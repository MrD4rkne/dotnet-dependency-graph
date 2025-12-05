use dotnet_dependency_parser::graph::DependencyId;
use eframe::App;
use egui::Context;
use egui_file_dialog::FileDialog;
use std::collections::HashMap;
use std::collections::HashSet;
use std::time::{Duration, Instant};

use crate::dependency_panel::DependencyPanel;
use crate::dependency_panel::SearchOptions;
use crate::graph_widget::{CachedNodeData, GraphWidget};
use crate::session::Session;

struct NodeCacheManager {
    cache: Option<HashMap<DependencyId, CachedNodeData>>,
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
        graph: &dotnet_dependency_parser::graph::DependencyGraph,
        positions: &HashMap<DependencyId, (f32, f32)>,
        visible_nodes: &HashSet<DependencyId>,
        zoom: f32,
        pan_offset: egui::Vec2,
    ) -> &HashMap<DependencyId, CachedNodeData> {
        let zoom_changed = zoom != self.old_zoom;
        let pan_changed = pan_offset != self.old_pan;
        if zoom_changed || pan_changed {
            self.cache = None;
        }
        if self.cache.is_none() {
            let cache = crate::graph_widget::compute_node_cache(
                graph,
                positions,
                visible_nodes,
                zoom,
                pan_offset,
            );
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

pub struct DependencyApp {
    file_dialog: FileDialog,
    current_dgspec_file: Option<Session>,
    pan_offset: egui::Vec2,
    zoom: f32,
    dragging_node: Option<DependencyId>,
    error_text: Option<String>,
    fps_counter: FpsCounter,
    cache_manager: NodeCacheManager,
    drag_happened: bool,
    package_filter: String,
    search_options: SearchOptions,
}

impl DependencyApp {
    pub fn new() -> Self {
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
            package_filter: String::new(),
            search_options: SearchOptions::default(),
        }
    }

    fn render_menu_bar(&mut self, ctx: &Context) {
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Open file").clicked() {
                        self.file_dialog.pick_file();
                    }

                    if self.current_dgspec_file.is_some() && ui.button("Merge file").clicked() {
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
    }

    fn handle_file_dialog(&mut self, ctx: &Context) {
        self.file_dialog.update(ctx);

        if let Some(path) = self.file_dialog.take_picked() {
            let new_file = Session::load_from(path);
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
    }

    fn render_central_panel(&mut self, ctx: &Context) {
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
                    crate::graph_widget::ViewState {
                        pan_offset: &mut self.pan_offset,
                        zoom: &mut self.zoom,
                    },
                    crate::graph_widget::InteractionState {
                        dragging_node: &mut self.dragging_node,
                        node_positions: &mut file.node_positions,
                        drag_happened: &mut self.drag_happened,
                    },
                    crate::graph_widget::GraphData {
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
    }

    fn render_error_window(&mut self, ctx: &Context) {
        if let Some(error_message) = self.error_text.clone() {
            egui::Window::new("Error").show(ctx, |ui| {
                ui.label(&error_message);

                if ui.button("Ok").clicked() {
                    self.error_text = None;
                }
            });
        }
    }

    fn render_packages_view(&mut self, ctx: &Context) {
        if let Some(file) = &mut self.current_dgspec_file {
            egui::SidePanel::left("nodes_panel").show(ctx, |ui| {
                ui.add(DependencyPanel::new(
                    &file.graph,
                    &mut file.visible_nodes,
                    &mut self.package_filter,
                    &mut self.search_options,
                ));
            });
        }
    }
}
impl App for DependencyApp {
    fn update(&mut self, ctx: &Context, _: &mut eframe::Frame) {
        self.fps_counter.update();
        self.render_menu_bar(ctx);
        self.handle_file_dialog(ctx);
        if self.drag_happened {
            self.cache_manager.invalidate();
        }
        self.drag_happened = false;

        // Render left side first not to overlay over central panel.
        self.render_packages_view(ctx);
        self.render_central_panel(ctx);

        self.render_error_window(ctx);
        ctx.request_repaint();
    }
}
