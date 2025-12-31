use crate::node;
use crate::visualize;
use dotnet_dependency_parser::graph::{DependencyGraph, DependencyId};
use eframe::egui::{Pos2, Rect, Vec2};
use std::collections::{BTreeMap, HashMap, HashSet};

#[derive(Debug)]
pub(crate) struct CachedNodeData {
    pub(crate) initial_position: Pos2,
    pub(crate) unzoomed_width: f32,
    pub(crate) unzoomed_height: f32,
    pub(crate) rect: Rect,
}

impl CachedNodeData {
    pub(crate) fn new(screen_pos: Pos2, unzoomed_width: f32, unzoomed_height: f32) -> Self {
        Self {
            initial_position: screen_pos,
            unzoomed_width,
            unzoomed_height,
            rect: Rect::from_center_size(Pos2::ZERO, Vec2::new(unzoomed_width, unzoomed_height)),
        }
    }
}

#[derive(Debug)]
pub(crate) struct GraphCache {
    node_cache: HashMap<DependencyId, CachedNodeData>,
    dependency_tree: BTreeMap<String, Vec<DependencyId>>,
}

impl GraphCache {
    pub(crate) fn new(
        graph: &dotnet_dependency_parser::graph::DependencyGraph,
        positions: &HashMap<DependencyId, (f32, f32)>,
        visible_nodes: &HashSet<DependencyId>,
    ) -> Self {
        let node_cache = compute_nodes_cache(graph, positions, visible_nodes);
        let dependency_tree = group_packages_by_name(graph);
        Self {
            node_cache,
            dependency_tree,
        }
    }

    pub(crate) fn dependency_tree(&self) -> &BTreeMap<String, Vec<DependencyId>> {
        &self.dependency_tree
    }

    pub(crate) fn node_cache_mut(&mut self) -> &mut HashMap<DependencyId, CachedNodeData> {
        &mut self.node_cache
    }
}

fn compute_nodes_cache(
    graph: &DependencyGraph,
    positions: &HashMap<DependencyId, (f32, f32)>,
    visible_nodes: &HashSet<DependencyId>,
) -> HashMap<DependencyId, CachedNodeData> {
    puffin::profile_scope!("compute_nodes_cache");
    let mut cache = HashMap::new();
    for id in visible_nodes.iter() {
        if let Some(&pos) = positions.get(id) {
            let text = node::get_display_text(graph.get(*id).expect("Node should exist"));
            let (width, height) = visualize::calculate_dimensions_from_text(text);
            cache.insert(
                *id,
                CachedNodeData::new(Pos2::new(pos.0, pos.1), width, height),
            );
        }
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
