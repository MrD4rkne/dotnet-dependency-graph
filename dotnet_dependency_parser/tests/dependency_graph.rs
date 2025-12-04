use dotnet_dependency_parser::graph::{DependencyGraph, DependencyId, DependencyInfo, Framework};

#[test]
fn test_new_graph_is_empty() {
    let graph = DependencyGraph::new();
    assert_eq!(graph.iter().count(), 0);
}

#[test]
fn test_add_project() {
    let mut graph = DependencyGraph::new();
    let project_path = "/path/to/project.csproj".to_string();

    let id = graph.add_project(project_path.clone(), None).unwrap();

    // Verify the dependency was added
    assert_eq!(graph.iter().count(), 1);

    // Verify we can retrieve the project
    let dep_info = graph.get(&id).expect("Project should exist");
    match dep_info {
        DependencyInfo::Project(info) => {
            assert_eq!(info.path, project_path);
        }
        _ => panic!("Expected Project dependency"),
    }
}

#[test]
fn test_add_multiple_projects() {
    let mut graph = DependencyGraph::new();

    let proj1 = graph
        .add_project("/path/to/proj1.csproj".to_string(), None)
        .unwrap();
    let proj2 = graph
        .add_project("/path/to/proj2.csproj".to_string(), None)
        .unwrap();
    let proj3 = graph
        .add_project("/path/to/proj3.csproj".to_string(), None)
        .unwrap();

    assert_eq!(graph.iter().count(), 3);
    assert!(graph.get(&proj1).is_some());
    assert!(graph.get(&proj2).is_some());
    assert!(graph.get(&proj3).is_some());
}

#[test]
fn test_add_duplicate_project_returns_same_id() {
    let mut graph = DependencyGraph::new();
    let project_path = "/path/to/project.csproj".to_string();

    let id1 = graph.add_project(project_path.clone(), None).unwrap();
    let id2 = graph.add_project(project_path.clone(), None).unwrap();

    // Should only have one dependency
    assert_eq!(graph.iter().count(), 1);

    // IDs should be the same
    assert_eq!(id1, id2);
}

#[test]
fn test_add_package_with_version() {
    let mut graph = DependencyGraph::new();
    let package_name = "Newtonsoft.Json".to_string();
    let version = Some("13.0.1".to_string());

    let id = graph
        .add_package(package_name.clone(), version.clone())
        .unwrap();

    assert_eq!(graph.iter().count(), 1);

    let dep_info = graph.get(&id).expect("Package should exist");
    match dep_info {
        DependencyInfo::Package(_) => {
            assert_eq!(dep_info.name(), package_name);
            assert_eq!(dep_info.version(), version.as_ref());
        }
        _ => panic!("Expected Package dependency"),
    }
}

#[test]
fn test_add_package_without_version() {
    let mut graph = DependencyGraph::new();
    let package_name = "MyPackage".to_string();

    let id = graph.add_package(package_name.clone(), None).unwrap();

    assert_eq!(graph.iter().count(), 1);

    let dep_info = graph.get(&id).expect("Package should exist");
    match dep_info {
        DependencyInfo::Package(_) => {
            assert_eq!(dep_info.name(), package_name);
            assert_eq!(dep_info.version(), None);
        }
        _ => panic!("Expected Package dependency"),
    }
}

#[test]
fn test_add_duplicate_package_returns_same_id() {
    let mut graph = DependencyGraph::new();
    let package_name = "Newtonsoft.Json".to_string();
    let version = Some("1.0.0".to_string());

    let id1 = graph
        .add_package(package_name.clone(), version.clone())
        .unwrap();
    let id2 = graph
        .add_package(package_name.clone(), version.clone())
        .unwrap();

    // Should only have one dependency
    assert_eq!(graph.iter().count(), 1);

    // IDs should be the same
    assert_eq!(id1, id2);
}

#[test]
fn test_add_duplicate_package_without_version_returns_same_id() {
    let mut graph = DependencyGraph::new();
    let package_name = "Newtonsoft.Json".to_string();
    let version = None;

    let id1 = graph
        .add_package(package_name.clone(), version.clone())
        .unwrap();
    let id2 = graph
        .add_package(package_name.clone(), version.clone())
        .unwrap();

    // Should only have one dependency
    assert_eq!(graph.iter().count(), 1);

    // IDs should be the same
    assert_eq!(id1, id2);
}

