use crate::visualize::Zoomed;
use crate::visualize::constants;

use eframe::egui::Pos2;
use eframe::egui::Rect;
use eframe::egui::Vec2;

pub(crate) struct GraphCache {
    padding: Zoomed<f32>,
    corner_radius: Zoomed<f32>,
    border_width: Zoomed<f32>,
    font_size: Zoomed<f32>,
}

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

impl GraphCache {
    pub(crate) fn new(zoom: f32) -> Self {
        GraphCache {
            padding: Zoomed::new(constants::NODE_PADDING, zoom),
            corner_radius: Zoomed::new(constants::NODE_CORNER_RADIUS, zoom),
            border_width: Zoomed::new(constants::NODE_BORDER_WIDTH, zoom),
            font_size: Zoomed::new(constants::FONT_SIZE, zoom),
        }
    }
}
