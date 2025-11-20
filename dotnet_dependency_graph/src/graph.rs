use std::collections::{HashMap, HashSet};

#[derive(Hash, PartialEq, Eq, Clone, Debug)]
pub struct Id {
    name: String,
}

#[derive(Debug)]
pub struct Project {
    pub id: Id,
    pub name: String,
    pub frameworks: HashMap<Framework, FrameworkEntry>,
    pub reverse_dependencies: HashSet<(Framework, Id)>,
}

#[derive(Hash, PartialEq, Eq, Clone, Debug)]
pub struct Framework {
    pub id: String,
}

#[derive(PartialEq, Eq, Debug)]
pub struct FrameworkEntry {
    pub dependencies: HashSet<Id>,
}

impl Id {
    fn new(name: String) -> Self {
        Self { name }
    }
}

impl Project {
    pub fn new(name: String) -> Self {
        Self {
            id: Id::new(name.clone()),
            name,
            frameworks: HashMap::new(),
            reverse_dependencies: HashSet::new(),
        }
    }
}

impl Framework {
    pub fn new(id: String) -> Self {
        Self { id }
    }
}

impl FrameworkEntry {
    fn new() -> Self {
        Self {
            dependencies: HashSet::new(),
        }
    }
}

#[derive(Debug)]
pub struct ProjectTree {
    projects: HashMap<Id, Project>,
    frameworks: HashSet<Framework>,
}

impl ProjectTree {
    pub fn new() -> Self {
        Self {
            projects: HashMap::new(),
            frameworks: HashSet::new(),
        }
    }

    pub fn link_projects(&mut self, project: &Id, dependency: &Id, framework: &Framework) {
        let src = self
            .projects
            .get_mut(project)
            .expect("Project is not defined inside the tree");
        src.frameworks
            .entry(framework.clone())
            .or_insert(FrameworkEntry::new())
            .dependencies
            .insert(dependency.clone());

        let dep = self
            .projects
            .get_mut(dependency)
            .expect("Dependency is not defined inside the tree");
        dep.reverse_dependencies
            .insert((framework.clone(), project.clone()));

        self.frameworks.insert(framework.clone());
    }

    pub fn get(&self, project: &Id) -> Option<&Project> {
        self.projects.get(project)
    }

    pub fn insert(&mut self, name: String) -> Id {
        let new_project = Project::new(name);
        let id = new_project.id.clone();

        self.projects.entry(id.clone()).or_insert(new_project);
        id
    }

    pub fn iter(&self) -> impl Iterator<Item = (&Id, &Project)> {
        self.projects.iter()
    }

    pub fn frameworks_iter(&self) -> impl Iterator<Item = &Framework> {
        self.frameworks.iter()
    }
}
