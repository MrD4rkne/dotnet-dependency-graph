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
pub fn layout_sugiyama<V, E>(
    g: &StableDiGraph<V, E>,
) -> Vec<(HashMap<NodeIndex, (f64, f64)>, f64, f64)> {
    let vertex_size = |_idx: NodeIndex, _v: &V| -> (f64, f64) { (60.0f64, 24.0f64) };
    let layouts = from_graph(g, &vertex_size, &Config::default());

    layouts
        .into_iter()
        .map(|(vec_layout, width, height)| {
            let mut map = HashMap::with_capacity(vec_layout.len());
            for (idx, (x, y)) in vec_layout {
                map.insert(idx, (x, y));
            }
            (map, width, height)
        })
        .collect()
}
