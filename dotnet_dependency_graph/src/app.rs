use anyhow::Error;
use dotnet_dependency_parser::graph::DependencyId;
use eframe::App;
use eframe::egui::Context;
use egui_file_dialog::FileDialog;
use std::collections::HashMap;
use std::collections::HashSet;
use std::time::{Duration, Instant};

use crate::dependency_panel::DependencyPanel;
use crate::dependency_panel::SearchOptions;
use crate::graph::CachedNodeData;
use crate::graph::graph_widget::GraphWidget;
use crate::parser;
use crate::session::Session;

/// Handles file dialog operations.
struct FileDialogHandler {
    file_dialog: FileDialog,
    mode: OpenFileMode,
}

#[derive(PartialEq, Eq)]
enum OpenFileMode {
    Replace,
    Merge,
    None,
}

impl FileDialogHandler {
    fn new() -> Self {
        Self {
            file_dialog: FileDialog::new(),
            mode: OpenFileMode::None,
        }
    }

    fn handle(
        &mut self,
        app_state: &mut AppState,
        cache_manager: &mut NodeCacheManager,
    ) -> Result<(), Error> {
        if self.mode == OpenFileMode::None {
            return Ok(());
        }

        let Some(path) = self.file_dialog.take_picked() else {
            return Ok(());
        };

        let new_graph = match parser::parse_with_supported_parsers(&path) {
            Ok(graph) => graph,
            Err(e) => {
                return Err(anyhow::anyhow!("Failed to parse file: {}", e));
            }
        };

        match (&mut *app_state, &self.mode) {
            (AppState::FileLoaded(session), OpenFileMode::Merge) => {
                if let Err(e) = session.merge(new_graph) {
                    return Err(anyhow::anyhow!("Failed to merge: {}", e));
                }
            }
            _ => match Session::load_from(path, new_graph) {
                Ok(session) => {
                    *app_state = AppState::FileLoaded(Box::new(session));
                }
                Err(e) => {
                    return Err(anyhow::anyhow!("Failed to load session: {}", e));
                }
            },
        }

        cache_manager.invalidate();
        self.mode = OpenFileMode::None;

        Ok(())
    }

    fn render(&mut self, ctx: &Context, app_state: &mut AppState) {
        self.file_dialog.update(ctx);

        eframe::egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            eframe::egui::MenuBar::new().ui(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Open file").clicked() {
                        self.open_for_replace();
                    }

                    if matches!(app_state, AppState::FileLoaded(_))
                        && ui.button("Merge file").clicked()
                    {
                        self.open_for_merge();
                    }
                });
            });

            if let AppState::FileLoaded(file) = app_state {
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
                            file.selected_framework = Some(fw.clone());
                        }
                    }
                });
            }
        });
    }

    fn open_for_replace(&mut self) {
        self.mode = OpenFileMode::Replace;
        self.file_dialog.pick_file();
    }

    fn open_for_merge(&mut self) {
        self.mode = OpenFileMode::Merge;
        self.file_dialog.pick_file();
    }
}

/// Handles central panel rendering.
struct CentralPanelRenderer<'a> {
    pan_offset: &'a mut eframe::egui::Vec2,
    zoom: &'a mut f32,
    dragging_node: &'a mut Option<DependencyId>,
    fps_counter: &'a FpsCounter,
    cache_manager: &'a mut NodeCacheManager,
}

impl<'a> CentralPanelRenderer<'a> {
    fn new(
        pan_offset: &'a mut eframe::egui::Vec2,
        zoom: &'a mut f32,
        dragging_node: &'a mut Option<DependencyId>,
        fps_counter: &'a FpsCounter,
        cache_manager: &'a mut NodeCacheManager,
    ) -> Self {
        Self {
            pan_offset,
            zoom,
            dragging_node,
            fps_counter,
            cache_manager,
        }
    }

    fn render(&mut self, ctx: &Context, app_state: &mut AppState) {
        eframe::egui::CentralPanel::default().show(ctx, |ui| {
            if let AppState::FileLoaded(file) = app_state {
                let mut node_cache = self.cache_manager.get_or_compute(
                    &file.graph,
                    &file.node_positions,
                    &file.visible_nodes,
                    *self.zoom,
                    *self.pan_offset,
                );

                ui.label(format!(
                    "File: {}",
                    file.path.file_name().unwrap_or_default().to_string_lossy()
                ));

                ui.add(GraphWidget::new(
                    crate::graph::graph_widget::ViewState::new(self.pan_offset, self.zoom),
                    crate::graph::graph_widget::InteractionState::new(self.dragging_node),
                    crate::graph::graph_widget::GraphData::new(
                        &file.graph,
                        &file.selected_framework,
                        &file.visible_nodes,
                    ),
                    &mut node_cache,
                ));

                // Show controls
                ui.with_layout(
                    eframe::egui::Layout::bottom_up(eframe::egui::Align::LEFT),
                    |ui| {
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
                    },
                );
            } else {
                ui.label("Choose a file to visualize dependencies.");
            }
        });
    }
}

