#[cfg(feature = "puffin-server")]
use anyhow::Result as AnyhowResult;
use eframe::run_native;
use std::any::Any;

mod app;
mod dependency_panel;
mod graph;
mod node;
mod parser;
mod session;
mod state;
mod visualize;

use crate::app::DependencyApp;

fn main() -> std::result::Result<(), eframe::Error> {
    let server = maybe_start_puffin_server();

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

#[cfg(feature = "puffin-server")]
fn start_puffin_server() -> AnyhowResult<puffin_http::Server> {
    puffin::set_scopes_on(true); // tell puffin to collect data

    let addr = "127.0.0.1:8585";
    match puffin_http::Server::new(addr) {
        Ok(puffin_server) => {
            println!(
                "Run:  cargo install puffin_viewer && puffin_viewer --url {}",
                addr
            );
            Ok(puffin_server)
        }
        Err(err) => Err(err),
    }
}

// Helper that returns an opaque boxed server when the `puffin-server`
// feature is enabled, or None when it's not. Using type erasure here avoids
// referencing `puffin_http` in code that must compile without the feature.
#[cfg(feature = "puffin-server")]
fn maybe_start_puffin_server() -> Option<Box<dyn Any + Send>> {
    match start_puffin_server() {
        Ok(s) => Some(Box::new(s)),
        Err(e) => {
            eprintln!("Failed to start puffin server: {}", e);
            None
        }
    }
}

#[cfg(not(feature = "puffin-server"))]
fn maybe_start_puffin_server() -> Option<Box<dyn Any + Send>> {
    None
}
