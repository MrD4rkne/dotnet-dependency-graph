use eframe::{App, run_native};
use egui::Context;
use egui_file_dialog::FileDialog;
use nuget_dgspec_parser::graph::DependencyGraph;
use std::path::PathBuf;
mod graph;

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
}

impl DependencyApp {
    fn new() -> Self {
        Self {
            file_dialog: FileDialog::new(),
            current_dgspec_file: None,
            graph: None,
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

        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(file) = &self.current_dgspec_file {
                ui.label(format!("Picked file: {:?}", &file.path));
            } else {
                ui.label("Choose the file.");
            }
        });

        self.file_dialog.update(ctx);
        if let Some(path) = self.file_dialog.take_picked() {
            self.current_dgspec_file = Some(File::new(path.to_path_buf()));

            self.graph = graph::load_dgspec_from_file(path.to_path_buf()).ok();
            dbg!(&self.graph);
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
