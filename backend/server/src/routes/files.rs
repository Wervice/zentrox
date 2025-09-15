use std::{
    fs,
    io::Read,
    os::unix::fs::{MetadataExt, PermissionsExt},
    path::PathBuf,
    time::UNIX_EPOCH,
};

use crate::SinglePath;
use actix_multipart::form::{MultipartForm, tempfile::TempFile, text::Text};
use actix_web::{
    HttpResponse,
    web::{Json, Query},
};
use log::{error, info};
use serde::{Deserialize, Serialize};
use utils::{
    status_com::{ErrorCode, MessageRes},
    users::NativeUser,
};
use utoipa::ToSchema;

#[utoipa::path(
    get,
    path = "/private/files/download",
    responses(
        (status = 200, body = Vec<u8>),
        (status = 404, description = "File not found")
    ),
    params(("path" = String, Query)),
    tags = ["private", "files"]
)]
/// Read file contents
pub async fn download_file(info: Query<SinglePath>) -> HttpResponse {
    let path_for_logging = info.path.to_string_lossy();

    info!("File download started for {}", path_for_logging);

    if std::path::Path::new(&info.path).exists() {
        error!("File {} does not exist.", path_for_logging);
        let f = fs::read(&info.path);
        match f {
            Ok(fh) => {
                HttpResponse::Ok().body(fh.bytes().map(|x| x.unwrap_or(0_u8)).collect::<Vec<u8>>())
            }
            Err(_) => {
                HttpResponse::InternalServerError().json(ErrorCode::FileError.as_error_message())
            }
        }
    } else {
        HttpResponse::NotFound().json(ErrorCode::FileDoesNotExist.as_error_message())
    }
}

#[derive(Serialize, ToSchema)]
struct FilesListRes {
    content: Vec<(String, FileSystemEntry)>,
}

/// List directory contents
#[utoipa::path(
    get,
    path = "/private/files/directoryReading",
    responses(
        (status = 200, body = FilesListRes),
        (status = 404, description = "Directory not found")
    ),
    params(("path" = String, Query)),
    tags = ["private", "files"]
)]
pub async fn files_list(info: Query<SinglePath>) -> HttpResponse {
    if !&info.path.exists() {
        return HttpResponse::NotFound().json(ErrorCode::DirectoryDoesNotExist.as_error_message());
    }

    match fs::read_dir(&info.path) {
        Ok(contents) => {
            let mut result: Vec<(String, FileSystemEntry)> = Vec::new();
            for e in contents {
                let e_unwrap = &e.unwrap();
                let file_name = &e_unwrap.file_name().into_string().unwrap();
                let is_file = e_unwrap.metadata().unwrap().is_file();
                let is_dir = e_unwrap.metadata().unwrap().is_dir();
                let is_symlink = e_unwrap.metadata().unwrap().is_symlink();

                if is_file {
                    result.push((file_name.to_string(), FileSystemEntry::File))
                } else if is_dir {
                    result.push((file_name.to_string(), FileSystemEntry::Directory))
                } else if is_symlink {
                    result.push((file_name.to_string(), FileSystemEntry::Symlink))
                } else {
                    result.push((file_name.to_string(), FileSystemEntry::Unknown))
                }
            }
            HttpResponse::Ok().json(FilesListRes { content: result })
        }
        Err(_) => {
            HttpResponse::InternalServerError().json(ErrorCode::DirectoryError.as_error_message())
        }
    }
}

