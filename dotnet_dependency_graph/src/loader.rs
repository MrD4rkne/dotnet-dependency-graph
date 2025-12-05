use dotnet_dependency_parser::graph::{DependencyGraph, DependencyId, Layout};
use std::path::PathBuf;

use crate::file::Session;
use crate::parser;
use crate::visualize;

pub fn load_file(path: PathBuf) -> Result<Session, Box<dyn std::error::Error + Send + Sync>> {
    let graph = parser::parse_with_supported_parsers(&path)?;
    let layouts = calculate_layout(&graph);
    let node_positions = visualize::join_layouts(layouts);
    Ok(Session::new(path, graph, node_positions))
}

pub fn calculate_layout(graph: &DependencyGraph) -> Vec<Layout<DependencyId>> {
    graph.layout(&visualize::calculate_size)
}
