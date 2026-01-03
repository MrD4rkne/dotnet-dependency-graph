use dotnet_dependency_parser::graph::{DependencyGraph, DependencyId, Framework};
use eframe::egui::{Painter, Rect, Response, Sense, Ui, Widget, containers::Scene};
use std::collections::{HashMap, HashSet};

use crate::graph::node_cache::GraphCache;
use crate::graph::node_cache::NodeData;
use crate::session::InteractionState;
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

// Grouped parameters for graph data
pub(crate) struct GraphData<'a> {
    graph: &'a DependencyGraph,
    visible_nodes: &'a HashSet<DependencyId>,
}

impl<'a> GraphData<'a> {
    pub(crate) fn new(
        graph: &'a DependencyGraph,
        visible_nodes: &'a HashSet<DependencyId>,
    ) -> Self {
        Self {
            graph,
            visible_nodes,
        }
    }
}

pub(crate) struct GraphWidget<'a> {
    view_state: ViewState<'a>,
    interaction_state: &'a mut InteractionState,
    graph_data: GraphData<'a>,
    node_cache: &'a mut GraphCache,
}

impl<'a> GraphWidget<'a> {
    pub(crate) fn new(
        view_state: ViewState<'a>,
        interaction_state: &'a mut InteractionState,
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

    fn handle_dependency_selection(&mut self) {
        if let Some(sel) = self.interaction_state.dependency_to_pan_to() {
            let cache = self
                .node_cache
                .node_cache()
                .get(&sel)
                .expect("All nodes should be in cache");
            let size = self.view_state.scene_rect.size();
            *self.view_state.scene_rect = Rect::from_center_size(*cache.position(), size);
            self.interaction_state.set_dependency_to_pan_to(None);
        }
    }
}

impl<'a> Widget for GraphWidget<'a> {
    fn ui(mut self, ui: &mut Ui) -> Response {
        let scene = Scene::new().zoom_range(ZOOM_MIN..=ZOOM_MAX);
        self.handle_dependency_selection();

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

                let is_selected = match self.interaction_state.highlighted_dependency() {
                    Some(sel) => sel == *id,
                    None => false,
                };

                draw_single_node(
                    *id,
                    cache,
                    dep.name(),
                    ui,
                    self.interaction_state,
                    is_selected,
                );
            }

            if let Some(framework) = self.interaction_state.selected_framework() {
                draw_all_edges(
                    self.node_cache.node_cache(),
                    ui.painter(),
                    self.graph_data.graph,
                    framework,
                    self.graph_data.visible_nodes,
                    self.interaction_state.highlighted_dependency(),
                );
            }
        });
        inner.response
    }
}

/// Draw all edges for the given framework
fn draw_all_edges(
    cache: &HashMap<DependencyId, NodeData>,
    painter: &Painter,
    graph: &DependencyGraph,
    framework: &Framework,
    visible_nodes: &HashSet<DependencyId>,
    selected_dependency: Option<DependencyId>,
) {
    puffin::profile_function!();
    for src_id in visible_nodes.iter() {
        puffin::profile_scope!("draw_edges_for_node");

        let src_data = cache
            .get(src_id)
            .expect("All nodes data should be in the cache.");
        let src_rect = src_data.rect();

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
                .rect();
            // Highlight edges adjacent to selected dependency
            let highlight = match selected_dependency {
                Some(sel) => sel == *src_id || sel == dst_id,
                None => false,
            };

            visualize::draw_edge(painter, src_rect, dst_rect, highlight);
        }
    }
}

/// Draw a single node and handle its dragging interaction
fn draw_single_node(
    id: DependencyId,
    cache: &mut NodeData,
    text: &str,
    ui: &mut Ui,
    interaction_state: &mut InteractionState,
    selected: bool,
) {
    puffin::profile_function!();
    visualize::draw_node(text, ui.painter(), cache, selected);
    handle_node_interactions(id, cache, ui, interaction_state, text);
}

/// Handle dragging interaction for a single node
fn handle_node_interactions(
    id: DependencyId,
    data: &mut NodeData,
    ui: &mut Ui,
    state: &mut InteractionState,
    text: &str,
) {
    let node_response = ui.interact(data.rect(), ui.id().with(id), Sense::click_and_drag());

    if node_response.drag_started() {
        state.set_dragged_node(Some(id));
    }

    if node_response.dragged() && state.dragged_node() == Some(id) {
        let delta = node_response.drag_delta();
        data.move_by(delta);
    }

    if node_response.drag_stopped() {
        state.set_dragged_node(None);
    }

    if node_response.double_clicked() {
        state.set_dependency_to_pan_to(Some(id));
    } else if node_response.clicked() {
        state.select_dependency(Some(id));
    }

    if node_response.hovered() {
        state.highlight_dependency(Some(id));
    }

    node_response.on_hover_text(text);
}

const ZOOM_MIN: f32 = 0.1;
const ZOOM_MAX: f32 = 3.0;
