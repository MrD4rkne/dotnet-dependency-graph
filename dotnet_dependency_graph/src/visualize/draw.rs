use egui::{FontId, Pos2, Rect, Sense, Stroke, Vec2};
use nuget_dgspec_parser::graph::{DependencyId, DependencyInfo, Layout};
use std::collections::HashMap;

use super::Zoomed;
use super::constants;

pub fn calculate_size(_id: &DependencyId, dep: &DependencyInfo) -> (f64, f64) {
    let text = match dep {
        DependencyInfo::Project(proj) => {
            // Extract just the project name from the full path
            if let Some(file_name) = std::path::Path::new(&proj.path).file_stem()
                && let Some(name_str) = file_name.to_str()
            {
                name_str
            } else {
                &proj.path
            }
        }
        DependencyInfo::Package(pck) => &pck.name,
    };

    // Calculate width based on text length
    let text_width = (text.len() as f64) * constants::CHAR_WIDTH + constants::TEXT_PADDING;
    let width = text_width.clamp(constants::MIN_WIDTH, constants::MAX_WIDTH);
    (width, constants::NODE_HEIGHT as f64)
}

pub fn draw_node(
    ui: &mut egui::Ui,
    text: &str,
    position: Pos2,
    painter: &egui::Painter,
    zoom: f32,
) -> Rect {
    // Create zoomed values for all properties
    let width = Zoomed::new(constants::NODE_WIDTH, zoom);
    let height = Zoomed::new(constants::NODE_HEIGHT, zoom);
    let padding = Zoomed::new(constants::NODE_PADDING, zoom);
    let corner_radius = Zoomed::new(constants::NODE_CORNER_RADIUS, zoom);
    let border_width = Zoomed::new(constants::NODE_BORDER_WIDTH, zoom);
    let font_size = Zoomed::new(constants::FONT_SIZE, zoom);

    let max_text_width = width - padding;

    let rect = Rect::from_center_size(position, Vec2::new(width.to_value(), height.to_value()));

    // Draw rectangle background
    painter.rect_filled(
        rect,
        corner_radius.to_value(),
        constants::NODE_BACKGROUND_COLOR,
    );

    // Draw rectangle border
    painter.rect_stroke(
        rect,
        corner_radius.to_value(),
        Stroke::new(border_width.to_value(), constants::NODE_BORDER_COLOR),
        egui::epaint::StrokeKind::Middle,
    );

    // Draw text with truncation - scale font with zoom
    let font = FontId::proportional(font_size.to_value());
    let full_galley = painter.layout_no_wrap(text.to_string(), font.clone(), constants::TEXT_COLOR);

    // Check if text needs truncation
    let (display_text, text_truncated) = if full_galley.size().x > max_text_width.to_value() {
        // Truncate and add ellipsis
        let mut truncated = text.to_string();
        while !truncated.is_empty() {
            let test_text = format!("{}...", truncated);
            let test_galley =
                painter.layout_no_wrap(test_text.clone(), font.clone(), constants::TEXT_COLOR);
            if test_galley.size().x <= max_text_width.to_value() {
                break;
            }
            truncated.pop();
        }
        (format!("{}...", truncated), true)
    } else {
        (text.to_string(), false)
    };

    let galley = painter.layout_no_wrap(display_text, font, constants::TEXT_COLOR);
    let text_pos = rect.center() - Vec2::new(galley.size().x / 2.0, galley.size().y / 2.0);
    painter.galley(text_pos, galley, constants::TEXT_COLOR);

    // Show tooltip on hover with full name if truncated
    if text_truncated {
        let response = ui.interact(rect, ui.id().with(text).with("tooltip"), Sense::hover());
        response.on_hover_text(text);
    }

    rect
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

    // TODO: handle f64->f32
    result
        .into_iter()
        .map(|(key, (x, y))| (key, (x as f32, y as f32)))
        .collect()
}
