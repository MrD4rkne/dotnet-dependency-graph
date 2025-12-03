use dotnet_dependency_parser::graph::DependencyGraph;
use dotnet_dependency_parser::parsing::dgspec::{
    create_dependency_graph, parse_dependency_graph_spec,
};
use dotnet_dependency_parser::parsing::project_assets::{
    create_dependency_graph_from_assets, parse_project_assets,
};
use std::fs;
use std::path::Path;

/// Enum for supported parsers
enum SupportedParser {
    Dgspec(DgspecParser),
    ProjectAssets(ProjectAssetsParser),
}

impl SupportedParser {
    /// Returns all supported parsers
    pub fn all() -> Vec<Self> {
        vec![
            SupportedParser::Dgspec(DgspecParser),
            SupportedParser::ProjectAssets(ProjectAssetsParser),
        ]
    }

    /// Returns supported extensions for this parser
    pub fn extensions(&self) -> Vec<&'static str> {
        match self {
            SupportedParser::Dgspec(_) => DgspecParser::extensions(),
            SupportedParser::ProjectAssets(_) => ProjectAssetsParser::extensions(),
        }
    }

    /// Returns true if this parser supports the file
    pub fn does_support_extension(&self, filename: &str) -> bool {
        self.extensions().iter().any(|ext| filename.ends_with(ext))
    }

    /// Tries to parse the file using this parser
    pub fn parse(
        &self,
        path: &std::path::Path,
    ) -> Result<DependencyGraph, Box<dyn std::error::Error + Send + Sync>> {
        match self {
            SupportedParser::Dgspec(p) => p.parse(path),
            SupportedParser::ProjectAssets(p) => p.parse(path),
        }
    }
}

/// Tries to parse with the first parser that matches the file extension
pub fn parse_with_supported_parsers(
    path: &std::path::Path,
) -> Result<DependencyGraph, Box<dyn std::error::Error + Send + Sync>> {
    let filename = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
    for parser in SupportedParser::all() {
        if parser.does_support_extension(filename) {
            return parser.parse(path);
        }
    }
    Err("Unsupported file type".into())
}

trait DependencyFileParser {
    /// Returns supported file extensions
    fn extensions() -> Vec<&'static str>;

    /// Parses the file at the given path and returns a DependencyGraph
    fn parse(
        &self,
        path: &Path,
    ) -> Result<DependencyGraph, Box<dyn std::error::Error + Send + Sync>>;
}

struct DgspecParser;

impl DependencyFileParser for DgspecParser {
    fn extensions() -> Vec<&'static str> {
        vec!["nuget.dgspec.json"]
    }

    fn parse(
        &self,
        path: &Path,
    ) -> Result<DependencyGraph, Box<dyn std::error::Error + Send + Sync>> {
        let contents = fs::read_to_string(path)?;
        let dgspec = parse_dependency_graph_spec(&contents)
            .map_err(|_| std::io::Error::other("Couldn't parse file's content"))?;
        create_dependency_graph(dgspec)
    }
}

struct ProjectAssetsParser;

impl DependencyFileParser for ProjectAssetsParser {
    fn extensions() -> Vec<&'static str> {
        vec!["project.assets.json"]
    }

    fn parse(
        &self,
        path: &Path,
    ) -> Result<DependencyGraph, Box<dyn std::error::Error + Send + Sync>> {
        let contents = fs::read_to_string(path)?;
        let assets = parse_project_assets(&contents)
            .map_err(|_| std::io::Error::other("Couldn't parse file's content"))?;
        Ok(create_dependency_graph_from_assets(assets))
    }
}

#[test]
fn test_dgspec_parser_supports_extension() {
    assert!(DgspecParser::does_support_extension(
        "foo.nuget.dgspec.json"
    ));
    assert!(!DgspecParser::does_support_extension(
        "foo.project.assets.json"
    ));
}

#[test]
fn test_project_assets_parser_supports_extension() {
    assert!(ProjectAssetsParser::does_support_extension(
        "foo.project.assets.json"
    ));
    assert!(!ProjectAssetsParser::does_support_extension(
        "foo.nuget.dgspec.json"
    ));
}
