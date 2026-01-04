use eframe::egui::{ComboBox, Context, Slider, Window};

#[derive(Clone, Debug, PartialEq, Copy)]
enum RankingType {
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
enum CrossingMinimization {
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
pub struct LayoutConfig {
    /// spacing between nodes in the same layer
    node_spacing: f64,
    /// spacing between layers
    layer_spacing: u32,
    // how nodes are ranked and ditributed
    ranking_type: RankingType,
    // Which heuristic to use when minimizing edge crossings.
    c_minimization: CrossingMinimization,
    // whether to try to minimize crosses of the edges by swapping nodes in the same layer.
    minimize_crosses: bool,
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

#[derive(Default)]
pub(crate) struct LayoutWindow {
    visible: bool,
    config: LayoutConfig,
}

impl LayoutWindow {
    pub(crate) fn update(&mut self, ctx: &Context) {
        Window::new("Layout Configuration")
            .open(&mut self.visible)
            .resizable(false)
            .show(ctx, |ui| {
                ui.label("Node spacing");
                ui.add(Slider::new(&mut self.config.node_spacing, 1.0..=40.0))
                .on_hover_text("Spacing between nodes, both in the same and adjacent layers");

                ui.label("Layer spacing");
                ui.add(Slider::new(&mut self.config.layer_spacing, 1..=20))
                .on_hover_text("Space between layers");

                ui.horizontal(|ui| {
                    ui.label("Ranking method:");
                    ComboBox::from_id_salt("ranking_method")
                        .selected_text(format!("{:?}", self.config.ranking_type))
                        .show_ui(ui, |ui| {
                            ui.selectable_value(
                                &mut self.config.ranking_type,
                                RankingType::ForceUp,
                                "Force nodes up",
                            )
                            .on_hover_text("Move the vertices as far up as possible");
                            ui.selectable_value(
                                &mut self.config.ranking_type,
                                RankingType::ForceDown,
                                "Force nodes down",
                            )
                            .on_hover_text("Move the vertices as far down as possible");
                            ui.selectable_value(
                                &mut self.config.ranking_type,
                                RankingType::Original,
                                "Original",
                            )
                            .on_hover_text("Firstly move vertices up, then down")
                        });
                });

                ui.checkbox(
                    &mut self.config.minimize_crosses,
                    "Try to minimize edge crossing",
                )
                .on_hover_text("Should the algo try to minimize edge crossing again, after positioning the nodes in layers, by swapping nodes in the same layer. This might significantly prolong the calculations");

                ui.horizontal(|ui| {
                    ui.label("Crossing edges minimalization method:");
                    ComboBox::from_id_salt("crossing_edges_minimalizing_method")
                        .selected_text(format!("{:?}", self.config.c_minimization))
                        .show_ui(ui, |ui| {
                            ui.selectable_value(
                                &mut self.config.c_minimization,
                                CrossingMinimization::Median,
                                "Median",
                            );
                            ui.selectable_value(
                                &mut self.config.c_minimization,
                                CrossingMinimization::Barycenter,
                                "Barycenter",
                            );
                        });
                });

                if ui.button("Reset to defaults").clicked() {
                    self.config = LayoutConfig::default();
                }
            });
    }

    pub(crate) fn request_show(&mut self) {
        self.visible = true;
    }

    pub(crate) fn get_config(&self) -> LayoutConfig {
        self.config
    }
}
