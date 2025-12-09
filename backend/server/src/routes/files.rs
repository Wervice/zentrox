use crate::SinglePath;
use actix_multipart::form::{MultipartForm, tempfile::TempFile, text::Text};
use actix_web::{
    HttpResponse,
    web::{Json, Query},
};
use log::{error, info};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::Read,
    os::unix::fs::{MetadataExt, PermissionsExt},
    path::PathBuf,
};
use utils::{
    status_com::{ErrorCode, MessageRes},
    time::time_to_unix,
    users::NativeUser,
};
use utoipa::ToSchema;

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
enum FileSystemEntry {
    File,
    Directory,
    Symlink,
    Unknown,
}

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
pub async fn download(info: Query<SinglePath>) -> HttpResponse {
    info!("File download started for {:?}", info.path);

    if !info.path.exists() {
        error!("File {:?} does not exist.", info.path);
        return HttpResponse::NotFound().json(ErrorCode::FileDoesNotExist.as_error_message());
    }
    let file = fs::read(&info.path);
    if let Ok(handle) = file {
        HttpResponse::Ok().body(handle.bytes().map(|x| x.unwrap()).collect::<Vec<u8>>())
    } else {
        HttpResponse::InternalServerError().json(ErrorCode::FileError.as_error_message())
    }
}

