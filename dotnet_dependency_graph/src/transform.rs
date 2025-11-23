use nuget_dgspec_parser::graph::DependencyInfo;

pub fn get_display_text(info: &DependencyInfo) -> String {
    match info {
        DependencyInfo::Project(proj) => {
            // Extract just the project name from the full path
            if let Some(file_name) = std::path::Path::new(&proj.path).file_stem()
                && let Some(name_str) = file_name.to_str()
            {
                return name_str.to_string();
            }
            proj.path.clone()
        }
        DependencyInfo::Package(pck) => {
            format!("{}@{}", pck.name, pck.version.clone().unwrap_or_default())
        }
        DependencyInfo::Unknown(unknown) => {
            format!(
                "{}@{}",
                unknown.name,
                unknown.version.clone().unwrap_or_default()
            )
        }
    }
}
