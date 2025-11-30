use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// Represents the root structure of a .NET dependency graph specification (dgSpec) file,
/// which describes the dependencies and restore information for projects and packages.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DependencyGraphSpec {
    /// The format version of the dgSpec.
    pub format: u32,
    /// Restore settings, keyed by some identifier (e.g., project paths).
    #[serde(default)]
    pub restore: HashMap<String, Value>,
    /// A map of project paths to their package specifications.
    pub projects: HashMap<String, PackageSpec>,
}

/// Represents a package specification within the dependency graph.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PackageSpec {
    /// The version of the package.
    pub version: Option<String>,
    /// Restore information specific to this package/project.
    pub restore: Option<ProjectRestore>,
    /// Target frameworks and their associated information.
    pub frameworks: Option<HashMap<String, TargetFrameworkInformation>>,
}

/// Contains restore metadata for a project.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectRestore {
    /// Unique name of the project.
    pub project_unique_name: Option<String>,
    /// Display name of the project.
    pub project_name: Option<String>,
    /// File system path to the project.
    pub project_path: Option<String>,
    /// Path where packages are stored.
    pub packages_path: Option<String>,
    /// Output path for the project.
    pub output_path: Option<String>,
    /// Style of the project (e.g., "PackageReference").
    pub project_style: Option<String>,
    /// Whether the project supports cross-targeting.
    pub cross_targeting: Option<bool>,
    /// Paths to configuration files.
    pub config_file_paths: Option<Vec<String>>,
    /// Original target frameworks specified.
    pub original_target_frameworks: Option<Vec<String>>,
    /// Sources for package restore.
    #[serde(default)]
    pub sources: HashMap<String, Value>,
    /// Frameworks and their restore information.
    pub frameworks: Option<HashMap<String, RestoreFramework>>,
    /// Properties for handling warnings.
    pub warning_properties: Option<WarningProperties>,
    /// Properties for restore auditing.
    pub restore_audit_properties: Option<RestoreAuditProperties>,
    /// SDK analysis level.
    #[serde(rename = "SdkAnalysisLevel")]
    pub sdk_analysis_level: Option<String>,
}

/// Represents a framework in the context of restore operations.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RestoreFramework {
    /// Alias for the target framework.
    pub target_alias: Option<String>,
    /// References to other projects.
    #[serde(default)]
    pub project_references: HashMap<String, ProjectReference>,
}

/// A reference to another project.
#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectReference {
    /// Path to the referenced project.
    #[serde(rename = "projectPath", alias = "project_path")]
    pub project_path: Option<String>,
}

/// Properties for configuring warnings during restore.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WarningProperties {
    /// List of warnings to treat as errors.
    pub warn_as_error: Option<Vec<String>>,
}

/// Properties for configuring restore auditing.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RestoreAuditProperties {
    /// Whether auditing is enabled.
    pub enable_audit: Option<String>,
    /// Level of auditing.
    pub audit_level: Option<String>,
    /// Mode of auditing.
    pub audit_mode: Option<String>,
}

/// Information about a target framework, including dependencies.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TargetFrameworkInformation {
    /// Library dependencies for this framework.
    pub dependencies: Option<HashMap<String, LibraryDependency>>,
    /// Download dependencies.
    pub download_dependencies: Option<Vec<DownloadDependency>>,
    /// Framework references.
    pub framework_references: Option<HashMap<String, FrameworkReference>>,
    /// Imported frameworks.
    pub imports: Option<Vec<String>>,
    /// Alias for the target framework.
    pub target_alias: Option<String>,
    /// Path to the runtime identifier graph.
    pub runtime_identifier_graph_path: Option<String>,
    /// Whether asset target fallback is enabled.
    pub asset_target_fallback: Option<bool>,
    /// Whether to warn for this framework.
    pub warn: Option<bool>,
}

/// A dependency that needs to be downloaded.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadDependency {
    /// Name of the dependency.
    pub name: Option<String>,
    /// Version of the dependency.
    pub version: Option<String>,
}

/// A reference to a framework.
#[derive(Debug, Serialize, Deserialize)]
pub struct FrameworkReference {
    /// Private assets for this reference.
    #[serde(rename = "privateAssets", alias = "private_assets")]
    pub private_assets: Option<String>,
}

/// A library dependency.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LibraryDependency {
    /// Target of the dependency.
    pub target: Option<String>,
    /// Version of the dependency.
    pub version: Option<String>,
}

/// Parses a JSON string into a DependencyGraphSpec.
///
/// # Arguments
/// * `s` - The JSON string to parse.
///
/// # Returns
/// A Result containing the parsed DependencyGraphSpec or a serde_json error.
///
/// # Example
/// ```
/// use dotnet_dependency_parser::parsing::dgspec::parse_dependency_graph_spec;
///
/// let json = r#"{
///   "format": 1,
///   "projects": {}
/// }"#;
/// let spec = parse_dependency_graph_spec(json).expect("Failed to parse the dgspec");
/// println!("Parsed spec: {:?}", spec);
/// ```
pub fn parse_dependency_graph_spec(s: &str) -> serde_json::Result<DependencyGraphSpec> {
    serde_json::from_str(s)
}
