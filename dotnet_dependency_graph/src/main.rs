use eframe::run_native;

mod app;
mod dependency_panel;
mod graph_widget;
mod node;
mod parser;
mod session;
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
