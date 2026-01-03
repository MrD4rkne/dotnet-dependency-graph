use dotnet_dependency_parser::graph::{DependencyGraph, DependencyId, DependencyInfo};
use eframe::egui::{self, Response, Ui, Widget, WidgetText};
use regex::Regex;
use std::collections::{BTreeMap, HashSet};

use crate::{graph::GraphCache, session::InteractionController};

/// Options for configuring search behavior in the packages panel.
#[derive(Debug, Clone)]
pub(crate) struct SearchOptions {
    /// The type of search to perform (text or regex).
    kind: SearchKind,
    /// If true, only match complete words.
    whole_word: bool,
    /// If true, perform case-sensitive matching.
    case_sensitive: bool,
}

impl Default for SearchOptions {
    fn default() -> Self {
        Self {
            kind: SearchKind::Text,
            whole_word: false,
            case_sensitive: false,
        }
    }
}

/// Search kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SearchKind {
    /// Compare raw text.
    Text,
    /// Use regex matching.
    Regex,
}

/// Struct responsible for handling matching of dependency name according to pattern and seatch options.
struct Searcher {
    pattern: String,
    regex: Option<Regex>,
}

impl Searcher {
    fn new(options: &SearchOptions, pattern: &str) -> Self {
        let pat = pattern.trim().to_string();

        let regex = if pat.is_empty() {
            None
        } else {
            let mut regex_pattern = String::new();
            if !options.case_sensitive {
                regex_pattern.push_str("(?i)");
            }
            if options.whole_word {
                regex_pattern.push_str("\\b");
            }
            if options.kind == SearchKind::Regex {
                regex_pattern.push_str(&pat);
            } else {
                regex_pattern.push_str(&regex::escape(&pat));
            }
            if options.whole_word {
                regex_pattern.push_str("\\b");
            }
            // Compile regex; use `.ok()` to avoid panicking on invalid user-provided regex.
            // `is_valid` will reflect failure to compile and UI will indicate invalid input.
            Regex::new(&regex_pattern).ok()
        };

        Self {
            pattern: pat,
            regex,
        }
    }

    fn is_match(&self, hay: &str) -> bool {
        if self.pattern.is_empty() {
            return true;
        }

        if let Some(ref r) = self.regex {
            r.is_match(hay)
        } else {
            // Invalid regex: show all packages
            true
        }
    }

    fn is_valid(&self) -> bool {
        if self.pattern.is_empty() {
            return true;
        }
        self.regex.is_some()
    }

    fn match_range(&self, hay: &str) -> Option<(usize, usize)> {
        if self.pattern.is_empty() {
            return None;
        }

        if let Some(ref r) = self.regex {
            r.find(hay).map(|m| (m.start(), m.end()))
        } else {
            None
        }
    }
}

enum Action {
    DeselectAll, // Desellect all visible deps
    SelectAll,   // Select all visible deps
}

/// A panel widget for displaying and filtering dependency packages.  
pub(crate) struct DependencyPanel<'a> {
    /// Text pattern used to filter dependencies in the list.
    filter: &'a mut String,
    /// Additional search options.
    search_options: &'a mut SearchOptions,
    graph: &'a DependencyGraph,
    visible_nodes: &'a mut HashSet<DependencyId>,
    cache: &'a mut GraphCache,
    interaction_state: &'a mut InteractionController,
}

impl<'a> DependencyPanel<'a> {
    pub(crate) fn new(
        filter: &'a mut String,
        search_options: &'a mut SearchOptions,
        graph: &'a DependencyGraph,
        visible_nodes: &'a mut HashSet<DependencyId>,
        cache: &'a mut GraphCache,
        interaction_state: &'a mut InteractionController,
    ) -> Self {
        Self {
            filter,
            search_options,
            graph,
            visible_nodes,
            cache,
            interaction_state,
        }
    }

    fn show_search_box(&mut self, ui: &mut Ui, searcher: &Searcher) {
        puffin::profile_function!();
        ui.take_available_width();
        ui.horizontal(|ui| {
            ui.label("Filter:");
            let original_visuals = ui.visuals().clone();
            if !searcher.is_valid() {
                ui.visuals_mut().override_text_color = Some(eframe::egui::Color32::RED);
                ui.visuals_mut().widgets.inactive.bg_stroke =
                    eframe::egui::Stroke::new(1.0, eframe::egui::Color32::RED);
                ui.visuals_mut().widgets.hovered.bg_stroke =
                    eframe::egui::Stroke::new(1.0, eframe::egui::Color32::RED);
                ui.visuals_mut().widgets.active.bg_stroke =
                    eframe::egui::Stroke::new(1.0, eframe::egui::Color32::RED);
            }
            ui.text_edit_singleline(self.filter);
            *ui.visuals_mut() = original_visuals;
        });
    }

