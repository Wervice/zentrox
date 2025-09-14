use serde::Serialize;
use utoipa::ToSchema;

use crate::sudo::{SudoExecutionResult, SwitchedUserCommand};
use std::{
    fmt::Display,
    process::{Command, Stdio},
};

#[derive(Debug)]
pub enum PackageManagerError {
    /// While executing a command with sudo, an error occurred
    SudoError,

    /// Executing a command failed
    ExecutionError,

    /// The package manager that was detected on the system is not supported
    UnsupportedPackageManager,
}

impl Display for PackageManagerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Self::SudoError => f.write_str("Sudo failed"),
            Self::ExecutionError => f.write_str("Execution failed"),
            Self::UnsupportedPackageManager => f.write_str("Package manager not supported"),
        }
    }
}

// TODO Centralize commands using this enum and traits
#[derive(Serialize, Debug, ToSchema)]
pub enum PackageManagers {
    Apt,
    Dnf,
    Pacman,
}

/// Try to run a command and return if it succeeded or failed
#[doc(hidden)]
fn try_command(c: &str) -> bool {
    Command::new(c).spawn().is_ok()
}

/// Detect which package manager to use by trying to run the commands for apt, dnf and pacman.
pub fn get_package_manager() -> Result<PackageManagers, PackageManagerError> {
    if try_command("apt") {
        Ok(PackageManagers::Apt)
    } else if try_command("dnf") {
        Ok(PackageManagers::Dnf)
    } else if try_command("pacman") {
        Ok(PackageManagers::Pacman)
    } else {
        Err(PackageManagerError::UnsupportedPackageManager)
    }
}

/// Remove all orphaned packages on the system, that are detected by the systems package manager to
/// be orphaned.
///
/// * `password` - Sudo password to execute with root privileges
pub fn remove_orphaned_packages(password: String) -> Result<(), PackageManagerError> {
    let package_mamager = get_package_manager().unwrap();

    let command = match package_mamager {
        PackageManagers::Apt => "apt autoremove".to_string(),
        PackageManagers::Dnf => "dnf autoremove".to_string(),
        PackageManagers::Pacman => {
            let mut packages = String::from_utf8(
                std::process::Command::new("pacman")
                    .arg("-Qdtq")
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped())
                    .spawn()
                    .unwrap()
                    .wait_with_output()
                    .unwrap()
                    .stdout,
            )
            .unwrap()
            .replace("\n", " ");
            packages = packages.trim().to_string();
            format!("pacman --noconfirm -Rc {packages}")
        }
    };

    match SwitchedUserCommand::new(password, command.to_string()).spawn() {
        SudoExecutionResult::Success(_) => Ok(()),
        SudoExecutionResult::Unauthorized => Err(PackageManagerError::SudoError),
        SudoExecutionResult::WrongPassword => Err(PackageManagerError::SudoError),
        SudoExecutionResult::ExecutionError(_) => Err(PackageManagerError::ExecutionError),
    }
}

/// Install a package using the systems package manager.
///
/// * `name` - Name of the package to install
/// * `password` - Sudo password for root privileges
pub fn install_package(name: String, password: String) -> Result<(), PackageManagerError> {
    let package_manager = get_package_manager().unwrap();

    let command = match package_manager {
        PackageManagers::Apt => format!("apt install {name} -y -q"), // FIX direct string
        // interpolation without
        // sanitization
        PackageManagers::Dnf => format!("dnf install {name} -y -q"),
        PackageManagers::Pacman => format!("pacman --noconfirm -Sy {name}"),
    };

    match SwitchedUserCommand::new(password, command.to_string()).spawn() {
        SudoExecutionResult::Success(_) => Ok(()),
        SudoExecutionResult::Unauthorized => Err(PackageManagerError::SudoError),
        SudoExecutionResult::WrongPassword => Err(PackageManagerError::SudoError),
        SudoExecutionResult::ExecutionError(_) => Err(PackageManagerError::ExecutionError),
    }
}

/// Remove a package using the systems package manager.
///
/// * `name` - Name of the package to install
/// * `password` - Sudo password for root privileges
pub fn remove_package(name: String, password: String) -> Result<(), PackageManagerError> {
    let package_manager = get_package_manager().unwrap();

    let command = match package_manager {
        PackageManagers::Apt => format!("apt remove {name} -y -q"),
        PackageManagers::Dnf => format!("dnf remove {name} -y -q"),
        PackageManagers::Pacman => format!("pacman --noconfirm -R {name}"),
    };

    match SwitchedUserCommand::new(password, command.to_string()).spawn() {
        SudoExecutionResult::Success(_) => Ok(()),
        SudoExecutionResult::Unauthorized => Err(PackageManagerError::SudoError),
        SudoExecutionResult::WrongPassword => Err(PackageManagerError::SudoError),
        SudoExecutionResult::ExecutionError(_) => Err(PackageManagerError::ExecutionError),
    }
}

/// Update a package using the systems package manager.
///
/// * `name` - Name of the package to install
/// * `password` - Sudo password for root privileges
pub fn update_package(name: String, password: String) -> Result<(), PackageManagerError> {
    let package_manager = get_package_manager().unwrap();

    let command = match package_manager {
        PackageManagers::Apt => format!("apt --only-upgrade install {name} -y -q"),
        PackageManagers::Dnf => format!("dnf update {name} -y -q"),
        PackageManagers::Pacman => format!("pacman --noconfirm -S {name}"),
    };

    match SwitchedUserCommand::new(password, command.to_string()).spawn() {
        SudoExecutionResult::Success(_) => Ok(()),
        SudoExecutionResult::Unauthorized => Err(PackageManagerError::SudoError),
        SudoExecutionResult::WrongPassword => Err(PackageManagerError::SudoError),
        SudoExecutionResult::ExecutionError(_) => Err(PackageManagerError::ExecutionError),
    }
}