#[derive(Serialize, ToSchema)]
struct FilesListRes {
    #[schema(value_type = Vec<(String, FileSystemEntry)>)]
    content: Vec<(PathBuf, FileSystemEntry)>,
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
pub async fn list(info: Query<SinglePath>) -> HttpResponse {
    if !info.path.exists() {
        return HttpResponse::NotFound().json(ErrorCode::DirectoryDoesNotExist.as_error_message());
    }

    if let Ok(reading) = fs::read_dir(&info.path) {
        let serializable_contents = reading
            .map(|entry| {
                let e_unwraped = entry.unwrap();
                let metadata = e_unwraped.metadata().unwrap();
                let variant = {
                    if metadata.is_dir() {
                        FileSystemEntry::Directory
                    } else if metadata.is_file() {
                        FileSystemEntry::File
                    } else if metadata.is_symlink() {
                        FileSystemEntry::Symlink
                    } else {
                        FileSystemEntry::Unknown
                    }
                };
                (e_unwraped.path(), variant)
            })
            .collect();
        HttpResponse::Ok().json(FilesListRes {
            content: serializable_contents,
        })
    } else {
        HttpResponse::InternalServerError().json(ErrorCode::DirectoryError.as_error_message())
    }
}

/// Delete a file-path
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
pub async fn delete(info: Query<SinglePath>) -> HttpResponse {
    if !info.path.exists() {
        return HttpResponse::NotFound().json(ErrorCode::FileDoesNotExist.as_error_message());
    }

    if info.path.is_file() || info.path.is_symlink() {
        if fs::remove_file(&info.path).is_ok() {
            return HttpResponse::Ok().json(MessageRes::from("The file has been deleted."));
        }
        HttpResponse::InternalServerError().json(ErrorCode::FileError.as_error_message())
    } else {
        if fs::remove_dir_all(&info.path).is_ok() {
            return HttpResponse::Ok().json(MessageRes::from("The directory has been deleted."));
        }
        HttpResponse::InternalServerError().json(ErrorCode::DirectoryError.as_error_message())
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
pub async fn move_to(json: Json<MovePathReq>) -> HttpResponse {
    if !json.origin.exists() {
        HttpResponse::NotFound().json(ErrorCode::FileDoesNotExist.as_error_message());
    }

    if json.destination.exists() {
        return HttpResponse::Conflict().json(ErrorCode::FileError);
    }

    if fs::rename(&json.origin, &json.destination).is_ok() {
        return HttpResponse::Ok().json(MessageRes::from("The directory has been renamed."));
    }
    HttpResponse::InternalServerError().json(ErrorCode::FileError.as_error_message())
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
pub async fn burn(info: Query<SinglePath>) -> HttpResponse {
    if info.path.is_dir() {
        return HttpResponse::UnprocessableEntity()
            .json(ErrorCode::DirectoryError.as_error_message());
    }

    let size = fs::metadata(&info.path).unwrap().len();
    let r_s = (0..size).map(|_| rand::random::<u8>()).collect::<Vec<u8>>();
    match fs::write(&info.path, r_s) {
        Ok(_) => {
            let _ = fs::remove_file(&info.path);
            HttpResponse::Ok().json(MessageRes::from("The file has been burned."))
        }
        Err(_) => HttpResponse::InternalServerError().json(ErrorCode::FileError.as_error_message()),
    }
}

fn recursive_size_of_directory(path: &PathBuf) -> u64 {
    let mut size = 0_u64;
    if let Ok(contents) = fs::read_dir(path) {
        contents.for_each(|f| {
            if let Ok(metadata) = f.as_ref().unwrap().metadata() {
                if metadata.is_dir() {
                    size += recursive_size_of_directory(&f.unwrap().path());
                } else {
                    size += metadata.size();
                }
            }
        });
    }
    size
}

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
struct GetMetadataRes {
    permissions: u16,
    owner: NativeUser,
    size: u64,
    entry_type: FileSystemEntry,
    created: u128,
    modified: u128,
    filename: String,
    absolute_path: String,
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
pub async fn metadata(info: Query<SinglePath>) -> HttpResponse {
    let metadata = match fs::metadata(&info.path) {
        Ok(m) => m,
        Err(_) => {
            return HttpResponse::InternalServerError()
                .json(ErrorCode::FileError.as_error_message());
        }
    };

    let permissions = metadata.permissions().mode() & 0o777;
    let permissions_octal_string = format!("{permissions:o}");
    let permissions_octal: u16 = permissions_octal_string.parse().unwrap();

    let created_stamp = time_to_unix(metadata.created().unwrap());
    let modified_stamp = time_to_unix(metadata.modified().unwrap());

    let size = if metadata.is_dir() {
        recursive_size_of_directory(&info.path)
    } else {
        metadata.size()
    };

    HttpResponse::Ok().json(GetMetadataRes {
        permissions: permissions_octal,
        size,
        owner: NativeUser::from_uid(metadata.uid()).unwrap(),
        modified: modified_stamp,
        created: created_stamp,
        entry_type: {
            if metadata.is_dir() {
                FileSystemEntry::Directory
            } else if metadata.is_file() {
                FileSystemEntry::File
            } else if metadata.is_symlink() {
                FileSystemEntry::Symlink
            } else {
                FileSystemEntry::Unknown
            }
        },
        absolute_path: info.path.to_string_lossy().to_string(),
        filename: info.path.file_name().unwrap().to_string_lossy().to_string(),
    })
}

#[derive(MultipartForm, ToSchema)]
pub struct UploadFileForm {
    #[multipart(limit = "32 GiB")]
    #[schema(value_type = [u8])]
    file: TempFile,
    #[schema(value_type = String)]
    path: Text<PathBuf>,
}

/// Uploads a file to the host system.
#[utoipa::path(
    post,
    path = "/private/files/upload",
    responses((status = 200)),
    request_body(content_type = "mutlipart/form-data", content = UploadFileForm,description = "The path and file to upload."),
    tags = ["private", "files"]
)]
pub async fn upload(MultipartForm(form): MultipartForm<UploadFileForm>) -> HttpResponse {
    let filename = match form.file.file_name {
        Some(v) => v,
        None => {
            return HttpResponse::BadRequest().json(ErrorCode::InsufficientData.as_error_message());
        }
    };
    let intended_path = form.path.join(filename);
    let temp_path = form.file.file.path();
    let cpy = fs::copy(temp_path, &intended_path);
    let unk = fs::remove_file(temp_path);

    if cpy.is_err() || unk.is_err() {
        HttpResponse::InternalServerError().json(ErrorCode::FileError.as_error_message());
    }

    HttpResponse::Ok().json(MessageRes::from("The upload has been finished."))
}