#[test]
fn test_different_package_versions_are_different() {
    let mut graph = DependencyGraph::new();
    let package_name = "Newtonsoft.Json".to_string();

    let id1 = graph
        .add_package(package_name.clone(), Some("13.0.1".to_string()))
        .unwrap();
    let id2 = graph
        .add_package(package_name.clone(), Some("12.0.3".to_string()))
        .unwrap();
    let id3 = graph.add_package(package_name.clone(), None).unwrap();

    assert_eq!(graph.iter().count(), 3);
    assert_ne!(id1, id2);
    assert_ne!(id1, id3);
    assert_ne!(id2, id3);
}

#[test]
fn test_different_package_names_are_different() {
    let mut graph = DependencyGraph::new();
    let package_version = Some("1.2.3".to_string());

    let id1 = graph
        .add_package("package A".to_string(), package_version.clone())
        .unwrap();
    let id2 = graph
        .add_package("package B".to_string(), package_version.clone())
        .unwrap();

    assert_eq!(graph.iter().count(), 2);
    assert_ne!(id1, id2);
}

#[test]
fn test_add_relation_between_projects() {
    let mut graph = DependencyGraph::new();

    let proj1 = graph
        .add_project("/path/to/proj1.csproj".to_string(), None)
        .unwrap();
    let proj2 = graph
        .add_project("/path/to/proj2.csproj".to_string(), None)
        .unwrap();

    let framework = Framework::new("net8.0".to_string());

    let result = graph.add_relation(proj1.clone(), proj2.clone(), framework.clone());

    assert!(result.is_ok());

    // Check that the relation was added
    let deps: Vec<_> = graph
        .get_direct_dependencies_in_framework(&proj1, &framework)
        .unwrap()
        .collect();
    assert_eq!(deps.len(), 1);
    assert_eq!(deps[0].id(), &proj2);
}

#[test]
fn test_add_relation_project_to_package() {
    let mut graph = DependencyGraph::new();

    let proj = graph
        .add_project("/path/to/proj.csproj".to_string(), None)
        .unwrap();
    let pkg = graph
        .add_package("Newtonsoft.Json".to_string(), Some("13.0.1".to_string()))
        .unwrap();

    let framework = Framework::new("net8.0".to_string());

    let result = graph.add_relation(proj.clone(), pkg.clone(), framework.clone());

    assert!(result.is_ok());

    let deps: Vec<_> = graph
        .get_direct_dependencies_in_framework(&proj, &framework)
        .unwrap()
        .collect();
    assert_eq!(deps.len(), 1);
    assert_eq!(deps[0].id(), &pkg);
}

#[test]
fn test_add_relation_with_nonexistent_source() {
    let mut graph = DependencyGraph::new();

    let proj = graph
        .add_project("/path/to/proj.csproj".to_string(), None)
        .unwrap();
    let fake_id = DependencyId::ProjectId("/fake/project.csproj".to_string());

    let framework = Framework::new("net8.0".to_string());

    let result = graph.add_relation(fake_id, proj, framework);

    assert!(result.is_err());
}

#[test]
fn test_add_relation_with_nonexistent_target() {
    let mut graph = DependencyGraph::new();

    let proj = graph
        .add_project("/path/to/proj.csproj".to_string(), None)
        .unwrap();
    let fake_id = DependencyId::PackageId("FakePackage".to_string(), None);

    let framework = Framework::new("net8.0".to_string());

    let result = graph.add_relation(proj, fake_id, framework);

    assert!(result.is_err());
}

