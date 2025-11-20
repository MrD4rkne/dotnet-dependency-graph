use serde_json::Value;
use std::{fs, path::PathBuf, process::Command};

use nuget_dgspec_parser::models::parse_dependency_graph_spec;

#[test]
fn dgspec_for_project_and_libraries_dependencies_deserializing_then_serializing_produces_equivalent_json()
-> std::io::Result<()> {
    // Arrange
    let crate_root = std::env::current_dir()?;
    let sln_dir = crate_root
        .join("tests")
        .join("data")
        .join("project_with_two_frameworks");

    clean_dotnet_sln(&sln_dir)?;
    restore_dotnet_sln(&sln_dir)?;

    let dgspec_files = get_dgspecs_from_dir(&sln_dir)?;
    assert!(
        !dgspec_files.is_empty(),
        "no .nuget.dgspec.json files found in {:?}",
        sln_dir
    );

    for path_to_dgspec in dgspec_files {
        let dgspec_content = fs::read_to_string(&path_to_dgspec)?;
        let mut original_content: Value = serde_json::from_str(&dgspec_content)?;

        // Act
        let parsed_spec = parse_dependency_graph_spec(&dgspec_content)?;
        let mut serialized_deserialized_content = serde_json::to_value(&parsed_spec)?;

        remove_nulls(&mut original_content);
        remove_nulls(&mut serialized_deserialized_content);

        // Assert
        assert_eq!(
            original_content, serialized_deserialized_content,
            "roundtrip mismatch for file {:?}",
            path_to_dgspec
        );
    }

    Ok(())
}

fn clean_dotnet_sln(sln_path: &std::path::Path) -> std::io::Result<()> {
    let status = Command::new("dotnet")
        .arg("clean")
        .current_dir(sln_path)
        .status()?;
    match status.success() {
        true => Ok(()),
        false => Err(std::io::Error::other(
            format!("Dotnet clean failed with exit status: {}", status),
        )),
    }
}

fn restore_dotnet_sln(sln_path: &std::path::Path) -> std::io::Result<()> {
    let status = Command::new("dotnet")
        .arg("restore")
        .current_dir(sln_path)
        .status()?;
    match status.success() {
        true => Ok(()),
        false => Err(std::io::Error::other(
            format!("Dotnet restore failed with exit status: {}", status),
        )),
    }
}

// Remove the nulls from the json value.
fn remove_nulls(v: &mut Value) {
    match v {
        Value::Object(map) => {
            let keys: Vec<String> = map.keys().cloned().collect();
            for k in keys {
                if let Some(child) = map.get_mut(&k) {
                    if *child == Value::Null {
                        map.remove(&k);
                    } else {
                        remove_nulls(child);
                    }
                }
            }
        }
        Value::Array(arr) => {
            for item in arr.iter_mut() {
                remove_nulls(item);
            }
        }
        _ => {}
    }
}

fn get_dgspecs_from_dir(dir: &std::path::Path) -> std::io::Result<Vec<PathBuf>> {
    let mut results = Vec::new();
    add_dgspecs_from_dir(dir, &mut results)?;
    Ok(results)
}

fn add_dgspecs_from_dir(dir: &std::path::Path, out: &mut Vec<PathBuf>) -> std::io::Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path_to_file = entry.path();
        if path_to_file.is_dir() {
            add_dgspecs_from_dir(&path_to_file, out)?;
        } else if let Some(name) = path_to_file.file_name().and_then(|s| s.to_str())
        && name.ends_with(".nuget.dgspec.json") {
            out.push(path_to_file);
        }
    }
    Ok(())
}
