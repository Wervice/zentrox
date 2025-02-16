/// APT, DNF, PacMan bindings to
/// install packages, remove package, list installed/available/unnecessary packages
use crate::sudo::{SudoExecutionResult, SwitchedUserCommand};
use std::{fs, process::Command, process::Stdio};

#[derive(Debug)]
pub enum PackageManagerError {
    SudoError,
    UnknownPackageManager,
    ExecutionError
}

/// Determines which package manager is used by the system.
///
/// The only supported package managers are apt (debian-based), dnf (rhel-based) and pacman
/// (arch-based)
/// If no supported package manager is founed None will be returned.
/// package managers, an Err is returned.
pub fn get_package_manager() -> Option<String> {
    let os_release = fs::read_to_string("/etc/os-release").unwrap();
    let mut id = "".to_string();
    for line in os_release.lines() {
        if line.starts_with("ID=") {
            id = line.split("ID=").nth(1).unwrap_or("").to_string();
        }
    }
    let debain_distros = [
        "debian",
        "linuxmint",
        "ubuntu",
        "xubuntu",
        "lubuntu",
        "zorin",
        "elementary",
        "kali",
        "parrot",
        "deepin",
        "pop",
        "raspberryos",
    ];
    let redhat_distros = ["rhel", "fedora", "centos", "mageia"];
    let arch_distros = ["arch", "manajaro", "arch", "blackarch"];
    if debain_distros.contains(&id.as_str()) {
        Some("apt".to_string())
    } else if redhat_distros.contains(&id.as_str()) {
        Some("dnf".to_string())
    } else if arch_distros.contains(&id.as_str()) {
        Some("pacman".to_string())
    } else {
        // Use commands to see which ones are supported.
        if Command::new("apt").spawn().is_ok() {
            Some("apt".to_string())
        } else if Command::new("dnf").spawn().is_ok() {
            Some("dnf".to_string())
        } else if Command::new("pacman").spawn().is_ok() {
            Some("pacman".to_string())
        } else {
            None
        }
    }
}

/// Autoremvoes every package the systems says is not relevant.
///
/// This only works on apt, dnf and pacman based systems.
/// If the function is called on a system that does not use one of the descriped
/// package managers, an Err is returned.
/// * `password` - Password used to run sudo

pub fn remove_orphaned_packages(password: String) -> Result<(), PackageManagerError> {
    let package_mamager = get_package_manager().unwrap();

    let command: String;

    if package_mamager == "apt" {
        command = "apt autoremove".to_string();
    } else if package_mamager == "dnf" {
        command = "dnf autoremove".to_string();
    } else if package_mamager == "pacman" {
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
        command = format!("pacman --noconfirm -Rc {}", packages);
    } else {
        return Err(PackageManagerError::UnknownPackageManager);
    }

    match SwitchedUserCommand::new(password, command.to_string()).spawn() {
        SudoExecutionResult::Success(_) => Ok(()),
        SudoExecutionResult::Unauthorized => Err(PackageManagerError::SudoError),
        SudoExecutionResult::WrongPassword => Err(PackageManagerError::SudoError),
        SudoExecutionResult::ExecutionError(_) => Err(PackageManagerError::ExecutionError)
    }
}

/// Installs package on the system.
///
/// This only works on apt, dnf and pacman based systems.
/// If the function is called on a system that does not use one of the descriped
/// package managers, an Err is returned.
/// * `name` - Name of the package
/// * `password` - Password used to run sudo
pub fn install_package(name: String, password: String) -> Result<(), PackageManagerError> {
    let package_manager = get_package_manager().unwrap();

    let command;

    if package_manager == "apt" {
        command = format!("apt install {} -y -q", name);
    } else if package_manager == "dnf" {
        command = format!("dnf install {} -y -q", name);
    } else if package_manager == "pacman" {
        command = format!("pacman --noconfirm -Sy {}", name);
    } else {
        return Err(PackageManagerError::UnknownPackageManager);
    }

     match SwitchedUserCommand::new(password, command.to_string()).spawn() {
        SudoExecutionResult::Success(_) => Ok(()),
        SudoExecutionResult::Unauthorized => Err(PackageManagerError::SudoError),
        SudoExecutionResult::WrongPassword => Err(PackageManagerError::SudoError),
        SudoExecutionResult::ExecutionError(_) => Err(PackageManagerError::ExecutionError)
    }
}

