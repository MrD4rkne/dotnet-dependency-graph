use anyhow::Error;
use dotnet_dependency_parser::graph::DependencyGraph;
use eframe::App;
use eframe::egui::Context;
use egui_file_dialog::FileDialog;
use puffin::GlobalProfiler;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use crate::background::{BackgroundWindow, PollResult};
use crate::dependency_panel::SearchOptions;
use crate::dependency_panel::{DepPanel, DependencyPanel};
use crate::graph::graph_widget::GraphWidget;
use crate::layout_options::LayoutConfig;
use crate::layout_options::LayoutWindow;
use crate::parser;
use crate::session::Session;

/// Handles file dialog operations.
struct FileDialogHandler {
    file_dialog: FileDialog,
    mode: FileMode,
}

#[derive(PartialEq, Eq)]
enum FileMode {
    Replace,
    Merge,
    Save,
    Load,
    None,
}

impl FileDialogHandler {
    fn new() -> Self {
        Self {
            file_dialog: FileDialog::new(),
            mode: FileMode::None,
        }
    }

    fn handle(&mut self, app_state: &mut AppState) -> Result<(), Error> {
        if self.mode == FileMode::None {
            return Ok(());
        }

        let Some(path) = self.file_dialog.take_picked() else {
            return Ok(());
        };

        match (&self.mode, &app_state) {
            (FileMode::Merge | FileMode::Replace, _) => {
                *app_state = AppState::LoadingFile(BackgroundWindow::new(
                    "Loading file...".to_string(),
                    move || match parser::parse_with_supported_parsers(&path) {
                        Ok(graph) => Ok(graph),
                        Err(e) => Err(anyhow::anyhow!("Failed to parse file: {}", e)),
                    },
                ));
            }
            (FileMode::Save, AppState::FileLoaded(session)) => {
                crate::state::save_state(session, path)?;
            }
            (FileMode::Load, _) => {
                *app_state = AppState::FileLoaded(Box::new(crate::state::load_state(path)?));
            }
            _ => {}
        }

        self.mode = FileMode::None;

        Ok(())
    }

    fn render(&mut self, ctx: &Context) {
        self.file_dialog.update(ctx);
    }

    fn open_for_replace(&mut self) {
        self.mode = FileMode::Replace;
        self.file_dialog.pick_file();
    }

    fn open_for_merge(&mut self) {
        self.mode = FileMode::Merge;
        self.file_dialog.pick_file();
    }

    fn open_for_save(&mut self) {
        self.mode = FileMode::Save;
        self.file_dialog.save_file();
    }

    fn open_for_load(&mut self) {
        self.mode = FileMode::Load;
        self.file_dialog.pick_file();
    }
}

/// Handles central panel rendering.
struct CentralPanelRenderer<'a> {
    scene_rect: &'a mut eframe::egui::Rect,
    fps_counter: &'a FpsCounter,
}

impl<'a> CentralPanelRenderer<'a> {
    fn new(scene_rect: &'a mut eframe::egui::Rect, fps_counter: &'a FpsCounter) -> Self {
        Self {
            scene_rect,
            fps_counter,
        }
    }

