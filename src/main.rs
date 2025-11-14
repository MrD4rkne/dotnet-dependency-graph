use eframe::{run_native, App, CreationContext};
use egui::{Context, PopupAnchor};
use egui_graphs::{
    DefaultEdgeShape, Graph,
    GraphView, SettingsInteraction, SettingsNavigation, DefaultNodeShape,
    default_edge_transform, default_node_transform, to_graph_custom
};

use std::cell::RefCell;
use std::rc::Rc;

use petgraph::{
    stable_graph::{DefaultIx, StableGraph, NodeIndex},
    Directed,
};

mod dependency;

struct DependencyNode {
    dep: Dependency,
    ix: Option<NodeIndex>,
}

impl DependencyNode{
    fn new(dep: Dependency) -> Self{
        Self { dep, ix: None }
    }
}

use crate::dependency::Dependency;

const GLYPH_CLOCKWISE: &str = "↻";
const GLYPH_ANTICLOCKWISE: &str = "↺";

pub struct AnimatedNodesApp {
    g: Graph<Rc<RefCell<DependencyNode>>, (), Directed, DefaultIx, DefaultNodeShape>,
    deps: Vec<Rc<RefCell<DependencyNode>>>
}

impl AnimatedNodesApp {
    fn new(_: &CreationContext<'_>, deps: Vec<Dependency>) -> Self {
        let mut deps: Vec<Rc<RefCell<DependencyNode>>> = deps.into_iter()
        .map(|x| Rc::new(RefCell::new(DependencyNode::new(x))))
        .collect();

        let g = generate_graph(&mut deps);
        Self {
            g: to_graph_custom(
                &g,
                default_node_transform,
                default_edge_transform,
            ),
            deps
        }
    }
}

impl App for AnimatedNodesApp {
    fn update(&mut self, ctx: &Context, _: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add(
                &mut GraphView::<_, _, _, _, DefaultNodeShape, DefaultEdgeShape>::new(&mut self.g)
                    .with_navigations(
                        &SettingsNavigation::default()
                            .with_fit_to_screen_enabled(false)
                            .with_zoom_and_pan_enabled(true),
                    )
                    .with_interactions(
                        &SettingsInteraction::default()
                            .with_dragging_enabled(true)
                            .with_node_selection_enabled(true)
                            .with_edge_selection_enabled(true),
                    ),
            );
        });

        egui::SidePanel::right("right_panel").show(ctx, |ui| {
            ui.label("Select a node to change its label");
            if ui.button("reset").clicked() {
                ui.label("clicked");
            }
        });
    }
}

fn generate_graph(deps: &mut Vec<Rc<RefCell<DependencyNode>>>) -> StableGraph<Rc<RefCell<DependencyNode>>, ()> {
    let mut g = StableGraph::new();

    for dep in deps{
        let ix = g.add_node(Rc::clone(dep));
        dep.borrow_mut().ix = Some(ix);
    }

    g
}

fn main() {
    let mut deps = Vec::new();
    deps.push(Dependency::new("a".to_string()));
    deps.push(Dependency::new("b".to_string()));
    deps.push(Dependency::new("c".to_string()));

    let native_options = eframe::NativeOptions::default();
    run_native(
        "animated",
        native_options,
        Box::new(move |cc| Ok(Box::new(AnimatedNodesApp::new(cc, deps)))),
    )
    .unwrap();
}

mod node {
    #[derive(Clone, Debug)]
    pub struct NodeData {
        pub clockwise: bool,
    }
}