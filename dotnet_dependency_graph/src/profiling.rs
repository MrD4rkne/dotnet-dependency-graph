#[cfg(feature = "profiling")]
macro_rules! profile_scope {
    ($name:expr) => {
        ::puffin::profile_scope!($name);
    };
}

#[cfg(not(feature = "profiling"))]
macro_rules! profile_scope {
    ($name:expr) => {};
}

#[cfg(feature = "profiling")]
macro_rules! profile_frame {
    () => {
        ::puffin::GlobalProfiler::lock().new_frame();
    };
}

#[cfg(not(feature = "profiling"))]
macro_rules! profile_frame {
    () => {};
}

#[cfg(feature = "profiling")]
fn start_puffin_server() -> ::anyhow::Result<::puffin_http::Server> {
    ::puffin::set_scopes_on(true); // tell puffin to collect data

    let addr = "127.0.0.1:8585";
    match ::puffin_http::Server::new(addr) {
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

// Helper that returns an opaque boxed server when the `profiling`
// feature is enabled, or None when it's not. Using type erasure here avoids
// referencing `puffin_http` in code that must compile without the feature.
#[cfg(feature = "profiling")]
pub(crate) fn maybe_start_puffin_server() -> Option<Box<dyn ::std::any::Any + Send>> {
    match start_puffin_server() {
        Ok(s) => Some(Box::new(s)),
        Err(e) => {
            eprintln!("Failed to start puffin server: {}", e);
            None
        }
    }
}

#[cfg(not(feature = "profiling"))]
pub(crate) fn maybe_start_puffin_server() -> Option<Box<dyn ::std::any::Any + Send>> {
    None
}
