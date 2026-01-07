use std::time::{Duration, Instant};

pub(crate) struct FpsCounter {
    last_update: Instant,
    frames_since_last: u32,
    current_fps: f32,
}

impl Default for FpsCounter {
    fn default() -> Self {
        Self {
            last_update: Instant::now(),
            frames_since_last: 0,
            current_fps: 0.0,
        }
    }
}

impl FpsCounter {
    pub(crate) fn update(&mut self) {
        self.frames_since_last += 1;
        if self.last_update.elapsed() >= Duration::from_secs(1) {
            self.current_fps = self.frames_since_last as f32;
            self.frames_since_last = 0;
            self.last_update = Instant::now();
        }
    }

    pub(crate) fn fps(&self) -> f32 {
        self.current_fps
    }
}
