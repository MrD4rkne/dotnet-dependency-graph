#[derive(Clone, Debug, PartialEq, Copy)]
pub(crate) enum RankingType {
    ForceUp,
    ForceDown,
    Original,
}

impl From<RankingType> for dotnet_dependency_parser::graph::algo::RankingType {
    fn from(ranking_type: RankingType) -> dotnet_dependency_parser::graph::algo::RankingType {
        match ranking_type {
            RankingType::ForceUp => dotnet_dependency_parser::graph::algo::RankingType::Up,
            RankingType::ForceDown => dotnet_dependency_parser::graph::algo::RankingType::Down,
            RankingType::Original => dotnet_dependency_parser::graph::algo::RankingType::Original,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Copy)]
pub(crate) enum CrossingMinimization {
    Median,
    Barycenter,
}

impl From<CrossingMinimization> for dotnet_dependency_parser::graph::algo::CrossingMinimization {
    fn from(
        cm: CrossingMinimization,
    ) -> dotnet_dependency_parser::graph::algo::CrossingMinimization {
        match cm {
            CrossingMinimization::Median => {
                dotnet_dependency_parser::graph::algo::CrossingMinimization::Median
            }
            CrossingMinimization::Barycenter => {
                dotnet_dependency_parser::graph::algo::CrossingMinimization::Barycenter
            }
        }
    }
}

/// Configuration for layout algorithm.
#[derive(Clone, Debug, PartialEq, Copy)]
pub(crate) struct LayoutConfig {
    /// spacing between nodes in the same layer
    pub(crate) node_spacing: f64,
    /// spacing between layers
    pub(crate) layer_spacing: u32,
    // how nodes are ranked and distributed
    pub(crate) ranking_type: RankingType,
    // Which heuristic to use when minimizing edge crossings.
    pub(crate) c_minimization: CrossingMinimization,
    // whether to try to minimize crosses of the edges by swapping nodes in the same layer.
    pub(crate) minimize_crosses: bool,
}

impl Default for LayoutConfig {
    fn default() -> Self {
        Self {
            node_spacing: 10.0,
            layer_spacing: 10,
            ranking_type: RankingType::ForceDown,
            c_minimization: CrossingMinimization::Barycenter,
            minimize_crosses: true,
        }
    }
}

impl From<LayoutConfig> for dotnet_dependency_parser::graph::algo::Config {
    fn from(layout_config: LayoutConfig) -> Self {
        dotnet_dependency_parser::graph::algo::Config {
            minimum_length: layout_config.layer_spacing,
            vertex_spacing: layout_config.node_spacing,
            ranking_type: layout_config.ranking_type.into(),
            dummy_vertices: true,
            dummy_size: 1.0,
            c_minimization: layout_config.c_minimization.into(),
            transpose: layout_config.minimize_crosses,
        }
    }
}
