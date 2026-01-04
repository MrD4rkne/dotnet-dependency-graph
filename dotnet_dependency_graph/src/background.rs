use std::time::Duration;

use eframe::egui::{Context, Window};
use poll_promise::Promise;

pub(crate) struct BackgroundWindow<T>
where
    T: Send + 'static,
{
    promise: Promise<T>,
    title: String,
}

pub(crate) enum PollResult<T>
where
    T: Send + 'static,
{
    Pending(BackgroundWindow<T>),
    Ready(T),
}

impl<T> BackgroundWindow<T>
where
    T: Send + 'static,
{
    pub(crate) fn new<F>(title: String, f: F) -> Self
    where
        F: FnOnce() -> T + Send + 'static,
        T: Send + 'static,
    {
        let promise = poll_promise::Promise::spawn_thread("slow_operation", f);
        Self { promise, title }
    }

    pub(crate) fn update(self, ctx: &Context) -> PollResult<T> {
        match self.promise.try_take() {
            Ok(result) => PollResult::Ready(result),
            Err(promise) => {
                Window::new("Background work")
                    .resizable(false)
                    .show(ctx, |ui| {
                        ui.vertical_centered_justified(|ui| ui.label(&self.title))
                    });
                ctx.request_repaint_after(Duration::from_millis(50));
                PollResult::Pending(BackgroundWindow {
                    promise,
                    title: self.title,
                })
            }
        }
    }
}
