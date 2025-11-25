use nuget_dgspec_parser::assets_models::parse_project_assets;
use nuget_dgspec_parser::graph::from_assets::create_dependency_graph_from_assets;
use nuget_dgspec_parser::graph::{DependencyGraph, DependencyId, Framework};
use nuget_dgspec_parser::models::{DependencyGraphSpec, LibraryDependency, ProjectReference};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// Load dependency graph from project.assets.json file
pub fn load_assets_from_file(path: PathBuf) -> std::io::Result<DependencyGraph> {
    let contents = fs::read_to_string(path)?;
    let assets = parse_project_assets(&contents)
        .map_err(|e| std::io::Error::other(format!("Couldn't parse file's content: {}", e)))?;
    Ok(create_dependency_graph_from_assets(assets))
}

/// Load dependency graph from .dgspec.json file (legacy)
pub fn load_dgspec_from_file(path: PathBuf) -> std::io::Result<DependencyGraph> {
    let contents = fs::read_to_string(path)?;
    let dgspec = nuget_dgspec_parser::models::parse_dependency_graph_spec(&contents)
        .map_err(|_| std::io::Error::other("Couldn't parse file's content"))?;
    create_dependency_graph(dgspec).map_err(std::io::Error::other)
}

fn create_dependency_graph(
    spec: DependencyGraphSpec,
) -> Result<DependencyGraph, Box<dyn std::error::Error + Send + Sync>> {
    let mut graph = DependencyGraph::new();

    for (project, spec) in spec.projects {
        let project_id = graph.add_project(project, spec.version)?;
        if let Some(frameworks) = spec.frameworks {
            for (framework, framework_entry) in frameworks {
                let framework = Framework::new(framework);
                if let Some(libs) = framework_entry.dependencies {
                    add_libs(&mut graph, project_id.clone(), framework, libs)?;
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
                )?;
            }
        }
    }

    Ok(graph)
}

fn add_libs(
    graph: &mut DependencyGraph,
    project_id: DependencyId,
    framework: Framework,
    libs: HashMap<String, LibraryDependency>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    for (dep, info) in libs {
        let dep_id = graph.add_package(dep, info.version)?;
        graph.add_relation(project_id.clone(), dep_id, framework.clone())?;
    }

    Ok(())
}

fn add_projs(
    graph: &mut DependencyGraph,
    project_id: DependencyId,
    framework: Framework,
    projs: HashMap<String, ProjectReference>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    for (dep, _) in projs {
        let dep_id = graph.add_project(dep, None)?;
        graph.add_relation(project_id.clone(), dep_id, framework.clone())?;
    }

    Ok(())
}
