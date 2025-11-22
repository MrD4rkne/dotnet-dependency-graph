use eframe::Frame;
use egui::{Color32, Painter, Pos2, Rect, Response, Sense, Stroke, Ui, Vec2, Widget};
use nuget_dgspec_parser::graph::{DependencyGraph, DependencyId, Framework};
use std::collections::HashMap;

use crate::visualize;

pub struct GraphWidget<'a> {
    graph: &'a DependencyGraph,
    pan_offset: &'a mut Vec2,
    zoom: &'a mut f32,
    node_positions: &'a mut HashMap<DependencyId, (f32, f32)>,
    dragging_node: &'a mut Option<DependencyId>,
    selected_framework: &'a Option<Framework>,
}

impl<'a> GraphWidget<'a> {
    pub fn new(
        graph: &'a DependencyGraph,
        pan_offset: &'a mut Vec2,
        zoom: &'a mut f32,
        node_positions: &'a mut HashMap<DependencyId, (f32, f32)>,
        dragging_node: &'a mut Option<DependencyId>,
        selected_framework: &'a Option<Framework>,
    ) -> Self {
        Self {
            graph,
            pan_offset,
            zoom,
            node_positions,
            dragging_node,
            selected_framework,
        }
    }

    /// Get positions of the nodes, as they can be set from layout data or from drag able node_positions.
    fn get_positions(&self) -> &HashMap<DependencyId, (f32, f32)> {
        self.node_positions
    }

    fn try_draw_edges(&self, painter: &Painter, response: &Response) {
        match self.selected_framework {
            Some(framework) => self.draw_edges(painter, response, framework),
            None => (),
        }
    }

    fn draw_edges(&self, painter: &Painter, response: &Response, framework: &Framework) {
        let positions = self.get_positions();
        let transform = |pos: Pos2| -> Pos2 {
            let centered = pos.to_vec2() * *self.zoom + *self.pan_offset;
            response.rect.min + centered
        };

        // Draw edges (arrows) between dependencies
        for (src_id, _) in self.graph.iter() {
            if let Some((src_x, src_y)) = positions.get(src_id) {
                let src_pos = Pos2::new(*src_x, *src_y);
                let src_screen = transform(src_pos);

                // Get all dependencies of this node
                for edge in self
                    .graph
                    .get_direct_dependencies_in_framework(src_id, framework.clone())
                {
                    let dst_id = edge.get_id();
                    if let Some((dst_x, dst_y)) = positions.get(dst_id) {
                        let dst_pos = Pos2::new(*dst_x, *dst_y);
                        let dst_screen = transform(dst_pos);

                        // Draw line
                        painter.line_segment(
                            [src_screen, dst_screen],
                            Stroke::new(2.0, Color32::from_rgb(100, 100, 100)),
                        );

                        // Draw arrow head
                        let dir = (dst_screen - src_screen).normalized();
                        let arrow_size = 10.0;
                        let perp = Vec2::new(-dir.y, dir.x);
                        let tip = dst_screen - dir * 30.0; // Offset from node edge

                        painter.line_segment(
                            [tip, tip - dir * arrow_size + perp * arrow_size * 0.5],
                            Stroke::new(2.0, Color32::from_rgb(100, 100, 100)),
                        );
                        painter.line_segment(
                            [tip, tip - dir * arrow_size - perp * arrow_size * 0.5],
                            Stroke::new(2.0, Color32::from_rgb(100, 100, 100)),
                        );
                    }
                }
            }
        }
    }

    fn draw_nodes(&mut self, ui: &mut Ui, painter: &Painter, response: &Response) {
        let positions: Vec<_> = self
            .node_positions
            .iter()
            .map(|(id, &pos)| (id.clone(), pos))
            .collect();
        for (id, (x, y)) in positions {
            let pos = Pos2::new(x, y);
            let screen_pos = response.rect.min + pos.to_vec2() * *self.zoom + *self.pan_offset;

            let text = get_node_text(
                self.graph
                    .get(&id)
                    .expect("Dep from layout should be in the graph"),
            );
            let rect = visualize::draw_node(ui, &text, screen_pos, painter, *self.zoom);

            handle_dragging(
                &id,
                rect,
                ui,
                self.dragging_node,
                self.node_positions,
                *self.zoom,
            );
        }
    }

    fn handle_interactions(&mut self, response: &Response, ui: &Ui) {
        // Handle panning (only when not dragging a node)
        if response.dragged() && self.dragging_node.is_none() {
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

        // Draw edges first (behind nodes)
        self.try_draw_edges(&painter, &response);

        // Draw nodes on top
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

fn handle_dragging(
    id: &DependencyId,
    rect: Rect,
    ui: &mut Ui,
    dragged_node: &mut Option<DependencyId>,
    node_positions: &mut HashMap<DependencyId, (f32, f32)>,
    zoom: f32,
) {
    let node_response = ui.interact(rect, ui.id().with(id), Sense::drag());
    if node_response.drag_started() {
        *dragged_node = Some(id.clone());
    }

    if node_response.dragged() && dragged_node.as_ref() == Some(id) {
        let delta = node_response.drag_delta() / zoom;
        if let Some((x, y)) = node_positions.get_mut(id) {
            *x += delta.x;
            *y += delta.y;
        }
    }

    if node_response.drag_stopped() {
        *dragged_node = None;
    }
}
