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
        let scene = Scene::new().zoom_range(ZOOM_MIN..=ZOOM_MAX);

        let inner = scene.show(ui, self.view_state.scene_rect, |ui| {
            // Draw nodes in scene coordinates.
            puffin::profile_function!();
            for id in self.graph_data.visible_nodes.iter() {
                puffin::profile_scope!("per_visible_node");

                let cache = self
                    .node_cache
                    .node_cache_mut()
                    .get_mut(id)
                    .expect("All nodes should have cache");
                let dep = self
                    .graph_data
                    .graph
                    .get(*id)
                    .expect("Visible node should be in graph");

                draw_single_node(id, cache, dep.name(), ui, &mut self.interaction_state);
            }

            if let Some(framework) = self.graph_data.selected_framework.as_ref() {
                draw_all_edges(
                    self.node_cache.node_cache(),
                    ui.painter(),
                    self.graph_data.graph,
                    framework,
                    self.graph_data.visible_nodes,
                );
            }
        });
        inner.response
    }
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
        puffin::profile_scope!("draw_edges_for_node");

        let src_data = cache
            .get(src_id)
            .expect("All nodes data should be in the cache.");
        let src_rect = src_data.rect;

        let deps = graph
            .get_direct_dependencies_in_framework(*src_id, framework)
            .expect("Node should be in the graph.");
        for edge in deps {
            puffin::profile_scope!("per_edge");
            let dst_id = edge.to();

            // Only draw edges to visible nodes
            if !visible_nodes.contains(&dst_id) {
                continue;
            }

            let dst_rect = cache
                .get(&dst_id)
                .expect("All nodes data should be in the cache.")
                .rect;
            visualize::draw_edge(painter, src_rect, dst_rect);
        }
    }
}

/// Draw a single node and handle its dragging interaction
fn draw_single_node(
    id: &DependencyId,
    cache: &mut CachedNodeData,
    text: &str,
    ui: &mut Ui,
    interaction_state: &mut InteractionState,
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
    state: &mut InteractionState,
    text: &str,
) {
    let node_response = ui.interact(data.rect, ui.id().with(id), Sense::drag());

    if node_response.drag_started() {
        *state.dragging_node = Some(*id);
    }

    if node_response.dragged() && state.dragging_node.as_ref() == Some(id) {
        let delta = node_response.drag_delta();
        data.position += delta;
    }

    if node_response.drag_stopped() {
        *state.dragging_node = None;
    }

    node_response.on_hover_text(text);
}

const ZOOM_MIN: f32 = 0.1;
const ZOOM_MAX: f32 = 3.0;
