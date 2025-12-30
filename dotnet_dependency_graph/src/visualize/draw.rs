use crate::graph::CachedNodeData;
use dotnet_dependency_parser::graph::{DependencyId, DependencyInfo, Layout};
use eframe::egui::TextFormat;
use eframe::egui::text::LayoutJob;
use eframe::egui::{Color32, FontId, Painter, Pos2, Rect, Stroke, Vec2};
use std::collections::HashMap;

use super::Zoomed;
use super::constants;

// Edge drawing constants
const EDGE_STROKE_WIDTH: f32 = 2.0;
const EDGE_COLOR: Color32 = Color32::from_rgb(100, 100, 100);
const ARROW_SIZE: f32 = 10.0;
const ARROW_HEAD_WIDTH_FACTOR: f32 = 0.5;
const LINE_HEIGHT: f32 = 20.0;

pub(crate) fn calculate_dimensions_from_text(text: &str) -> (f32, f32) {
    let (line_count, max_line_length) = get_lines_count_with_max_length(text);

    let text_width = (max_line_length as f64) * constants::CHAR_WIDTH + constants::TEXT_PADDING;
    let width = text_width.clamp(constants::MIN_WIDTH, constants::MAX_WIDTH) as f32;

    let height = (LINE_HEIGHT * line_count as f32 + constants::NODE_PADDING * 2.0)
        .max(constants::NODE_HEIGHT);

    (width, height)
}

pub(crate) struct State {
    zoom: f32,
    pan_offset: Vec2,
}

impl State {
    pub(crate) fn new(zoom: f32, pan_offset: Vec2) -> Self {
        Self { zoom, pan_offset }
    }

    pub(crate) fn transform(&self, pos: (f32, f32)) -> Pos2 {
        transform_position(pos, self.zoom, self.pan_offset)
    }

    pub(crate) fn transform_pos(&self, pos: Pos2) -> Pos2 {
        self.transform((pos.x, pos.y))
    }

    pub(crate) fn zoom(&self) -> f32 {
        self.zoom
    }
}

fn transform_position(pos: (f32, f32), zoom: f32, pan_offset: Vec2) -> Pos2 {
    let pos_vec = Pos2::new(pos.0, pos.1);
    (pos_vec.to_vec2() * zoom + pan_offset).to_pos2()
}

fn get_lines_count_with_max_length(text: &str) -> (usize, usize) {
    text.lines()
        .fold((0usize, 0usize), |(count, max_len), line| {
            (count + 1, max_len.max(line.len()))
        })
}

pub(crate) fn calculate_size(_id: &DependencyId, dep: &DependencyInfo) -> (f64, f64) {
    let (width, height) = calculate_dimensions_from_text(dep.name());
    (width as f64, height as f64)
}

pub(crate) fn draw_node(
    _ui: &mut eframe::egui::Ui,
    text: &str,
    painter: &eframe::egui::Painter,
    cache: &mut CachedNodeData,
    state: &State,
) {
    let width = Zoomed::new(cache.unzoomed_width, state.zoom());
    let height = Zoomed::new(cache.unzoomed_height, state.zoom());
    let padding = Zoomed::new(constants::NODE_PADDING, state.zoom());
    let corner_radius = Zoomed::new(constants::NODE_CORNER_RADIUS, state.zoom());
    let border_width = Zoomed::new(constants::NODE_BORDER_WIDTH, state.zoom());
    let font_size = Zoomed::new(constants::FONT_SIZE, state.zoom());
    let max_text_width = width - padding;

    let position = state.transform_pos(cache.initial_position);
    let rect = Rect::from_center_size(position, Vec2::new(width.into_value(), height.into_value()));

    // Draw rectangle background
    painter.rect_filled(
        rect,
        corner_radius.into_value(),
        constants::NODE_BACKGROUND_COLOR,
    );

    // Draw rectangle border
    painter.rect_stroke(
        rect,
        corner_radius.into_value(),
        Stroke::new(border_width.into_value(), constants::NODE_BORDER_COLOR),
        eframe::egui::epaint::StrokeKind::Middle,
    );

    let label_job = create_label(text, font_size, height, padding, max_text_width);
    let galley = painter.layout_job(label_job);

    // Center the text in the node
    let text_pos = Pos2::new(
        rect.center().x - galley.size().x / 2.0,
        rect.center().y - galley.size().y / 2.0,
    );

    painter.galley(text_pos, galley, constants::TEXT_COLOR);

    cache.rect = rect;
}

