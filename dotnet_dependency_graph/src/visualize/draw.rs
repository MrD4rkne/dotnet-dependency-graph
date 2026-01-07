use dotnet_dependency_parser::graph::{DependencyId, DependencyInfo, Layout};
use eframe::egui::TextFormat;
use eframe::egui::text::LayoutJob;
use eframe::egui::{FontId, Painter, Pos2, Rect, Stroke, Vec2};
use std::collections::HashMap;

mod constants {
    use eframe::egui::Color32;

    // Node dimensions
    pub(crate) const NODE_HEIGHT: f32 = 60.0;
    pub(crate) const NODE_PADDING: f32 = 16.0;
    pub(crate) const NODE_CORNER_RADIUS: f32 = 4.0;
    pub(crate) const NODE_BORDER_WIDTH: f32 = 2.5;
    pub(crate) const HIGHLIGHTED_NODE_BORDER_WIDTH: f32 = NODE_BORDER_WIDTH + 1.5;

    // Edge dimensions
    pub(crate) const EDGE_STROKE_WIDTH: f32 = 3.0;
    pub(crate) const HIGHLIGHTED_EDGE_STROKE_WIDTH: f32 = EDGE_STROKE_WIDTH + 1.0;
    pub(crate) const ARROW_SIZE: f32 = 14.0;
    pub(crate) const HIGHLIGHTED_ARROW_SIZE: f32 = ARROW_SIZE + 3.0;
    pub(crate) const ARROW_HEAD_WIDTH_FACTOR: f32 = 0.5;

    // Text sizing constants
    pub(crate) const CHAR_WIDTH: f64 = 8.0;
    pub(crate) const TEXT_PADDING: f64 = 32.0;
    pub(crate) const MIN_WIDTH: f64 = 120.0;
    pub(crate) const MAX_WIDTH: f64 = 300.0;
    pub(crate) const FONT_SIZE: f32 = 16.0;
    pub(crate) const LINE_HEIGHT: f32 = 20.0;

    // Colors
    pub(crate) const NODE_BACKGROUND_COLOR: Color32 = Color32::from_rgb(70, 130, 180);
    pub(crate) const NODE_BORDER_COLOR: Color32 = Color32::from_rgb(30, 60, 100);
    pub(crate) const TEXT_COLOR: Color32 = Color32::WHITE;
    pub(crate) const EDGE_COLOR: Color32 = Color32::from_rgb(100, 100, 100);
    pub(crate) const HIGHLIGHTED_NODE_BACKGROUND: Color32 = Color32::from_rgb(255, 165, 0);
    pub(crate) const HIGHLIGHTED_NODE_BORDER_COLOR: Color32 = Color32::from_rgb(200, 100, 0);
    pub(crate) const HIGHLIGHTED_EDGE_COLOR: Color32 = Color32::from_rgb(220, 120, 20);

    // Layout constants
    pub(crate) const LAYOUT_SPACING: f64 = 100.0;
}

pub(crate) fn calculate_dimensions_from_text(text: &str) -> (f32, f32) {
    let (line_count, max_line_length) = get_lines_count_with_max_length(text);

    let text_width = (max_line_length as f64) * constants::CHAR_WIDTH + constants::TEXT_PADDING;
    let width = text_width.clamp(constants::MIN_WIDTH, constants::MAX_WIDTH) as f32;

    let height = (constants::LINE_HEIGHT * line_count as f32 + constants::NODE_PADDING * 2.0)
        .max(constants::NODE_HEIGHT);

    (width, height)
}

fn get_lines_count_with_max_length(text: &str) -> (usize, usize) {
    text.lines()
        .fold((0usize, 0usize), |(count, max_len), line| {
            (count + 1, max_len.max(line.len()))
        })
}

pub(crate) fn calculate_size(dep: &DependencyInfo) -> (f32, f32) {
    calculate_dimensions_from_text(dep.name())
}

