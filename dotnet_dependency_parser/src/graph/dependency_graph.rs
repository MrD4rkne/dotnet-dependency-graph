use petgraph::stable_graph::{NodeIndex, StableDiGraph};
use std::collections::{HashMap, HashSet};
use std::hash::Hash;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum DependencyId {
    ProjectId(String),
    PackageId(String, Option<String>),
}

pub trait DependencyWithId {
    fn id(&self) -> DependencyId;
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ProjectInfo {
    path: String,
    version: Option<String>,
}

impl ProjectInfo {
    fn new(path: String, version: Option<String>) -> Self {
        Self { path, version }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PackageInfo {
    name: String,
    version: Option<String>,
}

impl PackageInfo {
    fn new(name: String, version: Option<String>) -> Self {
        Self { name, version }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum DependencyInfo {
    Project(ProjectInfo),
    Package(PackageInfo),
}

impl DependencyWithId for DependencyInfo {
    fn id(&self) -> DependencyId {
        match self {
            DependencyInfo::Project(info) => DependencyId::ProjectId(info.path.clone()),
            DependencyInfo::Package(info) => {
                DependencyId::PackageId(info.name.clone(), info.version.clone())
            }
        }
    }
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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

    pub fn id(&self) -> &DependencyId {
        &self.to
    }

    pub fn from(&self) -> &DependencyId {
        &self.from
    }

    pub fn to(&self) -> &DependencyId {
        &self.to
    }

    pub fn framework(&self) -> &Framework {
        &self.target_framework
    }
}

#[derive(Debug)]
pub struct DependencyGraph {
    graph: StableDiGraph<DependencyId, DepEdge>,
    info: HashMap<DependencyId, DependencyInfo>,
    ix_by_id: HashMap<DependencyId, NodeIndex>,
    id_by_name: HashMap<String, Vec<DependencyId>>,
    frameworks: HashSet<Framework>,
}

impl Default for DependencyGraph {
    fn default() -> Self {
        Self {
            graph: StableDiGraph::<DependencyId, DepEdge>::new(),
            info: HashMap::new(),
            ix_by_id: HashMap::new(),
            id_by_name: HashMap::new(),
            frameworks: HashSet::new(),
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct DependencyNotFound;

impl std::fmt::Display for DependencyNotFound {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Dependency not found in the graph")
    }
}

impl std::error::Error for DependencyNotFound {}

#[derive(Debug, Default, Clone)]
pub struct DependencyCycle;

impl std::fmt::Display for DependencyCycle {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Dependency cycle detected")
    }
}

impl std::error::Error for DependencyCycle {}

#[derive(Debug, Default, Clone)]
pub struct DifferentDependencyType;

impl std::fmt::Display for DifferentDependencyType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Dependencies with same name but different types")
    }
}

impl std::error::Error for DifferentDependencyType {}

impl DependencyGraph {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_project(
        &mut self,
        path: String,
        version: Option<String>,
    ) -> Result<DependencyId, DifferentDependencyType> {
        let project = DependencyInfo::Project(ProjectInfo::new(path, version));
        self.add_dependency(project)
    }

    pub fn add_package(
        &mut self,
        name: String,
        version: Option<String>,
    ) -> Result<DependencyId, DifferentDependencyType> {
        let lib = DependencyInfo::Package(PackageInfo::new(name, version));
        self.add_dependency(lib)
    }

    /// Ensures a dependency is in the graph. Returns id to it.
    ///
    /// Returns **Error** if there is already a dependency with same name but different type.
    fn add_dependency(
        &mut self,
        dependency: DependencyInfo,
    ) -> Result<DependencyId, DifferentDependencyType> {
        let id = dependency.id();
        if self.info.contains_key(&id) {
            return Ok(id);
        }

        let existing_versions = self.id_by_name.get(dependency.name());
        if let Some(vec) = existing_versions
            && let Some(existing_id) = vec.first()
            && let Some(lib) = self.info.get(existing_id)
            && std::mem::discriminant(&dependency) != std::mem::discriminant(lib)
        {
            return Err(DifferentDependencyType);
        }

        self.id_by_name
            .entry(dependency.name().to_string())
            .or_default()
            .push(id.clone());
        self.info.insert(id.clone(), dependency);

        let node_ix = self.graph.add_node(id.clone());
        self.ix_by_id.insert(id.clone(), node_ix);

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
    ) -> Option<&DependencyInfo> {
        if let Some(vec) = self.id_by_name.get(name) {
            if let Some(id) = vec.iter().find(|id| {
                self.info.get(id).is_some_and(|info| match info {
                    DependencyInfo::Package(p) => p.version == version,
                    DependencyInfo::Project(p) => p.version == version,
                })
            }) {
                return self.info.get(id);
            }

            if let Some(info) = vec.first().and_then(|id| self.get(id)) {
                match info {
                    DependencyInfo::Package(_) => {
                        let id = self.add_package(name.to_string(), version).ok()?;
                        return self.info.get(&id);
                    }
                    DependencyInfo::Project(_) => {
                        let id = self.add_project(name.to_string(), version).ok()?;
                        return self.info.get(&id);
                    }
                }
            }
        }

        None
    }

    pub fn get(&self, id: &DependencyId) -> Option<&DependencyInfo> {
        self.info.get(id)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&DependencyId, &DependencyInfo)> {
        self.info.iter()
    }

    /// Get direct dependencies of the dependency.
    ///
    /// Returns **Error** if dependency with the provided id was not from this graph.
    fn get_direct_dependencies(
        &self,
        id: &DependencyId,
    ) -> Result<impl Iterator<Item = &DepEdge>, DependencyNotFound> {
        if let Some(index) = self.ix_by_id.get(id) {
            Ok(self
                .graph
                .edges_directed(*index, petgraph::Direction::Outgoing)
                .map(|edge| edge.weight()))
        } else {
            Err(DependencyNotFound)
        }
    }

    pub fn get_direct_dependencies_in_framework(
        &self,
        id: &DependencyId,
        framework: &Framework,
    ) -> Result<impl Iterator<Item = &DepEdge>, DependencyNotFound> {
        Ok(self
            .get_direct_dependencies(id)?
            .filter(move |edge| edge.framework() == framework))
    }

    /// Get direct reverse dependencies of the dependency.
    ///
    /// Returns **Error** if dependency with the provided id was not from this graph.
    pub fn get_direct_reverse_dependencies(
        &self,
        id: &DependencyId,
    ) -> Result<impl Iterator<Item = &DepEdge>, DependencyNotFound> {
        if let Some(index) = self.ix_by_id.get(id) {
            Ok(self
                .graph
                .edges_directed(*index, petgraph::Direction::Incoming)
                .map(|edge| edge.weight()))
        } else {
            Err(DependencyNotFound)
        }
    }

    pub fn add_relation(
        &mut self,
        from: DependencyId,
        to: DependencyId,
        framework: Framework,
    ) -> Result<(), DependencyNotFound> {
        let source = self.ix_by_id.get(&from);
        let dependency = self.ix_by_id.get(&to);

        if let Some(source) = source
            && let Some(target) = dependency
        {
            _ = self.frameworks.insert(framework.clone());

            let edge = DepEdge::new(from, to, framework);
            _ = self.graph.add_edge(*source, *target, edge);

            return Ok(());
        }

        Err(DependencyNotFound)
    }

    pub fn iter_frameworks(&self) -> impl Iterator<Item = &Framework> {
        self.frameworks.iter()
    }

    pub fn layout(
        &self,
        vertex_size: &impl Fn(&DependencyId, &DependencyInfo) -> (f64, f64),
    ) -> Vec<super::algo::Layout<DependencyId>> {
        let vertex_size_fn = |_: NodeIndex, id: &DependencyId| -> (f64, f64) {
            let dep = self
                .info
                .get(id)
                .expect("Dependency info from graph should be in info");
            vertex_size(id, dep)
        };
        super::algo::layout_sugiyama(&self.graph, &vertex_size_fn)
    }
}
