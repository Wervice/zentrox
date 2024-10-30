use std::path::{Path, PathBuf};
use std::process::Command;
use uuid::Uuid;

/// Compresses a video or audio filvideo or audio fileng ffmpeg
/// * `video_path` &Path Path of the video file to compress
/// The function returns a temporary file location, somewhere in /tmp
pub fn compress_media(media_path: &Path) -> Result<PathBuf, String> {
    // Check if ffmpeg is installed
    let ffmpeg_command = Command::new("ffmpeg").arg("-version").spawn();
    let ffmpeg_works = ffmpeg_command.is_ok();

    if !ffmpeg_works {
        return Err(String::from("ffmpeg is not installed"));
    }

    let extension = media_path.extension().unwrap().to_str().unwrap_or("");

    // Determine file output location
    let output_location =
        Path::new("/tmp").join(format!("{}.{}", Uuid::new_v4().to_string(), extension).to_string());

    dbg!(&output_location);

    // The ffmpeg command, it is not being called in this expression
    let mut ffmpeg_command = Command::new("ffmpeg");
    ffmpeg_command
        .arg("-i")
        .arg(media_path.to_string_lossy().to_string());

    if is_video(&extension) {
        // Video compression
        ffmpeg_command.args([
            "-vcodec",
            "libvpx-vp9",
            "-crf",
            "35",
            "-preset",
            "ultrafast",
            "-acodec",
            "libopus",
            "-b:a",
            "128k",
            "-threads",
            "4",
        ]);
    } else if is_audio(&extension) {
        // Audio compression
        ffmpeg_command.args(["-acodec", "libmp3lame", "-b:a", "128k", "-f", "mp3"]);
    } else {
        return Err(String::from("Unsupported media type"));
    }

    ffmpeg_command.arg(&output_location);

    match ffmpeg_command.spawn() {
        Ok(mut s) => {
            if !s.wait().unwrap().success() {
                return Err(String::from(
                    "FFMpeg did not exit with a neutral status code.",
                ));
            }
        }
        Err(_e) => {
            return Err(String::from(
                "FFMpeg was unable to compress the provided file.",
            ))
        }
    }

    return Ok(PathBuf::from(output_location.to_string_lossy().to_string()));
}

fn is_video(extension: &str) -> bool {
    matches!(
        extension.to_lowercase().as_str(),
        "mp4" | "mkv" | "webm" | "avi" | "mov" | "flv" | "wmv"
    )
}
fn is_audio(extension: &str) -> bool {
    matches!(
        extension.to_lowercase().as_str(),
        "mp3" | "aac" | "wav" | "flac" | "m4a" | "ogg" | "opus"
    )
}
