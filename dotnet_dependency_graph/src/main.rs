use eframe::{App, run_native};
use egui::{Context, Painter, Pos2, Sense, Ui};
use egui_file_dialog::FileDialog;
use nuget_dgspec_parser::graph::{DependencyGraph, DependencyId, DependencyInfo};
use std::collections::HashMap;
use std::path::PathBuf;
mod parse;
mod visualize;

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
    pos: Option<Vec<(HashMap<DependencyId, (f64, f64)>, f64, f64)>>,
    pan_offset: egui::Vec2,
    zoom: f32,
}

impl DependencyApp {
    fn new() -> Self {
        Self {
            file_dialog: FileDialog::new(),
            current_dgspec_file: None,
            graph: None,
            pos: None,
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

            self.pos = Some(layouts);
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
                && let Some(layouts) = &self.pos
            {
                let (response, mut painter) =
                    ui.allocate_painter(ui.available_size(), Sense::click_and_drag());

                // Handle panning
                if response.dragged() {
                    self.pan_offset += response.drag_delta();
                }

                // Handle zoom with mouse wheel
                if response.hovered() {
                    let scroll = ui.input(|i| i.smooth_scroll_delta.y);
                    if scroll.abs() > 0.1 {
                        self.zoom *= 1.0 + scroll * 0.001;
                        self.zoom = self.zoom.clamp(0.1, 3.0);
                    }
                }

                draw_graph(
                    graph,
                    layouts,
                    ui,
                    &mut painter,
                    self.pan_offset,
                    self.zoom,
                    &response,
                );

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

fn draw_graph(
    graph: &DependencyGraph,
    layouts: &Vec<(HashMap<DependencyId, (f64, f64)>, f64, f64)>,
    ui: &mut Ui,
    painter: &mut Painter,
    pan_offset: egui::Vec2,
    zoom: f32,
    response: &egui::Response,
) {
    // Only draw the first layout component (avoid duplicates)
    if let Some((layout, _zoom, _size)) = layouts.first() {
        if layout.is_empty() {
            return;
        }

        let transform = |pos: Pos2| -> Pos2 {
            let centered = pos.to_vec2() * zoom + pan_offset;
            response.rect.min + centered
        };

        for (id, (x, y)) in layout {
            // Convert layout coordinates to screen coordinates with zoom and pan
            let pos = Pos2::new(*x as f32, *y as f32);
            let screen_pos = transform(pos);

            let text = get_displayed_text(
                graph
                    .get(id)
                    .expect("Dep from layout should be in the graph"),
            );
            let _rect = visualize::draw_node(ui, text, screen_pos, painter, zoom);
        }
    }
}

fn get_displayed_text(dep: &DependencyInfo) -> &str {
    match dep {
        DependencyInfo::Project(proj) => {
            // Extract just the project name from the full path
            if let Some(file_name) = std::path::Path::new(&proj.path).file_stem()
                && let Some(name_str) = file_name.to_str()
            {
                return name_str;
            }
            &proj.path
        }
        DependencyInfo::Package(pck) => &pck.name,
    }
}

fn calculate_layout(graph: &DependencyGraph) -> Vec<(HashMap<DependencyId, (f64, f64)>, f64, f64)> {
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
