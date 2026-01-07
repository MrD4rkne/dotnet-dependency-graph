use dotnet_dependency_parser::graph::algo::Config;
use dotnet_dependency_parser::graph::{DependencyGraph, DependencyGraphError, DependencyId};
use std::collections::HashMap;
use std::collections::HashSet;
use std::path::PathBuf;

use super::layout::LayoutConfig;
use crate::core::node_cache::GraphCache;
use crate::ui::interactions::InteractionController;
use crate::visualize;

#[derive(Debug)]
pub(crate) struct Session {
    pub(crate) path: PathBuf,
    pub(crate) graph: DependencyGraph,
    pub(crate) visible_nodes: HashSet<DependencyId>,
    pub(crate) cache: GraphCache,
    pub(crate) interaction_state: InteractionController,
}

impl Session {
    pub(crate) fn load_from(
        path: PathBuf,
        graph: DependencyGraph,
        config: LayoutConfig,
    ) -> Session {
        let positions = calculate_positions(&graph, config);
        let visible_nodes = graph.iter().map(|(id, _)| id).collect();
        Session::new(path, graph, positions, visible_nodes)
    }

    pub(crate) fn merge(
        &mut self,
        graph: DependencyGraph,
        config: LayoutConfig,
    ) -> Result<(), DependencyGraphError> {
        self.graph.merge(graph)?;
        self.recalculate_layout(config);
        Ok(())
    }

    pub(crate) fn recalculate_layout(&mut self, config: LayoutConfig) {
        self.cache = GraphCache::new(&self.graph, &calculate_positions(&self.graph, config));
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
            interaction_state: InteractionController::default(),
        }
    }
}

fn calculate_positions(
    graph: &DependencyGraph,
    config: LayoutConfig,
) -> HashMap<DependencyId, (f32, f32)> {
    let config: Config = config.into();
    let layouts = graph.layout_with_config(&visualize::calculate_size, &config);
    visualize::join_layouts(layouts)
}
