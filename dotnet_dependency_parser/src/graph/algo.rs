use petgraph::graph::NodeIndex;
use petgraph::stable_graph::StableDiGraph;
use std::collections::HashMap;

use rust_sugiyama::configure::Config;
use rust_sugiyama::from_graph;

#[derive(Clone)]
pub struct Layout<V> {
    pub positions: HashMap<V, (f64, f64)>,
    pub width: f64,
    pub height: f64,
}

impl<V> Layout<V> {
    pub fn new(positions: HashMap<V, (f64, f64)>, width: f64, height: f64) -> Self {
        Self {
            positions,
            width,
            height,
        }
    }
}

/// Run Sugiyama layout on a StableDiGraph.
/// Returns a Vec of per-connected-component results. Each result contains:
///   - HashMap<NodeIndex, (x,y)> for node coordinates
///   - width, height of that component layout
pub fn layout_sugiyama<V: std::cmp::Eq + std::hash::Hash + Clone, E>(
    g: &StableDiGraph<V, E>,
    vertex_size: &impl Fn(NodeIndex, &V) -> (f64, f64),
) -> Vec<Layout<NodeIndex>> {
    let layouts = from_graph(g, &vertex_size, &Config::default());
    layouts
        .into_iter()
        .map(|(vec_layout, width, height)| {
            let map: HashMap<NodeIndex, (f64, f64)> = vec_layout.into_iter().collect();
            Layout::new(map, width, height)
        })
        .collect()
}
