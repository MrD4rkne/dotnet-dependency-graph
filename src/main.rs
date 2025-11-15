use eframe::{run_native, App, CreationContext};
use egui::{Context};
use egui_graphs::{
    DefaultEdgeShape, Graph,
    GraphView, SettingsInteraction, SettingsNavigation, DefaultNodeShape,
    default_edge_transform, default_node_transform, to_graph_custom
};

use std::collections::HashMap;

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
    checked: bool,
}

impl DependencyNode{
    fn new(dep: Dependency) -> Self{
        Self { dep, ix: None, checked: false, }
    }
}

use crate::dependency::Dependency;

type GraphType = Graph<Rc<RefCell<DependencyNode>>, (), Directed, DefaultIx, DefaultNodeShape>;

pub struct AnimatedNodesApp {
    g: GraphType,
    deps: HashMap<String, Rc<RefCell<DependencyNode>>>,
    // UI helpers
    new_name: String,
    rename_target: Option<String>,
    rename_buffer: String,
}

impl AnimatedNodesApp {
    fn new(_: &CreationContext<'_>, deps: Vec<Dependency>) -> Self {
        // convert input Vec<Dependency> into a HashMap<String, Rc<RefCell<DependencyNode>>>
        let mut deps_map: HashMap<String, Rc<RefCell<DependencyNode>>> = deps
            .into_iter()
            .map(|d| {
            let rc = Rc::new(RefCell::new(DependencyNode::new(d)));
            let name = rc.borrow().dep.name.clone();
            (name, rc)
            })
            .collect();

        // build the petgraph StableGraph from the vector
    let g = generate_graph(&mut deps_map);

        // build and return the app
        Self {
            g: to_graph_custom(
                &g,
                |n| {
                    default_node_transform(n);
                    let name = n.payload().borrow().dep.name.clone();
                    n.set_label(name);
                },
                default_edge_transform,
            ),
            deps: deps_map,
            new_name: String::new(),
            rename_target: None,
            rename_buffer: String::new(),
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
        egui::SidePanel::left("deps_panel").show(ctx, |ui| {
            ui.heading("Dependencies");
            ui.horizontal(|ui| {
                ui.label("New:");
                ui.text_edit_singleline(&mut self.new_name);
                if ui.button("Add").clicked() {
                    if !self.new_name.trim().is_empty() && !self.deps.contains_key(&self.new_name) {
                        let d = Dependency::new(self.new_name.clone());
                        let rc = Rc::new(RefCell::new(DependencyNode::new(d)));
                        let ix = self.g.add_node(Rc::clone(&rc));
                        rc.borrow_mut().ix = Some(ix);
                        rc.borrow_mut().checked = true;
                        self.deps.insert(self.new_name.clone(), rc);
                        self.new_name.clear();
                        ctx.request_repaint();
                    }
                }
            });

            // (we iterate keys below to avoid borrow issues when mutating `self.deps`)

            // collect keys to avoid borrowing issues when renaming/removing
            let keys: Vec<String> = self.deps.keys().cloned().collect();
            for key in keys {
                if let Some(dep_rc) = self.deps.get(&key) {
                    let dep = Rc::clone(dep_rc);
                    ui.horizontal(|ui| {
                        ui.label(dep.borrow().dep.name.clone());

                        let mut checked = dep.borrow().checked;
                        if ui.checkbox(&mut checked, "Add to graph").changed(){
                            dep.borrow_mut().checked = checked;
                            if checked {
                                let ix = self.g.add_node(Rc::clone(&dep));
                                dep.borrow_mut().ix = Some(ix);
                                dbg!("Added");
                            }
                            else{
                                let mut dep_mut = dep.borrow_mut();
                                if let Some(ix) = dep_mut.ix {
                                    let _ = self.g.remove_node(ix);
                                    dep_mut.ix = None;
                                    dbg!("Removed");
                                }
                            }
                        }

                        if ui.button("Rename").clicked() {
                            self.rename_target = Some(key.clone());
                            self.rename_buffer = key.clone();
                        }
                    });
                }
            }

            // Rename UI
            if let Some(target) = &self.rename_target {
                // clone key so we don't hold an immutable borrow while mutating `self`
                let target_key = target.clone();
                ui.separator();
                ui.label(format!("Rename '{}' ->", target_key));
                ui.horizontal(|ui| {
                    ui.text_edit_singleline(&mut self.rename_buffer);
                    if ui.button("Confirm").clicked() {
                        let new_name = self.rename_buffer.trim().to_string();
                        if !new_name.is_empty() {
                            if new_name != target_key {
                                // avoid collision
                                if self.deps.contains_key(&new_name) {
                                    // name exists, ignore for now (could show warning)
                                } else {
                                    if let Some(rc) = self.deps.remove(&target_key) {
                                        rc.borrow_mut().dep.name = new_name.clone();
                                        self.deps.insert(new_name.clone(), rc);
                                    }
                                }
                            }
                        }
                        self.rename_target = None;
                        self.rename_buffer.clear();
                        ctx.request_repaint();
                    }
                    if ui.button("Cancel").clicked() {
                        self.rename_target = None;
                    }
                });
            }
        });
    }
}

fn generate_graph(deps: &mut HashMap<String,Rc<RefCell<DependencyNode>>>) -> StableGraph<Rc<RefCell<DependencyNode>>, ()> {
    let mut g = StableGraph::new();

    for (_, dep) in deps.iter(){
        let ix = g.add_node(Rc::clone(dep));
        dep.borrow_mut().ix = Some(ix);
        dep.borrow_mut().checked = true;
    }

    for (_, dep) in deps.iter(){
        let src = dep.borrow().ix.expect("ALl nodes should have been added");
        for target in &dep.borrow().dep.deps {
            let target = deps.get(target).expect("All nodes should have been added")
                .borrow()
                .ix.expect("All nodes should have been added");
            g.add_edge(src.clone(), target.clone(), ());
        }
    }

    g
}

// transform_graph was removed because it's unused and referenced a non-local `g`.

fn insert_edges_for(graph: &mut GraphType, deps: &HashMap<String,Rc<RefCell<DependencyNode>>>, dependency: Rc<RefCell<DependencyNode>>){
    let src = dependency.borrow().ix.expect("ALl nodes should have been added");
    for target in &dependency.borrow().dep.deps {
        let target = deps.get(target).expect("All nodes should have been added")
            .borrow()
            .ix.expect("All nodes should have been added");
        graph.add_edge(src.clone(), target.clone(), ());
    }
}

fn main() {
    let mut deps = Vec::new();
    deps.push(Dependency::new("a".to_string()));
    deps.push(Dependency::new("b".to_string()));
    deps.push(Dependency::new("c".to_string()));

    deps[0].deps.push("b".to_string());
    deps[2].deps.push("b".to_string());

    let native_options = eframe::NativeOptions::default();
    run_native(
        "animated",
        native_options,
        Box::new(move |cc| Ok(Box::new(AnimatedNodesApp::new(cc, deps)))),
    )
    .unwrap();
}