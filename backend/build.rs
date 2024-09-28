use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    let install_dir = env::var("CARGO_INSTALL_ROOT").unwrap_or_else(|_| {
        let home = env::var("CARGO_HOME").unwrap_or_else(|_| env::var("HOME").unwrap());
        format!("{}/.your_project", home)
    });

    let install_path = PathBuf::from(install_dir);
    fs::create_dir_all(&install_path).unwrap();

    // Copy your static files to the install path
    let files = ["ftp.py", "install.bash", "robots.txt", "static/"];
    for file in &files {
        fs::copy(file, install_path.join(file)).expect("Failed to copy file");
    }

    println!("cargo:rerun-if-changed=ftp.py");
    println!("cargo:rerun-if-changed=install.bash");
    println!("cargo:rerun-if-changed=robots.txt");
}