#[test]
fn test_multiple_relations_same_framework() {
    let mut graph = DependencyGraph::new();

    let proj = graph
        .add_project("/path/to/proj.csproj".to_string(), None)
        .unwrap();
    let pkg1 = graph.add_package("Package1".to_string(), None).unwrap();
    let pkg2 = graph.add_package("Package2".to_string(), None).unwrap();
    let pkg3 = graph.add_package("Package3".to_string(), None).unwrap();

    let framework = Framework::new("net8.0".to_string());

    graph
        .add_relation(proj.clone(), pkg1.clone(), framework.clone())
        .unwrap();
    graph
        .add_relation(proj.clone(), pkg2.clone(), framework.clone())
        .unwrap();
    graph
        .add_relation(proj.clone(), pkg3.clone(), framework.clone())
        .unwrap();

    let deps: Vec<_> = graph
        .get_direct_dependencies_in_framework(&proj, &framework)
        .unwrap()
        .collect();
    assert_eq!(deps.len(), 3);
}

#[test]
fn test_relations_with_different_frameworks() {
    let mut graph = DependencyGraph::new();

    let proj = graph
        .add_project("/path/to/proj.csproj".to_string(), None)
        .unwrap();
    let pkg1 = graph.add_package("Package1".to_string(), None).unwrap();
    let pkg2 = graph.add_package("Package2".to_string(), None).unwrap();

    let net8 = Framework::new("net8.0".to_string());
    let net7 = Framework::new("net7.0".to_string());

    graph
        .add_relation(proj.clone(), pkg1.clone(), net8.clone())
        .unwrap();
    graph
        .add_relation(proj.clone(), pkg2.clone(), net7.clone())
        .unwrap();

    let deps_net8: Vec<_> = graph
        .get_direct_dependencies_in_framework(&proj, &net8)
        .unwrap()
        .collect();
    let deps_net7: Vec<_> = graph
        .get_direct_dependencies_in_framework(&proj, &net7)
        .unwrap()
        .collect();

    assert_eq!(deps_net8.len(), 1);
    assert_eq!(deps_net8[0].id(), &pkg1);

    assert_eq!(deps_net7.len(), 1);
    assert_eq!(deps_net7[0].id(), &pkg2);
}

#[test]
fn test_edge_from_and_to_are_correct() {
    let mut graph = DependencyGraph::new();

    let proj1 = graph
        .add_project("/path/to/proj1.csproj".to_string(), None)
        .unwrap();
    let proj2 = graph
        .add_project("/path/to/proj2.csproj".to_string(), None)
        .unwrap();

    let framework = Framework::new("net8.0".to_string());

    graph
        .add_relation(proj1.clone(), proj2.clone(), framework.clone())
        .unwrap();

    // Get the edge from proj1 to proj2
    let deps: Vec<_> = graph
        .get_direct_dependencies_in_framework(&proj1, &framework)
        .unwrap()
        .collect();

    assert_eq!(deps.len(), 1);
    let edge = deps[0];

    // Verify from and to are correct
    assert_eq!(edge.from(), &proj1);
    assert_eq!(edge.to(), &proj2);
    assert_eq!(edge.id(), &proj2); // get_id() returns the target (to)
}

#[test]
fn test_edge_from_to_with_project_and_package() {
    let mut graph = DependencyGraph::new();

    let proj = graph
        .add_project("/path/to/proj.csproj".to_string(), None)
        .unwrap();
    let pkg = graph
        .add_package("Newtonsoft.Json".to_string(), Some("13.0.1".to_string()))
        .unwrap();

    let framework = Framework::new("net8.0".to_string());

    graph
        .add_relation(proj.clone(), pkg.clone(), framework.clone())
        .unwrap();

    let deps: Vec<_> = graph
        .get_direct_dependencies_in_framework(&proj, &framework)
        .unwrap()
        .collect();

    assert_eq!(deps.len(), 1);
    let edge = deps[0];

    // Verify the edge correctly represents proj -> pkg
    assert_eq!(edge.from(), &proj);
    assert_eq!(edge.to(), &pkg);
}