    fn show_mode_selection(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label("Mode:");
            ui.selectable_value(&mut self.search_options.kind, SearchKind::Text, "Text");
            ui.selectable_value(&mut self.search_options.kind, SearchKind::Regex, "Regex");
            ui.separator();
            ui.checkbox(&mut self.search_options.whole_word, "Match Whole Word");
            ui.checkbox(&mut self.search_options.case_sensitive, "Match Case");
        });
    }

    fn compute_dependencies_to_show_from_groups<'g>(
        groups: &'g BTreeMap<String, Vec<DependencyId>>,
        searcher: &Searcher,
    ) -> impl Iterator<Item = (&'g String, &'g Vec<DependencyId>)> {
        puffin::profile_function!();
        groups.iter().filter(|(name, _)| searcher.is_match(name))
    }

    fn show_selection_buttons(&mut self, ui: &mut Ui) -> Option<Action> {
        ui.separator();

        let mut action = None;

        ui.horizontal(|ui| {
            if ui.button("Select All").clicked() {
                action = Some(Action::SelectAll);
            }
            if ui.button("Deselect All").clicked() {
                action = Some(Action::DeselectAll);
            }
            if ui.button("Reset").clicked() {
                *self.filter = String::new();
                *self.visible_nodes = self.graph.iter().map(|x| x.0).collect();
            }
        });

        action
    }

    fn show_packages_and_update_visibility<'g>(
        ui: &mut Ui,
        graph: &DependencyGraph,
        visible_nodes: &mut HashSet<DependencyId>,
        dependencies_to_show: impl Iterator<Item = (&'g String, &'g Vec<DependencyId>)>,
        searcher: &Searcher,
        action: Option<Action>,
        interaction_state: &'g mut InteractionController,
    ) {
        puffin::profile_function!();
        ui.separator();

        eframe::egui::ScrollArea::vertical().show(ui, |ui| {
            for (name, versions) in dependencies_to_show {
                if versions.len() == 1 {
                    // Single version - show as flat checkbox
                    puffin::profile_scope!("show_package_single");
                    let id = versions[0];
                    show_checkbox(
                        ui,
                        visible_nodes,
                        id,
                        name,
                        Some(searcher),
                        interaction_state,
                    );
                    handle_action(visible_nodes, &id, &action);
                } else {
                    puffin::profile_scope!("show_package_multiple");
                    // Multiple versions - show as collapsing header with nested items
                    eframe::egui::CollapsingHeader::new(rich_text_for_label(name, searcher))
                        .default_open(false)
                        .show(ui, |ui| {
                            for id in versions {
                                puffin::profile_scope!("show_package");
                                let info = graph.get(*id).unwrap();
                                let version_label = info.version().unwrap_or("no version");
                                show_checkbox(
                                    ui,
                                    visible_nodes,
                                    *id,
                                    version_label,
                                    None,
                                    interaction_state,
                                );
                            }
                        });
                    puffin::profile_scope!("handle_action");
                    // Handle action here so this code will be invoked even if the header is collapsed.
                    for id in versions {
                        handle_action(visible_nodes, id, &action);
                    }
                }
            }
        });
    }
}

fn handle_action(
    visible_nodes: &mut HashSet<DependencyId>,
    id: &DependencyId,
    action: &Option<Action>,
) {
    match action {
        Some(Action::SelectAll) => {
            visible_nodes.insert(*id);
        }
        Some(Action::DeselectAll) => {
            visible_nodes.remove(id);
        }
        None => {}
    };
}

impl<'a> Widget for DependencyPanel<'a> {
    fn ui(mut self, ui: &mut Ui) -> Response {
        ui.vertical(|ui| {
            puffin::profile_function!();
            ui.heading("Packages");
            ui.separator();

            let searcher = Searcher::new(&*self.search_options, self.filter);

            self.show_search_box(ui, &searcher);
            self.show_mode_selection(ui);

            let action = self.show_selection_buttons(ui);
            let tree = self.cache.dependency_tree();
            let dependencies_to_show =
                Self::compute_dependencies_to_show_from_groups(tree, &searcher);

            Self::show_packages_and_update_visibility(
                ui,
                self.graph,
                self.visible_nodes,
                dependencies_to_show,
                &searcher,
                action,
                self.interaction_state,
            );
        })
        .response
    }
}

// Display checkbox. If searcher is provided, match fragment of text will be highlighted.
fn show_checkbox(
    ui: &mut Ui,
    visible_nodes: &mut HashSet<DependencyId>,
    id: DependencyId,
    label: &str,
    searcher: Option<&Searcher>,
    interaction_state: &mut InteractionController,
) {
    let mut is_visible = visible_nodes.contains(&id);
    let label = match searcher {
        Some(s) => rich_text_for_label(label, s),
        None => eframe::egui::WidgetText::Text(label.to_string()),
    };

    ui.horizontal(|ui| {
        // "" is intentional, in order to have both: select / deselect checkbox AND selectable label.
        if ui.checkbox(&mut is_visible, "").changed() {
            if is_visible {
                visible_nodes.insert(id);
            } else {
                visible_nodes.remove(&id);
            }
        }

        show_label_for_dependency(ui, interaction_state, visible_nodes, id, label);
    });
}