/// Handles error window rendering.
struct ErrorWindowRenderer<'a> {
    error_text: &'a mut Option<String>,
}

impl<'a> ErrorWindowRenderer<'a> {
    fn new(error_text: &'a mut Option<String>) -> Self {
        Self { error_text }
    }

    fn render(&mut self, ctx: &Context) {
        if let Some(error_message) = self.error_text.clone() {
            eframe::egui::Window::new("Error").show(ctx, |ui| {
                ui.label(&error_message);

                if ui.button("Ok").clicked() {
                    *self.error_text = None;
                }
            });
        }
    }
}

/// Handles packages view rendering.
struct PackagesViewRenderer<'a> {
    package_filter: &'a mut String,
    search_options: &'a mut SearchOptions,
}

impl<'a> PackagesViewRenderer<'a> {
    fn new(package_filter: &'a mut String, search_options: &'a mut SearchOptions) -> Self {
        Self {
            package_filter,
            search_options,
        }
    }

    fn render(&mut self, ctx: &Context, app_state: &mut AppState) {
        if let AppState::FileLoaded(file) = app_state {
            eframe::egui::SidePanel::left("nodes_panel").show(ctx, |ui| {
                ui.add(DependencyPanel::new(
                    &file.graph,
                    &mut file.visible_nodes,
                    self.package_filter,
                    self.search_options,
                ));
            });
        }
    }
}

/// Represents the current state of the application.
#[derive(Debug)]
enum AppState {
    /// No file is currently loaded.
    NoFile,
    /// A file is loaded and ready for visualization.
    FileLoaded(Box<Session>),
}

struct NodeCacheManager {
    cache: Option<HashMap<DependencyId, CachedNodeData>>,
}

impl NodeCacheManager {
    fn new() -> Self {
        Self { cache: None }
    }

    fn get_or_compute(
        &mut self,
        graph: &dotnet_dependency_parser::graph::DependencyGraph,
        positions: &HashMap<DependencyId, (f32, f32)>,
        visible_nodes: &HashSet<DependencyId>,
        zoom: f32,
        pan_offset: egui::Vec2,
    ) -> &mut HashMap<DependencyId, CachedNodeData> {
        if self.cache.is_none() {
            let cache = crate::graph::graph_widget::compute_nodes_cache(
                graph,
                positions,
                visible_nodes,
                zoom,
                pan_offset,
            );
            self.cache = Some(cache);
        }
        self.cache.as_mut().unwrap()
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

pub(crate) struct DependencyApp {
    app_state: AppState,
    pan_offset: eframe::egui::Vec2,
    zoom: f32,
    dragging_node: Option<DependencyId>,
    error_text: Option<String>,
    fps_counter: FpsCounter,
    cache_manager: NodeCacheManager,
    drag_happened: bool,
    package_filter: String,
    search_options: SearchOptions,
    file_dialog_handler: FileDialogHandler,
}

impl Default for DependencyApp {
    fn default() -> Self {
        Self {
            app_state: AppState::NoFile,
            pan_offset: eframe::egui::Vec2::ZERO,
            zoom: 1.0,
            dragging_node: None,
            error_text: None,
            fps_counter: FpsCounter::new(),
            cache_manager: NodeCacheManager::new(),
            drag_happened: false,
            package_filter: String::new(),
            search_options: SearchOptions::default(),
            file_dialog_handler: FileDialogHandler::new(),
        }
    }
}

impl DependencyApp {
    fn render_central_panel(&mut self, ctx: &Context) {
        let mut renderer = CentralPanelRenderer::new(
            &mut self.pan_offset,
            &mut self.zoom,
            &mut self.dragging_node,
            &self.fps_counter,
            &mut self.cache_manager,
        );
        renderer.render(ctx, &mut self.app_state);
    }

    fn render_error_window(&mut self, ctx: &Context) {
        let mut renderer = ErrorWindowRenderer::new(&mut self.error_text);
        renderer.render(ctx);
    }

    fn render_packages_view(&mut self, ctx: &Context) {
        let mut renderer =
            PackagesViewRenderer::new(&mut self.package_filter, &mut self.search_options);
        renderer.render(ctx, &mut self.app_state);
    }
}
impl App for DependencyApp {
    fn update(&mut self, ctx: &Context, _: &mut eframe::Frame) {
        self.fps_counter.update();
        self.file_dialog_handler.render(ctx, &mut self.app_state);
        if let Err(error) = self
            .file_dialog_handler
            .handle(&mut self.app_state, &mut self.cache_manager)
        {
            self.error_text = Some(error.to_string());
        }

        self.drag_happened = false;

        // Render left side first not to overlay over central panel.
        self.render_packages_view(ctx);
        self.render_central_panel(ctx);

        self.render_error_window(ctx);
        ctx.request_repaint();
    }
}
