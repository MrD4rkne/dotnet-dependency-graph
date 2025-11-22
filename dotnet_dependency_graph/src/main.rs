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
}

impl DependencyApp {
    fn new() -> Self {
        Self {
            file_dialog: FileDialog::new(),
            current_dgspec_file: None,
            graph: None,
            pos: None,
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

            let graph = parse::load_dgspec_from_file(path.to_path_buf()).expect("e");
            dbg!(&graph);

            self.pos = Some(calculate_layout(&graph));
            self.graph = Some(graph);
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            let (response, mut painter) =
                ui.allocate_painter(ui.available_size(), Sense::click_and_drag());
            if let Some(graph) = &self.graph
                && let Some(layouts) = &self.pos
            {
                draw_graph(&graph, &layouts, ui, &mut painter);
            }
        });
    }
}

fn draw_graph(
    graph: &DependencyGraph,
    layouts: &Vec<(HashMap<DependencyId, (f64, f64)>, f64, f64)>,
    ui: &mut Ui,
    painter: &mut Painter,
) {
    for (layout, zoom, size) in layouts {
        for (id, (x, y)) in layout {
            // TODO: handle cast?
            let pos = Pos2::new(*x as f32, *y as f32);
            let text = get_displayed_text(
                graph
                    .get(&id)
                    .expect("Dep from layout should be in the graph"),
            );
            let rect = visualize::draw_node(ui, text, pos, &painter, *zoom as f32);
        }
    }
}

fn get_displayed_text(dep: &DependencyInfo) -> &str {
    match dep {
        DependencyInfo::Project(proj) => &proj.path,
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
