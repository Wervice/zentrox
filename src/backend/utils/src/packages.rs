use serde::Serialize;
use utoipa::ToSchema;

use crate::sudo::{SudoCommand, SudoError, SudoOutput};
use std::{
    fmt::Display,
    process::{Command, Stdio},
};

struct CommandOutputStringified {
    stdout: String,
    stderr: String,
}

/// Run a command capturing it standard output and error, returning those values as Strings.
fn run_with_output(c: &mut Command) -> CommandOutputStringified {
    let x = c
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .stdin(Stdio::null())
        .spawn()
        .unwrap()
        .wait_with_output()
        .unwrap();

    CommandOutputStringified {
        stdout: stdout_to_string(x.stdout),
        stderr: stdout_to_string(x.stderr),
    }
}

#[derive(Debug)]
pub enum PackageManagerError {
    /// While executing a command with sudo, an error occurred
    SudoError,

    /// Executing a command failed
    ExecutionError,

    /// The package manager that was detected on the system is not supported
    UnsupportedPackageManager,
}

#[derive(Serialize, Debug, ToSchema)]
pub enum PackageManager {
    Apt,
    Dnf,
    Pacman,
}

impl Display for PackageManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PackageManager::Apt => f.write_str("apt"),
            PackageManager::Dnf => f.write_str("dnf"),
            PackageManager::Pacman => f.write_str("pacman"),
        }
    }
}

/// Try to run a command and return if it succeeded or failed
#[doc(hidden)]
fn try_command(c: &str) -> bool {
    Command::new(c).output().is_ok()
}

/// Detect which package manager to use by trying to run the commands for apt, dnf and pacman.
pub fn get_package_manager() -> Result<PackageManager, PackageManagerError> {
    if try_command("apt") {
        Ok(PackageManager::Apt)
    } else if try_command("dnf") {
        Ok(PackageManager::Dnf)
    } else if try_command("pacman") {
        Ok(PackageManager::Pacman)
    } else {
        Err(PackageManagerError::UnsupportedPackageManager)
    }
}

/// Remove all orphaned packages on the system, that are detected by the systems package manager to
/// be orphaned.
///
/// * `password` - Sudo password to execute with root privileges
pub fn remove_orphaned_packages(password: String) -> Result<SudoOutput, SudoError> {
    let package_mamager = get_package_manager().unwrap();

    let args = match package_mamager {
        PackageManager::Apt | PackageManager::Dnf => vec!["autoremove".to_string()],
        PackageManager::Pacman => {
            let packages =
                run_with_output(std::process::Command::new("pacman").arg("-Qdtq")).stdout;
            let packages_formatted = packages
                .split("\n")
                .filter(|x| !x.is_empty())
                .map(String::from)
                .collect::<Vec<String>>();
            [
                vec!["--noconfirm".to_string(), "-Rc".to_string()],
                packages_formatted,
            ]
            .concat()
        }
    };

    SudoCommand::new(password, package_mamager.to_string())
        .args(args)
        .output()
}

/// Install a package using the systems package manager.
///
/// * `name` - Name of the package to install
/// * `password` - Sudo password for root privileges
pub fn install_package(name: String, password: String) -> Result<SudoOutput, SudoError> {
    let package_manager = get_package_manager().unwrap();

    let args = match package_manager {
        PackageManager::Apt | PackageManager::Dnf => vec![
            "install".to_string(),
            name,
            "-y".to_string(),
            "-q".to_string(),
        ],
        PackageManager::Pacman => vec!["--noconfirm".to_string(), "-Sy".to_string(), name],
    };

    SudoCommand::new(password, package_manager.to_string())
        .args(args)
        .output()
}

/// Remove a package using the systems package manager.
///
/// * `name` - Name of the package to install
/// * `password` - Sudo password for root privileges
pub fn remove_package(name: String, password: String) -> Result<SudoOutput, SudoError> {
    let package_manager = get_package_manager().unwrap();

    let args = match package_manager {
        PackageManager::Apt | PackageManager::Dnf => vec![
            "remove".to_string(),
            name,
            "-y".to_string(),
            "-q".to_string(),
        ],
        PackageManager::Pacman => vec!["--noconfirm".to_string(), "-R".to_string(), name],
    };

    SudoCommand::new(password, package_manager.to_string())
        .args(args)
        .output()
}

/// Update a package using the systems package manager.
///
/// * `name` - Name of the package to install
/// * `password` - Sudo password for root privileges
pub fn update_package(name: String, password: String) -> Result<SudoOutput, SudoError> {
    let package_manager = get_package_manager().unwrap();

    let args = match package_manager {
        PackageManager::Apt => vec![
            "--only-upgrade".to_string(),
            "install".to_string(),
            name,
            "-y".to_string(),
            "-q".to_string(),
        ],
        PackageManager::Dnf => vec![
            "update".to_string(),
            name,
            "-y".to_string(),
            "-q".to_string(),
        ],
        PackageManager::Pacman => vec!["--noconfirm".to_string(), "-S".to_string(), name],
    };

    SudoCommand::new(password, package_manager.to_string())
        .args(args)
        .output()
}

