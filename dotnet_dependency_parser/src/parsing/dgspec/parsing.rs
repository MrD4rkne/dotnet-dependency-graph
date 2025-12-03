use crate::graph::{DependencyGraph, DependencyId, Framework};
use crate::parsing::dgspec::{DependencyGraphSpec, LibraryDependency, ProjectReference};
use std::collections::HashMap;

pub fn create_dependency_graph(
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
