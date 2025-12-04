use dotnet_dependency_parser::graph::{DependencyGraph, DependencyId, Framework, Layout};
use eframe::{App, run_native};
use egui::Context;
use egui_file_dialog::FileDialog;
use std::collections::HashMap;
use std::collections::HashSet;
use std::path::PathBuf;
use std::time::{Duration, Instant};

mod graph_widget;
mod node;
mod parser;
mod visualize;

use graph_widget::{CachedNodeData, GraphWidget};

struct NodeCacheManager {
    cache: Option<std::collections::HashMap<DependencyId, CachedNodeData>>,
    old_zoom: f32,
    old_pan: egui::Vec2,
}

impl NodeCacheManager {
    fn new() -> Self {
        Self {
            cache: None,
            old_zoom: 1.0,
            old_pan: egui::Vec2::ZERO,
        }
    }

    fn get_or_compute(
        &mut self,
        graph: &DependencyGraph,
        positions: &std::collections::HashMap<DependencyId, (f32, f32)>,
        visible_nodes: &std::collections::HashSet<DependencyId>,
        zoom: f32,
        pan_offset: egui::Vec2,
    ) -> &std::collections::HashMap<DependencyId, CachedNodeData> {
        let zoom_changed = zoom != self.old_zoom;
        let pan_changed = pan_offset != self.old_pan;
        if zoom_changed || pan_changed {
            self.cache = None;
        }
        if self.cache.is_none() {
            let cache =
                graph_widget::compute_node_cache(graph, positions, visible_nodes, zoom, pan_offset);
            self.cache = Some(cache);
        }
        self.old_zoom = zoom;
        self.old_pan = pan_offset;
        self.cache.as_ref().unwrap()
    }

    fn invalidate(&mut self) {
        self.cache = None;
    }
}

struct FpsCounter {
    last_update: Instant,
    frames_since_last: u32,
    current_fps: f32,
}

impl FpsCounter {
    fn new() -> Self {
        Self {
            last_update: Instant::now(),
            frames_since_last: 0,
            current_fps: 0.0,
        }
    }

    fn update(&mut self) {
        self.frames_since_last += 1;
        if self.last_update.elapsed() >= Duration::from_secs(1) {
            self.current_fps = self.frames_since_last as f32;
            self.frames_since_last = 0;
            self.last_update = Instant::now();
        }
    }

    fn fps(&self) -> f32 {
        self.current_fps
    }
}

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
        let all_dep_ids = graph.iter().map(|(id, _)| id.clone()).collect();
        Self {
            path,
            graph,
            node_positions,
            selected_framework: None,
            visible_nodes: all_dep_ids,
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
    fps_counter: FpsCounter,
    cache_manager: NodeCacheManager,
    drag_happened: bool,
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
            fps_counter: FpsCounter::new(),
            cache_manager: NodeCacheManager::new(),
            drag_happened: false,
        }
    }
}

impl App for DependencyApp {
    fn update(&mut self, ctx: &Context, _: &mut eframe::Frame) {
        // Calculate FPS
        self.fps_counter.update();

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
                                fw.name(),
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
                Ok(loaded_file) => {
                    self.current_dgspec_file = Some(loaded_file);
                    self.cache_manager.invalidate();
                }
                Err(e) => {
                    self.error_text = Some(format!("Failed to load dgspec file: {}", e));
                }
            }
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(file) = &mut self.current_dgspec_file {
                let node_cache = self.cache_manager.get_or_compute(
                    &file.graph,
                    &file.node_positions,
                    &file.visible_nodes,
                    self.zoom,
                    self.pan_offset,
                );

                ui.label(format!(
                    "File: {}",
                    file.path.file_name().unwrap_or_default().to_string_lossy()
                ));

                ui.add(GraphWidget::new(
                    graph_widget::ViewState {
                        pan_offset: &mut self.pan_offset,
                        zoom: &mut self.zoom,
                    },
                    graph_widget::InteractionState {
                        dragging_node: &mut self.dragging_node,
                        node_positions: &mut file.node_positions,
                        drag_happened: &mut self.drag_happened,
                    },
                    graph_widget::GraphData {
                        graph: &file.graph,
                        selected_framework: &file.selected_framework,
                        visible_nodes: &file.visible_nodes,
                    },
                    node_cache,
                ));

                // Show controls
                ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                    ui.label(format!(
                        "Zoom: {:.1}x | Pan: ({:.0}, {:.0}) | FPS: {:.0}",
                        self.zoom,
                        self.pan_offset.x,
                        self.pan_offset.y,
                        self.fps_counter.fps()
                    ));
                    ui.label(
                        "Mouse wheel to zoom | Drag background to pan | Drag nodes to move them",
                    );
                });
            } else {
                ui.label("Choose a file to visualize dependencies.");
            }
        });

        // Invalidate cache if drag happened
        if self.drag_happened {
            self.cache_manager.invalidate();
        }
        self.drag_happened = false;

        if let Some(error_message) = self.error_text.clone() {
            egui::Window::new("Error").show(ctx, |ui| {
                ui.label(&error_message);

                if ui.button("Ok").clicked() {
                    self.error_text = None;
                }
            });
        }

        // Request continuous repaint for accurate FPS calculation
        ctx.request_repaint();
    }
}

fn load_file(path: PathBuf) -> Result<File, Box<dyn std::error::Error + Send + Sync>> {
    let graph = parser::parse_with_supported_parsers(&path)?;
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
