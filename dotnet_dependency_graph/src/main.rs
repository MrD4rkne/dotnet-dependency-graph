use eframe::{App, run_native};
use egui::Context;
use egui_file_dialog::FileDialog;
use nuget_dgspec_parser::graph::{DependencyGraph, DependencyId, Framework, Layout};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

mod graph_widget;
mod parse;
mod visualize;

use graph_widget::GraphWidget;

struct File {
    path: PathBuf,
    graph: DependencyGraph,
    node_positions: HashMap<DependencyId, (f32, f32)>,
    selected_framework: Option<Framework>,
    visible_nodes: HashSet<DependencyId>,
}

impl File {
    fn new(
        path: PathBuf,
        graph: DependencyGraph,
        node_positions: HashMap<DependencyId, (f32, f32)>,
    ) -> Self {
        // Initialize all nodes as visible
        let visible_nodes: HashSet<DependencyId> = graph.iter().map(|(id, _)| id.clone()).collect();

        Self {
            path,
            graph,
            node_positions,
            selected_framework: None,
            visible_nodes,
        }
    }
}

struct DependencyApp {
    file_dialog: FileDialog,
    current_dgspec_file: Option<File>,
    pan_offset: egui::Vec2,
    zoom: f32,
    dragging_node: Option<DependencyId>,
    error_text: Option<String>,
    package_filter: String,
}

impl DependencyApp {
    fn new() -> Self {
        Self {
            file_dialog: FileDialog::new(),
            current_dgspec_file: None,
            pan_offset: egui::Vec2::ZERO,
            zoom: 1.0,
            dragging_node: None,
            error_text: None,
            package_filter: String::new(),
        }
    }
}

impl App for DependencyApp {
    fn update(&mut self, ctx: &Context, _: &mut eframe::Frame) {
        // --- Menu Bar using Top Panel ---
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Open file").clicked() {
                        self.file_dialog.pick_file();
                    }
                });
            });

            if let Some(file) = &mut self.current_dgspec_file {
                ui.horizontal(|ui| {
                    ui.label("Framework:");
                    for fw in file.graph.iter_frameworks() {
                        if ui
                            .selectable_label(
                                file.selected_framework.as_ref() == Some(fw),
                                fw.get_name(),
                            )
                            .clicked()
                        {
                            file.selected_framework = Some(fw.clone())
                        }
                    }
                });
            }
        });

        self.file_dialog.update(ctx);
        if let Some(path) = self.file_dialog.take_picked() {
            let new_file = load_file(path);
            match new_file {
                Ok(loaded_file) => self.current_dgspec_file = Some(loaded_file),
                Err(e) => {
                    self.error_text = Some(format!("Failed to load dgspec file: {}", e));
                }
            }
        }

        if let Some(file) = &mut self.current_dgspec_file {
            // Side panel with node list
            egui::SidePanel::left("nodes_panel").show(ctx, |ui| {
                ui.heading("Packages");
                ui.separator();

                // Add search/filter box
                ui.horizontal(|ui| {
                    ui.label("Filter:");
                    ui.text_edit_singleline(&mut self.package_filter);
                });

                ui.separator();

                // Add Select All / Deselect All buttons
                ui.horizontal(|ui| {
                    if ui.button("Select All").clicked() {
                        file.visible_nodes = file.graph.iter().map(|(id, _)| id.clone()).collect();
                    }
                    if ui.button("Deselect All").clicked() {
                        file.visible_nodes.clear();
                    }
                });

                ui.separator();

                egui::ScrollArea::vertical().show(ui, |ui| {
                    let mut nodes: Vec<_> = file.graph.iter().collect();
                    // Sort for consistent ordering
                    nodes.sort_by(|a, b| get_display_name(a.1).cmp(&get_display_name(b.1)));

                    // Apply filter
                    let filter_lower = self.package_filter.to_lowercase();
                    let filtered_nodes: Vec<_> = if filter_lower.is_empty() {
                        nodes
                    } else {
                        nodes
                            .into_iter()
                            .filter(|(_, info)| {
                                get_display_name(info)
                                    .to_lowercase()
                                    .contains(&filter_lower)
                            })
                            .collect()
                    };

                    for (id, info) in filtered_nodes {
                        let mut is_visible = file.visible_nodes.contains(id);
                        let display_name = get_display_name(info);

                        if ui.checkbox(&mut is_visible, &display_name).changed() {
                            if is_visible {
                                file.visible_nodes.insert(id.clone());
                            } else {
                                file.visible_nodes.remove(id);
                            }
                        }
                    }
                });
            });
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(file) = &mut self.current_dgspec_file {
                ui.label(format!(
                    "File: {}",
                    file.path.file_name().unwrap_or_default().to_string_lossy()
                ));

                ui.add(GraphWidget::new(
                    &file.graph,
                    &mut self.pan_offset,
                    &mut self.zoom,
                    &mut file.node_positions,
                    &mut self.dragging_node,
                    &file.selected_framework,
                    &file.visible_nodes,
                ));

                // Show controls
                ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                    ui.label(format!(
                        "Zoom: {:.1}x | Pan: ({:.0}, {:.0})",
                        self.zoom, self.pan_offset.x, self.pan_offset.y
                    ));
                    ui.label(
                        "Mouse wheel to zoom | Drag background to pan | Drag nodes to move them",
                    );
                });
            } else {
                ui.label("Choose a .dgspec file to visualize dependencies.");
            }
        });

        if let Some(error_message) = self.error_text.clone() {
            egui::Window::new("Error").show(ctx, |ui| {
                ui.label(&error_message);

                if ui.button("Ok").clicked() {
                    self.error_text = None;
                }
            });
        }
    }
}

fn load_file(path: PathBuf) -> std::io::Result<File> {
    let graph = parse::load_dgspec_from_file(path.to_path_buf())?;
    let layouts = calculate_layout(&graph);
    let node_positions = visualize::join_layouts(layouts);
    Ok(File::new(path, graph, node_positions))
}

fn calculate_layout(graph: &DependencyGraph) -> Vec<Layout<DependencyId>> {
    graph.layout(&visualize::calculate_size)
}

fn get_display_name(dep: &nuget_dgspec_parser::graph::DependencyInfo) -> String {
    use nuget_dgspec_parser::graph::DependencyInfo;

    match dep {
        DependencyInfo::Project(proj) => {
            // Extract just the project name from the full path
            if let Some(file_name) = std::path::Path::new(&proj.path).file_stem()
                && let Some(name_str) = file_name.to_str()
            {
                return name_str.to_string();
            }
            proj.path.clone()
        }
        DependencyInfo::Package(pck) => {
            format!(
                "{}@{}",
                pck.name,
                pck.version.clone().unwrap_or_else(|| "?".to_string())
            )
        }
    }
}

fn main() -> Result<(), eframe::Error> {
    let native_options = eframe::NativeOptions::default();
    run_native(
        "Dotnet Dependency Viewer",
        native_options,
        Box::new(move |_| Ok(Box::new(DependencyApp::new()))),
    )
}
