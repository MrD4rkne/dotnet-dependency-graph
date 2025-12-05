use dotnet_dependency_parser::graph::{DependencyGraph, DependencyId, Framework};
use std::collections::HashMap;
use std::collections::HashSet;
use std::path::PathBuf;

pub struct Session {
    pub path: PathBuf,
    pub graph: DependencyGraph,
    pub node_positions: HashMap<DependencyId, (f32, f32)>,
    pub selected_framework: Option<Framework>,
    pub visible_nodes: HashSet<DependencyId>,
}

impl Session {
    pub fn new(
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
