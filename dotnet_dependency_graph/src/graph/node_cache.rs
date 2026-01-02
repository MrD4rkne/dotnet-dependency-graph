use crate::node;
use crate::visualize;
use dotnet_dependency_parser::graph::{DependencyGraph, DependencyId};
use eframe::egui::{Pos2, Rect, Vec2};
use std::collections::{BTreeMap, HashMap};

#[derive(Debug)]
pub(crate) struct CachedNodeData {
    pub(crate) position: Pos2,
    pub(crate) width: f32,
    pub(crate) height: f32,
    pub(crate) rect: Rect,
}

impl CachedNodeData {
    pub(crate) fn new(screen_pos: Pos2, width: f32, height: f32) -> Self {
        Self {
            position: screen_pos,
            width,
            height,
            rect: Rect::from_center_size(Pos2::ZERO, Vec2::new(width, height)),
        }
    }
}

#[derive(Debug)]
pub(crate) struct GraphCache {
    node_cache: HashMap<DependencyId, CachedNodeData>,
    dependency_tree: BTreeMap<String, Vec<DependencyId>>,
    // Keep track of the last selected dependency so UI can react to selection changes
    last_selected_dependency: Option<DependencyId>,
}

impl GraphCache {
    pub(crate) fn new(
        graph: &dotnet_dependency_parser::graph::DependencyGraph,
        positions: &HashMap<DependencyId, (f32, f32)>,
    ) -> Self {
        let node_cache = compute_nodes_cache(graph, positions);
        let dependency_tree = group_packages_by_name(graph);
        Self {
            node_cache,
            dependency_tree,
            last_selected_dependency: None,
        }
    }

    pub(crate) fn dependency_tree(&self) -> &BTreeMap<String, Vec<DependencyId>> {
        &self.dependency_tree
    }

    pub(crate) fn node_cache_mut(&mut self) -> &mut HashMap<DependencyId, CachedNodeData> {
        &mut self.node_cache
    }

    pub(crate) fn node_cache(&self) -> &HashMap<DependencyId, CachedNodeData> {
        &self.node_cache
    }

    pub(crate) fn last_selected(&self) -> Option<DependencyId> {
        self.last_selected_dependency
    }

    pub(crate) fn set_last_selected(&mut self, v: Option<DependencyId>) {
        self.last_selected_dependency = v;
    }
}

fn compute_nodes_cache(
    graph: &DependencyGraph,
    positions: &HashMap<DependencyId, (f32, f32)>,
) -> HashMap<DependencyId, CachedNodeData> {
    puffin::profile_scope!("compute_nodes_cache");
    let mut cache = HashMap::new();
    for (id, info) in graph.iter() {
        let text = node::get_display_text(info);
        let (width, height) = visualize::calculate_dimensions_from_text(text);
        let (x, y) = positions.get(&id).copied().unwrap_or((0.0_f32, 0.0_f32));
        cache.insert(id, CachedNodeData::new(Pos2::new(x, y), width, height));
    }
    cache
}

fn group_packages_by_name(graph: &DependencyGraph) -> BTreeMap<String, Vec<DependencyId>> {
    let mut groups: BTreeMap<String, Vec<DependencyId>> = BTreeMap::new();
    for (id, info) in graph.iter() {
        let name = node::get_display_text(info);
        groups.entry(name.to_string()).or_default().push(id);
    }
    groups
}
