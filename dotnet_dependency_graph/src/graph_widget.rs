use egui::{Painter, Pos2, Response, Sense, Ui, Vec2, Widget};
use nuget_dgspec_parser::graph::{DependencyGraph, DependencyId};
use std::collections::HashMap;

use crate::visualize;

pub type LayoutData = Vec<(HashMap<DependencyId, (f64, f64)>, f64, f64)>;

pub struct GraphWidget<'a> {
    graph: &'a DependencyGraph,
    layouts: &'a LayoutData,
    pan_offset: &'a mut Vec2,
    zoom: &'a mut f32,
}

impl<'a> GraphWidget<'a> {
    pub fn new(
        graph: &'a DependencyGraph,
        layouts: &'a LayoutData,
        pan_offset: &'a mut Vec2,
        zoom: &'a mut f32,
    ) -> Self {
        Self {
            graph,
            layouts,
            pan_offset,
            zoom,
        }
    }

    fn draw_nodes(&self, ui: &mut Ui, painter: &Painter, response: &Response) {
        // Only draw the first layout component (avoid duplicates)
        if let Some((layout, _zoom, _size)) = self.layouts.first() {
            if layout.is_empty() {
                return;
            }

            let transform = |pos: Pos2| -> Pos2 {
                let centered = pos.to_vec2() * *self.zoom + *self.pan_offset;
                response.rect.min + centered
            };

            for (id, (x, y)) in layout {
                let pos = Pos2::new(*x as f32, *y as f32);
                let screen_pos = transform(pos);

                let text = get_node_text(
                    self.graph
                        .get(id)
                        .expect("Dep from layout should be in the graph"),
                );
                visualize::draw_node(ui, &text, screen_pos, painter, *self.zoom);
            }
        }
    }

    fn handle_interactions(&mut self, response: &Response, ui: &Ui) {
        // Handle panning
        if response.dragged() {
            *self.pan_offset += response.drag_delta();
        }

        // Handle zoom with mouse wheel
        if response.hovered() {
            let scroll = ui.input(|i| i.smooth_scroll_delta.y);
            if scroll.abs() > 0.1 {
                *self.zoom *= 1.0 + scroll * 0.001;
                *self.zoom = self.zoom.clamp(0.1, 3.0);
            }
        }
    }
}

impl<'a> Widget for GraphWidget<'a> {
    fn ui(mut self, ui: &mut Ui) -> Response {
        let (response, painter) = ui.allocate_painter(ui.available_size(), Sense::click_and_drag());

        self.handle_interactions(&response, ui);
        self.draw_nodes(ui, &painter, &response);

        response
    }
}

fn get_node_text(dep: &nuget_dgspec_parser::graph::DependencyInfo) -> String {
    use nuget_dgspec_parser::graph::DependencyInfo;

    match dep {
        DependencyInfo::Project(proj) => {
            // Extract just the project name from the full path
            if let Some(file_name) = std::path::Path::new(&proj.path).file_stem()
                && let Some(name_str) = file_name.to_str()
            {
                return name_str.to_string();
            }
            proj.path.clone()
        }
        DependencyInfo::Package(pck) => {
            format!("{}@{}", pck.name, pck.version.clone().unwrap_or_default())
        }
    }
}
