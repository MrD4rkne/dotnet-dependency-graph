use egui::{FontId, Pos2, Rect, Stroke, Vec2};
use nuget_dgspec_parser::graph::{DependencyId, DependencyInfo, Layout};
use std::collections::HashMap;

use super::Zoomed;
use super::constants;

pub fn calculate_size(
    _id: &DependencyId,
    dep: &DependencyInfo,
    get_node_text: impl FnOnce(&DependencyInfo) -> String,
) -> (f64, f64) {
    let text = get_node_text(dep);

    // Handle multi-line text
    let lines: Vec<&str> = text.lines().collect();
    let max_line_length = lines.iter().map(|line| line.len()).max().unwrap_or(0);

    // Calculate width based on longest line
    let text_width = (max_line_length as f64) * constants::CHAR_WIDTH + constants::TEXT_PADDING;
    let width = text_width.clamp(constants::MIN_WIDTH, constants::MAX_WIDTH);

    // Calculate height based on number of lines
    let line_count = lines.len().max(1);
    // Height scales linearly with the number of lines
    let height = constants::NODE_HEIGHT as f64 * line_count as f64;

    (width, height)
}

pub fn draw_node(
    _ui: &mut egui::Ui,
    text: &str,
    position: Pos2,
    painter: &egui::Painter,
    zoom: f32,
) -> (Rect, bool) {
    // Create zoomed values for all properties
    let width = Zoomed::new(constants::NODE_WIDTH, zoom);
    let height = Zoomed::new(constants::NODE_HEIGHT, zoom);
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

    // Handle multi-line text by splitting on newlines
    let font = FontId::proportional(font_size.into_value());
    let lines: Vec<&str> = text.lines().collect();
    let mut text_truncated = false;

    // Process each line
    let mut display_lines = Vec::new();
    for line in &lines {
        let full_galley =
            painter.layout_no_wrap(line.to_string(), font.clone(), constants::TEXT_COLOR);

        // Check if line needs truncation
        if full_galley.size().x > max_text_width.into_value() {
            // Truncate and add ellipsis
            let mut truncated = line.to_string();
            while !truncated.is_empty() {
                let test_text = format!("{}...", truncated);
                let test_galley =
                    painter.layout_no_wrap(test_text.clone(), font.clone(), constants::TEXT_COLOR);
                if test_galley.size().x <= max_text_width.into_value() {
                    break;
                }
                truncated.pop();
            }
            display_lines.push(format!("{}...", truncated));
            text_truncated = true;
        } else {
            display_lines.push(line.to_string());
        }
    }

    // Calculate total height of text block
    let line_height = font_size.into_value() * 1.2; // Line spacing
    let total_text_height = line_height * display_lines.len() as f32;

    // Draw each line centered vertically within the node
    let start_y = rect.center().y - (total_text_height / 2.0) + (line_height / 2.0);

    for (i, line) in display_lines.iter().enumerate() {
        let galley = painter.layout_no_wrap(line.clone(), font.clone(), constants::TEXT_COLOR);
        let y_offset = start_y + (i as f32 * line_height);
        let text_pos = Pos2::new(
            rect.center().x - galley.size().x / 2.0,
            y_offset - galley.size().y / 2.0,
        );
        painter.galley(text_pos, galley, constants::TEXT_COLOR);
    }

    (rect, text_truncated)
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
