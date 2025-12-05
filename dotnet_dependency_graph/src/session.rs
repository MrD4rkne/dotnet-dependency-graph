use dotnet_dependency_parser::graph::{DependencyGraph, DependencyId, Framework};
use std::collections::HashMap;
use std::collections::HashSet;
use std::path::PathBuf;

use crate::parser;
use crate::visualize;
use dotnet_dependency_parser::graph::Layout;

pub(crate) struct Session {
    pub path: PathBuf,
    pub graph: DependencyGraph,
    pub node_positions: HashMap<DependencyId, (f32, f32)>,
    pub selected_framework: Option<Framework>,
    pub visible_nodes: HashSet<DependencyId>,
}

impl Session {
    pub fn load_from(path: PathBuf) -> Result<Session, Box<dyn std::error::Error + Send + Sync>> {
        let graph = parser::parse_with_supported_parsers(&path)?;
        let layouts = calculate_layout(&graph);
        let node_positions = visualize::join_layouts(layouts);
        Ok(Session::new(path, graph, node_positions))
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
}

pub fn calculate_layout(graph: &DependencyGraph) -> Vec<Layout<DependencyId>> {
    graph.layout(&visualize::calculate_size)
}
