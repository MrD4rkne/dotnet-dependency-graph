use dotnet_dependency_parser::graph::{
    DependencyGraph, DependencyGraphError, DependencyId, Framework,
};
use std::collections::HashMap;
use std::collections::HashSet;
use std::path::PathBuf;

use crate::graph::GraphCache;
use crate::visualize;
use dotnet_dependency_parser::graph::Layout;

#[derive(Debug)]
pub(crate) struct Session {
    pub(crate) path: PathBuf,
    pub(crate) graph: DependencyGraph,
    pub(crate) node_positions: HashMap<DependencyId, (f32, f32)>,
    pub(crate) selected_framework: Option<Framework>,
    pub(crate) visible_nodes: HashSet<DependencyId>,
    pub(crate) cache: GraphCache,
}

impl Session {
    pub(crate) fn load_from(path: PathBuf, graph: DependencyGraph) -> Session {
        let positions = calculate_positions(&graph);
        Session::new(path, graph, positions)
    }

    pub(crate) fn load_from_saved(
        path: PathBuf,
        graph: DependencyGraph,
        node_positions: HashMap<DependencyId, (f32, f32)>,
        visible_nodes: HashSet<DependencyId>,
    ) -> Session {
        let cache = GraphCache::new(&graph, &node_positions);
        Self {
            path,
            graph,
            node_positions,
            selected_framework: None,
            visible_nodes,
            cache,
        }
    }

    pub(crate) fn merge(&mut self, graph: DependencyGraph) -> Result<(), DependencyGraphError> {
        self.graph.merge(graph)?; // Use atomic merge for safety
        self.node_positions = calculate_positions(&self.graph);
        Ok(())
    }

    fn new(
        path: PathBuf,
        graph: DependencyGraph,
        node_positions: HashMap<DependencyId, (f32, f32)>,
    ) -> Self {
        let all_dep_ids = graph.iter().map(|(id, _)| id).collect();
        let cache = GraphCache::new(&graph, &node_positions);
        Self {
            path,
            graph,
            node_positions,
            selected_framework: None,
            visible_nodes: all_dep_ids,
            cache,
        }
    }
}

fn calculate_positions(graph: &DependencyGraph) -> HashMap<DependencyId, (f32, f32)> {
    let layouts = calculate_layout(graph);
    visualize::join_layouts(layouts)
}

pub(crate) fn calculate_layout(graph: &DependencyGraph) -> Vec<Layout<DependencyId>> {
    puffin::profile_scope!("calculate_layout");
    graph.layout(&visualize::calculate_size)
}
