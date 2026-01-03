use petgraph::graph::NodeIndex;
use petgraph::stable_graph::StableDiGraph;

pub use rust_sugiyama::configure::Config;
pub use rust_sugiyama::configure::CrossingMinimization;
pub use rust_sugiyama::configure::RankingType;
use rust_sugiyama::from_graph;

#[derive(Clone)]
pub struct Layout<V> {
    pub positions: Vec<(V, (f64, f64))>,
    pub width: f64,
    pub height: f64,
}

impl<V> Layout<V> {
    pub fn new(positions: Vec<(V, (f64, f64))>, width: f64, height: f64) -> Self {
        Self {
            positions,
            width,
            height,
        }
    }
}

/// Run Sugiyama layout using a supplied configuration.
pub fn layout_sugiyama_with_config<V: std::cmp::Eq + std::hash::Hash + Clone, E>(
    g: &StableDiGraph<V, E>,
    vertex_size: &impl Fn(NodeIndex, &V) -> (f64, f64),
    cfg: &Config,
) -> Vec<Layout<NodeIndex>> {
    let layouts = from_graph(g, &vertex_size, cfg);
    layouts
        .into_iter()
        .map(|(vec_layout, width, height)| Layout::new(vec_layout, width, height))
        .collect()
}