fn create_label(
    text: &str,
    font_size: Zoomed<f32>,
    height: Zoomed<f32>,
    padding: Zoomed<f32>,
    max_text_width: Zoomed<f32>,
) -> LayoutJob {
    let font_id = FontId::proportional(font_size.into_value());
    let max_text_height = height - padding;

    let mut job = LayoutJob::default();
    job.append(
        text,
        0.0,
        TextFormat::simple(font_id, constants::TEXT_COLOR),
    );
    job.wrap.max_rows = ((max_text_height / font_size).into_value().floor() as usize).max(1);
    job.wrap.max_width = max_text_width.into_value();

    job
}

pub(crate) fn join_layouts(
    layouts: Vec<Layout<DependencyId>>,
) -> HashMap<DependencyId, (f32, f32)> {
    let mut result = HashMap::new();
    let mut offset_x = 0.0;
    for layout in layouts {
        let mut max_x: f64 = 0.0;
        for (id, (x, y)) in layout.positions {
            let new_x = x + offset_x;
            // Clamp here in one pass
            let x_clamped = new_x.clamp(f32::MIN as f64, f32::MAX as f64) as f32;
            let y_clamped = y.clamp(f32::MIN as f64, f32::MAX as f64) as f32;
            result.insert(id, (x_clamped, y_clamped));
            max_x = max_x.max(new_x);
        }
        offset_x = max_x + constants::LAYOUT_SPACING;
    }
    result
}

/// Calculate the intersection point of a line from rect center in given direction with rect boundary
fn rect_edge_point(rect: Rect, direction: Vec2) -> Pos2 {
    let center = rect.center();
    let half_size = rect.size() * 0.5;

    // Calculate intersection with each edge
    let t_x = if direction.x.abs() > 1e-6 {
        half_size.x / direction.x.abs()
    } else {
        f32::INFINITY
    };

    let t_y = if direction.y.abs() > 1e-6 {
        half_size.y / direction.y.abs()
    } else {
        f32::INFINITY
    };

    let t = t_x.min(t_y);
    if t.is_infinite() {
        center
    } else {
        center + direction * t
    }
}

/// Draw a single edge with arrow from source to destination
pub(crate) fn draw_edge(painter: &Painter, src_rect: Rect, dst_rect: Rect, zoom: f32) {
    let src_center = src_rect.center();
    let dst_center = dst_rect.center();

    // Calculate direction
    let dir = (dst_center - src_center).normalized();

    // Find edge points on rectangles
    let src_edge = rect_edge_point(src_rect, dir);
    let dst_edge = rect_edge_point(dst_rect, -dir);

    // Draw line from edge to edge
    painter.line_segment(
        [src_edge, dst_edge],
        Stroke::new(EDGE_STROKE_WIDTH, EDGE_COLOR),
    );

    // Draw arrow head at destination edge
    let perp = Vec2::new(-dir.y, dir.x);
    let arrow_size = Zoomed::new(ARROW_SIZE, zoom).into_value();

    // Two sides of the arrow
    painter.line_segment(
        [
            dst_edge,
            dst_edge - dir * arrow_size + perp * arrow_size * ARROW_HEAD_WIDTH_FACTOR,
        ],
        Stroke::new(EDGE_STROKE_WIDTH, EDGE_COLOR),
    );
    painter.line_segment(
        [
            dst_edge,
            dst_edge - dir * arrow_size - perp * arrow_size * ARROW_HEAD_WIDTH_FACTOR,
        ],
        Stroke::new(EDGE_STROKE_WIDTH, EDGE_COLOR),
    );
}
