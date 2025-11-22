use egui::{Color32, FontId, Pos2, Rect, Sense, Stroke, Vec2};
use nuget_dgspec_parser::graph::{DependencyId, DependencyInfo, Layout};
use std::collections::HashMap;

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

    // Base dimensions
    let base_height = 60.0;
    let char_width = 8.0; // Approximate character width at font size 16
    let padding = 32.0; // Left and right padding
    let min_width = 120.0;
    let max_width = 300.0;

    // Calculate width based on text length
    let text_width = (text.len() as f64) * char_width + padding;
    let width = text_width.max(min_width).min(max_width);

    (width, base_height)
}

pub fn draw_node(
    ui: &mut egui::Ui,
    text: &str,
    position: Pos2,
    painter: &egui::Painter,
    zoom: f32,
) -> Rect {
    // Scale width and height with zoom
    let base_width = 200.0;
    let base_height = 60.0;
    let width = base_width * zoom;
    let height = base_height * zoom;
    let max_text_width = width - 16.0 * zoom; // Scale padding too

    let rect = Rect::from_center_size(position, Vec2::new(width, height));

    // Draw rectangle background
    painter.rect_filled(rect, 4.0 * zoom, Color32::from_rgb(70, 130, 180));

    // Draw rectangle border
    painter.rect_stroke(
        rect,
        4.0 * zoom,
        Stroke::new(2.5 * zoom, Color32::from_rgb(30, 60, 100)),
        egui::epaint::StrokeKind::Middle,
    );

    // Draw text with truncation - scale font with zoom
    let font = FontId::proportional(16.0 * zoom);
    let full_galley = painter.layout_no_wrap(text.to_string(), font.clone(), Color32::WHITE);

    // Check if text needs truncation
    let (display_text, text_truncated) = if full_galley.size().x > max_text_width {
        // Truncate and add ellipsis
        let mut truncated = text.to_string();
        while !truncated.is_empty() {
            let test_text = format!("{}...", truncated);
            let test_galley =
                painter.layout_no_wrap(test_text.clone(), font.clone(), Color32::WHITE);
            if test_galley.size().x <= max_text_width {
                break;
            }
            truncated.pop();
        }
        (format!("{}...", truncated), true)
    } else {
        (text.to_string(), false)
    };

    let galley = painter.layout_no_wrap(display_text, font, Color32::WHITE);
    let text_pos = rect.center() - Vec2::new(galley.size().x / 2.0, galley.size().y / 2.0);
    painter.galley(text_pos, galley, Color32::WHITE);

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
        offset_x = max_x + 50.0; // padding
    }

    // TODO: handle f64->f32
    result
        .into_iter()
        .map(|(key, (x, y))| (key, (x as f32, y as f32)))
        .collect()
}
