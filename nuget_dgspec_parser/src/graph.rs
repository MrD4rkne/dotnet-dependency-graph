use petgraph::stable_graph::{NodeIndex, StableDiGraph};
use std::collections::HashMap;
use std::hash::Hash;
use std::slice::Iter;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PackageId {
    pub name: String,
    pub version: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum DependencyInfo {
    Project { path: String },
    Package(PackageId),
}

#[derive(Debug, Clone)]
pub enum DependencyKind {
    Direct,
    Transitive,
    ProjectReference,
    FrameworkReference,
}

#[derive(Debug, Clone)]
pub struct DepEdge {
    pub kind: DependencyKind,
    pub version_req: Option<String>,
    pub target_framework: Option<String>,
    pub runtime: bool,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
struct DependencyId {
    id: usize,
}

impl DependencyId {
    fn new(id: usize) -> Self {
        Self { id }
    }
}

#[derive(Debug)]
struct IncrementingIdFactory {
    next_id: usize,
}

impl IncrementingIdFactory {
    fn new() -> Self {
        Self { next_id: 0 }
    }

    fn get_new_id(&mut self) -> DependencyId {
        let id = self.next_id.clone();
        self.next_id += 1;
        DependencyId::new(id)
    }
}

#[derive(Debug)]
pub struct DependencyGraph {
    graph: StableDiGraph<DependencyInfo, DepEdge>,

    node_by_id: HashMap<DependencyId, DependencyInfo>,
    index: HashMap<DependencyInfo, DependencyId>,
    adjacency: HashMap<DependencyId, Vec<(DependencyId, DepEdge)>>,

    id_factory: IncrementingIdFactory,
}

impl DependencyGraph {
    pub fn new() -> Self {
        Self {
            graph: StableDiGraph::<DependencyInfo, DepEdge>::new(),
            index: HashMap::new(),
            node_by_id: HashMap::new(),
            adjacency: HashMap::new(),
            id_factory: IncrementingIdFactory::new(),
        }
    }

    pub fn get_direct_deps(
        &self,
        key: &DependencyInfo,
    ) -> Option<Iter<'_, (DependencyId, DepEdge)>> {
        self.index
            .get(key)
            .and_then(|dep_id| self.adjacency.get(dep_id).map(|vec| vec.iter()))
    }

    fn add_dependency(&mut self, dependency: DependencyInfo) -> DependencyId {
        let id = self
            .index
            .entry(dependency.clone())
            .or_insert(self.id_factory.get_new_id());
        self.node_by_id.insert(id.clone(), dependency);
        id.clone()
    }
}
