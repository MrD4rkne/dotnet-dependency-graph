use std::process::Command;

pub fn clean_dotnet_sln(sln_path: &std::path::Path) -> std::io::Result<()> {
    let status = Command::new("dotnet")
        .arg("clean")
        .current_dir(sln_path)
        .status()?;
    match status.success() {
        true => Ok(()),
        false => Err(std::io::Error::other(format!(
            "Dotnet clean failed with exit status: {}",
            status
        ))),
    }
}

pub fn restore_dotnet_sln(sln_path: &std::path::Path) -> std::io::Result<()> {
    let status = Command::new("dotnet")
        .arg("restore")
        .current_dir(sln_path)
        .status()?;
    match status.success() {
        true => Ok(()),
        false => Err(std::io::Error::other(format!(
            "Dotnet restore failed with exit status: {}",
            status
        ))),
    }
}
