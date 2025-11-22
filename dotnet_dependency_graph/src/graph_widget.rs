use egui::{Color32, Painter, Pos2, Response, Sense, Stroke, Ui, Vec2, Widget};
use nuget_dgspec_parser::graph::{DependencyGraph, DependencyId};
use std::collections::HashMap;

use crate::visualize;

pub type LayoutData = Vec<(HashMap<DependencyId, (f64, f64)>, f64, f64)>;

pub struct GraphWidget<'a> {
    graph: &'a DependencyGraph,
    layouts: &'a LayoutData,
    pan_offset: &'a mut Vec2,
    zoom: &'a mut f32,
    node_positions: &'a mut Option<HashMap<DependencyId, (f64, f64)>>,
    dragging_node: &'a mut Option<DependencyId>,
}

impl<'a> GraphWidget<'a> {
    pub fn new(
        graph: &'a DependencyGraph,
        layouts: &'a LayoutData,
        pan_offset: &'a mut Vec2,
        zoom: &'a mut f32,
        node_positions: &'a mut Option<HashMap<DependencyId, (f64, f64)>>,
        dragging_node: &'a mut Option<DependencyId>,
    ) -> Self {
        Self {
            graph,
            layouts,
            pan_offset,
            zoom,
            node_positions,
            dragging_node,
        }
    }

    fn get_positions(&self) -> &HashMap<DependencyId, (f64, f64)> {
        // Use custom positions if available, otherwise use layout positions
        if let Some(positions) = self.node_positions.as_ref() {
            positions
        } else if let Some((layout, _zoom, _size)) = self.layouts.first() {
            layout
        } else {
            panic!("No layout data available");
        }
    }

    fn draw_edges(&self, painter: &Painter, response: &Response) {
        let positions = self.get_positions();

        let transform = |pos: Pos2| -> Pos2 {
            let centered = pos.to_vec2() * *self.zoom + *self.pan_offset;
            response.rect.min + centered
        };

        // Draw edges (arrows) between dependencies
        for (src_id, _) in self.graph.iter() {
            if let Some((src_x, src_y)) = positions.get(&src_id) {
                let src_pos = Pos2::new(*src_x as f32, *src_y as f32);
                let src_screen = transform(src_pos);

                // Get all dependencies of this node
                for edge in self.graph.get_direct_dependencies(&src_id) {
                    let dst_id = edge.get_id();
                    if let Some((dst_x, dst_y)) = positions.get(dst_id) {
                        let dst_pos = Pos2::new(*dst_x as f32, *dst_y as f32);
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
        let positions = self.get_positions().clone();

        let transform = |pos: Pos2| -> Pos2 {
            let centered = pos.to_vec2() * *self.zoom + *self.pan_offset;
            response.rect.min + centered
        };

        for (id, (x, y)) in &positions {
            let pos = Pos2::new(*x as f32, *y as f32);
            let screen_pos = transform(pos);

            let text = get_node_text(
                self.graph
                    .get(id)
                    .expect("Dep from layout should be in the graph"),
            );
            let rect = visualize::draw_node(ui, &text, screen_pos, painter, *self.zoom);

            // Handle node dragging
            let node_response = ui.interact(rect, ui.id().with(id), Sense::drag());

            if node_response.drag_started() {
                *self.dragging_node = Some(id.clone());
            }

            if node_response.dragged() && self.dragging_node.as_ref() == Some(id) {
                let delta = node_response.drag_delta() / *self.zoom;

                // Initialize custom positions if not already done
                if self.node_positions.is_none() {
                    *self.node_positions = Some(positions.clone());
                }

                // Update position
                if let Some(positions) = self.node_positions.as_mut() {
                    if let Some((x, y)) = positions.get_mut(id) {
                        *x += delta.x as f64;
                        *y += delta.y as f64;
                    }
                }
            }

            if node_response.drag_stopped() {
                *self.dragging_node = None;
            }
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
        self.draw_edges(&painter, &response);

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
