use std::path::PathBuf;
use std::process::Command;

use std::sync::Once;
static INIT: Once = Once::new();

// Prepare simple dotnet project - clean and restore it.
// The function uses "execute once" synchronization.
pub(crate) fn prepare_simple_dotnet_project() -> PathBuf {
    let crate_root = std::env::current_dir().expect("Failed when getting current dir");
    let sln_dir = crate_root
        .join("tests")
        .join("data")
        .join("project_with_two_frameworks");
    INIT.call_once(|| {
        clean_dotnet_sln(&sln_dir).expect("dotnet clean failed during test setup");
        restore_dotnet_sln(&sln_dir).expect("dotnet restore failed during test setup");
    });

    sln_dir
}

fn clean_dotnet_sln(sln_path: &std::path::Path) -> std::io::Result<()> {
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

fn restore_dotnet_sln(sln_path: &std::path::Path) -> std::io::Result<()> {
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
