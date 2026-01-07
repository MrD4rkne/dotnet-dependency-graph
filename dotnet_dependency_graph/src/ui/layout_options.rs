use eframe::egui::{ComboBox, Context, Slider, Window};

use crate::core::layout::{CrossingMinimization, LayoutConfig, RankingType};

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
