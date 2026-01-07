use eframe::run_native;

#[macro_use]
mod profiling;
mod app;
mod core;
mod ui;
mod visualize;

use crate::app::DependencyApp;

fn main() -> std::result::Result<(), eframe::Error> {
    let server = profiling::maybe_start_puffin_server();

    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 1024.0])
            .with_drag_and_drop(true),

        ..Default::default()
    };

    let result = run_native(
        "Dotnet Dependency Viewer",
        options,
        Box::new(move |_| Ok(Box::new(DependencyApp::default()))),
    );

    // Keep `server` alive for the duration of the app if present.
    let _ = server;

    result
}
