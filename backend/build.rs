use std::env;
use std::fs;
use std::process::Command;
use std::path::Path;

fn main() {
    // Get the current working directory
    let current_dir = env::current_dir().expect("Failed to get current directory");
    let install_script_path = current_dir.join("install.bash");

    // Execute the install.bash script
    let status = Command::new("bash")
        .arg(install_script_path)
        .status()
        .expect("Failed to execute install.bash");

    if !status.success() {
        panic!("install.bash script failed with status: {:?}", status);
    }
}