/// Delete a file path
#[utoipa::path(
    post,
    path = "/private/files/delete",
    responses(
        (status = 200),
        (status = 404, description = "Path does not exist."),
        (status = 403, description = "The user lacks permissions to delete the file.")
    ),
    params(("path" = String, Query)),
    tags = ["private", "files"]
)]
pub async fn delete_file(info: Query<SinglePath>) -> HttpResponse {
    let file_path = &info.path;

    if !std::path::Path::new(&file_path).exists() {
        return HttpResponse::NotFound().json(ErrorCode::FileDoesNotExist.as_error_message());
    }

    let metadata = fs::metadata(&file_path).unwrap();
    let is_file = metadata.is_file();
    let is_dir = metadata.is_dir();
    let is_link = metadata.is_symlink();
    let has_permissions = !metadata.permissions().readonly();

    if (is_file || is_link) && has_permissions {
        match fs::remove_file(&file_path) {
            Ok(_) => HttpResponse::Ok().json(MessageRes::from("The file has been deleted.")),
            Err(_) => {
                HttpResponse::InternalServerError().json(ErrorCode::FileError.as_error_message())
            }
        }
    } else if is_dir && has_permissions {
        match fs::remove_dir_all(&file_path) {
            Ok(_) => HttpResponse::Ok().json(MessageRes::from("The directory has been deleted.")),
            Err(_) => HttpResponse::InternalServerError()
                .json(ErrorCode::DirectoryError.as_error_message()),
        }
    } else {
        HttpResponse::Forbidden().json(ErrorCode::MissingSystemPermissions.as_error_message())
    }
}

#[derive(Deserialize, ToSchema)]
pub struct MovePathReq {
    #[schema(value_type = String)]
    origin: PathBuf,
    #[schema(value_type = String)]
    destination: PathBuf,
}

/// Move file path
#[utoipa::path(
    post,
    path = "/private/files/move",
    responses(
        (status = 200),
        (status = 404, description = "Path does not exist."),
        (status = 403, description = "The user lacks permissions to rename the path.")
    ),
    request_body = inline(MovePathReq),
    tags = ["private", "files"]
)]
pub async fn move_path(json: Json<MovePathReq>) -> HttpResponse {
    if !json.origin.exists() || json.destination.exists() {
        HttpResponse::NotFound().json(ErrorCode::FileDoesNotExist.as_error_message());
    }

    let metadata = fs::metadata(&json.origin).unwrap();
    let has_permissions = !metadata.permissions().readonly();
    if has_permissions && !&json.destination.exists() {
        match fs::rename(&json.origin, &json.destination) {
            Ok(_) => HttpResponse::Ok().json(MessageRes::from("The directory has been renamed.")),
            Err(_) => {
                HttpResponse::InternalServerError().json(ErrorCode::FileError.as_error_message())
            }
        }
    } else {
        HttpResponse::Forbidden().json(ErrorCode::MissingSystemPermissions.as_error_message())
    }
}

/// Delete file
///
/// Overwrites the file with random data and removes it.
#[utoipa::path(
    post,
    path = "/private/files/burn",
    responses(
        (status = 200),
        (status = 404, description = "Path does not exist."),
        (status = 403, description = "The user lacks permissions to burn this file.")
    ),
    params(("path" = String, Query)),
    tags = ["private", "files"]
)]
pub async fn burn_file(info: Query<SinglePath>) -> HttpResponse {
    let path = &info.path;

    if path.is_dir() {
        return HttpResponse::UnprocessableEntity()
            .json(ErrorCode::DirectoryError.as_error_message());
    }

    if !path.exists() {
        return HttpResponse::NotFound().json(ErrorCode::FileDoesNotExist.as_error_message());
    }

    let metadata = fs::metadata(&path).unwrap();
    let has_permissions = !metadata.permissions().readonly();

    if !has_permissions {
        return HttpResponse::Forbidden()
            .json(ErrorCode::MissingSystemPermissions.as_error_message());
    }

    let size = metadata.len();
    let r_s = (0..size).map(|_| rand::random::<u8>()).collect::<Vec<u8>>();
    match fs::write(path.clone(), r_s) {
        Ok(_) => {
            let _ = fs::remove_file(&path);
            HttpResponse::Ok().json(MessageRes::from("The file has been burned."))
        }
        Err(_) => HttpResponse::InternalServerError().json(ErrorCode::FileError.as_error_message()),
    }
}

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
enum FileSystemEntry {
    File,
    Directory,
    Symlink,
    Unknown,
}

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
struct GetMetadataRes {
    permissions: i32,
    owner: NativeUser,
    size: u64,
    entry_type: FileSystemEntry,
    created: Option<u64>,
    modified: Option<u64>,
    filename: String,
    absolute_path: String,
}

