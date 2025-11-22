use petgraph::graph::NodeIndex;
use petgraph::stable_graph::StableDiGraph;
use std::collections::HashMap;

use rust_sugiyama::configure::Config;
use rust_sugiyama::from_graph;

/// Run rust-sugiyama layout on a StableDiGraph.
/// Returns a Vec of per-connected-component results. Each result contains:
///   - HashMap<NodeIndex, (x,y)> for node coordinates
///   - width, height of that component layout
///
/// This uses a simple constant vertex size; you can provide a closure to compute node sizes
/// from node data if you need label-aware sizing.
pub fn layout_sugiyama<V: std::cmp::Eq + std::hash::Hash + Clone, E>(
    g: &StableDiGraph<V, E>,
    vertex_size: &impl Fn(NodeIndex, &V) -> (f64, f64),
) -> Vec<(HashMap<V, (f64, f64)>, f64, f64)> {
    let layouts = from_graph(g, &vertex_size, &Config::default());
    layouts
        .into_iter()
        .map(|(vec_layout, width, height)| {
            let mut map = HashMap::with_capacity(vec_layout.len());
            for (idx, (x, y)) in vec_layout {
                let weight = g
                    .node_weight(idx)
                    .expect("All nodes from calculated layout should be in original graph.");
                map.insert(weight.clone(), (x, y));
            }
            (map, width, height)
        })
        .collect()
}