pub(crate) fn draw_node(
    text: &str,
    painter: &eframe::egui::Painter,
    rect: Rect,
    highlighted: bool,
) {
    profile_function!();
    let max_text_width = rect.width() - constants::NODE_PADDING;
    let (bg_color, border_color, border_width) = if highlighted {
        (
            constants::HIGHLIGHTED_NODE_BACKGROUND,
            constants::HIGHLIGHTED_NODE_BORDER_COLOR,
            constants::HIGHLIGHTED_NODE_BORDER_WIDTH,
        )
    } else {
        (
            constants::NODE_BACKGROUND_COLOR,
            constants::NODE_BORDER_COLOR,
            constants::NODE_BORDER_WIDTH,
        )
    };

    {
        profile_scope!("paint");
        // Draw rectangle background
        painter.rect_filled(rect, constants::NODE_CORNER_RADIUS, bg_color);

        // Draw rectangle border
        painter.rect_stroke(
            rect,
            constants::NODE_CORNER_RADIUS,
            Stroke::new(border_width, border_color),
            eframe::egui::epaint::StrokeKind::Middle,
        );
    }

    let label_job = create_label(
        text,
        constants::FONT_SIZE,
        rect.height(),
        constants::NODE_PADDING,
        max_text_width,
    );
    {
        profile_scope!("paint_text");
        let galley = painter.layout_job(label_job);

        // Center the text in the node
        let text_pos = Pos2::new(
            rect.center().x - galley.size().x / 2.0,
            rect.center().y - galley.size().y / 2.0,
        );

        {
            profile_scope!("galley");
            painter.galley(text_pos, galley, constants::TEXT_COLOR);
        }
    }
}

fn create_label(
    text: &str,
    font_size: f32,
    height: f32,
    padding: f32,
    max_text_width: f32,
) -> LayoutJob {
    profile_function!();
    let font_id = FontId::proportional(font_size);
    let max_text_height = height - padding;

    let mut job = LayoutJob::default();
    job.append(
        text,
        0.0,
        TextFormat::simple(font_id, constants::TEXT_COLOR),
    );
    job.wrap.max_rows = ((max_text_height / font_size).floor() as usize).max(1);
    job.wrap.max_width = max_text_width;

    job
}

pub(crate) fn join_layouts(
    layouts: impl Iterator<Item = Layout<DependencyId>>,
) -> HashMap<DependencyId, (f32, f32)> {
    profile_function!();
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
pub(crate) fn draw_edge(painter: &Painter, src_rect: Rect, dst_rect: Rect, highlighted: bool) {
    profile_function!();
    let src_center = src_rect.center();
    let dst_center = dst_rect.center();

    // Calculate direction
    let dir = (dst_center - src_center).normalized();

    // Find edge points on rectangles
    let src_edge = rect_edge_point(src_rect, dir);
    let dst_edge = rect_edge_point(dst_rect, -dir);

    let (edge_color, arrow_size, stroke_width) = if highlighted {
        (
            constants::HIGHLIGHTED_EDGE_COLOR,
            constants::HIGHLIGHTED_ARROW_SIZE,
            constants::HIGHLIGHTED_EDGE_STROKE_WIDTH,
        )
    } else {
        (
            constants::EDGE_COLOR,
            constants::ARROW_SIZE,
            constants::EDGE_STROKE_WIDTH,
        )
    };

    // Draw line from edge to edge
    painter.line_segment([src_edge, dst_edge], Stroke::new(stroke_width, edge_color));

    // Draw arrow head at destination edge
    let perp = Vec2::new(-dir.y, dir.x);

    // Two sides of the arrow
    painter.line_segment(
        [
            dst_edge,
            dst_edge - dir * arrow_size + perp * arrow_size * constants::ARROW_HEAD_WIDTH_FACTOR,
        ],
        Stroke::new(stroke_width, edge_color),
    );
    painter.line_segment(
        [
            dst_edge,
            dst_edge - dir * arrow_size - perp * arrow_size * constants::ARROW_HEAD_WIDTH_FACTOR,
        ],
        Stroke::new(stroke_width, edge_color),
    );
}
