use std::fs::{self, DirEntry};
use std::path::PathBuf;

pub fn visit_dirs(dir: PathBuf) -> Result<std::vec::IntoIter<DirEntry>, std::io::Error> {
    let mut e: Vec<DirEntry> = Vec::new();

    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                visit_dirs(path)?.for_each(|ce| e.push(ce));
            } else {
                e.push(entry)
            }
        }
    }

    Ok(e.into_iter())
}
