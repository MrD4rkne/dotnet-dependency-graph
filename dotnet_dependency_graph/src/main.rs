use eframe::{App, run_native};
use egui::Context;
use egui_file_dialog::FileDialog;
use nuget_dgspec_parser::graph::{DependencyGraph, DependencyId, Layout};
use std::collections::HashMap;
use std::path::PathBuf;

mod graph_widget;
mod parse;
mod visualize;

use graph_widget::GraphWidget;

struct File {
    path: PathBuf,
    graph: DependencyGraph,
    node_positions: HashMap<DependencyId, (f32, f32)>,
}

impl File {
    fn new(
        path: PathBuf,
        graph: DependencyGraph,
        node_positions: HashMap<DependencyId, (f32, f32)>,
    ) -> Self {
        Self {
            path,
            graph,
            node_positions,
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

fn main() -> Result<(), eframe::Error> {
    let native_options = eframe::NativeOptions::default();
    run_native(
        "Dotnet Dependency Viewer",
        native_options,
        Box::new(move |_| Ok(Box::new(DependencyApp::new()))),
    )
}
