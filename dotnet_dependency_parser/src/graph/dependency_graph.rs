use petgraph::stable_graph::{NodeIndex, StableDiGraph};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use thiserror::Error;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct DependencyId {
    ix: NodeIndex,
}

impl DependencyId {
    fn new(ix: NodeIndex) -> Self {
        Self { ix }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct ProjectInfo {
    path: String,
    version: Option<String>,
}

impl ProjectInfo {
    fn new(path: String, version: Option<String>) -> Self {
        Self { path, version }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct PackageInfo {
    name: String,
    version: Option<String>,
}

impl PackageInfo {
    fn new(name: String, version: Option<String>) -> Self {
        Self { name, version }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub enum DependencyInfo {
    Project(ProjectInfo),
    Package(PackageInfo),
}

impl DependencyInfo {
    /// Get name of the dependency.
    pub fn name(&self) -> &str {
        match self {
            DependencyInfo::Project(info) => &info.path,
            DependencyInfo::Package(info) => &info.name,
        }
    }

    /// Get version of the dependency.
    pub fn version(&self) -> Option<&str> {
        match self {
            DependencyInfo::Project(info) => info.version.as_deref(),
            DependencyInfo::Package(info) => info.version.as_deref(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct Framework {
    name: String,
}

impl Framework {
    pub fn new(name: String) -> Self {
        Self { name }
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

#[derive(Debug, Clone)]
pub struct DepEdge {
    from: DependencyId,
    to: DependencyId,
    target_framework: Framework,
}

impl DepEdge {
    fn new(from: DependencyId, to: DependencyId, target_framework: Framework) -> Self {
        Self {
            from,
            to,
            target_framework,
        }
    }

    pub fn from(&self) -> DependencyId {
        self.from
    }

    pub fn to(&self) -> DependencyId {
        self.to
    }

    pub fn framework(&self) -> &Framework {
        &self.target_framework
    }
}

#[derive(Debug, Clone)]
pub struct DependencyGraph {
    graph: StableDiGraph<DependencyInfo, DepEdge>,
    id_by_name: HashMap<String, Vec<DependencyId>>,
    frameworks: HashSet<Framework>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SerializableGraph<T> {
    pub nodes: Vec<(usize, DependencyInfo)>,
    pub edges: Vec<(usize, usize, Framework)>,
    pub node_metadata: Option<HashMap<usize, T>>,
}

#[derive(Error, Debug)]
pub enum SerializableGraphError<T> {
    #[error("Dependency not found in the graph")]
    DependencyNotFound(
        Vec<DependencyId>,
        Box<DependencyGraph>,
        Option<HashMap<DependencyId, T>>,
    ),
    #[error("Invalid ids")]
    InvalidIds(Vec<usize>, SerializableGraph<T>),
    #[error("Couldn't add some of the dependencies")]
    CouldntAddDeps(Vec<DependencyGraphError>),
}

impl DependencyGraph {
    pub fn try_into_serializable<T>(
        self,
        metadata: Option<HashMap<DependencyId, T>>,
    ) -> Result<SerializableGraph<T>, SerializableGraphError<T>> {
        // Let's start with validating if each DependencyId is valid.
        if let Some(metadata_map) = &metadata {
            let nonexisting_ids: Vec<_> = metadata_map
                .iter()
                .filter(|(id, _)| self.is_in_graph(**id))
                .map(|(id, _)| *id)
                .collect();
            if !nonexisting_ids.is_empty() {
                return Err(SerializableGraphError::DependencyNotFound(
                    nonexisting_ids,
                    Box::new(self),
                    metadata,
                ));
            }
        }

        let (nodes_from_graph, edges_from_graph) = self.graph.into_nodes_edges_iters();

        let nodes = nodes_from_graph
            .map(|node| (node.index.index(), node.weight))
            .collect();
        let edges = edges_from_graph
            .map(|edge| {
                (
                    edge.source.index(),
                    edge.target.index(),
                    edge.weight.target_framework,
                )
            })
            .collect();
        let metadata = metadata.map(|met| {
            met.into_iter()
                .map(|(id, value)| (id.ix.index(), value))
                .collect()
        });
        Ok(SerializableGraph {
            nodes,
            edges,
            node_metadata: metadata,
        })
    }
}

type NodeMetadata<T> = Option<HashMap<DependencyId, T>>;

impl<T> SerializableGraph<T> {
    /// Recreate a `DependencyGraph` from a `SerializableGraph`.
    pub fn from_serializable(
        self,
    ) -> Result<(DependencyGraph, NodeMetadata<T>), SerializableGraphError<T>> {
        type NodeMapping = HashMap<usize, DependencyId>;

        let mut graph = DependencyGraph::new();

        // Collect valid node ids
        let ids: HashSet<usize> = self.nodes.iter().map(|(id, _)| *id).collect();

        // Find invalid ids referenced by metadata or edges
        let invalid_ids: Vec<usize> = self
            .node_metadata
            .iter()
            .flat_map(|m| m.keys().copied())
            .chain(self.edges.iter().map(|(from, _, _)| *from))
            .chain(self.edges.iter().map(|(_, to, _)| *to))
            .filter(|id| !ids.contains(id))
            .collect();

        if !invalid_ids.is_empty() {
            return Err(SerializableGraphError::InvalidIds(invalid_ids, self));
        }

        // Add nodes, collecting mapping from original usize id to new DependencyId.
        let mut mapping: NodeMapping = HashMap::new();
        let mut errors: Vec<DependencyGraphError> = Vec::new();

        for (index, info) in self.nodes {
            match graph.add_dependency(info) {
                Ok(id) => {
                    mapping.insert(index, id);
                }
                Err(e) => {
                    errors.push(e);
                }
            }
        }

        if !errors.is_empty() {
            return Err(SerializableGraphError::CouldntAddDeps(errors));
        }

        // Add edges using the mapping (mapping must contain all referenced ids).
        for (from, to, framework) in self.edges {
            let from_id = *mapping
                .get(&from)
                .expect("Validated ids missing from mapping");
            let to_id = *mapping
                .get(&to)
                .expect("Validated ids missing from mapping");
            graph
                .add_relation(from_id, to_id, framework)
                .expect("Ids have been validated in the previous steps.");
        }

        // Re-map metadata to use DependencyId keys.
        let metadata: NodeMetadata<T> = self.node_metadata.map(|met| {
            met.into_iter()
                .map(|(id, value)| (*mapping.get(&id).unwrap(), value))
                .collect()
        });

        Ok((graph, metadata))
    }
}

impl Default for DependencyGraph {
    fn default() -> Self {
        Self {
            graph: StableDiGraph::<DependencyInfo, DepEdge>::new(),
            id_by_name: HashMap::new(),
            frameworks: HashSet::new(),
        }
    }
}

#[derive(Error, Debug)]
pub enum DependencyGraphError {
    #[error("Dependency not found in the graph")]
    DependencyNotFound,

    #[error("Dependencies with same name but different types")]
    DifferentDependencyType,

    #[error("Graph operation failed: {message}")]
    GraphOperation { message: String },

    #[error("Merge failed for dependency '{name}': {reason}")]
    MergeFailed { name: String, reason: String },
}

impl DependencyGraph {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_project(
        &mut self,
        path: String,
        version: Option<String>,
    ) -> Result<DependencyId, DependencyGraphError> {
        let project = DependencyInfo::Project(ProjectInfo::new(path, version));
        self.add_dependency(project)
    }

    pub fn add_package(
        &mut self,
        name: String,
        version: Option<String>,
    ) -> Result<DependencyId, DependencyGraphError> {
        let lib = DependencyInfo::Package(PackageInfo::new(name, version));
        self.add_dependency(lib)
    }

    /// Ensures a dependency is in the graph. Returns id to it.
    ///
    /// Returns **Error** if there is already a dependency with same name but different type.
    fn add_dependency(
        &mut self,
        dependency: DependencyInfo,
    ) -> Result<DependencyId, DependencyGraphError> {
        let existing_versions = self.id_by_name.get(dependency.name());
        if let Some(vec) = existing_versions {
            if let Some(existing_id) = vec.first()
                && let Some(existing_dep) = self.graph.node_weight(existing_id.ix)
                && std::mem::discriminant(&dependency) != std::mem::discriminant(existing_dep)
            {
                return Err(DependencyGraphError::DifferentDependencyType);
            }

            if let Some(existing_id) = vec.iter().find(|dep| {
                self.graph
                    .node_weight(dep.ix)
                    .is_some_and(|x| *x == dependency)
            }) {
                return Ok(*existing_id);
            }
        }

        let name = dependency.name().to_string();
        let id = DependencyId::new(self.graph.add_node(dependency));
        self.id_by_name.entry(name).or_default().push(id);

        Ok(id)
    }

    /// Gets or creates a dependency (package or project) with the specified name and version.  
    /// - If a dependency with this name and version already exists - return it.  
    /// - If a dependency with this name exists (but different version) - create a new one with the same type and provided version and return it.  
    /// - Otherwise - return None  
    pub fn get_or_create_if_exists(
        &mut self,
        name: &str,
        version: Option<String>,
    ) -> Option<DependencyId> {
        if let Some(vec) = self.id_by_name.get(name) {
            if let Some(id) = vec.iter().find(|id| {
                self.get(**id)
                    .map(|info| info.version() == version.as_deref())
                    .unwrap_or(false)
            }) {
                return Some(*id);
            }

            if let Some(info) = vec.first().and_then(|id| self.get(*id)) {
                match info {
                    DependencyInfo::Package(_) => {
                        return self.add_package(name.to_string(), version).ok();
                    }
                    DependencyInfo::Project(_) => {
                        return self.add_project(name.to_string(), version).ok();
                    }
                }
            }
        }

        None
    }

    pub fn get(&self, id: DependencyId) -> Option<&DependencyInfo> {
        self.graph.node_weight(id.ix)
    }

    pub fn iter(&self) -> impl Iterator<Item = (DependencyId, &DependencyInfo)> {
        self.graph.node_indices().filter_map(move |ix| {
            self.graph
                .node_weight(ix)
                .map(|info| (DependencyId::new(ix), info))
        })
    }

    /// Get direct dependencies of the dependency.
    ///
    /// Returns **Error** if dependency with the provided id was not from this graph.
    fn get_direct_dependencies(
        &self,
        id: DependencyId,
    ) -> Result<impl Iterator<Item = &DepEdge>, DependencyGraphError> {
        if self.is_in_graph(id) {
            Ok(self.graph.edges(id.ix).map(|edge_ref| edge_ref.weight()))
        } else {
            Err(DependencyGraphError::DependencyNotFound)
        }
    }

    pub fn get_direct_dependencies_in_framework(
        &self,
        id: DependencyId,
        framework: &Framework,
    ) -> Result<impl Iterator<Item = &DepEdge>, DependencyGraphError> {
        Ok(self
            .get_direct_dependencies(id)?
            .filter(move |edge| edge.framework() == framework))
    }

    /// Get direct reverse dependencies of the dependency.
    ///
    /// Returns **Error** if dependency with the provided id was not from this graph.
    pub fn get_direct_reverse_dependencies(
        &self,
        id: DependencyId,
    ) -> Result<impl Iterator<Item = &DepEdge>, DependencyGraphError> {
        if self.is_in_graph(id) {
            Ok(self
                .graph
                .edges_directed(id.ix, petgraph::Direction::Incoming)
                .map(|edge| edge.weight()))
        } else {
            Err(DependencyGraphError::DependencyNotFound)
        }
    }

    pub fn add_relation(
        &mut self,
        from: DependencyId,
        to: DependencyId,
        framework: Framework,
    ) -> Result<(), DependencyGraphError> {
        self.frameworks.insert(framework.clone());
        let edge = DepEdge::new(from, to, framework);
        self.graph
            .try_add_edge(from.ix, to.ix, edge)
            .map(|_| ())
            .map_err(|err| match err {
                petgraph::graph::GraphError::NodeMissed(_) => {
                    DependencyGraphError::DependencyNotFound
                }
                _ => DependencyGraphError::GraphOperation {
                    message: format!("{:?}", err),
                },
            })
    }

    fn is_in_graph(&self, id: DependencyId) -> bool {
        self.graph.contains_node(id.ix)
    }

    pub fn iter_frameworks(&self) -> impl Iterator<Item = &Framework> {
        self.frameworks.iter()
    }

    pub fn layout(
        &self,
        vertex_size: &impl Fn(&DependencyId, &DependencyInfo) -> (f64, f64),
    ) -> Vec<super::algo::Layout<DependencyId>> {
        let vertex_size_fn = |ix: NodeIndex, dep: &DependencyInfo| -> (f64, f64) {
            let id = DependencyId::new(ix);
            vertex_size(&id, dep)
        };
        super::algo::layout_sugiyama(&self.graph, &vertex_size_fn)
            .into_iter()
            .map(|layout| {
                let map = layout
                    .positions
                    .into_iter()
                    .map(|(ix, pos)| (DependencyId::new(ix), pos))
                    .collect();
                super::algo::Layout::new(map, layout.width, layout.height)
            })
            .collect()
    }

    /// Merge another graph into this one atomically.
    pub fn merge(&mut self, graph: DependencyGraph) -> Result<(), DependencyGraphError> {
        // Check for type conflicts before merging
        let conflicting_name = graph.id_by_name.iter().find_map(|(name, other_ids)| {
            self.id_by_name.get(name).and_then(|self_ids| {
                other_ids
                    .first()
                    .zip(self_ids.first())
                    .and_then(|(other_id, self_id)| {
                        let other_dep = graph.graph.node_weight(other_id.ix)?;
                        let self_dep = self.graph.node_weight(self_id.ix)?;
                        (std::mem::discriminant(other_dep) != std::mem::discriminant(self_dep))
                            .then_some(name)
                    })
            })
        });

        if let Some(name) = conflicting_name {
            return Err(DependencyGraphError::MergeFailed {
                name: name.clone(),
                reason: "type conflict".to_string(),
            });
        }

        // Map from graph's DependencyId to self's DependencyId
        let mut id_map: HashMap<NodeIndex, DependencyId> = HashMap::new();

        // Add all dependencies from graph to self
        for (id, info) in graph.iter() {
            let new_id = match info {
                DependencyInfo::Project(proj) => self
                    .add_project(proj.path.clone(), proj.version.clone())
                    .unwrap(),
                DependencyInfo::Package(pkg) => self
                    .add_package(pkg.name.clone(), pkg.version.clone())
                    .unwrap(),
            };
            id_map.insert(id.ix, new_id);
        }

        // Add all edges from graph to self
        let (_, edges) = graph.graph.into_nodes_edges_iters();
        for edge_ref in edges {
            let from = id_map.get(&edge_ref.weight.from().ix).unwrap();
            let to = id_map.get(&edge_ref.weight.to().ix).unwrap();
            self.add_relation(*from, *to, edge_ref.weight.framework().clone())?;
        }

        Ok(())
    }
}

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
    let dep_info = graph.get(id).expect("Project should exist");
    match dep_info {
        DependencyInfo::Project(_) => {
            assert_eq!(dep_info.name(), project_path);
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
    assert!(graph.get(proj1).is_some());
    assert!(graph.get(proj2).is_some());
    assert!(graph.get(proj3).is_some());
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

    let dep_info = graph.get(id).expect("Package should exist");
    match dep_info {
        DependencyInfo::Package(_) => {
            assert_eq!(dep_info.name(), package_name);
            assert_eq!(dep_info.version(), version.as_deref());
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

    let dep_info = graph.get(id).expect("Package should exist");
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

    let result = graph.add_relation(proj1, proj2, framework.clone());

    assert!(result.is_ok());

    // Check that the relation was added
    let deps: Vec<_> = graph
        .get_direct_dependencies_in_framework(proj1, &framework)
        .unwrap()
        .collect();
    assert_eq!(deps.len(), 1);
    assert_eq!(deps[0].to(), proj2);
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

    let result = graph.add_relation(proj, pkg, framework.clone());

    assert!(result.is_ok());

    let deps: Vec<_> = graph
        .get_direct_dependencies_in_framework(proj, &framework)
        .unwrap()
        .collect();
    assert_eq!(deps.len(), 1);
    assert_eq!(deps[0].to(), pkg);
}

#[test]
fn test_add_relation_with_nonexistent_source() {
    let mut graph = DependencyGraph::new();

    let proj = graph
        .add_project("/path/to/proj.csproj".to_string(), None)
        .unwrap();
    let fake_id = DependencyId::new(NodeIndex::new(12));

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
    let fake_id = DependencyId::new(NodeIndex::new(67));

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

    graph.add_relation(proj, pkg1, framework.clone()).unwrap();
    graph.add_relation(proj, pkg2, framework.clone()).unwrap();
    graph.add_relation(proj, pkg3, framework.clone()).unwrap();

    let deps: Vec<_> = graph
        .get_direct_dependencies_in_framework(proj, &framework)
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

    graph.add_relation(proj, pkg1, net8.clone()).unwrap();
    graph.add_relation(proj, pkg2, net7.clone()).unwrap();

    let deps_net8: Vec<_> = graph
        .get_direct_dependencies_in_framework(proj, &net8)
        .unwrap()
        .collect();
    let deps_net7: Vec<_> = graph
        .get_direct_dependencies_in_framework(proj, &net7)
        .unwrap()
        .collect();

    assert_eq!(deps_net8.len(), 1);
    assert_eq!(deps_net8[0].to(), pkg1);

    assert_eq!(deps_net7.len(), 1);
    assert_eq!(deps_net7[0].to(), pkg2);
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

    graph.add_relation(proj1, proj2, framework.clone()).unwrap();

    // Get the edge from proj1 to proj2
    let deps: Vec<_> = graph
        .get_direct_dependencies_in_framework(proj1, &framework)
        .unwrap()
        .collect();

    assert_eq!(deps.len(), 1);
    let edge = deps[0];

    // Verify from and to are correct
    assert_eq!(edge.from(), proj1);
    assert_eq!(edge.to(), proj2);
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

    graph.add_relation(proj, pkg, framework.clone()).unwrap();

    let deps: Vec<_> = graph
        .get_direct_dependencies_in_framework(proj, &framework)
        .unwrap()
        .collect();

    assert_eq!(deps.len(), 1);
    let edge = deps[0];

    // Verify the edge correctly represents proj -> pkg
    assert_eq!(edge.from(), proj);
    assert_eq!(edge.to(), pkg);
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

    graph.add_relation(proj, pkg1, framework.clone()).unwrap();
    graph.add_relation(proj, pkg2, framework.clone()).unwrap();
    graph.add_relation(proj, pkg3, framework.clone()).unwrap();

    let deps: Vec<_> = graph
        .get_direct_dependencies_in_framework(proj, &framework)
        .unwrap()
        .collect();

    assert_eq!(deps.len(), 3);

    // All edges should have the same source (from)
    for edge in &deps {
        assert_eq!(edge.from(), proj);
    }

    // Collect all target IDs
    let target_ids: Vec<DependencyId> = deps.iter().map(|e| e.to()).collect();
    assert!(target_ids.contains(&pkg1));
    assert!(target_ids.contains(&pkg2));
    assert!(target_ids.contains(&pkg3));
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
    graph.add_relation(proj1, pkg, framework.clone()).unwrap();
    graph.add_relation(proj2, pkg, framework.clone()).unwrap();

    // Get reverse dependencies of the package
    let reverse_deps: Vec<_> = graph
        .get_direct_reverse_dependencies(pkg)
        .unwrap()
        .collect();

    assert_eq!(reverse_deps.len(), 2);

    // For reverse dependencies (incoming edges), the edges still have:
    // - from: the source that depends on pkg (proj1 or proj2)
    // - to: pkg
    for edge in &reverse_deps {
        assert_eq!(edge.to(), pkg);
    }

    let source_ids: Vec<DependencyId> = reverse_deps.iter().map(|e| e.from()).collect();
    assert!(source_ids.contains(&proj1));
    assert!(source_ids.contains(&proj2));
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
    graph.add_relation(app, lib, framework.clone()).unwrap();
    graph.add_relation(lib, pkg, framework.clone()).unwrap();

    // Check app -> lib edge
    let app_deps: Vec<_> = graph
        .get_direct_dependencies_in_framework(app, &framework)
        .unwrap()
        .collect();
    assert_eq!(app_deps.len(), 1);
    assert_eq!(app_deps[0].from(), app);
    assert_eq!(app_deps[0].to(), lib);

    // Check lib -> pkg edge
    let lib_deps: Vec<_> = graph
        .get_direct_dependencies_in_framework(lib, &framework)
        .unwrap()
        .collect();
    assert_eq!(lib_deps.len(), 1);
    assert_eq!(lib_deps[0].from(), lib);
    assert_eq!(lib_deps[0].to(), pkg);
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

    graph.add_relation(proj1, pkg, framework.clone()).unwrap();
    graph.add_relation(proj2, pkg, framework.clone()).unwrap();

    let reverse_deps: Vec<_> = graph
        .get_direct_reverse_dependencies(pkg)
        .unwrap()
        .collect();
    // We should have 2 reverse dependencies (proj1 and proj2 depend on pkg)
    assert_eq!(reverse_deps.len(), 2);

    // The edges contain the target ID (pkg), not the source IDs
    // So all reverse dep edges should point to pkg
    let reverse_dep_ids: Vec<DependencyId> = reverse_deps.iter().map(|e| e.to()).collect();
    assert!(reverse_dep_ids.iter().all(|id| *id == pkg));
}

#[test]
fn test_no_reverse_dependencies_for_root_node() {
    let mut graph = DependencyGraph::new();

    let proj = graph
        .add_project("/path/to/proj.csproj".to_string(), None)
        .unwrap();
    let pkg = graph.add_package("LeafPackage".to_string(), None).unwrap();

    let framework = Framework::new("net8.0".to_string());

    graph.add_relation(proj, pkg, framework.clone()).unwrap();

    // proj has no reverse dependencies (it's a root node)
    let reverse_deps: Vec<_> = graph
        .get_direct_reverse_dependencies(proj)
        .unwrap()
        .collect();
    assert_eq!(reverse_deps.len(), 0);
}

#[test]
fn test_get_direct_dependencies_returns_error_for_nonexistent_dependency() {
    let graph = DependencyGraph::new();
    let fake_id = DependencyId::new(NodeIndex::new(17));

    // This should return an error
    let non_existing_framework = Framework::new("net8.0".to_string());
    let result = graph.get_direct_dependencies_in_framework(fake_id, &non_existing_framework);

    assert!(result.is_err());
}

#[test]
fn test_get_reverse_dependencies_returns_error_for_nonexistent_dependency() {
    let graph = DependencyGraph::new();
    let fake_id = DependencyId::new(NodeIndex::new(17));

    // This should return an error
    let result = graph.get_direct_reverse_dependencies(fake_id);

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

    graph.add_relation(proj1, proj2, net8.clone()).unwrap();
    graph.add_relation(proj1, proj2, net7.clone()).unwrap();
    graph.add_relation(proj1, proj2, net6.clone()).unwrap();

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
fn test_package_id_equality() {
    let id1 = DependencyId::new(NodeIndex::new(1));
    let id2 = DependencyId::new(NodeIndex::new(1));
    let id3 = DependencyId::new(NodeIndex::new(2));
    let id4 = DependencyId::new(NodeIndex::new(5));

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
    graph.add_relation(app, lib1, framework.clone()).unwrap();
    graph.add_relation(app, lib2, framework.clone()).unwrap();

    // App depends on pkg1
    graph.add_relation(app, pkg1, framework.clone()).unwrap();

    // Lib1 depends on pkg2 and pkg3
    graph.add_relation(lib1, pkg2, framework.clone()).unwrap();
    graph.add_relation(lib1, pkg3, framework.clone()).unwrap();

    // Lib2 depends on pkg1 (shared dependency)
    graph.add_relation(lib2, pkg1, framework.clone()).unwrap();

    // Verify app dependencies
    let app_deps: Vec<_> = graph
        .get_direct_dependencies_in_framework(app, &framework)
        .unwrap()
        .collect();
    assert_eq!(app_deps.len(), 3);

    // Verify lib1 dependencies
    let lib1_deps: Vec<_> = graph
        .get_direct_dependencies_in_framework(lib1, &framework)
        .unwrap()
        .collect();
    assert_eq!(lib1_deps.len(), 2);

    // Verify lib2 dependencies
    let lib2_deps: Vec<_> = graph
        .get_direct_dependencies_in_framework(lib2, &framework)
        .unwrap()
        .collect();
    assert_eq!(lib2_deps.len(), 1);

    // Verify pkg1 has reverse dependencies
    let pkg1_reverse: Vec<_> = graph
        .get_direct_reverse_dependencies(pkg1)
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
    let fake_id = DependencyId::new(NodeIndex::new(5));

    assert!(graph.get(fake_id).is_none());
}

#[test]
fn test_merge_graphs() {
    let mut graph1 = DependencyGraph::new();
    let mut graph2 = DependencyGraph::new();

    // Add dependencies to first graph
    let proj1 = graph1
        .add_project("proj1.csproj".to_string(), None)
        .unwrap();
    let pkg1 = graph1
        .add_package("Package1".to_string(), Some("1.0.0".to_string()))
        .unwrap();

    // Add dependencies to second graph
    let proj2 = graph2
        .add_project("proj2.csproj".to_string(), None)
        .unwrap();
    let pkg2 = graph2
        .add_package("Package2".to_string(), Some("2.0.0".to_string()))
        .unwrap();

    // Add a relation in each graph
    let framework = Framework::new("net8.0".to_string());
    graph1.add_relation(proj1, pkg1, framework.clone()).unwrap();
    graph2.add_relation(proj2, pkg2, framework.clone()).unwrap();

    // Verify initial state
    assert_eq!(graph1.iter().count(), 2); // proj1, pkg1
    assert_eq!(graph2.iter().count(), 2); // proj2, pkg2

    // Merge graph2 into graph1
    graph1.merge(graph2).unwrap();

    // Verify merged state - should have all 4 dependencies
    assert_eq!(graph1.iter().count(), 4);

    // Check that all dependencies are present
    let deps: Vec<_> = graph1.iter().map(|(_, info)| info.name()).collect();
    assert!(deps.contains(&"proj1.csproj"));
    assert!(deps.contains(&"Package1"));
    assert!(deps.contains(&"proj2.csproj"));
    assert!(deps.contains(&"Package2"));

    // Check that old ids for the original graph work.
    assert!(
        graph1
            .get(proj1)
            .expect("Id from old graph should work")
            .name()
            == "proj1.csproj"
    );
    assert!(
        graph1
            .get(proj1)
            .expect("Id from old graph should work")
            .version()
            .is_none()
    );
    assert!(
        graph1
            .get(pkg1)
            .expect("Id from old graph should work")
            .name()
            == "Package1"
    );
    assert!(
        graph1
            .get(pkg1)
            .expect("Id from old graph should work")
            .version()
            == Some("1.0.0")
    );
}

#[test]
fn test_merge_graphs_with_common_dependencies() {
    let mut graph1 = DependencyGraph::new();
    let mut graph2 = DependencyGraph::new();

    let proj1 = graph1
        .add_project("proj1.csproj".to_string(), None)
        .unwrap();
    let pkg1 = graph1
        .add_package("Package1".to_string(), Some("1.0.0".to_string()))
        .unwrap();

    let proj2 = graph2
        .add_project("proj2.csproj".to_string(), None)
        .unwrap();
    let pkg2 = graph2
        .add_package("Package1".to_string(), Some("1.0.0".to_string()))
        .unwrap();

    // Add a relation in each graph
    let framework = Framework::new("net8.0".to_string());
    graph1.add_relation(proj1, pkg1, framework.clone()).unwrap();
    graph2.add_relation(proj2, pkg2, framework.clone()).unwrap();

    // Verify initial state
    assert_eq!(graph1.iter().count(), 2);
    assert_eq!(graph2.iter().count(), 2);

    // Merge graph2 into graph1
    graph1.merge(graph2).unwrap();

    assert_eq!(graph1.iter().count(), 3);

    // Check that all dependencies are present
    let deps: Vec<_> = graph1.iter().map(|(_, info)| info.name()).collect();
    assert!(deps.contains(&"proj1.csproj"));
    assert!(deps.contains(&"Package1"));
    assert!(deps.contains(&"proj2.csproj"));

    // Check if Package1 have reverse deps from both graphs.
    // Find proj2 in the merged graph (should exist after merge)
    let (proj2, _) = graph1
        .iter()
        .find(|x| x.1.name() == "proj2.csproj")
        .unwrap();

    // Both proj1 and proj2 should have reverse dependencies to pkg1 (with correct framework)
    let reverse_edges: Vec<_> = graph1
        .get_direct_reverse_dependencies(pkg1)
        .unwrap()
        .collect();

    // There should be two reverse edges: from proj1 and proj2 to pkg1
    assert_eq!(reverse_edges.len(), 2);

    // Check that both expected edges exist (ignoring edge instance identity, but matching from, to, and framework)
    let expected_sources: HashSet<_> = [proj1, proj2].into_iter().collect();
    let actual_sources: HashSet<_> = reverse_edges.iter().map(|e| e.from()).collect();
    assert_eq!(expected_sources, actual_sources);

    for edge in &reverse_edges {
        assert_eq!(edge.to(), pkg1);
        assert_eq!(edge.framework(), &Framework::new("net8.0".to_string()));
    }
}

#[test]
fn test_merge_graphs_on_different_type_should_fail() {
    let mut graph1 = DependencyGraph::new();
    let mut graph2 = DependencyGraph::new();

    // Add dependencies to first graph
    let proj1 = graph1
        .add_project("proj1.csproj".to_string(), None)
        .unwrap();
    let pkg1 = graph1
        .add_package("Package1".to_string(), Some("1.0.0".to_string()))
        .unwrap();

    // Add dependencies to second graph
    let proj2 = graph2
        .add_package("proj1.csproj".to_string(), None)
        .unwrap();
    let pkg2 = graph2
        .add_package("Package2".to_string(), Some("2.0.0".to_string()))
        .unwrap();

    // Add a relation in each graph
    let framework = Framework::new("net8.0".to_string());
    graph1.add_relation(proj1, pkg1, framework.clone()).unwrap();
    graph2.add_relation(proj2, pkg2, framework.clone()).unwrap();

    // Verify initial state
    assert_eq!(graph1.iter().count(), 2); // proj1, pkg1
    assert_eq!(graph2.iter().count(), 2); // proj2, pkg2

    // Merge graph2 into graph1
    let error = graph1.merge(graph2).unwrap_err();
    match error {
        DependencyGraphError::MergeFailed { reason, .. } => {
            assert!(
                reason.contains("type conflict"),
                "Expected type conflict, got: {}",
                reason
            );
        }
        DependencyGraphError::DifferentDependencyType => {
            // Acceptable, but MergeFailed is expected from merge
        }
        _ => panic!(
            "Expected MergeFailed or DifferentDependencyType, got: {:?}",
            error
        ),
    }
}