    fn render(&mut self, ctx: &Context, app_state: &mut AppState) {
        eframe::egui::CentralPanel::default().show(ctx, |ui| {
            if let AppState::FileLoaded(file) = app_state {
                ui.add(GraphWidget::new(
                    crate::graph::graph_widget::ViewState::new(self.scene_rect),
                    &mut file.interaction_state,
                    crate::graph::graph_widget::GraphData::new(
                        &file.graph,
                        &file.visible_nodes,
                    ),
                    &mut file.cache,
                ));

                // Show controls
                ui.with_layout(
                    eframe::egui::Layout::bottom_up(eframe::egui::Align::LEFT),
                    |ui| {
                        ui.label(format!("FPS: {:.0}", self.fps_counter.fps()));
                        ui.label("Click to select and pan to the dependency | Hover to highlight | Drag nodes to move them");
                        ui.label("Ctrl + mouse wheel to zoom | Drag background to pan");
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
            eframe::egui::SidePanel::left("list_panel")
                .max_width(600.0)
                .resizable(true)
                .show(ctx, |ui| {
                    ui.add(DependencyPanel::new(
                        self.package_filter,
                        self.search_options,
                        &file.graph,
                        &mut file.visible_nodes,
                        &mut file.cache,
                        &mut file.interaction_state,
                    ));
                });
            eframe::egui::TopBottomPanel::bottom("dependency_panel")
                .min_height(100.0)
                .max_height(600.0)
                .resizable(true)
                .show(ctx, |ui| {
                    ui.add(DepPanel::new(&file.graph, &mut file.interaction_state))
                });
        }
    }
}

/// Represents the current state of the application.
enum AppState {
    /// No file is currently loaded.
    NoFile,
    // File is being loaded in the background.
    LoadingFile(BackgroundWindow<Result<DependencyGraph, Error>>),
    CalculatingLayout(BackgroundWindow<Box<Session>>),
    /// A file is loaded and ready for visualization.
    FileLoaded(Box<Session>),
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
    scene_rect: eframe::egui::Rect,
    error_text: Option<String>,
    fps_counter: FpsCounter,
    package_filter: String,
    search_options: SearchOptions,
    file_dialog_handler: FileDialogHandler,
    layout_config: LayoutWindow,
    background: Option<BackgroundWindow<u32>>,
}

impl Default for DependencyApp {
    fn default() -> Self {
        Self {
            app_state: AppState::NoFile,
            scene_rect: eframe::egui::Rect::from_min_size(
                eframe::egui::Pos2::ZERO,
                eframe::egui::Vec2::splat(1000.0),
            ),
            error_text: None,
            fps_counter: FpsCounter::new(),
            package_filter: String::new(),
            search_options: SearchOptions::default(),
            file_dialog_handler: FileDialogHandler::new(),
            layout_config: LayoutWindow::default(),
            background: None,
        }
    }
}

impl DependencyApp {
    fn render_central_panel(&mut self, ctx: &Context) {
        let mut renderer = CentralPanelRenderer::new(&mut self.scene_rect, &self.fps_counter);
        renderer.render(ctx, &mut self.app_state);
    }

    fn render_error_window(&mut self, ctx: &Context) {
        let mut renderer = ErrorWindowRenderer::new(&mut self.error_text);
        renderer.render(ctx);
    }

    fn render_packages_view(&mut self, ctx: &Context) {
        puffin::profile_function!();
        let mut renderer =
            PackagesViewRenderer::new(&mut self.package_filter, &mut self.search_options);
        renderer.render(ctx, &mut self.app_state);
    }

    fn render_menu(&mut self, ctx: &Context) {
        eframe::egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            eframe::egui::MenuBar::new().ui(ui, |ui| {
                ui.menu_button("File", |ui| {
                    ui.menu_button("Parse", |ui| {
                        if ui.button("Open file").clicked() {
                            self.file_dialog_handler.open_for_replace();
                        }

                        if matches!(self.app_state, AppState::FileLoaded(_))
                            && ui.button("Merge file").clicked()
                        {
                            self.file_dialog_handler.open_for_merge();
                        }
                    });

                    ui.menu_button("State", |ui| {
                        if ui.button("Load from file").clicked() {
                            self.file_dialog_handler.open_for_load();
                        }
                        if matches!(self.app_state, AppState::FileLoaded(_))
                            && ui.button("Save to file").clicked()
                        {
                            self.file_dialog_handler.open_for_save();
                        }
                    });
                });

                ui.menu_button("Layout", |ui| {
                    if ui.button("Config").clicked() {
                        self.layout_config.request_show();
                    }

                    if let AppState::FileLoaded(session) = &mut self.app_state
                        && ui.button("Recalculate").clicked()
                    {
                        session.recalculate_layout(self.layout_config.get_config());
                    }
                });

                ui.menu_button("Job", |ui| {
                    if ui.button("Do").clicked() {
                        self.background =
                            Some(BackgroundWindow::new("Counting...".to_string(), || {
                                std::thread::sleep(Duration::from_secs(3));
                                2
                            }));
                    }
                });
            });

            if let AppState::FileLoaded(file) = &mut self.app_state {
                ui.horizontal(|ui| {
                    ui.label("Framework:");
                    for fw in file.graph.iter_frameworks() {
                        if ui
                            .selectable_label(
                                file.interaction_state.selected_framework() == Some(fw),
                                fw.name(),
                            )
                            .clicked()
                        {
                            file.interaction_state.publish(
                                crate::session::InteractionEvent::SelectFramework(fw.clone()),
                            );
                        }
                    }
                });
            }
        });
    }
}

impl App for DependencyApp {
    fn update(&mut self, ctx: &Context, _: &mut eframe::Frame) {
        GlobalProfiler::lock().new_frame();

        let previous_state = std::mem::replace(&mut self.app_state, AppState::NoFile);
        self.app_state = match previous_state {
            AppState::LoadingFile(bw) => match bw.update(ctx) {
                PollResult::Pending(bw) => AppState::LoadingFile(bw),
                PollResult::Ready(graph_result) => match graph_result {
                    Ok(graph) => {
                        let config = self.layout_config.get_config();
                        AppState::CalculatingLayout(BackgroundWindow::new(
                            "Calculating layout...".to_string(),
                            move || Box::new(Session::load_from(graph, config)),
                        ))
                    }
                    Err(err) => {
                        self.error_text = Some(err.to_string());
                        AppState::NoFile
                    }
                },
            },
            AppState::CalculatingLayout(bw) => match bw.update(ctx) {
                PollResult::Pending(bw) => AppState::CalculatingLayout(bw),
                PollResult::Ready(session) => AppState::FileLoaded(session),
            },
            other => other,
        };

        // Apply interaction events published in the previous frame.
        // Also, reset the per_frame state.
        if let AppState::FileLoaded(file) = &mut self.app_state {
            file.interaction_state
                .process_pending(&mut file.visible_nodes);
        }

        self.fps_counter.update();
        self.render_menu(ctx);
        self.file_dialog_handler.render(ctx);
        if let Err(error) = self.file_dialog_handler.handle(&mut self.app_state) {
            self.error_text = Some(error.to_string());
        }

        self.background = match self.background.take() {
            None => None,
            Some(w) => match w.update(ctx) {
                crate::background::PollResult::Ready(v) => {
                    dbg!(v);
                    None
                }
                crate::background::PollResult::Pending(v) => Some(v),
            },
        };

        self.render_error_window(ctx);

        // Render left side first not to overlay over central panel. It MUST be kept in this order.
        self.render_packages_view(ctx);
        self.render_central_panel(ctx);

        self.layout_config.update(ctx);
    }
}
