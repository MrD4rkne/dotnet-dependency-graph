use eframe::egui::{Pos2, Rect, Vec2};

pub(crate) struct CachedNodeData {
    pub(crate) initial_position: Pos2,
    pub(crate) unzoomed_width: f32,
    pub(crate) unzoomed_height: f32,
    pub(crate) rect: Rect,
}

impl CachedNodeData {
    pub(crate) fn new(screen_pos: Pos2, unzoomed_width: f32, unzoomed_height: f32) -> Self {
        Self {
            initial_position: screen_pos,
            unzoomed_width,
            unzoomed_height,
            rect: Rect::from_center_size(Pos2::ZERO, Vec2::new(unzoomed_width, unzoomed_height)),
        }
    }
}
