use dotnet_dependency_parser::graph::{DependencyGraph, DependencyId, Framework};
use std::collections::HashMap;
use std::collections::HashSet;
use std::path::PathBuf;

use crate::parser;
use crate::visualize;
use dotnet_dependency_parser::graph::DifferentDependencyType;
use dotnet_dependency_parser::graph::Layout;

#[derive(Debug)]
pub(crate) struct Session {
    pub path: PathBuf,
    pub graph: DependencyGraph,
    pub node_positions: HashMap<DependencyId, (f32, f32)>,
    pub selected_framework: Option<Framework>,
    pub visible_nodes: HashSet<DependencyId>,
}

impl Session {
    pub fn load_from(
        path: PathBuf,
        graph: DependencyGraph,
    ) -> Result<Session, Box<dyn std::error::Error + Send + Sync>> {
        let positions = Session::calculate_positions(&graph);
        Ok(Session::new(path, graph, positions))
    }

    pub fn merge(
        &mut self,
        graph: DependencyGraph,
    ) -> Result<(), Vec<(DependencyId, DifferentDependencyType)>> {
        let result = self.graph.merge(graph); // todo: what on failure?

        if result.is_ok() {
            self.node_positions = Self::calculate_positions(&self.graph);
        }

        result
    }

    fn new(
        path: PathBuf,
        graph: DependencyGraph,
        node_positions: HashMap<DependencyId, (f32, f32)>,
    ) -> Self {
        let all_dep_ids = graph.iter().map(|(id, _)| id).collect();
        Self {
            path,
            graph,
            node_positions,
            selected_framework: None,
            visible_nodes: all_dep_ids,
        }
    }

    fn calculate_positions(graph: &DependencyGraph) -> HashMap<DependencyId, (f32, f32)> {
        let layouts = calculate_layout(&graph);
        visualize::join_layouts(layouts)
    }
}

pub fn calculate_layout(graph: &DependencyGraph) -> Vec<Layout<DependencyId>> {
    graph.layout(&visualize::calculate_size)
}
