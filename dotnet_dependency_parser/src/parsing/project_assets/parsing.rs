use std::collections::HashMap;

use super::models::{Library, LibraryType, ProjectAssets, ProjectInfo, TargetLibrary};
use crate::graph::{DependencyGraph, DependencyId, DependencyWithId, Framework};

/// Creates a DependencyGraph from a ProjectAssets structure.
///
/// This function parses the resolved dependency information from project.assets.json,
/// which includes both direct and transitive dependencies.
///
/// # Arguments
/// * `assets` - The parsed ProjectAssets structure
///
/// # Returns
/// A DependencyGraph containing all dependencies and their relationships
pub fn create_dependency_graph_from_assets(assets: ProjectAssets) -> DependencyGraph {
    let mut graph = DependencyGraph::new();
    // Get the project path and add to graph.
    let project_id = assets.project.and_then(|p| parse_project(&mut graph, p));

    // Process all "libraries" section to add all dependencies (Projects and Packages) to the graph.
    parse_libraries(&mut graph, assets.libraries);

    // Process each target framework
    assets
        .targets
        .into_iter()
        .for_each(|(target_framework, libraries)| {
            let framework = Framework::new(target_framework);
            // Process each library in this target framework
            libraries
                .into_iter()
                .for_each(|(library_id, library_info)| {
                    parse_library_entry(&mut graph, &framework, library_id, library_info);
                });
        });

    _ = project_id
        .zip(assets.project_file_dependency_groups)
        .map(|(id, g)| parse_project_file_dependency_groups(&mut graph, &id, g));

    graph
}

fn parse_libraries(graph: &mut DependencyGraph, libraries: HashMap<String, Library>) {
    libraries.into_iter().for_each(|(key, library)| {
        let (name, version) = parse_library_id(key);
        _ = match library.library_type {
            LibraryType::Project => graph.add_project(name, version).ok(),
            LibraryType::Package => graph.add_package(name, version).ok(),
            _ => return, // TODO: support all
        }
    });
}

fn parse_project_file_dependency_groups(
    graph: &mut DependencyGraph,
    proj_id: &DependencyId,
    deps: HashMap<String, Vec<String>>,
) {
    deps.into_iter()
        .flat_map(|(framework, deps_vec)| {
            deps_vec
                .into_iter()
                .map(move |dep| (framework.clone(), dep))
        })
        .for_each(|(framework, dep)| {
            let (name, version) = parse_dep_requirement(dep);
            let dep_id = graph
                .get_or_create_if_exists(&name, version)
                .map(|x| x.id());
            if let Some(dep_id) = dep_id {
                _ = graph.add_relation(proj_id.clone(), dep_id, Framework::new(framework));
            }
        });
}

fn parse_dep_requirement(req: String) -> (String, Option<String>) {
    let parts: Vec<&str> = req.split_whitespace().collect();
    if parts.len() == 1 {
        (parts[0].to_string(), None)
    } else if parts.len() == 3 && parts[1] == ">=" {
        // TODO: parse more
        (parts[0].to_string(), Some(parts[2].to_string()))
    } else {
        // Fallback: assume first part is name, ignore others
        (parts[0].to_string(), None)
    }
}

fn parse_library_entry(
    graph: &mut DependencyGraph,
    target_framework: &Framework,
    library_id: String,
    library_info: TargetLibrary,
) {
    let (name, version) = parse_library_id(library_id);
    let id = match library_info.library_type {
        LibraryType::Package => graph.add_package(name.to_string(), version),
        LibraryType::Project => graph.add_project(name.to_string(), version),
        _ => return,
    };

    let id = match id {
        Ok(id) => id,
        Err(_) => return,
    };

    // Add relations for dependencies
    library_info
        .dependencies
        .into_iter()
        .for_each(|(dep_name, dep_version)| {
            let dep_id = graph
                .get_or_create_if_exists(&dep_name, Some(dep_version))
                .map(|x| x.id());
            if let Some(dep_id) = dep_id {
                _ = graph.add_relation(id.clone(), dep_id, target_framework.clone());
            }
        });
}

fn parse_project(graph: &mut DependencyGraph, proj: ProjectInfo) -> Option<DependencyId> {
    proj.restore
        .and_then(|r| r.project_path)
        .and_then(|path| graph.add_project(path, proj.version).ok())
}

/// Parses a library ID in format "name/version" into separate components.
fn parse_library_id(id: String) -> (String, Option<String>) {
    if let Some(pos) = id.rfind('/') {
        let name = &id[..pos];
        let version = &id[pos + 1..];
        (name.to_string(), Some(version.to_string()))
    } else {
        (id, None)
    }
}
