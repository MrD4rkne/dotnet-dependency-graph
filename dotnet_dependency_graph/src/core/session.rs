use dotnet_dependency_parser::graph::{DependencyGraph, DependencyGraphError, DependencyId};
use std::collections::HashMap;
use std::collections::HashSet;
use std::path::PathBuf;

use crate::core::events;
use crate::graph::GraphCache;
use crate::visualize;
use dotnet_dependency_parser::graph::Layout;

#[derive(Debug)]
pub(crate) struct Session {
    pub(crate) path: PathBuf,
    pub(crate) graph: DependencyGraph,
    pub(crate) visible_nodes: HashSet<DependencyId>,
    pub(crate) cache: GraphCache,
    pub(crate) interaction_state: events::InteractionController,
}

impl Session {
    pub(crate) fn load_from(path: PathBuf, graph: DependencyGraph) -> Session {
        let positions = calculate_positions(&graph);
        let visible_nodes = graph.iter().map(|(id, _)| id).collect();
        Session::new(path, graph, positions, visible_nodes)
    }

    pub(crate) fn merge(&mut self, graph: DependencyGraph) -> Result<(), DependencyGraphError> {
        self.graph.merge(graph)?;
        self.cache = GraphCache::new(&self.graph, &calculate_positions(&self.graph));
        Ok(())
    }

    pub(crate) fn new(
        path: PathBuf,
        graph: DependencyGraph,
        node_positions: HashMap<DependencyId, (f32, f32)>,
        visible_nodes: HashSet<DependencyId>,
    ) -> Self {
        let cache = GraphCache::new(&graph, &node_positions);
        Self {
            path,
            graph,
            visible_nodes,
            cache,
            interaction_state: events::InteractionController::default(),
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
