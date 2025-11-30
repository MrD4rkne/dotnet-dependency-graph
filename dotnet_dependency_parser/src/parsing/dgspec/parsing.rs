use crate::graph::{DependencyGraph, DependencyId, Framework};
use crate::parsing::dgspec::{DependencyGraphSpec, LibraryDependency, ProjectReference};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

pub fn load_dgspec_from_file(path: PathBuf) -> std::io::Result<DependencyGraph> {
    let contents = fs::read_to_string(path)?;
    let dgspec = crate::parsing::dgspec::parse_dependency_graph_spec(&contents)
        .map_err(|_| std::io::Error::other("Couldn't parse file's content"))?;
    Ok(create_dependency_graph(dgspec))
}

fn create_dependency_graph(spec: DependencyGraphSpec) -> DependencyGraph {
    let mut graph = DependencyGraph::new();

    for (project, spec) in spec.projects {
        let project_id = graph.add_project(project);
        if let Some(frameworks) = spec.frameworks {
            for (framework, framework_entry) in frameworks {
                let framework = Framework::new(framework);
                if let Some(libs) = framework_entry.dependencies {
                    add_libs(&mut graph, project_id.clone(), framework, libs);
                }
            }
        }

        if let Some(frameworks) = spec.restore.and_then(|x| x.frameworks) {
            for (framework, framework_entry) in frameworks {
                let framework = Framework::new(framework);
                add_projs(
                    &mut graph,
                    project_id.clone(),
                    framework,
                    framework_entry.project_references,
                );
            }
        }
    }

    graph
}

fn add_libs(
    graph: &mut DependencyGraph,
    project_id: DependencyId,
    framework: Framework,
    libs: HashMap<String, LibraryDependency>,
) {
    for (dep, info) in libs {
        let dep_id = graph.add_package(dep, info.version);
        graph
            .add_relation(project_id.clone(), dep_id, framework.clone())
            .expect("Both dependencies should be in the graph");
    }
}

fn add_projs(
    graph: &mut DependencyGraph,
    project_id: DependencyId,
    framework: Framework,
    projs: HashMap<String, ProjectReference>,
) {
    for (dep, _) in projs {
        let dep_id = graph.add_project(dep);
        graph
            .add_relation(project_id.clone(), dep_id, framework.clone())
            .expect("Both dependencies should be in the graph");
    }
}
