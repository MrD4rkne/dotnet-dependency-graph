use dotnet_dependency_parser::graph::DependencyInfo;

pub(crate) fn get_display_text(dep: &DependencyInfo) -> &str {
    dep.name()
}
