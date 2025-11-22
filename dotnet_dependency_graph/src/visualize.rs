use egui::{Color32, FontId, Pos2, Rect, Sense, Stroke, Vec2};
use nuget_dgspec_parser::graph::{DependencyId, DependencyInfo};

pub fn calculate_size(_: &DependencyId, _: &DependencyInfo) -> (f64, f64) {
    // TODO: fill placeholder
    (60.0f64, 24.0f64)
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
        let response = ui.interact(rect, ui.id().with("tooltip"), Sense::hover());
        response.on_hover_text(text);
    }

    rect
}
