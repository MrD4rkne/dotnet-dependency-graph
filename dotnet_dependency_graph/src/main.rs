use eframe::run_native;

mod app;
mod dependency_panel;
mod graph;
mod node;
mod parser;
mod session;
mod visualize;

use crate::app::DependencyApp;

fn main() -> Result<(), eframe::Error> {
    let native_options = eframe::NativeOptions::default();

    #[cfg(feature = "puffin-server")]
    start_puffin_server();

    run_native(
        "Dotnet Dependency Viewer",
        native_options,
        Box::new(move |_| Ok(Box::new(DependencyApp::default()))),
    )
}

#[cfg(feature = "puffin-server")]
fn start_puffin_server() {
    puffin::set_scopes_on(true); // tell puffin to collect data

    match puffin_http::Server::new("127.0.0.1:8585") {
        Ok(puffin_server) => {
            println!("Run:  cargo install puffin_viewer && puffin_viewer --url 127.0.0.1:8585");

            std::process::Command::new("puffin_viewer")
                .arg("--url")
                .arg("127.0.0.1:8585")
                .spawn()
                .ok();

            // We can store the server if we want, but in this case we just want
            // it to keep running. Dropping it closes the server, so let's not drop it!
            #[expect(clippy::mem_forget)]
            std::mem::forget(puffin_server);
        }
        Err(err) => {
            println!("Failed to start puffin server: {err}");
        }
    }
}
