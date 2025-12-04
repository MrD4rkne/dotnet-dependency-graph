use eframe::run_native;

mod app;
mod file;
mod graph_widget;
mod loader;
mod node;
mod parser;
mod visualize;

use crate::app::DependencyApp;

fn main() -> Result<(), eframe::Error> {
    let native_options = eframe::NativeOptions::default();
    run_native(
        "Dotnet Dependency Viewer",
        native_options,
        Box::new(move |_| Ok(Box::new(DependencyApp::new()))),
    )
}
