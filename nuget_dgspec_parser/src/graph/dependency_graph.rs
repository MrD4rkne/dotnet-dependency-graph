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
    pub path: String,
}

impl ProjectInfo {
    fn new(path: String) -> Self {
        Self { path }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PackageInfo {
    pub name: String,
    pub version: Option<String>,
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Framework {
    name: String,
}

impl Framework {
    pub fn new(name: String) -> Self {
        Self { name }
    }

    pub fn get_name(&self) -> &String {
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

    pub fn get_id(&self) -> &DependencyId {
        &self.to
    }

    pub fn get_framework(&self) -> &Framework {
        &self.target_framework
    }
}

#[derive(Debug)]
pub struct DependencyGraph {
    graph: StableDiGraph<DependencyId, DepEdge>,
    info: HashMap<DependencyId, DependencyInfo>,
    ix_by_id: HashMap<DependencyId, NodeIndex>,
    frameworks: HashSet<Framework>,
}

impl Default for DependencyGraph {
    fn default() -> Self {
        Self {
            graph: StableDiGraph::<DependencyId, DepEdge>::new(),
            info: HashMap::new(),
            ix_by_id: HashMap::new(),
            frameworks: HashSet::new(),
        }
    }
}

#[derive(Debug, Default)]
pub struct DependencyNotFound;

#[derive(Debug, Default)]
pub struct DependencyCycle;

impl DependencyGraph {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_project(&mut self, path: String) -> DependencyId {
        let project = DependencyInfo::Project(ProjectInfo::new(path));
        self.add_dependency(project)
    }

    pub fn add_package(&mut self, name: String, version: Option<String>) -> DependencyId {
        let lib = DependencyInfo::Package(PackageInfo::new(name, version));
        self.add_dependency(lib)
    }

    /// Ensures a dependency is in the graph. Returns id to it.
    fn add_dependency(&mut self, dependency: DependencyInfo) -> DependencyId {
        let dependency = self.info.entry(dependency.id()).or_insert_with(|| {
            self.ix_by_id
                .insert(dependency.id(), self.graph.add_node(dependency.id()));
            dependency
        });
        dependency.id()
    }

    pub fn get(&self, id: &DependencyId) -> Option<&DependencyInfo> {
        self.info.get(id)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&DependencyId, &DependencyInfo)> {
        self.info.iter()
    }

    /// Get direct dependencies of the dependency.
    ///
    /// **Panics** if dependency with the provided id was not from this graph.
    fn get_direct_dependencies(&self, id: &DependencyId) -> impl Iterator<Item = &DepEdge> {
        if let Some(index) = self.ix_by_id.get(id) {
            self.graph
                .edges_directed(*index, petgraph::Direction::Outgoing)
                .map(|edge| edge.weight())
        } else {
            panic!("The dependency is not available in the graph");
        }
    }

    pub fn get_direct_dependencies_in_framework(
        &self,
        id: &DependencyId,
        framework: Framework,
    ) -> impl Iterator<Item = &DepEdge> {
        self.get_direct_dependencies(id)
            .filter(move |edge| *edge.get_framework() == framework)
    }

    /// Get direct reverse dependencies of the dependency.
    ///
    /// **Panics** if dependency with the provided id was not from this graph.
    pub fn get_direct_reverse_dependencies(
        &self,
        id: &DependencyId,
    ) -> impl Iterator<Item = &DepEdge> {
        if let Some(index) = self.ix_by_id.get(id) {
            self.graph
                .edges_directed(*index, petgraph::Direction::Incoming)
                .map(|edge| edge.weight())
        } else {
            panic!("The dependency is not available in the graph");
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
        let vertex_size = |_: NodeIndex, id: &DependencyId| -> (f64, f64) {
            let dep = self.get(id).expect("Node from graph should be in info");
            vertex_size(id, dep)
        };
        super::algo::layout_sugiyama(&self.graph, &vertex_size)
    }
}
