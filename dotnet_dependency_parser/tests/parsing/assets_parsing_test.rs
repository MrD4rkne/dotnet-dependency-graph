use dotnet_dependency_parser::graph::{DependencyInfo, Framework};
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

    let mut serilog_id = None;
    let mut serilog_console_id = None;
    let mut serilog_debug_id = None;

    for (id, info) in graph.iter() {
        match info {
            DependencyInfo::Package(_) => {
                if info.name() == "Serilog" {
                    assert_eq!(info.version(), Some("4.0.0"));
                    serilog_id = Some(id);
                }
                if info.name() == "Serilog.Sinks.Console" && info.version() == Some("6.1.1") {
                    serilog_console_id = Some(id);
                }
                if info.name() == "Serilog.Sinks.Debug" {
                    serilog_debug_id = Some(id);
                }
            }
            DependencyInfo::Project(_) => {}
        }
    }

    assert!(
        serilog_id.is_some(),
        "Serilog package should be found (transitive dependency)"
    );
    assert!(
        serilog_console_id.is_some(),
        "Serilog.Sinks.Console should be found"
    );
    assert!(
        serilog_debug_id.is_some(),
        "Serilog.Sinks.Debug should be found"
    );

    // Use net8.0 framework for testing
    let framework = Framework::new("net8.0".to_string());

    let deps: Vec<_> = graph
        .get_direct_dependencies_in_framework(serilog_console_id.unwrap(), &framework)
        .unwrap()
        .collect();

    assert!(
        deps.iter().any(|edge| edge.to() == serilog_id.unwrap()),
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