/// Update all packages on the system using the systems package manager.
///
/// * `password` - Sudo password for root privileges.
pub fn update_all_packages(password: String) -> Result<SudoOutput, SudoError> {
    let package_manager = get_package_manager().unwrap();

    let args = match package_manager {
        PackageManager::Apt | PackageManager::Dnf => vec!["update", "-y", "-q"],
        PackageManager::Pacman => vec!["--noconfirm", "-Su"],
    };

    SudoCommand::new(password, package_manager.to_string())
        .args(args)
        .output()
}

/// Refresh the package managers repositories or database.
/// This is useful for detecting possible updates.
///
/// * `password` - Sudo password for root privileges
pub fn update_database(password: String) -> Result<SudoOutput, SudoError> {
    let package_manager = get_package_manager().unwrap();

    match package_manager {
        PackageManager::Apt => SudoCommand::new(password, "apt")
            .arg("update")
            .arg("-y")
            .output(),
        PackageManager::Dnf => SudoCommand::new(password, "dnf")
            .arg("makecache")
            .arg("-y")
            .output(),
        PackageManager::Pacman => SudoCommand::new(password, "pacman")
            .arg("-Syy")
            .arg("--noconfirm")
            .output(),
    }
}

/// Convert a Vec<u8> (vector of bytes) to a lossy String with UTF-8 encoding.
#[doc(hidden)]
fn stdout_to_string(bytes: Vec<u8>) -> String {
    String::from_utf8_lossy(&bytes).to_string()
}

/// List all packages installed on the system.
pub fn list_installed_packages() -> Vec<String> {
    let package_manager = get_package_manager().unwrap();

    match package_manager {
        PackageManager::Apt => {
            let command = run_with_output(Command::new("apt").arg("list").arg("--installed"));
            command
                .stdout
                .lines()
                .filter_map(|e| {
                    let entry = e.to_string();
                    let split = entry.split("/");
                    let collection = split.collect::<Vec<&str>>();
                    if collection.len() != 2 {
                        None
                    } else {
                        Some(collection[0].to_string())
                    }
                })
                .collect::<Vec<String>>()
        }
        PackageManager::Dnf => {
            let command = run_with_output(Command::new("dnf").arg("list").arg("installed"));
            command
                .stdout
                .lines()
                .filter_map(|e| {
                    let entry = e.to_string();
                    let split = entry.split(".");
                    let collection = split.collect::<Vec<&str>>();
                    if collection.len() != 2 {
                        None
                    } else {
                        Some(collection[0].to_string())
                    }
                })
                .collect::<Vec<String>>()
        }
        PackageManager::Pacman => {
            let command = run_with_output(Command::new("pacman").arg("--noconfirm").arg("-Qq"));
            command
                .stdout
                .lines()
                .filter_map(|e| {
                    let entry = e.to_string();
                    let split = entry.split("/");
                    let collection = split.collect::<Vec<&str>>();
                    if collection.len() != 1 {
                        None
                    } else {
                        Some(collection[0].to_string())
                    }
                })
                .collect::<Vec<String>>()
        }
    }
}

/// List all packages on the system that could be updated.
pub fn list_updates() -> Result<Vec<String>, PackageManagerError> {
    let package_manager = get_package_manager().unwrap();

    match package_manager {
        PackageManager::Apt => {
            let command = run_with_output(Command::new("apt").arg("list").arg("--upgradable"));
            Ok(command
                .stdout
                .lines()
                .filter_map(|e| {
                    let split = e.split("/");
                    let collection = split.collect::<Vec<&str>>();
                    if collection.len() != 2 {
                        None
                    } else {
                        Some(collection[0].to_string())
                    }
                })
                .collect::<Vec<String>>())
        }
        PackageManager::Dnf => {
            let command = run_with_output(Command::new("dnf").arg("list-update"));
            Ok(command
                .stdout
                .lines()
                .filter_map(|e| {
                    let split = e.split(".");
                    let collection = split.collect::<Vec<&str>>();
                    if collection.len() != 2 {
                        None
                    } else {
                        Some(collection[0].to_string())
                    }
                })
                .collect::<Vec<String>>())
        }
        PackageManager::Pacman => Err(PackageManagerError::UnsupportedPackageManager),
    }
}