#[test]
fn test_multiple_edges_from_same_source() {
    let mut graph = DependencyGraph::new();

    let proj = graph
        .add_project("/path/to/proj.csproj".to_string(), None)
        .unwrap();
    let pkg1 = graph.add_package("Package1".to_string(), None).unwrap();
    let pkg2 = graph.add_package("Package2".to_string(), None).unwrap();
    let pkg3 = graph.add_package("Package3".to_string(), None).unwrap();

    let framework = Framework::new("net8.0".to_string());

    graph
        .add_relation(proj.clone(), pkg1.clone(), framework.clone())
        .unwrap();
    graph
        .add_relation(proj.clone(), pkg2.clone(), framework.clone())
        .unwrap();
    graph
        .add_relation(proj.clone(), pkg3.clone(), framework.clone())
        .unwrap();

    let deps: Vec<_> = graph
        .get_direct_dependencies_in_framework(&proj, &framework)
        .unwrap()
        .collect();

    assert_eq!(deps.len(), 3);

    // All edges should have the same source (from)
    for edge in &deps {
        assert_eq!(edge.from(), &proj);
    }

    // Collect all target IDs
    let target_ids: Vec<&DependencyId> = deps.iter().map(|e| e.to()).collect();
    assert!(target_ids.contains(&&pkg1));
    assert!(target_ids.contains(&&pkg2));
    assert!(target_ids.contains(&&pkg3));
}

#[test]
fn test_reverse_dependencies_edge_from_to() {
    let mut graph = DependencyGraph::new();

    let proj1 = graph
        .add_project("/path/to/proj1.csproj".to_string(), None)
        .unwrap();
    let proj2 = graph
        .add_project("/path/to/proj2.csproj".to_string(), None)
        .unwrap();
    let pkg = graph
        .add_package("SharedPackage".to_string(), None)
        .unwrap();

    let framework = Framework::new("net8.0".to_string());

    // Both projects depend on the package
    graph
        .add_relation(proj1.clone(), pkg.clone(), framework.clone())
        .unwrap();
    graph
        .add_relation(proj2.clone(), pkg.clone(), framework.clone())
        .unwrap();

    // Get reverse dependencies of the package
    let reverse_deps: Vec<_> = graph
        .get_direct_reverse_dependencies(&pkg)
        .unwrap()
        .collect();

    assert_eq!(reverse_deps.len(), 2);

    // For reverse dependencies (incoming edges), the edges still have:
    // - from: the source that depends on pkg (proj1 or proj2)
    // - to: pkg
    for edge in &reverse_deps {
        assert_eq!(edge.to(), &pkg);
    }

    let source_ids: Vec<&DependencyId> = reverse_deps.iter().map(|e| e.from()).collect();
    assert!(source_ids.contains(&&proj1));
    assert!(source_ids.contains(&&proj2));
}

#[test]
fn test_chain_of_dependencies_edge_consistency() {
    let mut graph = DependencyGraph::new();

    let app = graph
        .add_project("/path/to/app.csproj".to_string(), None)
        .unwrap();
    let lib = graph
        .add_project("/path/to/lib.csproj".to_string(), None)
        .unwrap();
    let pkg = graph
        .add_package("CorePackage".to_string(), Some("1.0.0".to_string()))
        .unwrap();

    let framework = Framework::new("net8.0".to_string());

    // Create a chain: app -> lib -> pkg
    graph
        .add_relation(app.clone(), lib.clone(), framework.clone())
        .unwrap();
    graph
        .add_relation(lib.clone(), pkg.clone(), framework.clone())
        .unwrap();

    // Check app -> lib edge
    let app_deps: Vec<_> = graph
        .get_direct_dependencies_in_framework(&app, &framework)
        .unwrap()
        .collect();
    assert_eq!(app_deps.len(), 1);
    assert_eq!(app_deps[0].from(), &app);
    assert_eq!(app_deps[0].to(), &lib);

    // Check lib -> pkg edge
    let lib_deps: Vec<_> = graph
        .get_direct_dependencies_in_framework(&lib, &framework)
        .unwrap()
        .collect();
    assert_eq!(lib_deps.len(), 1);
    assert_eq!(lib_deps[0].from(), &lib);
    assert_eq!(lib_deps[0].to(), &pkg);
}

