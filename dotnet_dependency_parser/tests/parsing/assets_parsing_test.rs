use dotnet_dependency_parser::graph::{DependencyId, DependencyInfo, Framework};
use dotnet_dependency_parser::parsing::project_assets::{
    create_dependency_graph_from_assets, parse_project_assets,
};
use std::fs;

use super::dotnet_project;

#[test]
fn test_parse_project_assets_json() {
    // Arrange
    let content = get_assets_content();

    // Act
    let assets = parse_project_assets(&content).expect("Failed to parse project.assets.json");
    let graph = create_dependency_graph_from_assets(assets);

    // Assert - Check that we have the expected packages
    let mut found_serilog = false;
    let mut found_serilog_console = false;
    let mut found_serilog_debug = false;

    for (_id, info) in graph.iter() {
        match info {
            DependencyInfo::Package(_) => {
                if info.name() == "Serilog" {
                    found_serilog = true;
                    assert_eq!(info.version(), Some(&"4.0.0".to_string()));
                }
                if info.name() == "Serilog.Sinks.Console" {
                    found_serilog_console = true;
                }
                if info.name() == "Serilog.Sinks.Debug" {
                    found_serilog_debug = true;
                }
            }
            DependencyInfo::Project(_) => {}
        }
    }

    assert!(
        found_serilog,
        "Serilog package should be found (transitive dependency)"
    );
    assert!(
        found_serilog_console,
        "Serilog.Sinks.Console should be found"
    );
    assert!(found_serilog_debug, "Serilog.Sinks.Debug should be found");

    // Check that Serilog.Sinks.Console has Serilog as a dependency
    let serilog_console_id = DependencyId::PackageId(
        "Serilog.Sinks.Console".to_string(),
        Some("6.1.1".to_string()),
    );
    let serilog_id = DependencyId::PackageId("Serilog".to_string(), Some("4.0.0".to_string()));

    // Use net8.0 framework for testing
    let framework = Framework::new("net8.0".to_string());

    let deps: Vec<_> = graph
        .get_direct_dependencies_in_framework(&serilog_console_id, &framework)
        .unwrap()
        .collect();

    assert!(
        deps.iter().any(|edge| edge.to() == &serilog_id),
        "Serilog.Sinks.Console should depend on Serilog"
    );
}

#[test]
fn test_parse_project_assets_includes_all_frameworks() {
    // Arrange
    let content = get_assets_content();

    // Act
    let assets = parse_project_assets(&content).expect("Failed to parse project.assets.json");
    let graph = create_dependency_graph_from_assets(assets);

    // Assert - Check that we have both frameworks
    let frameworks: Vec<_> = graph.iter_frameworks().collect();

    assert!(
        frameworks.iter().any(|f| f.name() == "net8.0"),
        "Should have net8.0 framework"
    );
    assert!(
        frameworks.iter().any(|f| f.name() == "net9.0"),
        "Should have net9.0 framework"
    );
}

fn get_assets_content() -> String {
    // Arrange
    let crate_root = std::env::current_dir().expect("Invalid current dir");
    let sln_dir = crate_root
        .join("tests")
        .join("data")
        .join("project_with_two_frameworks");

    dotnet_project::clean_dotnet_sln(&sln_dir).expect("Failed on dotnet clean");
    dotnet_project::restore_dotnet_sln(&sln_dir).expect("Failed on dotnet restore");

    let assets_path = sln_dir
        .join("console1")
        .join("obj")
        .join("project.assets.json");
    fs::read_to_string(assets_path).expect("Failed while reading project.assets.json")
}
