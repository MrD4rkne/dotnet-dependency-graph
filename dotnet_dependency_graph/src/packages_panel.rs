use egui::Ui;
use nuget_dgspec_parser::graph::{DependencyGraph, DependencyId, DependencyInfo};
use std::collections::{BTreeMap, HashSet};

pub struct PackagesPanel<'a> {
    graph: &'a DependencyGraph,
    visible_nodes: &'a mut HashSet<DependencyId>,
    filter: &'a mut String,
}

impl<'a> PackagesPanel<'a> {
    pub fn new(
        graph: &'a DependencyGraph,
        visible_nodes: &'a mut HashSet<DependencyId>,
        filter: &'a mut String,
    ) -> Self {
        Self {
            graph,
            visible_nodes,
            filter,
        }
    }

    pub fn show(&mut self, ui: &mut Ui) {
        ui.heading("Packages");
        ui.separator();

        // Add search/filter box
        ui.horizontal(|ui| {
            ui.label("Filter:");
            ui.text_edit_singleline(self.filter);
        });

        ui.separator();

        // Add Select All / Deselect All buttons
        ui.horizontal(|ui| {
            if ui.button("Select All").clicked() {
                *self.visible_nodes = self.graph.iter().map(|(id, _)| id.clone()).collect();
            }
            if ui.button("Deselect All").clicked() {
                self.visible_nodes.clear();
            }
        });

        ui.separator();

        egui::ScrollArea::vertical().show(ui, |ui| {
            // Group nodes by name
            let mut groups: BTreeMap<String, Vec<(&DependencyId, &DependencyInfo)>> =
                BTreeMap::new();

            for (id, info) in self.graph.iter() {
                let name = get_display_name(info);
                groups.entry(name).or_default().push((id, info));
            }

            // Apply filter
            let filter_lower = self.filter.to_lowercase();

            for (name, mut versions) in groups {
                if !filter_lower.is_empty() && !name.to_lowercase().contains(&filter_lower) {
                    continue;
                }

                // Sort versions within each group
                versions.sort_by(|a, b| match (a.1, b.1) {
                    (DependencyInfo::Package(p1), DependencyInfo::Package(p2)) => {
                        p1.version.cmp(&p2.version)
                    }
                    _ => std::cmp::Ordering::Equal,
                });

                if versions.len() == 1 {
                    // Single version - show as flat checkbox
                    let (id, _) = versions[0];
                    show_checkbox(ui, self.visible_nodes, id.clone(), &name);
                } else {
                    // Multiple versions - show as collapsing header with nested items
                    egui::CollapsingHeader::new(&name)
                        .default_open(false)
                        .show(ui, |ui| {
                            for (id, info) in versions {
                                let version_label = get_checkbox_text(info);
                                show_checkbox(ui, self.visible_nodes, id.clone(), version_label);
                            }
                        });
                }
            }
        });
    }
}

fn get_checkbox_text(dep: &DependencyInfo) -> &str {
    match dep {
        DependencyInfo::Package(pck) => pck.version.as_deref().unwrap_or("no version"),
        DependencyInfo::Project(proj) => proj.version.as_deref().unwrap_or("no version"),
    }
}

fn show_checkbox(
    ui: &mut Ui,
    visible_nodes: &mut HashSet<DependencyId>,
    id: DependencyId,
    label: &str,
) {
    let mut is_visible = visible_nodes.contains(&id);
    if ui.checkbox(&mut is_visible, label).changed() {
        if is_visible {
            visible_nodes.insert(id);
        } else {
            visible_nodes.remove(&id);
        }
    }
}

fn get_display_name(dep: &DependencyInfo) -> String {
    dep.get_name().to_string()
}