fn show_label_for_dependency(
    ui: &mut Ui,
    interaction_state: &mut crate::session::InteractionController,
    visible_nodes: &mut HashSet<DependencyId>,
    id: DependencyId,
    label: WidgetText,
) {
    let is_selected = interaction_state.selected_dependency() == Some(id);
    ui.horizontal_wrapped(|ui| {
        let mut label = ui.selectable_label(is_selected, label);
        // If the representing Dependency is highlighted, highlight the label.
        // This way we highlight ALL labels that are connected to this Dependency.
        if interaction_state.highlighted_dependency() == Some(id) {
            label = label.highlight();
        }

        if label.clicked() {
            interaction_state.publish(crate::session::InteractionEvent::Select(id));
        }

        if label.hovered() {
            interaction_state.publish(crate::session::InteractionEvent::Highlight(id));
        }
    });
}

// Create WidgetText content, but highlight the sequence matched by the searcher.
fn rich_text_for_label(label: &str, searcher: &Searcher) -> eframe::egui::WidgetText {
    if let Some((start, end)) = searcher.match_range(label) {
        let mut job = eframe::egui::text::LayoutJob::default();
        job.append(&label[..start], 0.0, eframe::egui::TextFormat::default());
        job.append(
            &label[start..end],
            0.0,
            eframe::egui::TextFormat {
                color: eframe::egui::Color32::YELLOW,
                ..Default::default()
            },
        );
        job.append(&label[end..], 0.0, eframe::egui::TextFormat::default());
        job.into()
    } else {
        label.into()
    }
}

pub(crate) struct DepPanel<'a> {
    graph: &'a DependencyGraph,
    interaction_state: &'a mut crate::session::InteractionController,
    visible_nodes: &'a mut HashSet<DependencyId>,
}

impl<'a> DepPanel<'a> {
    pub(crate) fn new(
        graph: &'a DependencyGraph,
        interaction_state: &'a mut crate::session::InteractionController,
        visible_nodes: &'a mut HashSet<DependencyId>,
    ) -> Self {
        Self {
            graph,
            interaction_state,
            visible_nodes,
        }
    }

    fn show_column_with_deps(
        &mut self,
        ui: &mut Ui,
        text: impl Into<WidgetText>,
        id_salt: impl std::hash::Hash,
        deps: impl Iterator<Item = (DependencyId, &'a DependencyInfo)>,
    ) {
        ui.group(|ui| {
            ui.take_available_height();
            ui.label(text);
            egui::ScrollArea::vertical()
                .id_salt(id_salt)
                .show(ui, |ui| {
                    ui.take_available_width();
                    for (id, info) in deps {
                        show_label_for_dependency(
                            ui,
                            self.interaction_state,
                            self.visible_nodes,
                            id,
                            eframe::egui::WidgetText::Text(info.name().to_string()),
                        );
                    }
                });
        });
    }
}

impl<'a> Widget for DepPanel<'a> {
    fn ui(mut self, ui: &mut Ui) -> Response {
        ui.take_available_height();
        ui.vertical(|ui| {
            match (
                self.interaction_state.selected_dependency(),
                self.interaction_state.selected_framework().cloned(),
            ) {
                (Some(dep), Some(framework)) => {
                    ui.columns(2, |columns| {
                        let deps = self
                            .graph
                            .get_direct_dependencies_in_framework(dep, &framework)
                            .unwrap()
                            .map(|edge| (edge.to(), self.graph.get(edge.to()).unwrap()));
                        self.show_column_with_deps(
                            &mut columns[0],
                            "Direct dependencies",
                            "deps_scroll",
                            deps,
                        );

                        let reverse_deps = self
                            .graph
                            .get_direct_reverse_dependencies_in_framework(dep, &framework)
                            .unwrap()
                            .map(|edge| (edge.from(), self.graph.get(edge.from()).unwrap()));
                        self.show_column_with_deps(
                            &mut columns[1],
                            "Reverse direct dependencies",
                            "reverse_deps_scroll",
                            reverse_deps,
                        );
                    });
                }
                _ => {
                    let label = match (
                        self.interaction_state.selected_dependency(),
                        self.interaction_state.selected_framework().cloned(),
                    ) {
                        (Some(_), None) => "Select framework",
                        (None, Some(_)) => "Select dependency",
                        (None, None) => "Select framework and dependency",
                        _ => unreachable!(),
                    };

                    ui.with_layout(
                        eframe::egui::Layout::centered_and_justified(
                            eframe::egui::Direction::TopDown,
                        ),
                        |ui| ui.label(label),
                    );
                }
            }
        })
        .response
    }
}
