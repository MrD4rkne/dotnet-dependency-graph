use dotnet_dependency_parser::graph::{DependencyGraph, DependencyId, DependencyInfo};
use egui::{Response, Ui, Widget};
use regex::Regex;
use std::collections::{BTreeMap, HashSet};

use crate::node::get_display_text;

#[derive(Debug, Clone)]
pub(crate) struct SearchOptions {
    pub kind: SearchKind,
    pub whole_word: bool,
    pub case_sensitive: bool,
}

impl SearchOptions {
    pub fn new() -> Self {
        Self {
            kind: SearchKind::Text,
            whole_word: false,
            case_sensitive: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SearchKind {
    Text,
    Regex,
}

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

pub(crate) struct PackagesPanel<'a> {
    graph: &'a DependencyGraph,
    visible_nodes: &'a mut HashSet<DependencyId>,
    filter: &'a mut String,
    search_options: &'a mut SearchOptions,
}

impl<'a> PackagesPanel<'a> {
    pub fn new(
        graph: &'a DependencyGraph,
        visible_nodes: &'a mut HashSet<DependencyId>,
        filter: &'a mut String,
        search_options: &'a mut SearchOptions,
    ) -> Self {
        Self {
            graph,
            visible_nodes,
            filter,
            search_options,
        }
    }

    fn show(&mut self, ui: &mut Ui) {
        ui.heading("Packages");
        ui.separator();

        let searcher = Searcher::new(&*self.search_options, self.filter);

        self.show_search_box(ui, &searcher);
        self.show_mode_selection(ui);

        let groups = Self::group_packages_by_name(self.graph);
        let dependencies_to_show =
            Self::compute_dependencies_to_show_from_groups(groups, &searcher);

        self.show_selection_buttons(ui, &dependencies_to_show);

        self.show_packages(ui, &dependencies_to_show, &searcher);
    }

    fn show_search_box(&mut self, ui: &mut Ui, searcher: &Searcher) {
        ui.horizontal(|ui| {
            ui.label("Filter:");
            let original_visuals = ui.visuals().clone();
            if !searcher.is_valid() {
                ui.visuals_mut().override_text_color = Some(egui::Color32::RED);
                ui.visuals_mut().widgets.inactive.bg_stroke =
                    egui::Stroke::new(1.0, egui::Color32::RED);
                ui.visuals_mut().widgets.hovered.bg_stroke =
                    egui::Stroke::new(1.0, egui::Color32::RED);
                ui.visuals_mut().widgets.active.bg_stroke =
                    egui::Stroke::new(1.0, egui::Color32::RED);
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

    fn compute_dependencies_to_show_from_groups(
        groups: BTreeMap<String, Vec<(DependencyId, DependencyInfo)>>,
        searcher: &Searcher,
    ) -> Vec<(String, Vec<(DependencyId, DependencyInfo)>)> {
        groups
            .into_iter()
            .filter(|(name, _)| searcher.is_match(name))
            .map(|(name, mut versions)| {
                // Sort versions within each group
                versions.sort_by(|a, b| a.1.version().cmp(&b.1.version()));
                (name, versions)
            })
            .collect()
    }

    fn show_selection_buttons(
        &mut self,
        ui: &mut Ui,
        dependencies_to_show: &[(String, Vec<(DependencyId, DependencyInfo)>)],
    ) {
        ui.separator();

        ui.horizontal(|ui| {
            if ui.button("Select All").clicked() {
                for dep in dependencies_to_show {
                    dep.1.iter().for_each(|version| {
                        _ = self.visible_nodes.insert(version.0.clone());
                    });
                }
            }
            if ui.button("Deselect All").clicked() {
                for dep in dependencies_to_show {
                    dep.1.iter().for_each(|version| {
                        self.visible_nodes.remove(&version.0);
                    });
                }
            }
            if ui.button("Reset").clicked() {
                *self.visible_nodes = self.graph.iter().map(|(id, _)| id.clone()).collect();
                *self.filter = String::new();
            }
        });
    }

    fn show_packages(
        &mut self,
        ui: &mut Ui,
        dependencies_to_show: &[(String, Vec<(DependencyId, DependencyInfo)>)],
        searcher: &Searcher,
    ) {
        ui.separator();

        egui::ScrollArea::vertical().show(ui, |ui| {
            for (name, versions) in dependencies_to_show {
                if versions.len() == 1 {
                    // Single version - show as flat checkbox
                    let (id, _) = &versions[0];
                    show_checkbox(ui, self.visible_nodes, id.clone(), name, Some(searcher));
                } else {
                    // Multiple versions - show as collapsing header with nested items
                    egui::CollapsingHeader::new(rich_text_for_label(name, searcher))
                        .default_open(false)
                        .show(ui, |ui| {
                            for (id, info) in versions {
                                let version_label = info.version().unwrap_or("no version");
                                show_checkbox(
                                    ui,
                                    self.visible_nodes,
                                    id.clone(),
                                    version_label,
                                    None,
                                );
                            }
                        });
                }
            }
        });
    }

    fn group_packages_by_name(
        graph: &DependencyGraph,
    ) -> BTreeMap<String, Vec<(DependencyId, DependencyInfo)>> {
        let mut groups: BTreeMap<String, Vec<(DependencyId, DependencyInfo)>> = BTreeMap::new();
        for (id, info) in graph.iter() {
            let name = get_display_text(info);
            groups
                .entry(name.to_string())
                .or_default()
                .push((id.clone(), info.clone()));
        }
        groups
    }
}

impl<'a> Widget for PackagesPanel<'a> {
    fn ui(mut self, ui: &mut Ui) -> Response {
        ui.group(|ui| {
            self.show(ui);
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
) {
    let mut is_visible = visible_nodes.contains(&id);
    let label = match searcher {
        Some(s) => rich_text_for_label(label, s),
        None => egui::WidgetText::Text(label.to_string()),
    };

    if ui.checkbox(&mut is_visible, label).changed() {
        if is_visible {
            visible_nodes.insert(id);
        } else {
            visible_nodes.remove(&id);
        }
    }
}

// Create WidgetText content, but highlight the sequence matched by the searcher.
fn rich_text_for_label(label: &str, searcher: &Searcher) -> egui::WidgetText {
    if let Some((start, end)) = searcher.match_range(label) {
        let mut job = egui::text::LayoutJob::default();
        job.append(&label[..start], 0.0, egui::TextFormat::default());
        job.append(
            &label[start..end],
            0.0,
            egui::TextFormat {
                color: egui::Color32::YELLOW,
                ..Default::default()
            },
        );
        job.append(&label[end..], 0.0, egui::TextFormat::default());
        job.into()
    } else {
        label.into()
    }
}
