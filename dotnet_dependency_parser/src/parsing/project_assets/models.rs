use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents the root structure of a project.assets.json file,
/// which contains the resolved dependency graph including transitive dependencies.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectAssets {
    /// The version of the assets file format.
    pub version: u32,
    /// Targets containing the resolved dependency graph for each framework.
    pub targets: HashMap<String, HashMap<String, TargetLibrary>>,
    /// All libraries (packages and projects) used in the project.
    pub libraries: HashMap<String, Library>,
    /// Direct dependencies per target framework.
    pub project_file_dependency_groups: Option<HashMap<String, Vec<String>>>,
    /// Package folders where packages are stored.
    pub package_folders: Option<HashMap<String, serde_json::Value>>,
    /// Project-specific information.
    pub project: Option<ProjectInfo>,
}

/// Represents the type of a library.
#[derive(Debug, Serialize, Deserialize)]
pub enum LibraryType {
    /// Indicates that the library comes from compiling a .NET project.
    #[serde(rename = "project")]
    Project,
    /// Indicates that the library comes from compiling an external project (such as an MSBuild-based project)
    #[serde(rename = "externalProject")]
    ExternalProject,
    /// Indicates that the library comes from a NuGet Package
    #[serde(rename = "package")]
    Package,
    /// Indicates that the library comes from a stand-alone .NET Assembly
    #[serde(rename = "assembly")]
    Assembly,
    /// Indicates that the library comes from a .NET Assembly in a globally-accessible location such as the GAC or the Framework Reference Assemblies
    #[serde(rename = "reference")]
    Reference,
    /// Indicates that the library comes from a Windows Metadata Assembly (.winmd file)
    #[serde(rename = "winmd")]
    WinMD,
    /// Indicates that the library could not be resolved
    #[serde(rename = "unresolved")]
    Unresolved,
}

/// Represents a library in a specific target framework.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TargetLibrary {
    /// Type of the library.
    #[serde(rename = "type")]
    pub library_type: LibraryType,
    /// Dependencies of this library.
    #[serde(default)]
    pub dependencies: HashMap<String, String>,
    /// Framework for project dependencies.
    pub framework: Option<String>,
    /// Compile-time assets.
    #[serde(default)]
    pub compile: HashMap<String, serde_json::Value>,
    /// Runtime assets.
    #[serde(default)]
    pub runtime: HashMap<String, serde_json::Value>,
    /// Additional fields that we might not need.
    #[serde(flatten)]
    pub additional: HashMap<String, serde_json::Value>,
}

/// Represents a library in the libraries section.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Library {
    /// SHA512 hash of the package (for NuGet packages).
    pub sha512: Option<String>,
    /// Type of the library.
    #[serde(rename = "type")]
    pub library_type: LibraryType,
    /// Path to the package or project.
    pub path: String,
    /// MSBuild project path (for project references).
    pub msbuild_project: Option<String>,
    /// Files included in the package (for NuGet packages).
    #[serde(default)]
    pub files: Vec<String>,
}

/// Project-specific information from the assets file.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectInfo {
    /// Version of the project.
    pub version: Option<String>,
    /// Restore information.
    pub restore: Option<ProjectRestore>,
    /// Target frameworks and their information.
    pub frameworks: Option<HashMap<String, serde_json::Value>>,
}

/// Restore information for the project.
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
    /// Additional fields.
    #[serde(flatten)]
    pub additional: HashMap<String, serde_json::Value>,
}

/// Parses a JSON string into a ProjectAssets.
///
/// # Arguments
/// * `s` - The JSON string to parse.
///
/// # Returns
/// A Result containing the parsed ProjectAssets or a serde_json error.
///
/// # Example
/// ```
/// use dotnet_dependency_parser::parsing::project_assets::parse_project_assets;
///
/// let json = r#"{
///   "version": 3,
///   "targets": {},
///   "libraries": {}
/// }"#;
/// let assets = parse_project_assets(json).expect("Failed to parse project.assets.json");
/// println!("Parsed assets: {:?}", assets);
/// ```
pub fn parse_project_assets(s: &str) -> serde_json::Result<ProjectAssets> {
    serde_json::from_str(s)
}
