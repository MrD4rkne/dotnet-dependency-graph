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
    pub(crate) selected_framework: Option<Framework>,
    pub(crate) visible_nodes: HashSet<DependencyId>,
    pub(crate) selected_dependency: Option<DependencyId>,
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
            selected_framework: None,
            visible_nodes,
            selected_dependency: None,
            cache,
        }
    }

    pub(crate) fn merge(&mut self, graph: DependencyGraph) -> Result<(), DependencyGraphError> {
        self.graph.merge(graph)?;
        self.cache = GraphCache::new(&self.graph, &calculate_positions(&self.graph));
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
            selected_framework: None,
            visible_nodes: all_dep_ids,
            selected_dependency: None,
            cache,
        }
    }
}

fn calculate_positions(graph: &DependencyGraph) -> HashMap<DependencyId, (f32, f32)> {
    let layouts = calculate_layout(graph);
    visualize::join_layouts(layouts)
}

pub(crate) fn calculate_layout(graph: &DependencyGraph) -> Vec<Layout<DependencyId>> {
    puffin::profile_function!();
    graph.layout(&visualize::calculate_size)
}
