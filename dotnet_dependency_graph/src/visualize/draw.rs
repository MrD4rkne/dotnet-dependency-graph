use egui::text::LayoutJob;
use egui::{Color32, FontId, Painter, Pos2, Rect, Stroke, Vec2};
use nuget_dgspec_parser::graph::{DependencyId, DependencyInfo, Layout};
use std::collections::HashMap;

use super::Zoomed;
use super::constants;

// Edge drawing constants
const EDGE_STROKE_WIDTH: f32 = 2.0;
const EDGE_COLOR: Color32 = Color32::from_rgb(100, 100, 100);
const ARROW_SIZE: f32 = 10.0;
const ARROW_HEAD_WIDTH_FACTOR: f32 = 0.5;

pub fn calculate_dimensions_from_text(text: &str) -> (f32, f32) {
    let lines: Vec<&str> = text.lines().collect();
    let max_line_length = lines.iter().map(|line| line.len()).max().unwrap_or(0);

    let text_width = (max_line_length as f64) * constants::CHAR_WIDTH + constants::TEXT_PADDING;
    let width = text_width.clamp(constants::MIN_WIDTH, constants::MAX_WIDTH) as f32;

    let line_count = lines.len().max(1);
    // Use a reasonable line height (e.g., 20px per line) plus padding
    let line_height = 20.0;
    let height = (line_height * line_count as f32 + constants::NODE_PADDING * 2.0)
        .max(constants::NODE_HEIGHT);

    (width, height)
}

pub fn calculate_size(
    _id: &DependencyId,
    dep: &DependencyInfo,
    get_node_text: impl FnOnce(&DependencyInfo) -> String,
) -> (f64, f64) {
    let text = get_node_text(dep);
    let (width, height) = calculate_dimensions_from_text(&text);
    (width as f64, height as f64)
}

pub fn draw_node(
    _ui: &mut egui::Ui,
    text: &str,
    position: Pos2,
    painter: &egui::Painter,
    zoom: f32,
) -> Rect {
    let (unzoomed_width, unzoomed_height) = calculate_dimensions_from_text(text);
    let width = Zoomed::new(unzoomed_width, zoom);
    let height = Zoomed::new(unzoomed_height, zoom);
    let padding = Zoomed::new(constants::NODE_PADDING, zoom);
    let corner_radius = Zoomed::new(constants::NODE_CORNER_RADIUS, zoom);
    let border_width = Zoomed::new(constants::NODE_BORDER_WIDTH, zoom);
    let font_size = Zoomed::new(constants::FONT_SIZE, zoom);
    let max_text_width = width - padding;

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
        egui::epaint::StrokeKind::Middle,
    );

    let label_job = create_label(text.to_string(), font_size, height, padding, max_text_width);
    let galley = painter.layout_job(label_job);

    // Center the text in the node
    let text_pos = Pos2::new(
        rect.center().x - galley.size().x / 2.0,
        rect.center().y - galley.size().y / 2.0,
    );

    painter.galley(text_pos, galley, constants::TEXT_COLOR);

    rect
}

fn create_label(
    text: String,
    font_size: Zoomed<f32>,
    height: Zoomed<f32>,
    padding: Zoomed<f32>,
    max_text_width: Zoomed<f32>,
) -> LayoutJob {
    let font = FontId::proportional(font_size.into_value());

    // Calculate available space for text (with padding)
    let max_text_height = height - padding;

    // Use TextWrapMode::Truncate to handle both width and height truncation
    let mut job = LayoutJob::simple(
        text,
        font,
        constants::TEXT_COLOR,
        max_text_width.into_value(),
    );
    job.wrap.max_rows = ((max_text_height / font_size).floor() as usize).max(1);

    job
}

pub fn join_layouts(layouts: Vec<Layout<DependencyId>>) -> HashMap<DependencyId, (f32, f32)> {
    let mut result = HashMap::new();
    let mut offset_x = 0.0;
    for layout in layouts {
        let mut max_x: f64 = 0.0;
        for (id, (x, y)) in layout.positions {
            let new_x = x + offset_x;
            result.insert(id, (new_x, y));
            max_x = max_x.max(new_x);
        }
        offset_x = max_x + constants::LAYOUT_SPACING;
    }

    // In this context, layout coordinates are expected to be within reasonable bounds for UI rendering.
    result
        .into_iter()
        .map(|(key, (x, y))| {
            let x_clamped = x.clamp(f32::MIN as f64, f32::MAX as f64) as f32;
            let y_clamped = y.clamp(f32::MIN as f64, f32::MAX as f64) as f32;
            (key, (x_clamped, y_clamped))
        })
        .collect()
}

/// Calculate node rect for a given position and text
pub fn calculate_node_rect(text: &str, position: Pos2, zoom: f32) -> Rect {
    let (unzoomed_width, unzoomed_height) = calculate_dimensions_from_text(text);
    let width = Zoomed::new(unzoomed_width, zoom);
    let height = Zoomed::new(unzoomed_height, zoom);
    Rect::from_center_size(position, Vec2::new(width.into_value(), height.into_value()))
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
    center + direction * t
}

/// Draw a single edge with arrow from source to destination
pub fn draw_edge(painter: &Painter, src_rect: Rect, dst_rect: Rect, zoom: f32) {
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