/// List all packages that are available in the package managers repositories or database, but are
/// not installed.
pub fn list_available_packages() -> Result<Vec<String>, PackageManagerError> {
    let package_manager = get_package_manager().unwrap();

    match package_manager {
        PackageManager::Apt => {
            let command = run_with_output(Command::new("apt").arg("list"));

            let lines = &command.stdout.lines().filter(|x| !x.is_empty());
            let mut installed = Vec::new();
            let mut names = Vec::new();
            lines.clone().for_each(|l| {
                let line_s = l.split("/").collect::<Vec<&str>>();
                if line_s.len() != 2 {
                    return;
                };
                if line_s[1].contains("[installed") {
                    installed.push(line_s[0]);
                } else {
                    names.push(line_s[0])
                }
            });

            let mut vector = Vec::new();

            installed.sort();

            for e in names {
                if installed.binary_search(&e).is_err() {
                    vector.push(e.to_string());
                }
            }

            Ok(vector
                .iter()
                .cloned()
                .collect::<std::collections::HashSet<_>>()
                .into_iter()
                .collect::<Vec<String>>())
        }
        PackageManager::Dnf => {
            let command = run_with_output(Command::new("dnf").arg("list").arg("available"));
            let vector = &command
                .stdout
                .lines()
                .filter_map(|e| {
                    let split = e.split(".");
                    let collection = split.collect::<Vec<&str>>();
                    if collection.len() != 2 {
                        None
                    } else {
                        Some(collection[0].to_string())
                    }
                })
                .collect::<Vec<String>>();
            Ok(vector.to_vec())
        }
        PackageManager::Pacman => {
            let command = run_with_output(Command::new("pacman").arg("--noconfirm").arg("-Sl"));
            let vector = &command
                .stdout
                .lines()
                .filter_map(|e| {
                    let entry = e.to_string();
                    let split = entry.split(" ");
                    let collection = split.collect::<Vec<&str>>();
                    if !entry.contains("[installed") {
                        Some(collection[1].to_string())
                    } else {
                        None
                    }
                })
                .collect::<Vec<String>>();
            Ok(vector.to_vec())
        }
    }
}

/// List all packages that count as orphaned according to the package manager.
/// For more information look into `fn remove_orphaned_packages()`.
pub fn list_orphaned_packages() -> Result<Vec<String>, PackageManagerError> {
    let package_manager = get_package_manager().unwrap();

    match package_manager {
        PackageManager::Apt => {
            let command = run_with_output(Command::new("apt").arg("autoremove").arg("--dry-run"));
            let vector = &command
                .stdout
                .lines()
                .map(|e| {
                    let entry = e.to_string();
                    let split = entry.split(" - ");
                    let collection = split.collect::<Vec<&str>>();
                    if collection.len() != 2 {
                        String::from("")
                    } else {
                        collection[0].to_string()
                    }
                })
                .filter(|lines| lines.starts_with("Remv"))
                .collect::<Vec<String>>();
            Ok(vector.to_vec())
        }
        PackageManager::Dnf => {
            let command = run_with_output(Command::new("dnf").arg("repoquery").arg("--unneeded"));
            let vector = &command
                .stdout
                .lines()
                .filter_map(|e| {
                    let entry = e.to_string();
                    let split = entry.split(".");
                    let collection = split.collect::<Vec<&str>>();
                    if collection.len() != 2 {
                        None
                    } else {
                        Some(collection[0].to_string())
                    }
                })
                .collect::<Vec<String>>();
            Ok(vector.to_vec())
        }
        PackageManager::Pacman => {
            let command = run_with_output(Command::new("pacman").arg("--noconfirm").arg("-Qdtq"));
            let vector = &command
                .stdout
                .lines()
                .map(|e| e.to_string())
                .collect::<Vec<String>>();
            Ok(vector.to_vec())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{list_installed_packages, *};

    #[test]
    fn install_xterm() {
        let password =
            std::env::var("TEST_PASSWORD").expect("Requires TEST_PASSWORD environment variable");
        install_package("xterm".to_string(), password).expect("Failed to install xterm package");
    }

    #[test]
    fn remove_xterm() {
        let password =
            std::env::var("TEST_PASSWORD").expect("Requires TEST_PASSWORD environment variable");
        remove_package("xterm".to_string(), password).expect("Failed to install xterm package");
    }

    #[test]
    fn list_all_installed_packages() {
        list_installed_packages();
    }

    #[test]
    fn list_all_available_packages() {
        list_available_packages().expect("Failed to list available packages.");
    }

    #[test]
    fn list_all_orphaned_packages() {
        list_orphaned_packages().expect("Failed to list orphaned packages.");
    }

    #[test]
    fn list_all_outdated_packages() {
        list_updates().expect("Failed to list outdated packages.");
    }

    #[test]
    fn determine_package_manager() {
        get_package_manager().expect("Failed to list available packages.");
    }
}
