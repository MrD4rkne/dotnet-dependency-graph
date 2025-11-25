use egui::{Color32, Painter, Pos2, Rect, Response, Sense, Stroke, Ui, Vec2, Widget};
use nuget_dgspec_parser::graph::{DependencyGraph, DependencyId, DependencyInfo, Framework};
use std::collections::{HashMap, HashSet};

use crate::visualize;

// Constants
const EDGE_STROKE_WIDTH: f32 = 2.0;
const EDGE_COLOR: Color32 = Color32::from_rgb(100, 100, 100);
const ARROW_SIZE: f32 = 10.0;
const ARROW_TIP_OFFSET: f32 = 30.0;
const ARROW_HEAD_WIDTH_FACTOR: f32 = 0.5;
const ZOOM_MIN: f32 = 0.1;
const ZOOM_MAX: f32 = 3.0;
const ZOOM_SENSITIVITY: f32 = 0.001;
const SCROLL_THRESHOLD: f32 = 0.1;

pub struct GraphWidget<'a> {
    graph: &'a DependencyGraph,
    pan_offset: &'a mut Vec2,
    zoom: &'a mut f32,
    node_interaction_state: NodeInteractionState<'a>,
    selected_framework: &'a Option<Framework>,
    visible_nodes: &'a HashSet<DependencyId>,
}

impl<'a> GraphWidget<'a> {
    pub fn new(
        graph: &'a DependencyGraph,
        pan_offset: &'a mut Vec2,
        zoom: &'a mut f32,
        node_positions: &'a mut HashMap<DependencyId, (f32, f32)>,
        dragging_node: &'a mut Option<DependencyId>,
        selected_framework: &'a Option<Framework>,
        visible_nodes: &'a HashSet<DependencyId>,
    ) -> Self {
        Self {
            graph,
            pan_offset,
            zoom,
            node_interaction_state: NodeInteractionState {
                node_positions,
                dragging_node,
            },
            selected_framework,
            visible_nodes,
        }
    }

    fn try_draw_edges(&self, painter: &Painter, response: &Response) {
        if let Some(framework) = self.selected_framework {
            let ctx = DrawContext {
                zoom: *self.zoom,
                pan_offset: *self.pan_offset,
                rect_min: response.rect.min,
                graph: self.graph,
            };

            draw_all_edges(
                &ctx,
                self.node_interaction_state.node_positions,
                painter,
                framework,
                self.visible_nodes,
            );
        }
    }

    fn draw_nodes(
        &mut self,
        ui: &mut Ui,
        painter: &Painter,
        response: &Response,
        get_node_text: impl Fn(&DependencyInfo) -> String,
    ) {
        let ctx = DrawContext {
            zoom: *self.zoom,
            pan_offset: *self.pan_offset,
            rect_min: response.rect.min,
            graph: self.graph,
        };

        let positions: Vec<_> = self
            .node_interaction_state
            .node_positions
            .iter()
            .filter(|(id, _)| self.visible_nodes.contains(id))
            .map(|(id, &pos)| (id.clone(), pos))
            .collect();

        for (id, pos) in positions {
            draw_single_node(
                &id,
                pos,
                &ctx,
                ui,
                painter,
                &get_node_text,
                &mut self.node_interaction_state,
            );
        }
    }

    fn handle_interactions(&mut self, response: &Response, ui: &Ui) {
        handle_panning(
            response,
            self.pan_offset,
            self.node_interaction_state.dragging_node,
        );
        handle_zoom(response, ui, self.zoom);
    }
}

impl<'a> Widget for GraphWidget<'a> {
    fn ui(mut self, ui: &mut Ui) -> Response {
        let (response, painter) = ui.allocate_painter(ui.available_size(), Sense::click_and_drag());

        self.handle_interactions(&response, ui);

        // Draw edges first (behind nodes)
        self.try_draw_edges(&painter, &response);

        // Draw nodes on top
        self.draw_nodes(ui, &painter, &response, crate::transform::get_display_text);

        response
    }
}

fn transform_position(pos: (f32, f32), zoom: f32, pan_offset: Vec2, rect_min: Pos2) -> Pos2 {
    let zoom_wrapper = visualize::Zoomed::new(1.0, zoom);
    let pos_vec = Pos2::new(pos.0, pos.1);
    rect_min + pos_vec.to_vec2() * zoom_wrapper.into_value() + pan_offset
}

/// Context for drawing operations to reduce parameter passing
struct DrawContext<'a> {
    zoom: f32,
    pan_offset: Vec2,
    rect_min: Pos2,
    graph: &'a DependencyGraph,
}

impl<'a> DrawContext<'a> {
    fn transform(&self, pos: (f32, f32)) -> Pos2 {
        transform_position(pos, self.zoom, self.pan_offset, self.rect_min)
    }
}

