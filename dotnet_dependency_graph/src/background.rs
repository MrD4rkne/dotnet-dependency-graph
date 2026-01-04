use std::pin::Pin;
use std::task::Poll;

use eframe::egui::{Context, ProgressBar, Window};
use eventuals::EventualWriter;

struct Task {
    progress: eventuals::EventualReader<Progress>,
    last_progress: Option<Progress>,
}

#[derive(Clone, PartialEq, Eq)]
pub(crate) enum Progress {
    Percent(u8, Option<String>),
    Done,
}

#[derive(Default)]
pub(crate) struct BackgroundWindow {
    task: Option<Task>,
}

pub(crate) struct ProgressReporter {
    ctx: Context,
    writer: EventualWriter<Progress>,
}

impl ProgressReporter {
    fn new(ctx: Context, writer: EventualWriter<Progress>) -> Self {
        Self { ctx, writer }
    }

    pub(crate) fn publish(&mut self, progress: Progress) {
        self.writer.write(progress);
        self.ctx.request_repaint();
    }
}

impl BackgroundWindow {
    pub(crate) fn update(&mut self, ctx: &Context) {
        self.poll();

        if let Some(task) = &mut self.task {
            Window::new("Background work")
                .resizable(false)
                .show(ctx, |ui| {
                    ui.centered_and_justified(|ui| match &task.last_progress {
                        Some(Progress::Percent(p, msg)) => {
                            if let Some(msg) = msg {
                                ui.label(msg);
                            }

                            let frac = (*p as f32) / 100.0;
                            ui.add(ProgressBar::new(frac).show_percentage());
                        }
                        Some(Progress::Done) => {
                            ui.label("Done");
                        }
                        None => {
                            ui.label("Working...");
                        }
                    })
                });
        }
    }

    fn poll(&mut self) {
        self.task = match self.task.take() {
            None => None,
            Some(mut task) => {
                let mut next = task.progress.next();
                let waker = futures::task::noop_waker_ref();
                let mut cx = std::task::Context::from_waker(waker);
                match Pin::new(&mut next).poll(&mut cx) {
                    Poll::Ready(Ok(p)) => {
                        task.last_progress = Some(p);
                        Some(task)
                    }
                    Poll::Pending => Some(task),
                    Poll::Ready(Err(_)) => None,
                }
            }
        }
    }

    pub(crate) fn schedule_task<F, T>(&mut self, f: F, ctx: Context)
    where
        F: FnOnce(&mut ProgressReporter) -> T + Send + 'static,
        T: Send + 'static,
    {
        let (writer, eventual) = eventuals::Eventual::new();
        self.task = Some(Task {
            progress: eventual.subscribe(),
            last_progress: None,
        });

        std::thread::spawn(move || {
            let mut reporter = ProgressReporter::new(ctx, writer);
            f(&mut reporter);
        });
    }
}
