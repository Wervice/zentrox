use std::collections::HashMap;
use std::path::PathBuf;

pub fn guess_mime(path: PathBuf) -> Option<String> {
    let path_extension = path.extension()?.to_string_lossy().to_lowercase();
    let table = HashMap::from([
        // Audio MIME types
        ("mp3", "audio/mpeg"),
        ("opus", "audio/opus"),
        ("wav", "audio/wav"),
        ("flac", "audio/flac"),
        ("aac", "audio/aac"),
        ("ogg", "audio/ogg"),
        ("m4a", "audio/m4a"),
        ("weba", "audio/webm"),
        ("amr", "audio/amr"),
        // Video MIME types
        ("mp4", "video/mp4"),
        ("webm", "video/webm"),
        ("ogg", "video/ogg"),
        ("mov", "video/quicktime"),
        ("avi", "video/x-msvideo"),
        ("mkv", "video/x-matroska"),
        ("flv", "video/x-flv"),
        ("wmv", "video/x-ms-wmv"),
        ("3gp", "video/3gpp"),
        ("mpeg", "video/mpeg"),
    ]);

    if table.contains_key(&path_extension.as_str()) {
        Some(table.get(path_extension.as_str()).unwrap().to_string())
    } else {
        None
    }
}
