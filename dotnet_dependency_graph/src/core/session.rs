use dotnet_dependency_parser::graph::algo::Config;
use dotnet_dependency_parser::graph::{DependencyGraph, DependencyGraphError, DependencyId};
use std::collections::{BTreeMap, HashMap, HashSet};

use crate::core::layout::LayoutConfig;
use crate::ui::interactions::InteractionController;
use crate::visualize;

pub(crate) struct Session {
    pub(crate) graph: DependencyGraph,
    pub(crate) visible_nodes: HashSet<DependencyId>,
    pub(crate) node_positions: HashMap<DependencyId, (f32, f32)>,
    pub(crate) node_sizes: HashMap<DependencyId, (f32, f32)>,
    pub(crate) tree_by_name: BTreeMap<String, Vec<DependencyId>>,
    pub(crate) interaction_state: InteractionController,
}

impl Session {
    pub(crate) fn load_from(graph: DependencyGraph, config: LayoutConfig) -> Session {
        let visible_nodes = graph.iter().map(|(id, _)| id).collect();
        let sizes = compute_sizes(&graph);
        let positions = compute_positions(&graph, &sizes, config);
        let tree_map = group_packages_by_name(&graph);
        Session::new(graph, positions, sizes, tree_map, visible_nodes)
    }

    pub(crate) fn load_precomputed(
        graph: DependencyGraph,
        node_positions: HashMap<DependencyId, (f32, f32)>,
        visible_nodes: HashSet<DependencyId>,
    ) -> Session {
        let sizes = compute_sizes(&graph);
        let tree_map = group_packages_by_name(&graph);
        Session::new(graph, node_positions, sizes, tree_map, visible_nodes)
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
        self.node_positions = compute_positions(&self.graph, &self.node_sizes, config);
    }

    fn new(
        graph: DependencyGraph,
        node_positions: HashMap<DependencyId, (f32, f32)>,
        node_sizes: HashMap<DependencyId, (f32, f32)>,
        tree_by_name: BTreeMap<String, Vec<DependencyId>>,
        visible_nodes: HashSet<DependencyId>,
    ) -> Self {
        Self {
            graph,
            visible_nodes,
            node_positions,
            node_sizes,
            tree_by_name,
            interaction_state: InteractionController::default(),
        }
    }
}

fn compute_sizes(graph: &DependencyGraph) -> HashMap<DependencyId, (f32, f32)> {
    graph
        .iter()
        .map(|(id, info)| (id, visualize::calculate_size(info)))
        .collect()
}

fn group_packages_by_name(graph: &DependencyGraph) -> BTreeMap<String, Vec<DependencyId>> {
    let mut groups: BTreeMap<String, Vec<DependencyId>> = BTreeMap::new();
    for (id, info) in graph.iter() {
        groups.entry(info.name().to_string()).or_default().push(id);
    }
    groups
}

fn compute_positions(
    graph: &DependencyGraph,
    size_map: &HashMap<DependencyId, (f32, f32)>,
    config: LayoutConfig,
) -> HashMap<DependencyId, (f32, f32)> {
    let config: Config = config.into();
    let layouts = graph.layout_with_config(
        |id, _| {
            let (x, y) = size_map
                .get(&id)
                .expect("All nodes should have computed size cache");
            (*x as f64, *y as f64)
        },
        &config,
    );
    visualize::join_layouts(layouts)
}