#[test]
fn test_get_reverse_dependencies() {
    let mut graph = DependencyGraph::new();

    let proj1 = graph
        .add_project("/path/to/proj1.csproj".to_string(), None)
        .unwrap();
    let proj2 = graph
        .add_project("/path/to/proj2.csproj".to_string(), None)
        .unwrap();
    let pkg = graph
        .add_package("SharedPackage".to_string(), None)
        .unwrap();

    let framework = Framework::new("net8.0".to_string());

    graph
        .add_relation(proj1.clone(), pkg.clone(), framework.clone())
        .unwrap();
    graph
        .add_relation(proj2.clone(), pkg.clone(), framework.clone())
        .unwrap();

    let reverse_deps: Vec<_> = graph
        .get_direct_reverse_dependencies(&pkg)
        .unwrap()
        .collect();
    // We should have 2 reverse dependencies (proj1 and proj2 depend on pkg)
    assert_eq!(reverse_deps.len(), 2);

    // The edges contain the target ID (pkg), not the source IDs
    // So all reverse dep edges should point to pkg
    let reverse_dep_ids: Vec<&DependencyId> = reverse_deps.iter().map(|e| e.id()).collect();
    assert!(reverse_dep_ids.iter().all(|id| *id == &pkg));
}

#[test]
fn test_no_reverse_dependencies_for_root_node() {
    let mut graph = DependencyGraph::new();

    let proj = graph
        .add_project("/path/to/proj.csproj".to_string(), None)
        .unwrap();
    let pkg = graph.add_package("LeafPackage".to_string(), None).unwrap();

    let framework = Framework::new("net8.0".to_string());

    graph
        .add_relation(proj.clone(), pkg.clone(), framework.clone())
        .unwrap();

    // proj has no reverse dependencies (it's a root node)
    let reverse_deps: Vec<_> = graph
        .get_direct_reverse_dependencies(&proj)
        .unwrap()
        .collect();
    assert_eq!(reverse_deps.len(), 0);
}

#[test]
fn test_get_direct_dependencies_returns_error_for_nonexistent_dependency() {
    let graph = DependencyGraph::new();
    let fake_id = DependencyId::ProjectId("/fake/project.csproj".to_string());

    // This should return an error
    let non_existing_framework = Framework::new("net8.0".to_string());
    let result = graph.get_direct_dependencies_in_framework(&fake_id, &non_existing_framework);

    assert!(result.is_err());
}

#[test]
fn test_get_reverse_dependencies_returns_error_for_nonexistent_dependency() {
    let graph = DependencyGraph::new();
    let fake_id = DependencyId::ProjectId("/fake/project.csproj".to_string());

    // This should return an error
    let result = graph.get_direct_reverse_dependencies(&fake_id);

    assert!(result.is_err());
}

#[test]
fn test_iter_frameworks() {
    let mut graph = DependencyGraph::new();

    let proj1 = graph
        .add_project("/path/to/proj1.csproj".to_string(), None)
        .unwrap();
    let proj2 = graph
        .add_project("/path/to/proj2.csproj".to_string(), None)
        .unwrap();

    let net8 = Framework::new("net8.0".to_string());
    let net7 = Framework::new("net7.0".to_string());
    let net6 = Framework::new("net6.0".to_string());

    graph
        .add_relation(proj1.clone(), proj2.clone(), net8.clone())
        .unwrap();
    graph
        .add_relation(proj1.clone(), proj2.clone(), net7.clone())
        .unwrap();
    graph
        .add_relation(proj1.clone(), proj2.clone(), net6.clone())
        .unwrap();

    let frameworks: Vec<_> = graph.iter_frameworks().collect();
    assert_eq!(frameworks.len(), 3);
}

#[test]
fn test_iter_all_dependencies() {
    let mut graph = DependencyGraph::new();

    let _proj1 = graph
        .add_project("/path/to/proj1.csproj".to_string(), None)
        .unwrap();
    let _proj2 = graph
        .add_project("/path/to/proj2.csproj".to_string(), None)
        .unwrap();
    let _pkg1 = graph
        .add_package("Package1".to_string(), Some("1.0.0".to_string()))
        .unwrap();
    let _pkg2 = graph.add_package("Package2".to_string(), None).unwrap();

    let all_deps: Vec<_> = graph.iter().collect();
    assert_eq!(all_deps.len(), 4);
}

#[test]
fn test_dependency_id_equality() {
    let id1 = DependencyId::ProjectId("/path/to/proj.csproj".to_string());
    let id2 = DependencyId::ProjectId("/path/to/proj.csproj".to_string());
    let id3 = DependencyId::ProjectId("/path/to/other.csproj".to_string());

    assert_eq!(id1, id2);
    assert_ne!(id1, id3);
}

