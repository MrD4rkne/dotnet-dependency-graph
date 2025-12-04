+
use dotnet_dependency_parser::graph::{DependencyGraph, DependencyId, Framework};
use egui::{Painter, Pos2, Rect, Response, Sense, Ui, Vec2, Widget};
use std::collections::{HashMap, HashSet};

use crate::node;
use crate::visualize;

pub fn compute_node_cache(
    graph: &DependencyGraph,
    positions: &HashMap<DependencyId, (f32, f32)>,
    visible_nodes: &HashSet<DependencyId>,
    zoom: f32,
    pan_offset: Vec2,
) -> HashMap<DependencyId, CachedNodeData> {
    let mut cache = HashMap::new();
    let ctx = DrawContext {
        state: State {
            zoom,
            pan_offset,
            rect_min: Pos2::ZERO, // dummy
        },
        graph,
    };
    for id in visible_nodes.iter() {
        if let Some(&pos) = positions.get(id) {
            let screen_pos = ctx.transform(pos);
            let text = node::get_display_text(graph.get(id).expect("Node should exist"));
            let rect = visualize::calculate_node_rect(&text, screen_pos, zoom);
            cache.insert(
                id.clone(),
                CachedNodeData {
                    screen_pos,
                    rect,
                    text: text.to_string(),
                },
            );
        }
    }
    cache
}

// Cached data for node calculations per frame
pub struct CachedNodeData {
    screen_pos: Pos2,
    rect: Rect,
    text: String,
}

// Constants
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
    node_cache: &'a HashMap<DependencyId, CachedNodeData>,
    drag_happened: &'a mut bool,
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
        node_cache: &'a HashMap<DependencyId, CachedNodeData>,
        drag_happened: &'a mut bool,
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
            node_cache,
            drag_happened,
        }
    }

    fn try_draw_edges(
        &self,
        painter: &Painter,
        cache: &HashMap<DependencyId, CachedNodeData>,
    ) {
        if let Some(framework) = self.selected_framework {
            draw_all_edges(
                cache,
                painter,
                self.graph,
                framework,
                self.visible_nodes,
                *self.zoom,
            );
        }
    }

    fn draw_nodes(
        &mut self,
        ui: &mut Ui,
        painter: &Painter,
        cache: &HashMap<DependencyId, CachedNodeData>,
    ) {
        for id in self.visible_nodes.iter() {
            if let Some(data) = cache.get(id) {
                draw_single_node(
                    id,
                    data,
                    ui,
                    painter,
                    &mut self.node_interaction_state,
                    *self.zoom,
                    &mut self.drag_happened,
                );
            }
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
        self.try_draw_edges(&painter, self.node_cache);

        // Draw nodes on top
        self.draw_nodes(ui, &painter, self.node_cache);

        response
    }
}

fn transform_position(pos: (f32, f32), zoom: f32, pan_offset: Vec2, rect_min: Pos2) -> Pos2 {
    let pos_vec = Pos2::new(pos.0, pos.1);
    rect_min + pos_vec.to_vec2() * zoom + pan_offset
}

struct State {
    zoom: f32,
    pan_offset: Vec2,
    rect_min: Pos2,
}

/// Context for drawing operations to reduce parameter passing
struct DrawContext<'a> {
    state: State,
    graph: &'a DependencyGraph,
}

impl<'a> DrawContext<'a> {
    fn transform(&self, pos: (f32, f32)) -> Pos2 {
        transform_position(
            pos,
            self.state.zoom,
            self.state.pan_offset,
            self.state.rect_min,
        )
    }
}

/// Mutable state for node interactions
struct NodeInteractionState<'a> {
    dragging_node: &'a mut Option<DependencyId>,
    node_positions: &'a mut HashMap<DependencyId, (f32, f32)>,
}

/// Draw all edges for the given framework
fn draw_all_edges(
    cache: &HashMap<DependencyId, CachedNodeData>,
    painter: &Painter,
    graph: &DependencyGraph,
    framework: &Framework,
    visible_nodes: &HashSet<DependencyId>,
    zoom: f32,
) {
    for src_id in visible_nodes.iter() {
        if let Some(src_data) = cache.get(src_id) {
            let src_rect = src_data.rect;

            let deps = graph.get_direct_dependencies_in_framework(src_id, framework.clone());

            if let Ok(edges) = deps {
                for edge in edges {
                    let dst_id = edge.to();

                    // Only draw edges to visible nodes
                    if !visible_nodes.contains(dst_id) {
                        continue;
                    }

                    if let Some(dst_data) = cache.get(dst_id) {
                        let dst_rect = dst_data.rect;
                        visualize::draw_edge(painter, src_rect, dst_rect, zoom);
                    }
                }
            }
        }
    }
}

/// Draw a single node and handle its dragging interaction
fn draw_single_node(
    id: &DependencyId,
    data: &CachedNodeData,
    ui: &mut Ui,
    painter: &Painter,
    state: &mut NodeInteractionState,
    zoom: f32,
    drag_happened: &mut bool,
) {
    visualize::draw_node(ui, &data.text, data.screen_pos, painter, zoom);
    handle_node_drag(id, data.rect, ui, state, zoom, &data.text, drag_happened);
}

/// Handle dragging interaction for a single node
fn handle_node_drag(
    id: &DependencyId,
    rect: Rect,
    ui: &mut Ui,
    state: &mut NodeInteractionState,
    zoom: f32,
    text: &str,
    drag_happened: &mut bool,
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
        *drag_happened = true;
    }

    if node_response.drag_stopped() {
        *state.dragging_node = None;
    }

    node_response.on_hover_text(text);
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