/// Update all packages on the system using the systems package manager.
///
/// * `password` - Sudo password for root privileges.
pub fn update_all_packages(password: String) -> Result<(), PackageManagerError> {
    let package_manager = get_package_manager().unwrap();

    let command = match package_manager {
        PackageManagers::Apt => "apt upgrade -y -q",
        PackageManagers::Dnf => "dnf update -y -q",
        PackageManagers::Pacman => "pacman --noconfirm -Su",
    };

    match SwitchedUserCommand::new(password, command.to_string()).spawn() {
        SudoExecutionResult::Success(_) => Ok(()),
        SudoExecutionResult::Unauthorized => Err(PackageManagerError::SudoError),
        SudoExecutionResult::WrongPassword => Err(PackageManagerError::SudoError),
        SudoExecutionResult::ExecutionError(_) => Err(PackageManagerError::ExecutionError),
    }
}

/// Refresh the package managers repositories or database.
/// This is useful for detecting possible updates.
///
/// * `password` - Sudo password for root privileges
pub fn update_database(password: String) -> Result<(), PackageManagerError> {
    let package_manager = get_package_manager().unwrap();

    let execution = match package_manager {
        PackageManagers::Apt => SwitchedUserCommand::new(password, "apt")
            .arg("update")
            .arg("-y")
            .spawn(),
        PackageManagers::Dnf => SwitchedUserCommand::new(password, "dnf")
            .arg("makecache")
            .arg("-y")
            .spawn(),
        PackageManagers::Pacman => SwitchedUserCommand::new(password, "pacman")
            .arg("-Syy")
            .arg("--noconfirm")
            .spawn(),
    };
    match execution {
        SudoExecutionResult::Success(_) => Ok(()),
        SudoExecutionResult::Unauthorized => Err(PackageManagerError::SudoError),
        SudoExecutionResult::WrongPassword => Err(PackageManagerError::SudoError),
        SudoExecutionResult::ExecutionError(_) => Err(PackageManagerError::ExecutionError),
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
        PackageManagers::Apt => {
            let command = Command::new("apt")
                .arg("list")
                .arg("--installed")
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .unwrap()
                .wait_with_output()
                .unwrap();
            let output = stdout_to_string(command.stdout);
            output
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
        PackageManagers::Dnf => {
            let command = Command::new("dnf")
                .arg("list")
                .arg("installed")
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .unwrap()
                .wait_with_output()
                .unwrap();
            let output = stdout_to_string(command.stdout);
            output
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
        PackageManagers::Pacman => {
            let command = Command::new("pacman")
                .arg("--noconfirm")
                .arg("-Qq")
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .unwrap()
                .wait_with_output()
                .unwrap();
            let output = stdout_to_string(command.stdout);
            output
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
        PackageManagers::Apt => {
            let command = Command::new("apt")
                .arg("list")
                .arg("--upgradable")
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .unwrap()
                .wait_with_output()
                .unwrap();
            let output = stdout_to_string(command.stdout);
            Ok(output
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
                .collect::<Vec<String>>())
        }
        PackageManagers::Dnf => {
            let command = Command::new("dnf")
                .arg("list-update")
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .unwrap()
                .wait_with_output()
                .unwrap();
            let output = stdout_to_string(command.stdout);
            Ok(output
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
                .collect::<Vec<String>>())
        }
        PackageManagers::Pacman => Err(PackageManagerError::UnsupportedPackageManager),
    }
}

/// List all packages that are available in the package managers repositories or database, but are
/// not installed.
pub fn list_available_packages() -> Result<Vec<String>, PackageManagerError> {
    let package_manager = get_package_manager().unwrap();

    match package_manager {
        PackageManagers::Apt => {
            let command = Command::new("apt")
                .arg("list")
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .unwrap()
                .wait_with_output()
                .unwrap();
            let output = stdout_to_string(command.stdout);
            let lines = &output.lines().filter(|x| !x.is_empty());
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
        PackageManagers::Dnf => {
            let command = Command::new("dnf")
                .arg("list")
                .arg("available")
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .unwrap()
                .wait_with_output()
                .unwrap();
            let output = stdout_to_string(command.stdout);
            let vector = &output
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
        PackageManagers::Pacman => {
            let command = Command::new("pacman")
                .arg("--noconfirm")
                .arg("-Sl")
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .unwrap()
                .wait_with_output()
                .unwrap();
            let output = stdout_to_string(command.stdout);
            let vector = &output
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
        PackageManagers::Apt => {
            let command = Command::new("apt")
                .arg("autoremove")
                .arg("--dry-run")
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .unwrap()
                .wait_with_output()
                .unwrap();
            let output = String::from_utf8_lossy(&command.stdout).to_string();
            let vector = &output
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
        PackageManagers::Dnf => {
            let command = Command::new("dnf")
                .arg("repoquery")
                .arg("--unneeded")
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .unwrap()
                .wait_with_output()
                .unwrap();
            let output = String::from_utf8_lossy(&command.stdout).to_string();
            let vector = &output
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
        PackageManagers::Pacman => {
            let command = Command::new("pacman")
                .arg("--noconfirm")
                .arg("-Qdtq")
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .unwrap()
                .wait_with_output()
                .unwrap();

            let output = String::from_utf8_lossy(&command.stdout).to_string();
            let vector = &output
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