fn recursive_size_of_directory(path: &PathBuf) -> u64 {
    let mut size = 0_u64;
    if let Ok(contents) = fs::read_dir(path) {
        contents.for_each(|f| {
            if f.is_err() {
                return;
            }
            if let Ok(metadata) = f.as_ref().unwrap().metadata() {
                if metadata.is_symlink() || metadata.is_file() {
                    size += metadata.size();
                } else {
                    size += recursive_size_of_directory(&f.unwrap().path());
                }
            }
        });
    }
    size
}

/// Metadata of path
#[utoipa::path(
    get,
    path = "/private/files/metadata",
    responses(
        (status = 200, body = GetMetadataRes),
        (status = 404, description = "Path does not exist."),
    ),
    params(("path" = String, Query)),
    tags = ["private", "files"]
)]
pub async fn get_file_metadata(info: Query<SinglePath>) -> HttpResponse {
    let constructed_path = &info.path;
    if !constructed_path.exists() {
        return HttpResponse::NotFound().json(ErrorCode::FileDoesNotExist.as_error_message());
    }

    let metadata = match fs::metadata(constructed_path.clone()) {
        Ok(m) => m,
        Err(_) => {
            return HttpResponse::InternalServerError()
                .json(ErrorCode::FileError.as_error_message());
        }
    };

    let permissions = metadata.permissions().mode() & 0o777;
    let permissions_octal_string = format!("{permissions:o}");
    let permissions_octal: i32 = permissions_octal_string.parse().unwrap();

    // UNIX timestamp in seconds at which this file was created
    let created_stamp = match metadata.created() {
        Ok(v) => Some(
            v.duration_since(UNIX_EPOCH)
                .expect("Time went backwards.")
                .as_secs(),
        ),
        Err(_) => None,
    };

    // UNIX timestamp in seconds at which this file was last modified
    let modified_stamp = match metadata.modified() {
        Ok(v) => Some(
            v.duration_since(UNIX_EPOCH)
                .expect("Time went backwards.")
                .as_secs(),
        ),
        Err(_) => None,
    };

    let is_file = metadata.is_file();
    let is_dir = metadata.is_dir();
    let is_symlink = metadata.is_symlink();

    let size = if is_file || is_symlink {
        metadata.size()
    } else {
        recursive_size_of_directory(&constructed_path)
    };

    let owner = NativeUser::from_uid(metadata.uid()).unwrap();

    HttpResponse::Ok().json(GetMetadataRes {
        permissions: permissions_octal,
        size,
        owner,
        modified: modified_stamp,
        created: created_stamp,
        entry_type: {
            if is_dir {
                FileSystemEntry::Directory
            } else if is_file {
                FileSystemEntry::File
            } else if is_symlink {
                FileSystemEntry::Symlink
            } else {
                FileSystemEntry::Unknown
            }
        },
        absolute_path: constructed_path.to_str().unwrap().to_string(),
        filename: constructed_path
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string(),
    })
}

#[derive(MultipartForm, ToSchema)]
pub struct UploadFileForm {
    #[multipart(limit = "32 GiB")]
    #[schema(value_type = [u8])]
    file: TempFile,
    #[schema(value_type = String)]
    path: Text<String>,
}

/// Uploads a file to the host system.
#[utoipa::path(
    post,
    path = "/private/files/upload",
    responses((status = 200)),
    request_body(content_type = "mutlipart/form-data", content = UploadFileForm,description = "The path and file to upload."),
    tags = ["private", "files"]
)]
pub async fn upload_file(MultipartForm(form): MultipartForm<UploadFileForm>) -> HttpResponse {
    let intended_path = PathBuf::from(form.path.to_string());
    let filename = match form.file.file_name {
        Some(v) => v,
        None => {
            return HttpResponse::BadRequest().json(ErrorCode::InsufficientData.as_error_message());
        }
    };
    let intended_path = intended_path.join(filename);
    let temp_path = form.file.file.path();
    let cpy = fs::copy(temp_path, &intended_path);
    let unk = fs::remove_file(temp_path);

    if cpy.is_err() || unk.is_err() {
        HttpResponse::InternalServerError().json(ErrorCode::FileError.as_error_message());
    }

    HttpResponse::Ok().json(MessageRes::from("The upload has been finished."))
}