/// Removes package from the system.
///
/// This only works on apt, dnf and pacman based systems.
/// If the function is called on a system that does not use one of the descriped
/// package managers, an Err is returned.
///
/// * `name` - Name of the package
/// * `password` - Password used to run sudo
pub fn remove_package(name: String, password: String) -> Result<(), PackageManagerError> {
    let package_manager = get_package_manager().unwrap();

    let command;

    if package_manager == "apt" {
        command = format!("apt remove {} -y -q", name);
    } else if package_manager == "dnf" {
        command = format!("dnf remove {} -y -q", name);
    } else if package_manager == "pacman" {
        command = format!("pacman --noconfirm -R {}", name)
    } else {
        return Err(PackageManagerError::UnknownPackageManager);
    }

    
     match SwitchedUserCommand::new(password, command.to_string()).spawn() {
        SudoExecutionResult::Success(_) => Ok(()),
        SudoExecutionResult::Unauthorized => Err(PackageManagerError::SudoError),
        SudoExecutionResult::WrongPassword => Err(PackageManagerError::SudoError),
        SudoExecutionResult::ExecutionError(_) => Err(PackageManagerError::ExecutionError)
    }

}

/// Update a package to the next version.
///
/// This only works on apt, dnf and pacman based systems.
/// If the function is called on a system that does not use one of the descriped
/// package managers, an Err is returned.
///
/// * `name` - Name of the package
/// * `password` - Password used to run sudo
pub fn update_package(name: String, password: String) -> Result<(), PackageManagerError> {
    let package_manager = get_package_manager().unwrap();

    let command;

    if package_manager == "apt" {
        command = format!("apt --only-upgrade install {} -y -q", name);
    } else if package_manager == "dnf" {
        command = format!("dnf update {} -y -q", name);
    } else if package_manager == "pacman" {
        command = format!("pacman --noconfirm -S {}", name)
    } else {
        return Err(PackageManagerError::UnknownPackageManager);
    }

       
     match SwitchedUserCommand::new(password, command.to_string()).spawn() {
        SudoExecutionResult::Success(_) => Ok(()),
        SudoExecutionResult::Unauthorized => Err(PackageManagerError::SudoError),
        SudoExecutionResult::WrongPassword => Err(PackageManagerError::SudoError),
        SudoExecutionResult::ExecutionError(_) => Err(PackageManagerError::ExecutionError)
    }
}

pub fn update_all_packages(password: String) -> Result<(), PackageManagerError> {
    let package_manager = get_package_manager().unwrap();

    let command;
    if package_manager == "apt" {
        command = format!("apt upgrade -y -q");
    } else if package_manager == "dnf" {
        command = format!("dnf update -y -q");
    } else if package_manager == "pacman" {
        command = format!("pacman --noconfirm -Su")
    } else {
        return Err(PackageManagerError::UnknownPackageManager);
    }

    
       
     match SwitchedUserCommand::new(password, command.to_string()).spawn() {
        SudoExecutionResult::Success(_) => Ok(()),
        SudoExecutionResult::Unauthorized => Err(PackageManagerError::SudoError),
        SudoExecutionResult::WrongPassword => Err(PackageManagerError::SudoError),
        SudoExecutionResult::ExecutionError(_) => Err(PackageManagerError::ExecutionError)
    }

}

pub fn update_database(password: String) -> Result<(), PackageManagerError> {
    let package_manager = get_package_manager().unwrap();

    if package_manager == "apt" {
        match SwitchedUserCommand::new(password, "apt").arg("update").arg("-y").spawn() {
            SudoExecutionResult::Success(_) => Ok(()),
        SudoExecutionResult::Unauthorized => Err(PackageManagerError::SudoError),
        SudoExecutionResult::WrongPassword => Err(PackageManagerError::SudoError),
        SudoExecutionResult::ExecutionError(_) => Err(PackageManagerError::ExecutionError)

        }

    } else if package_manager == "dnf" {
        match SwitchedUserCommand::new(password, "dnf").arg("makecache").arg("-y").spawn() {
            
            SudoExecutionResult::Success(_) => Ok(()),
        SudoExecutionResult::Unauthorized => Err(PackageManagerError::SudoError),
        SudoExecutionResult::WrongPassword => Err(PackageManagerError::SudoError),
        SudoExecutionResult::ExecutionError(_) => Err(PackageManagerError::ExecutionError)

        }

    } else if package_manager == "pacman" {
        match SwitchedUserCommand::new(password, "pacman").arg("-Syy").arg("--noconfirm").spawn() {
            
            SudoExecutionResult::Success(_) => Ok(()),
        SudoExecutionResult::Unauthorized => Err(PackageManagerError::SudoError),
        SudoExecutionResult::WrongPassword => Err(PackageManagerError::SudoError),
        SudoExecutionResult::ExecutionError(_) => Err(PackageManagerError::ExecutionError)
        }
    } else {
        return Err(PackageManagerError::UnknownPackageManager);
    }

}

