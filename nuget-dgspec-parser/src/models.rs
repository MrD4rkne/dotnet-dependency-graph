use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DependencyGraphSpec {
    pub format: u32,
    #[serde(default)]
    pub restore: HashMap<String, Value>,
    pub projects: HashMap<String, PackageSpec>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PackageSpec {
    pub version: Option<String>,
    pub restore: Option<ProjectRestore>,
    pub frameworks: Option<HashMap<String, TargetFrameworkInformation>>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectRestore {
    pub project_unique_name: Option<String>,
    pub project_name: Option<String>,
    pub project_path: Option<String>,
    pub packages_path: Option<String>,
    pub output_path: Option<String>,
    pub project_style: Option<String>,
    pub cross_targeting: Option<bool>,
    pub config_file_paths: Option<Vec<String>>,
    pub original_target_frameworks: Option<Vec<String>>,
    #[serde(default)]
    pub sources: HashMap<String, Value>,
    pub frameworks: Option<HashMap<String, RestoreFramework>>,
    pub warning_properties: Option<WarningProperties>,
    pub restore_audit_properties: Option<RestoreAuditProperties>,
    #[serde(rename = "SdkAnalysisLevel")]
    pub sdk_analysis_level: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RestoreFramework {
    pub target_alias: Option<String>,
    #[serde(default)]
    pub project_references: HashMap<String, ProjectReference>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectReference {
    #[serde(rename = "projectPath", alias = "project_path")]
    pub project_path: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WarningProperties {
    pub warn_as_error: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RestoreAuditProperties {
    pub enable_audit: Option<String>,
    pub audit_level: Option<String>,
    pub audit_mode: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TargetFrameworkInformation {
    pub dependencies: Option<HashMap<String, LibraryDependency>>,
    pub download_dependencies: Option<Vec<DownloadDependency>>,
    pub framework_references: Option<HashMap<String, FrameworkReference>>,
    pub imports: Option<Vec<String>>,
    pub target_alias: Option<String>,
    pub runtime_identifier_graph_path: Option<String>,
    pub asset_target_fallback: Option<bool>,
    pub warn: Option<bool>,
    
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DownloadDependency {
    pub name: Option<String>,
    pub version: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FrameworkReference {
    #[serde(rename = "privateAssets", alias = "private_assets")]
    pub private_assets: Option<String>,
}

/// Convenience parser that deserializes a JSON string into `DependencyGraphSpec`.
pub fn parse_dependency_graph_spec(s: &str) -> serde_json::Result<DependencyGraphSpec> {
    serde_json::from_str(s)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LibraryDependency {
    pub target: Option<String>,
    pub version: Option<String>,
}