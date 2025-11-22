use eframe::{App, run_native};
use egui::Context;
use egui_file_dialog::FileDialog;
use nuget_dgspec_parser::graph::DependencyGraph;
use std::path::PathBuf;

mod graph_widget;
mod parse;
mod visualize;

use graph_widget::{GraphWidget, LayoutData};

struct File {
    path: PathBuf,
}

impl File {
    fn new(path: PathBuf) -> Self {
        Self { path }
    }
}

struct DependencyApp {
    file_dialog: FileDialog,
    current_dgspec_file: Option<File>,
    graph: Option<DependencyGraph>,
    layouts: Option<LayoutData>,
    pan_offset: egui::Vec2,
    zoom: f32,
}

impl DependencyApp {
    fn new() -> Self {
        Self {
            file_dialog: FileDialog::new(),
            current_dgspec_file: None,
            graph: None,
            layouts: None,
            pan_offset: egui::Vec2::ZERO,
            zoom: 1.0,
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
            self.current_dgspec_file = Some(File::new(path.to_path_buf()));

            let graph = parse::load_dgspec_from_file(path.to_path_buf())
                .expect("Failed to load dgspec file");

            let layouts = calculate_layout(&graph);
            println!(
                "Loaded dependency graph with {} nodes",
                graph.iter().count()
            );

            self.layouts = Some(layouts);
            self.graph = Some(graph);
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(file) = &self.current_dgspec_file {
                ui.label(format!(
                    "File: {}",
                    file.path.file_name().unwrap_or_default().to_string_lossy()
                ));
            } else {
                ui.label("Choose a .dgspec file to visualize dependencies.");
            }

            if let Some(graph) = &self.graph
                && let Some(layouts) = &self.layouts
            {
                ui.add(GraphWidget::new(
                    graph,
                    layouts,
                    &mut self.pan_offset,
                    &mut self.zoom,
                ));

                // Show controls
                ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                    ui.label(format!(
                        "Zoom: {:.1}x | Pan: ({:.0}, {:.0})",
                        self.zoom, self.pan_offset.x, self.pan_offset.y
                    ));
                    ui.label("Mouse wheel to zoom | Drag background to pan");
                });
            }
        });
    }
}

fn calculate_layout(graph: &DependencyGraph) -> LayoutData {
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