#[test]
fn test_package_id_equality() {
    let id1 = DependencyId::PackageId("Package".to_string(), Some("1.0.0".to_string()));
    let id2 = DependencyId::PackageId("Package".to_string(), Some("1.0.0".to_string()));
    let id3 = DependencyId::PackageId("Package".to_string(), Some("2.0.0".to_string()));
    let id4 = DependencyId::PackageId("Package".to_string(), None);

    assert_eq!(id1, id2);
    assert_ne!(id1, id3);
    assert_ne!(id1, id4);
    assert_ne!(id3, id4);
}

#[test]
fn test_framework_equality() {
    let fw1 = Framework::new("net8.0".to_string());
    let fw2 = Framework::new("net8.0".to_string());
    let fw3 = Framework::new("net7.0".to_string());

    assert_eq!(fw1, fw2);
    assert_ne!(fw1, fw3);
}

#[test]
fn test_complex_dependency_graph() {
    let mut graph = DependencyGraph::new();

    // Create a more complex graph structure
    let app = graph
        .add_project("/path/to/app.csproj".to_string(), None)
        .unwrap();
    let lib1 = graph
        .add_project("/path/to/lib1.csproj".to_string(), None)
        .unwrap();
    let lib2 = graph
        .add_project("/path/to/lib2.csproj".to_string(), None)
        .unwrap();

    let pkg1 = graph
        .add_package("Newtonsoft.Json".to_string(), Some("13.0.1".to_string()))
        .unwrap();
    let pkg2 = graph
        .add_package("Serilog".to_string(), Some("3.0.1".to_string()))
        .unwrap();
    let pkg3 = graph
        .add_package("EntityFramework".to_string(), Some("6.4.4".to_string()))
        .unwrap();

    let framework = Framework::new("net8.0".to_string());

    // App depends on lib1 and lib2
    graph
        .add_relation(app.clone(), lib1.clone(), framework.clone())
        .unwrap();
    graph
        .add_relation(app.clone(), lib2.clone(), framework.clone())
        .unwrap();

    // App depends on pkg1
    graph
        .add_relation(app.clone(), pkg1.clone(), framework.clone())
        .unwrap();

    // Lib1 depends on pkg2 and pkg3
    graph
        .add_relation(lib1.clone(), pkg2.clone(), framework.clone())
        .unwrap();
    graph
        .add_relation(lib1.clone(), pkg3.clone(), framework.clone())
        .unwrap();

    // Lib2 depends on pkg1 (shared dependency)
    graph
        .add_relation(lib2.clone(), pkg1.clone(), framework.clone())
        .unwrap();

    // Verify app dependencies
    let app_deps: Vec<_> = graph
        .get_direct_dependencies_in_framework(&app, &framework)
        .unwrap()
        .collect();
    assert_eq!(app_deps.len(), 3);

    // Verify lib1 dependencies
    let lib1_deps: Vec<_> = graph
        .get_direct_dependencies_in_framework(&lib1, &framework)
        .unwrap()
        .collect();
    assert_eq!(lib1_deps.len(), 2);

    // Verify lib2 dependencies
    let lib2_deps: Vec<_> = graph
        .get_direct_dependencies_in_framework(&lib2, &framework)
        .unwrap()
        .collect();
    assert_eq!(lib2_deps.len(), 1);

    // Verify pkg1 has reverse dependencies
    let pkg1_reverse: Vec<_> = graph
        .get_direct_reverse_dependencies(&pkg1)
        .unwrap()
        .collect();
    assert_eq!(pkg1_reverse.len(), 2); // app and lib2
}

#[test]
fn test_empty_graph_no_frameworks() {
    let graph = DependencyGraph::new();
    let frameworks: Vec<_> = graph.iter_frameworks().collect();
    assert_eq!(frameworks.len(), 0);
}

#[test]
fn test_get_nonexistent_dependency() {
    let graph = DependencyGraph::new();
    let fake_id = DependencyId::ProjectId("/fake/project.csproj".to_string());

    assert!(graph.get(&fake_id).is_none());
}
