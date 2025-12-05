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
    Dgspec,
    ProjectAssets,
}

impl SupportedParser {
    /// Returns all supported parsers
    pub fn all() -> Vec<Self> {
        vec![SupportedParser::Dgspec, SupportedParser::ProjectAssets]
    }

    /// Returns supported extensions for this parser
    pub fn extensions(&self) -> Vec<&'static str> {
        match self {
            SupportedParser::Dgspec => DgspecParser::extensions(),
            SupportedParser::ProjectAssets => ProjectAssetsParser::extensions(),
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
            SupportedParser::Dgspec => {
                let p = DgspecParser;
                p.parse(path)
            }
            SupportedParser::ProjectAssets => {
                let p = ProjectAssetsParser;
                p.parse(path)
            }
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
fn test_supported_parser_enumeration() {
    let all = SupportedParser::all();
    let exts: Vec<Vec<&str>> = all.iter().map(|p| p.extensions()).collect();
    assert!(exts.iter().any(|e| e.contains(&"nuget.dgspec.json")));
    assert!(exts.iter().any(|e| e.contains(&"project.assets.json")));
}

#[test]
fn test_try_parse_with_supported_parsers_unsupported() {
    let path = Path::new("foo.unknown.json");
    let result = parse_with_supported_parsers(path);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().to_string(), "Unsupported file type");
}