/// Mutable state for node interactions
struct NodeInteractionState<'a> {
    dragging_node: &'a mut Option<DependencyId>,
    node_positions: &'a mut HashMap<DependencyId, (f32, f32)>,
}

/// Draw all edges for the given framework
fn draw_all_edges(
    ctx: &DrawContext,
    positions: &HashMap<DependencyId, (f32, f32)>,
    painter: &Painter,
    framework: &Framework,
    visible_nodes: &HashSet<DependencyId>,
) {
    for src_id in visible_nodes.iter() {
        if let Some(&src_pos) = positions.get(src_id) {
            let src_screen = ctx.transform(src_pos);

            for edge in ctx
                .graph
                .get_direct_dependencies_in_framework(src_id, framework.clone())
            {
                let dst_id = edge.get_id();

                // Only draw edges to visible nodes
                if !visible_nodes.contains(dst_id) {
                    continue;
                }

                if let Some(&dst_pos) = positions.get(dst_id) {
                    let dst_screen = ctx.transform(dst_pos);
                    draw_edge(painter, src_screen, dst_screen);
                }
            }
        }
    }
}

/// Draw a single edge with arrow from source to destination
fn draw_edge(painter: &Painter, src: Pos2, dst: Pos2) {
    // Draw line
    painter.line_segment([src, dst], Stroke::new(EDGE_STROKE_WIDTH, EDGE_COLOR));

    // Draw arrow head
    let dir = (dst - src).normalized();
    let perp = Vec2::new(-dir.y, dir.x);
    let tip = dst - dir * ARROW_TIP_OFFSET;

    // Two sides of the arrow
    painter.line_segment(
        [
            tip,
            tip - dir * ARROW_SIZE + perp * ARROW_SIZE * ARROW_HEAD_WIDTH_FACTOR,
        ],
        Stroke::new(EDGE_STROKE_WIDTH, EDGE_COLOR),
    );
    painter.line_segment(
        [
            tip,
            tip - dir * ARROW_SIZE - perp * ARROW_SIZE * ARROW_HEAD_WIDTH_FACTOR,
        ],
        Stroke::new(EDGE_STROKE_WIDTH, EDGE_COLOR),
    );
}

/// Draw a single node and handle its dragging interaction
fn draw_single_node(
    id: &DependencyId,
    pos: (f32, f32),
    ctx: &DrawContext,
    ui: &mut Ui,
    painter: &Painter,
    get_node_text: &impl Fn(&DependencyInfo) -> String,
    state: &mut NodeInteractionState,
) {
    let screen_pos = ctx.transform(pos);

    let text = get_node_text(
        ctx.graph
            .get(id)
            .expect("Dep from layout should be in the graph"),
    );
    let (rect, text_truncated) = visualize::draw_node(ui, &text, screen_pos, painter, ctx.zoom);

    handle_node_drag(id, rect, ui, state, ctx.zoom, &text, text_truncated);
}

/// Handle dragging interaction for a single node
fn handle_node_drag(
    id: &DependencyId,
    rect: Rect,
    ui: &mut Ui,
    state: &mut NodeInteractionState,
    zoom: f32,
    text: &str,
    text_truncated: bool,
) {
    let zoom_wrapper = visualize::Zoomed::new(1.0, zoom);
    let node_response = ui.interact(rect, ui.id().with(id), Sense::drag());

    if node_response.drag_started() {
        *state.dragging_node = Some(id.clone());
    }

    if node_response.dragged() && state.dragging_node.as_ref() == Some(id) {
        let delta = node_response.drag_delta() / zoom_wrapper.into_value();
        if let Some((x, y)) = state.node_positions.get_mut(id) {
            *x += delta.x;
            *y += delta.y;
        }
    }

    if node_response.drag_stopped() {
        *state.dragging_node = None;
    }

    // Show tooltip on hover with full name if truncated
    if text_truncated && node_response.hovered() {
        node_response.on_hover_text(text);
    }
}

/// Handle panning of the graph view
fn handle_panning(
    response: &Response,
    pan_offset: &mut Vec2,
    dragging_node: &Option<DependencyId>,
) {
    if response.dragged() && dragging_node.is_none() {
        *pan_offset += response.drag_delta();
    }
}

/// Handle zoom with mouse wheel
fn handle_zoom(response: &Response, ui: &Ui, zoom: &mut f32) {
    if response.hovered() {
        let scroll = ui.input(|i| i.smooth_scroll_delta.y);
        if scroll.abs() > SCROLL_THRESHOLD {
            *zoom *= 1.0 + scroll * ZOOM_SENSITIVITY;
            *zoom = zoom.clamp(ZOOM_MIN, ZOOM_MAX);
        }
    }
}
