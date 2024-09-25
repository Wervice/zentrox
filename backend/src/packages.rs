/// APT, DNF, PacMan bindings to
/// install packages, remove package, list installed/available/unnecessary packages
use crate::sudo::SwitchedUserCommand;
use std::collections::{HashMap};
use std::{fs, process::Command};

/// Determines which package manager is used by the system.
///
/// The only supported package managers are apt (debian-based), dnf (rhel-based) and pacman
/// (arch-based)
/// If no supported package manager is founed None will be returned.
/// package managers, an Err is returned.
pub fn get_package_manager() -> Option<String> {
    let lsb_release_output = Command::new("lsb_release").arg("-is").output();
    let binding = match lsb_release_output {
        Ok(value) => String::from_utf8_lossy(&value.stdout).to_string(),
        Err(_) => String::new(),
    }
    .to_lowercase();
    let lsb_release_id_clear = binding.replace("\n", "");
    let lsb_release_id = lsb_release_id_clear.as_str();
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
    ];
    let redhat_distros = ["rhel", "fedora", "centos", "mageia"];
    let arch_distros = ["arch", "manajaro", "arch", "blackarch"];
    if debain_distros.contains(&lsb_release_id) {
        Some("apt".to_string())
    } else if redhat_distros.contains(&lsb_release_id) {
        Some("dnf".to_string())
    } else if arch_distros.contains(&lsb_release_id) {
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

pub fn auto_remove(password: String) -> Result<(), String> {
    let package_mamager = get_package_manager().unwrap();

    let command;

    if package_mamager == "apt" {
        command = "apt autoremove";
    } else if package_mamager == "dnf" {
        command = "dnf autoremove";
    } else if package_mamager == "pacman" {
        command = "pacman -R $(pacman -Qdtq)";
    } else {
        return Err("Unknow package manager".to_string());
    }

    let _ = SwitchedUserCommand::new(password, command.to_string()).spawn();

    Ok(())
}

/// Installs package on the system.
///
/// This only works on apt, dnf and pacman based systems.
/// If the function is called on a system that does not use one of the descriped
/// package managers, an Err is returned.
/// * `name` - Name of the package
/// * `password` - Password used to run sudo
pub fn install_package(name: String, password: String) -> Result<(), String> {
    let package_manager = get_package_manager().unwrap();

    let command;

    if package_manager == "apt" {
        command = format!("apt install {} -y -q", name);
    } else if package_manager == "dnf" {
        command = format!("dnf install {} -y -q", name);
    } else if package_manager == "pacman" {
        command = format!("pacman -Sy {} --noconfirm", name);
    } else {
        return Err("Unknown package manager".to_string());
    }

    let _ = SwitchedUserCommand::new(password, command).spawn();

    Ok(())
}

/// Removes package from the system.
///
/// This only works on apt, dnf and pacman based systems.
/// If the function is called on a system that does not use one of the descriped
/// package managers, an Err is returned.
///
/// * `name` - Name of the package
/// * `password` - Password used to run sudo
pub fn remove_package(name: String, password: String) -> Result<(), String> {
    let package_manager = get_package_manager().unwrap();

    let command;

    if package_manager == "apt" {
        command = format!("apt remove {} -y -q", name);
    } else if package_manager == "dnf" {
        command = format!("dnf remove {} -y -q", name);
    } else if package_manager == "pacman" {
        command = format!("pacman -R {} --noconfirm", name)
    } else {
        return Err("Unknown package manager".to_string());
    }

    let _ = SwitchedUserCommand::new(password, command).spawn();

    Ok(())
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
pub fn list_installed_packages() -> Result<Vec<String>, String> {
    let package_manager = get_package_manager().unwrap();
    if package_manager == "apt" {
        let command = Command::new("apt")
            .arg("list")
            .arg("--installed")
            .output()
            .unwrap();
        let output = String::from_utf8_lossy(&command.stdout).to_string();
        let vector = &output
            .lines()
            .map(|e| {
                let entry = e.to_string();
                let split = entry.split("/");
                let collection = split.collect::<Vec<&str>>();
                if collection.len() != 2 {
                    if entry.contains(&String::from("Listing...")) {
                        String::from("Skip")
                    } else {
                        String::from("")
                    }
                } else {
                    collection[0].to_string()
                }
            })
            .skip_while(|x| x != "Skip")
            .filter(|x| !x.is_empty())
            .collect::<Vec<String>>();
        Ok(vector.to_vec())
    } else if package_manager == "dnf" {
        let command = Command::new("dnf")
            .arg("list")
            .arg("installed")
            .output()
            .unwrap();
        let output = String::from_utf8_lossy(&command.stdout).to_string();
        let vector = &output
            .lines()
            .map(|e| {
                let entry = e.to_string();
                let split = entry.split(".");
                let collection = split.collect::<Vec<&str>>();
                if collection.len() != 2 {
                    if entry.contains(&String::from("Installed Packages")) {
                        String::from("Skip")
                    } else {
                        String::from("")
                    }
                } else {
                    collection[0].to_string()
                }
            })
            .skip_while(|x| x != "Skip")
            .filter(|x| !x.is_empty())
            .collect::<Vec<String>>();
        Ok(vector.to_vec())
    } else if package_manager == "pacman" {
        let command = Command::new("pacman").arg("-Qq").output().unwrap();
        let output = String::from_utf8_lossy(&command.stdout).to_string();
        let vector = &output
            .lines()
            .map(|e| {
                let entry = e.to_string();
                let split = entry.split("/");
                let collection = split.collect::<Vec<&str>>();
                if collection.len() != 1 {
                    String::from("")
                } else {
                    collection[0].to_string()
                }
            })
            .filter(|x| !x.is_empty())
            .collect::<Vec<String>>();
        Ok(vector.to_vec())
    } else {
        return Err("Unknow package manager".to_string());
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
pub fn list_available_packages() -> Result<Vec<String>, String> {
    
    fn is_version_number(s: &String) -> bool {
        let version_number_chars = ['.', '-', '0', '1', '2', '3', '4', '5', '6', '7', '8', '9'];
        let mut is = false;

        for c in s.chars() {
            if version_number_chars.contains(&c) {
                is = true;
            }
        }

        is
    }

    let package_manager = get_package_manager().unwrap();
    if package_manager == "apt" {
        let command = Command::new("apt").arg("list").output().unwrap();
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
            .output()
            .unwrap();
        let output = String::from_utf8_lossy(&command.stdout).to_string();
        let vector = &output
            .lines()
            .map(|e| {
                let entry = e.to_string();
                let split = entry.split(".");
                let collection = split.collect::<Vec<&str>>();
                if collection.len() != 2 {
                    if entry.contains(&String::from("Available Packages")) {
                        String::from("Skip")
                    } else {
                        String::from("")
                    }
                } else {
                    collection[0].to_string()
                }
            })
            .skip_while(|x| x != "Skip")
            .filter(|x| !x.is_empty())
            .collect::<Vec<String>>();
        Ok(vector.to_vec())
    } else if package_manager == "pacman" {
        let command = Command::new("pacman").arg("-Sl").output().unwrap();
        let output = String::from_utf8_lossy(&command.stdout).to_string();
        let vector = &output
            .lines()
            .filter_map(|e| {
                let entry = e.to_string();
                let split = entry.split(" ");
                let collection = split.collect::<Vec<&str>>();
                if !entry.contains("[installed") {
                    return Some(collection[1].to_string());
                } {
                    None
                }
            })
            .collect::<Vec<String>>();
        Ok(vector.to_vec())
    } else {
        return Err("Unknow package manager".to_string());
    }
}

/// Lists every package that can be removed according to the package manager.
///
/// This function only support apt, dnf and pacman.
/// If the function is called on a system that use a package manager that is not supported,
/// an error is returned.
///
/// For APT --dry-run is used.
pub fn list_autoremoveable_packages() -> Result<Vec<String>, String> {
    let package_manager = get_package_manager().unwrap();
    if package_manager == "apt" {
        let command = Command::new("apt")
            .arg("list")
            .arg("autoremove")
            .arg("--dry-run")
            .output()
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
            .output()
            .unwrap();
        let output = String::from_utf8_lossy(&command.stdout).to_string();
        let vector = &output
            .lines()
            .map(|e| {
                let entry = e.to_string();
                let split = entry.split(".");
                let collection = split.collect::<Vec<&str>>();
                if collection.len() != 2 {
                    if entry.contains(&String::from("Autoremove Packages")) {
                        String::from("Skip")
                    } else {
                        String::from("")
                    }
                } else {
                    collection[0].to_string()
                }
            })
            .skip_while(|x| x != "Skip")
            .skip(1)
            .collect::<Vec<String>>();
        Ok(vector.to_vec())
    } else if package_manager == "pacman" {
        let command = Command::new("pacman").arg("-Qdtq").output().unwrap();
        let output = String::from_utf8_lossy(&command.stdout).to_string();
        let vector = &output
            .lines()
            .map(|e| {
                let entry = e.to_string();
                let split = entry.split("/");
                let collection = split.collect::<Vec<&str>>();
                if collection.len() != 2 {
                    String::from("")
                } else {
                    collection[1].to_string()
                }
            })
            .skip(1)
            .collect::<Vec<String>>();
        Ok(vector.to_vec())
    } else {
        return Err("Unknow package manager".to_string());
    }
}

#[derive(Clone, Debug)]
pub struct DesktopApplication {
    pub name: String,
    pub exec_name: String,
}

/// Lists every application that has a desktop file in `/usr/share/applications/`.
///
/// Those desktop files are then parsed into DesktopApplication structs.
/// A DesktopApplication struct contains the pretty name of the application (i.e. Firefox or
/// Nautilus) and the exec_name like /bin/firefox or /usr/bin/nano
/// If an error occurs during this process, an error is returned in a Result.
pub fn list_desktop_applications() -> Result<Vec<DesktopApplication>, String> {
    if let Ok(value) = fs::read_dir("/usr/share/applications/") {
        let applications_folder_entires = value;

        let mut applications = Vec::new();

        for entry in applications_folder_entires {
            let entry_u = entry.unwrap();
            let entry_path = entry_u.path();
            let entry_extension = entry_u
                .file_name()
                .to_string_lossy()
                .split(".")
                .last()
                .unwrap()
                .to_string();

            if entry_extension == "desktop" {
                let entry_contents = fs::read_to_string(entry_path).unwrap();
                let entry_contents_lines_collected =
                    entry_contents.lines().skip(1).collect::<Vec<&str>>();
                let mut entry_contents_hashmap = HashMap::new();
                for l in entry_contents_lines_collected {
                    if let Some((k, v)) = l.split_once("=") {
                        entry_contents_hashmap.insert(k, v);
                    }
                }

                applications.push(DesktopApplication {
                    name: entry_contents_hashmap
                        .get("Name")
                        .unwrap_or(&"")
                        .to_string(),
                    exec_name: entry_contents_hashmap
                        .get("Exec")
                        .unwrap_or(&"")
                        .split(" ")
                        .collect::<Vec<&str>>()[0]
                        .to_string(),
                });
            }
        }

        Ok(applications)
    } else {
        Err("Failed to read dir".to_string())
    }
}