/// List every package, the package manager says is installed
///
/// The supported package managers are apt, dnf and pacman.
/// If the function is used on a distro with an unsupported package manager,
/// the function will return an Err.
///
/// # Example
/// ```
/// packages::list_installed_packages().unwrap()
/// // Returns vector of the names of all installed packages
///
/// ```
pub fn list_installed_packages() -> Result<Vec<String>, PackageManagerError> {
    let package_manager = get_package_manager().unwrap();
    if package_manager == "apt" {
        let command = Command::new("apt")
            .arg("list")
            .arg("--installed")
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
                let split = entry.split("/");
                let collection = split.collect::<Vec<&str>>();
                if collection.len() != 2 {
                    None
                } else {
                    Some(collection[0].to_string())
                }
            })
            .collect::<Vec<String>>();
        Ok(vector.to_vec())
    } else if package_manager == "dnf" {
        let command = Command::new("dnf")
            .arg("list")
            .arg("installed")
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
    } else if package_manager == "pacman" {
        let command = Command::new("pacman")
            .arg("--noconfirm")
            .arg("-Qq")
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
                let split = entry.split("/");
                let collection = split.collect::<Vec<&str>>();
                if collection.len() != 1 {
                    None
                } else {
                    Some(collection[0].to_string())
                }
            })
            .collect::<Vec<String>>();
        Ok(vector.to_vec())
    } else {
        return Err(PackageManagerError::UnknownPackageManager);
    }
}

/// List every package, that is out-dated according to the package manager.
/// This binding does *not* support pacman, as it would require a sudo password
///
/// Commands:
/// * `apt list --upgradable`
/// * `dnf list-update`
pub fn list_updates() -> Result<Vec<String>, PackageManagerError> {
    let package_manager = get_package_manager().unwrap();
    if package_manager == "apt" {
        let command = Command::new("apt")
            .arg("list")
            .arg("--upgradable")
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
                let split = entry.split("/");
                let collection = split.collect::<Vec<&str>>();
                if collection.len() != 2 {
                    None
                } else {
                    Some(collection[0].to_string())
                }
            })
            .collect::<Vec<String>>();
        Ok(vector.to_vec())
    } else if package_manager == "dnf" {
        let command = Command::new("dnf")
            .arg("list-update")
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
    } else if package_manager == "pacman" {
        let command = Command::new("pacman")
            .arg("-Qu")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap()
            .wait_with_output()
            .unwrap();
        let output = String::from_utf8_lossy(&command.stdout).to_string();
        let vector = &output
            .lines()
            .map(|e| e.split(" ").nth(0).unwrap_or(e).to_string())
            .collect::<Vec<String>>();

        Ok(vector.to_vec())

    } else {
        return Err(PackageManagerError::UnknownPackageManager);
    }
}

/// List all packages that are installed on the system.
///
/// This function only supports apt, dnf and pacman.
/// If a the function is called on a system that uses another package manager,
/// an error will be returned.
///
/// The function works the following:
/// ## APT
/// 1. `apt list` is called without root permissions.
/// 2. The result is split into lines and empty lines are removed.
/// 3. Two vectors are created. One (installed) list every package which appeared in the previous'
///    command output with the string [installed **(no closing ])**. Another vector (names) keeps
///    track over every package that does not have the strong [installed.
///    This is necessary due to apt treating packages for another arch as different packages, but
///    the function is not meant to treat those differently.
/// 4. Packages that are not in installed but in names are added to another vector called "vector".
/// 5. Vector gets returned
///
/// ## DNF
/// 1. `dnf list available` is called without root permissions.
/// 2. The result is split into lines.
/// 3. Everything before "Available Packages" is skipped.
/// 4. The package names are extracted from every line and are returned.
///
/// ## PacMan
/// 1. `pacman -Sl` is called without root permissions.
/// 2. The output is split into lines and returned as a vector.
pub fn list_available_packages() -> Result<Vec<String>, PackageManagerError> {
    let package_manager = get_package_manager().unwrap();
    if package_manager == "apt" {
        let command = Command::new("apt")
            .arg("list")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap()
            .wait_with_output()
            .unwrap();
        let output = String::from_utf8_lossy(&command.stdout).to_string();
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
    } else if package_manager == "dnf" {
        let command = Command::new("dnf")
            .arg("list")
            .arg("available")
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
    } else if package_manager == "pacman" {
        let command = Command::new("pacman")
            .arg("--noconfirm")
            .arg("-Sl")
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
    } else {
        return Err(PackageManagerError::UnknownPackageManager);
    }
}

/// Lists every package that can be removed according to the package manager.
///
/// This function only support apt, dnf and pacman.
/// If the function is called on a system that use a package manager that is not supported,
/// an error is returned.
///
/// For APT --dry-run is used.
pub fn list_orphaned_packages() -> Result<Vec<String>, PackageManagerError> {
    let package_manager = get_package_manager().unwrap();
    if package_manager == "apt" {
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
    } else if package_manager == "dnf" {
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
                    if entry.contains(&String::from("Autoremove Packages")) {
                        None
                    } else {
                        None
                    }
                } else {
                    Some(collection[0].to_string())
                }
            })
            .collect::<Vec<String>>();
        Ok(vector.to_vec())
    } else if package_manager == "pacman" {
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
    } else {
        return Err(PackageManagerError::UnknownPackageManager);
    }
}
