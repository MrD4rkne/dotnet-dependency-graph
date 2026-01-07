use dotnet_dependency_parser::graph::{DependencyGraph, DependencyId, Framework};
use eframe::egui::{Painter, Rect, Response, Sense, Ui, Widget, containers::Scene};
use eframe::egui::{Pos2, Vec2};
use std::collections::{HashMap, HashSet};

use crate::ui::interactions::{InteractionController, InteractionEvent};
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
    node_positions: &'a HashMap<DependencyId, (f32, f32)>,
    node_sizes: &'a HashMap<DependencyId, (f32, f32)>,
}

impl<'a> GraphData<'a> {
    pub(crate) fn new(
        graph: &'a DependencyGraph,
        visible_nodes: &'a HashSet<DependencyId>,
        node_positions: &'a HashMap<DependencyId, (f32, f32)>,
        node_sizes: &'a HashMap<DependencyId, (f32, f32)>,
    ) -> Self {
        Self {
            graph,
            visible_nodes,
            node_positions,
            node_sizes,
        }
    }

    fn get_data_by_id(&self, id: DependencyId) -> NodeData {
        let pos = self.node_positions.get(&id).unwrap();
        let size = self.node_sizes.get(&id).unwrap();
        NodeData {
            pos: Pos2::new(pos.0, pos.1),
            size: Vec2::new(size.0, size.1),
        }
    }
}

struct NodeData {
    pos: Pos2,
    size: Vec2,
}

impl NodeData {
    fn rect(&self) -> Rect {
        Rect::from_center_size(self.pos, self.size)
    }

    fn pos(&self) -> Pos2 {
        self.pos
    }
}

pub(crate) struct GraphWidget<'a> {
    view_state: ViewState<'a>,
    interaction_state: &'a mut InteractionController,
    graph_data: GraphData<'a>,
}

impl<'a> GraphWidget<'a> {
    pub(crate) fn new(
        view_state: ViewState<'a>,
        interaction_state: &'a mut InteractionController,
        graph_data: GraphData<'a>,
    ) -> Self {
        Self {
            view_state,
            interaction_state,
            graph_data,
        }
    }

    fn handle_panning_to_dependency(&mut self) {
        if let Some(sel) = self.interaction_state.panned_dependency() {
            let cache = self.graph_data.get_data_by_id(sel);
            let size = self.view_state.scene_rect.size();
            *self.view_state.scene_rect = Rect::from_center_size(cache.pos(), size);
        }
    }

    /// Draw all edges for the given framework
    fn draw_all_edges(
        painter: &Painter,
        graph_data: &GraphData,
        interaction_state: &InteractionController,
        framework: &Framework,
    ) {
        puffin::profile_function!();
        for src_id in graph_data.visible_nodes.iter() {
            puffin::profile_scope!("draw_edges_for_node");

            let src_data = graph_data.get_data_by_id(*src_id);
            let src_rect = src_data.rect();

            let deps = graph_data
                .graph
                .get_direct_dependencies_in_framework(*src_id, framework)
                .expect("Node should be in the graph.");
            for edge in deps {
                puffin::profile_scope!("per_edge");
                let dst_id = edge.to();

                // Only draw edges to visible nodes
                if !graph_data.visible_nodes.contains(&dst_id) {
                    continue;
                }

                let dst_rect = graph_data.get_data_by_id(dst_id).rect();
                // Highlight edges adjacent to selected dependency
                let highlight = match interaction_state.highlighted_dependency() {
                    Some(sel) => sel == *src_id || sel == dst_id,
                    None => false,
                };

                visualize::draw_edge(painter, src_rect, dst_rect, highlight);
            }
        }
    }
}

impl<'a> Widget for GraphWidget<'a> {
    fn ui(mut self, ui: &mut Ui) -> Response {
        let scene = Scene::new().zoom_range(ZOOM_MIN..=ZOOM_MAX);
        self.handle_panning_to_dependency();

        let graph_data = &self.graph_data;
        let interaction_state = &mut *self.interaction_state;

        let inner = scene.show(ui, self.view_state.scene_rect, move |ui| {
            // Draw nodes in scene coordinates.
            puffin::profile_function!();

            if let Some(framework) = interaction_state.selected_framework() {
                Self::draw_all_edges(ui.painter(), graph_data, interaction_state, framework);
            }

            draw_all_nodes(ui, graph_data, interaction_state);
        });
        inner.response
    }
}

fn draw_all_nodes(
    ui: &mut Ui,
    graph_data: &GraphData,
    interaction_state: &mut InteractionController,
) {
    for id in graph_data.visible_nodes.iter() {
        puffin::profile_scope!("per_visible_node");

        let cache = graph_data.get_data_by_id(*id);
        let dep = graph_data
            .graph
            .get(*id)
            .expect("Visible node should be in graph");

        let is_selected = match interaction_state.highlighted_dependency() {
            Some(sel) => sel == *id,
            None => false,
        };

        draw_single_node(
            *id,
            cache.rect(),
            dep.name(),
            ui,
            interaction_state,
            is_selected,
        );
    }
}

/// Draw a single node and handle its dragging interaction
fn draw_single_node(
    id: DependencyId,
    rect: Rect,
    text: &str,
    ui: &mut Ui,
    interaction_state: &mut InteractionController,
    highlighted: bool,
) {
    puffin::profile_function!();
    visualize::draw_node(text, ui.painter(), rect, highlighted);
    handle_node_interactions(id, rect, ui, interaction_state, text);
}

/// Handle all user interactions for a single node (dragging, clicking, hovering)
fn handle_node_interactions(
    id: DependencyId,
    rect: Rect,
    ui: &mut Ui,
    state: &mut InteractionController,
    text: &str,
) {
    let node_response = ui.interact(rect, ui.id().with(id), Sense::click_and_drag());

    if node_response.dragged() {
        let delta = node_response.drag_delta();
        state.publish(InteractionEvent::MoveBy(id, delta));
    }

    if node_response.clicked() {
        state.publish(InteractionEvent::Select(id));
    }

    if node_response.hovered() {
        state.publish(InteractionEvent::Highlight(id));
    }

    node_response.on_hover_text(text);
}

const ZOOM_MIN: f32 = 0.1;
const ZOOM_MAX: f32 = 3.0;
