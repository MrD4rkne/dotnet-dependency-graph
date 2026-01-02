use dotnet_dependency_parser::graph::{DependencyGraph, DependencyId, Framework};
use eframe::egui::{Painter, Rect, Response, Sense, Ui, Widget, containers::Scene};
use std::collections::{HashMap, HashSet};

use crate::graph::node_cache::CachedNodeData;
use crate::graph::node_cache::GraphCache;
use crate::visualize;

// Grouped parameters for view state
pub(crate) struct ViewState<'a> {
    /// Scene rect stored in app state; mutated by Scene::show to reflect pan/zoom.
    scene_rect: &'a mut Rect,
}

impl<'a> ViewState<'a> {
    pub(crate) fn new(scene_rect: &'a mut Rect) -> Self {
        Self { scene_rect }
    }
}

// Grouped parameters for interaction state
pub(crate) struct InteractionState<'a> {
    dragging_node: &'a mut Option<DependencyId>,
}

impl<'a> InteractionState<'a> {
    pub(crate) fn new(dragging_node: &'a mut Option<DependencyId>) -> Self {
        Self { dragging_node }
    }
}

// Grouped parameters for graph data
pub(crate) struct GraphData<'a> {
    graph: &'a DependencyGraph,
    selected_framework: &'a Option<Framework>,
    visible_nodes: &'a HashSet<DependencyId>,
}

impl<'a> GraphData<'a> {
    pub(crate) fn new(
        graph: &'a DependencyGraph,
        selected_framework: &'a Option<Framework>,
        visible_nodes: &'a HashSet<DependencyId>,
    ) -> Self {
        Self {
            graph,
            selected_framework,
            visible_nodes,
        }
    }
}

pub(crate) struct GraphWidget<'a> {
    view_state: ViewState<'a>,
    interaction_state: InteractionState<'a>,
    graph_data: GraphData<'a>,
    node_cache: &'a mut GraphCache,
}

impl<'a> GraphWidget<'a> {
    pub(crate) fn new(
        view_state: ViewState<'a>,
        interaction_state: InteractionState<'a>,
        graph_data: GraphData<'a>,
        node_cache: &'a mut GraphCache,
    ) -> Self {
        Self {
            view_state,
            interaction_state,
            graph_data,
            node_cache,
        }
    }
}

impl<'a> Widget for GraphWidget<'a> {
    fn ui(mut self, ui: &mut Ui) -> Response {
        // Use a Scene container to handle pan & zoom. Store the scene rect in
        // the application state (via ViewState) so the Scene can mutate it.
        let scene = Scene::new().zoom_range(ZOOM_MIN..=ZOOM_MAX);
        let node_cache = &mut *self.node_cache;
        let graph_data = &self.graph_data;
        let interaction_state = &mut self.interaction_state;
        let scene_rect = self.view_state.scene_rect;

        let inner = scene.show(ui, scene_rect, |ui| {
            // Draw nodes in scene coordinates.
            puffin::profile_function!();
            for id in graph_data.visible_nodes.iter() {
                puffin::profile_scope!("per_visible_node");

                let cache = node_cache
                    .node_cache_mut()
                    .get_mut(id)
                    .expect("All nodes should have cache");
                let dep = graph_data
                    .graph
                    .get(*id)
                    .expect("Visible node should be in graph");

                draw_single_node(
                    id,
                    cache,
                    dep.name(),
                    ui,
                    &mut NodeInteractionState {
                        dragging_node: interaction_state.dragging_node,
                    },
                );
            }

            if let Some(framework) = graph_data.selected_framework.as_ref() {
                draw_all_edges(
                    node_cache.node_cache(),
                    ui.painter(),
                    graph_data.graph,
                    framework,
                    graph_data.visible_nodes,
                );
            }
        });

        // Return the response of the inner scene UI so upstream can react to interactions.
        inner.response
    }
}

/// Mutable state for node interactions
struct NodeInteractionState<'a> {
    dragging_node: &'a mut Option<DependencyId>,
}

/// Draw all edges for the given framework
fn draw_all_edges(
    cache: &HashMap<DependencyId, CachedNodeData>,
    painter: &Painter,
    graph: &DependencyGraph,
    framework: &Framework,
    visible_nodes: &HashSet<DependencyId>,
) {
    puffin::profile_function!();
    for src_id in visible_nodes.iter() {
        puffin::profile_scope!("per_edge");
        if let Some(src_data) = cache.get(src_id) {
            let src_rect = src_data.rect;

            let deps = graph.get_direct_dependencies_in_framework(*src_id, framework);

            if let Ok(edges) = deps {
                for edge in edges {
                    let dst_id = edge.to();

                    // Only draw edges to visible nodes
                    if !visible_nodes.contains(&dst_id) {
                        continue;
                    }

                    if let Some(dst_data) = cache.get(&dst_id) {
                        let dst_rect = dst_data.rect;
                        visualize::draw_edge(painter, src_rect, dst_rect);
                    }
                }
            }
        }
    }
}

/// Draw a single node and handle its dragging interaction
fn draw_single_node(
    id: &DependencyId,
    cache: &mut CachedNodeData,
    text: &str,
    ui: &mut Ui,
    interaction_state: &mut NodeInteractionState,
) {
    puffin::profile_function!();
    visualize::draw_node(text, ui.painter(), cache);
    handle_node_drag(id, cache, ui, interaction_state, text);
}

/// Handle dragging interaction for a single node
fn handle_node_drag(
    id: &DependencyId,
    data: &mut CachedNodeData,
    ui: &mut Ui,
    state: &mut NodeInteractionState,
    text: &str,
) {
    let node_response = ui.interact(data.rect, ui.id().with(id), Sense::drag());

    if node_response.drag_started() {
        *state.dragging_node = Some(*id);
    }

    if node_response.dragged() && state.dragging_node.as_ref() == Some(id) {
        // In scene coordinates, drag_delta() is the delta in world-space already.
        let delta = node_response.drag_delta();
        data.position += delta;
    }

    if node_response.drag_stopped() {
        *state.dragging_node = None;
    }

    node_response.on_hover_text(text);
}

// Panning is now handled by Scene::register_pan_and_zoom.

const ZOOM_MIN: f32 = 0.1;
const ZOOM_MAX: f32 = 3.0;
// Zooming/panning behavior is supplied by `Scene`.
