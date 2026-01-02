use dotnet_dependency_parser::graph::{
    DependencyGraph, DependencyGraphError, DependencyId, Framework,
};
use std::collections::HashMap;
use std::collections::HashSet;
use std::path::PathBuf;

use crate::graph::GraphCache;
use crate::visualize;
use dotnet_dependency_parser::graph::Layout;

/// Holds data about interactions on the graph.
#[derive(Default, Debug)]
pub(crate) struct InteractionState {
    selected_dependency: Option<DependencyId>,
    selected_framework: Option<Framework>,
}

impl InteractionState {
    /// Returns the currently selected dependency identifier.
    pub(crate) fn selected_dependency(&self) -> Option<DependencyId> {
        self.selected_dependency
    }

    /// Returns the currently selected framework.
    ///
    /// Returns reference as Framework does not impl Copy trait.
    pub(crate) fn selected_framework(&self) -> Option<&Framework> {
        self.selected_framework.as_ref()
    }

    /// Change the selected dependency.
    pub(crate) fn select_dependency(&mut self, id: Option<DependencyId>) {
        self.selected_dependency = id;
    }

    /// Change the selected framework.
    pub(crate) fn select_framework(&mut self, framework: Framework) {
        self.selected_framework = Some(framework);
    }
}

#[derive(Debug)]
pub(crate) struct Session {
    pub(crate) path: PathBuf,
    pub(crate) graph: DependencyGraph,
    pub(crate) visible_nodes: HashSet<DependencyId>,
    pub(crate) cache: GraphCache,
    pub(crate) interaction_state: InteractionState,
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
            visible_nodes,
            cache,
            interaction_state: InteractionState::default(),
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
            visible_nodes: all_dep_ids,
            cache,
            interaction_state: InteractionState::default(),
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
