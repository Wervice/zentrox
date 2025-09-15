use actix_multipart::form::{MultipartForm, tempfile::TempFile, text::Text};
use actix_web::web::Json;
use actix_web::{HttpResponse, web::Data};
use diesel::prelude::*;
use futures::FutureExt;
use log::error;
use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::Read,
    path::{self, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};
use utils::status_com::ErrorCode;
use utils::{database::establish_connection, models, schema, status_com::MessageRes, vault};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{AppState, BackgroundTaskState};

#[derive(Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct VaultConfigurationReq {
    key: Option<String>,
    old_key: Option<String>,
    new_key: Option<String>,
}

fn get_vault_enabled() -> bool {
    use models::Configurations;
    use schema::Configuration::dsl::*;

    Configuration
        .select(Configurations::as_select())
        .first(&mut establish_connection())
        .unwrap()
        .vault_enabled
}

/// Configure Vault
///
/// This creates a new vault instance. If one is already found on the system, only the password is
/// updated to a new one.
#[utoipa::path(
    post,
    path = "/private/vault/configuration",
    request_body = VaultConfigurationReq,
    responses((status = 200), (status = 403, description = "The new old key could not be used to decrypt.")),
    tags = ["private", "vault"]
)]
// NOTE The following could use less indentations
pub async fn configure(json: Json<VaultConfigurationReq>) -> HttpResponse {
    use schema::Configuration::dsl::*;
    let vault_path = path::Path::new(&dirs::home_dir().unwrap())
        .join(".local")
        .join("share")
        .join("zentrox")
        .join("vault_directory");

    let connection = &mut establish_connection();

    if !get_vault_enabled() && !vault_path.exists() {
        if json.key.is_none() {
            return HttpResponse::BadRequest().json(ErrorCode::InsufficientData.as_error_message());
        }

        let key = &json.key.clone().unwrap();

        match fs::create_dir_all(&vault_path) {
            Ok(_) => {}
            Err(e) => {
                error!("The vault directory could not be created due to the following error: {e}");
                return HttpResponse::InternalServerError()
                    .json(ErrorCode::DirectoryError.as_error_message());
            }
        };

        let vault_file_contents = format!(
            "Vault created by {} at UNIX {}.",
            whoami::username(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Time went backwards.")
                .as_millis()
        );

        match fs::write(vault_path.join(".vault"), vault_file_contents) {
            Ok(_) => {}
            Err(e) => {
                error!("Creating the initial vault file failed with error: {e}");
                return HttpResponse::InternalServerError()
                    .json(ErrorCode::FileError.as_error_message());
            }
        }

        vault::encrypt_file(vault_path.join(".vault"), key);

        let database_vault_enable_update = diesel::update(Configuration)
            .set(vault_enabled.eq(true))
            .execute(connection);

        if let Err(database_error) = database_vault_enable_update {
            return HttpResponse::InternalServerError().json(
                ErrorCode::DatabaseUpdateFailed(database_error.to_string()).as_error_message(),
            );
        }
    } else if json.old_key.is_some() && json.new_key.is_some() {
        let old_key = json.old_key.clone().unwrap();
        let new_key = json.new_key.clone().unwrap();
        match vault::decrypt_file(vault_path.join(".vault"), &old_key.to_string()) {
            Some(_) => vault::encrypt_file(vault_path.join(".vault"), &old_key.to_string()),
            None => {
                return HttpResponse::Forbidden()
                    .json(ErrorCode::MissingVaultPermissions.as_error_message());
            }
        };

        match vault::decrypt_directory(vault_path.clone(), &old_key) {
            Ok(_) => {}
            Err(_) => {
                return HttpResponse::Forbidden()
                    .json(ErrorCode::MissingVaultPermissions.as_error_message());
            }
        };

        if let Err(_) = vault::encrypt_directory(vault_path.clone(), &new_key) {
            return HttpResponse::InternalServerError()
                .json(ErrorCode::EncryptionFailed.as_error_message());
        }
    } else {
        return HttpResponse::BadRequest().json(ErrorCode::InsufficientData.as_error_message());
    }

    HttpResponse::Ok().json(MessageRes::from("The vault has been configured."))
}

#[utoipa::path(get,
    path = "/private/vault/enabled",
    responses((status = 200, body = String)),
    tags = ["private", "vault"]
)]
/// Is vault ready for use
pub async fn is_configured() -> HttpResponse {
    return HttpResponse::Ok().body(get_vault_enabled().to_string());
}

// Vault Tree

#[derive(Serialize, ToSchema)]
struct VaultFsPathRes {
    tree: Vec<String>,
}

#[derive(Deserialize, ToSchema)]
pub struct VaultKeyReq {
    key: String,
}

/// List all paths in Zentrox Vault in a one-dimensional vector. This also decrypts the filenames
/// and folder names of the entries. A directory path always ends with /. The path never starts
/// with /, thus root is "".
fn list_paths(directory: PathBuf, key: String) -> Vec<String> {
    let read = fs::read_dir(directory).unwrap();
    let mut paths: Vec<String> = Vec::new();

    for entry in read {
        let entry_unwrap = &entry.unwrap();
        let entry_metadata = &entry_unwrap.metadata().unwrap();
        let is_file = entry_metadata.is_file() || entry_metadata.is_symlink();
        let path = &entry_unwrap.path().to_string_lossy().to_string().replace(
            &path::Path::new(&dirs::home_dir().unwrap())
                .join(".local")
                .join("share")
                .join("zentrox")
                .join("vault_directory")
                .to_string_lossy()
                .to_string(),
            "",
        );
        // TODO ^ Check if this is error prone

        if is_file {
            paths.push(
                path.to_string()
                    .split("/")
                    .filter(|x| !x.is_empty())
                    .map(|x| {
                        if x != ".vault" && !x.is_empty() {
                            match vault::decrypt_string_hash(x.to_string(), &key) {
                                Some(v) => v,
                                None => "Decryption Error".to_string(),
                            }
                        } else {
                            ".vault".to_string()
                        }
                    })
                    .collect::<Vec<String>>()
                    .join("/"),
            ); // Path of the file, while ignoring the path until (but still including) vault_directory.
        } else {
            paths.push(
                path.to_string()
                    .split("/")
                    .filter(|x| !x.is_empty())
                    .map(|x| {
                        if x != ".vault" && !x.is_empty() {
                            match vault::decrypt_string_hash(x.to_string(), &key) {
                                Some(v) => v,
                                None => "Decryption Error".to_string(),
                            }
                        } else {
                            ".vault".to_string()
                        }
                    })
                    .collect::<Vec<String>>()
                    .join("/")
                    .to_string()
                    + "/",
            ); // Path of the file, while ignoring the path until (but still including) vault_directory.
            for e in list_paths(entry_unwrap.path(), key.clone()) {
                paths.push(e); // Path of the file, while ignoring the path until (but still including) vault_directory.
            }
        }
    }
    paths
}

/// Vault paths as flat path tree
#[utoipa::path(
    post,
    path = "/private/vault/tree",
    request_body = VaultKeyReq,
    responses((status = 200, body = VaultFsPathRes), (status = 403, description = "The new key could not be used to decrypt.")),
    tags = ["private", "vault"]
)]
pub async fn tree(json: Json<VaultKeyReq>) -> HttpResponse {
    if get_vault_enabled() {
        let vault_path = path::Path::new(&dirs::home_dir().unwrap())
            .join(".local")
            .join("share")
            .join("zentrox")
            .join("vault_directory");

        let key = &json.key;

        match vault::decrypt_file(vault_path.join(".vault"), &key.to_string()) {
            Some(_) => vault::encrypt_file(vault_path.join(".vault"), &key.to_string()),
            None => {
                return HttpResponse::Forbidden()
                    .json(ErrorCode::MissingVaultPermissions.as_error_message());
            }
        };

        let paths = list_paths(vault_path, key.to_string());

        HttpResponse::Ok().json(VaultFsPathRes { tree: paths })
    } else {
        HttpResponse::BadRequest().json(ErrorCode::VaultUnconfigured)
    }
}

// Delete vault file
#[derive(Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct VaultDeleteReq {
    delete_path: String,
    key: String,
}

/// Burn vault file
///
/// Burns a file in Zentrox vault by overwriting it with random data
#[utoipa::path(
    post,
    path = "/private/vault/delete",
    request_body = VaultDeleteReq,
    responses((status = 200, description = "The file is being deleted. This may take several seconds, thus a Job UUID is provided.")),
    tags = ["private", "vault", "responding_job"]
)]
pub async fn delete_file(state: Data<AppState>, json: Json<VaultDeleteReq>) -> HttpResponse {
    let uuid = Uuid::new_v4();
    state
        .background_jobs
        .lock()
        .unwrap()
        .insert(uuid, BackgroundTaskState::Pending);

    drop(actix_web::web::block(move || {
        let sent_path = &json.delete_path;
        if sent_path == ".vault" {
            error!(".vault file may never be deleted.");
            state.background_jobs.lock().unwrap().insert(
                uuid,
                BackgroundTaskState::FailOutput(".vault file may never be deleted.".to_string()),
            );
        }

        let path = path::Path::new(&dirs::home_dir().unwrap().to_string_lossy().to_string())
            .join(".local")
            .join("share")
            .join("zentrox")
            .join("vault_directory")
            .join(
                sent_path
                    .split("/")
                    .filter(|x| !x.is_empty())
                    .map(|x| {
                        vault::encrypt_string_hash(x.to_string(), &json.key.to_string()).unwrap()
                    })
                    .collect::<Vec<String>>()
                    .join("/"),
            );

        if path.metadata().unwrap().is_file() {
            let file_size = fs::metadata(&path).unwrap().len();
            let mut i = 0;

            while i != 5 {
                let random_data = (0..file_size)
                    .map(|_| rand::random::<u8>())
                    .collect::<Vec<u8>>();
                let _ = fs::write(&path, random_data);
                i += 1;
            }

            match fs::remove_file(path) {
                Ok(_) => {}
                Err(_e) => {
                    error!("Failed to remove vault file.");
                    state.background_jobs.lock().unwrap().insert(
                        uuid,
                        BackgroundTaskState::FailOutput("Failed to remove vault file.".to_string()),
                    );
                }
            };
        } else {
            let _ = vault::burn_directory(path.to_string_lossy().to_string());
            match fs::remove_dir_all(path) {
                Ok(_) => {}
                Err(_e) => {
                    error!("Failed to remove vault directory.");
                    state.background_jobs.lock().unwrap().insert(
                        uuid,
                        BackgroundTaskState::FailOutput(
                            "Failed to remove vault directory.".to_string(),
                        ),
                    );
                }
            };
        }
        state
            .background_jobs
            .lock()
            .unwrap()
            .insert(uuid, BackgroundTaskState::Success);
    }));

    HttpResponse::Ok().body(uuid.to_string())
}

// Create new folder in vault
#[derive(Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct VaultNewFolderReq {
    directory_name: String,
    key: String,
}

/// Create new directory in vault.
#[utoipa::path(
    post,
    path = "/private/vault/directory",
    request_body = VaultNewFolderReq,
    responses((status = 200), (status = 500, description = "Directory name too long.")),
    tags = ["private", "vault"]
)]
pub async fn new_directory(json: Json<VaultNewFolderReq>) -> HttpResponse {
    let sent_path = &json.directory_name;

    if sent_path.split("/").last().unwrap().len() > 64 {
        return HttpResponse::InternalServerError()
            .json(ErrorCode::VaultPathTooLong.as_error_message());
    }

    let path = path::Path::new(&dirs::home_dir().unwrap().to_string_lossy().to_string())
        .join(".local")
        .join("share")
        .join("zentrox")
        .join("vault_directory")
        .join(
            sent_path
                .split("/")
                .filter(|x| !x.is_empty())
                .map(|x| vault::encrypt_string_hash(x.to_string(), &json.key.to_string()).unwrap())
                .collect::<Vec<String>>()
                .join("/"),
        );

    if path.exists() {
        return HttpResponse::Conflict().json(ErrorCode::DirectoryError);
    }

    let _ = fs::create_dir(&path);
    HttpResponse::Ok().json(MessageRes::from("A new directory has been created."))
}

// Upload vault file

#[derive(MultipartForm, ToSchema)]
pub struct VaultUploadForm {
    #[schema(value_type = String)]
    key: Text<String>,
    #[schema(value_type = String)]
    path: Text<String>,

    #[multipart(limit = "32 GiB")]
    #[schema(value_type = Vec<u8>)]
    file: TempFile,
}

/// Upload file to vault.
#[utoipa::path(
    post,
    path = "/private/vault/file",
    request_body(content_type = "mutlipart/form-data", content = VaultUploadForm,description = "The path and file to upload."),
    responses((status = 200), (status = 409, description = "File with same path already exists.")),
    tags = ["private", "vault"]
    )]
pub async fn upload(MultipartForm(form): MultipartForm<VaultUploadForm>) -> HttpResponse {
    let file_name = form
        .file
        .file_name
        .unwrap_or_else(|| "vault_default_file".to_string())
        .replace("..", "")
        .replace("/", "");

    let key = &form.key;

    if file_name == ".vault" {
        return HttpResponse::BadRequest().json(ErrorCode::FileError);
    }

    let base_path = path::Path::new(&dirs::home_dir().unwrap())
        .join(".local")
        .join("share")
        .join("zentrox")
        .join("vault_directory");

    let encrypted_path = form
        .path
        .split('/')
        .filter(|x| !x.is_empty()) // Filter out empty path entries
        .map(|x| vault::encrypt_string_hash(x.to_string(), key).unwrap())
        .collect::<Vec<String>>()
        .join("/");

    let in_vault_path = base_path
        .join(encrypted_path)
        .join(vault::encrypt_string_hash(file_name.to_string(), key).unwrap());

    if in_vault_path.exists() {
        return HttpResponse::Conflict().json(ErrorCode::FileError);
    }

    let tmp_file_path = form.file.file.path().to_owned();
    let _ = fs::copy(&tmp_file_path, &in_vault_path);

    let _ = tokio::fs::copy(&tmp_file_path, &in_vault_path).await;

    let block = actix_web::web::block(|| {
        let _ = tokio::fs::metadata(&tmp_file_path.clone()).then(async move |v| {
            let file_size = v.unwrap().len();
            let mut i = 0;
            while i != 5 {
                let random_data = (0..file_size)
                    .map(|_| rand::random::<u8>())
                    .collect::<Vec<u8>>();
                let _ = tokio::fs::write(&tmp_file_path, random_data);
                i += 1;
            }
            let _ = tokio::fs::remove_file(&tmp_file_path);
        });
    });

    drop(block);

    vault::encrypt_file(in_vault_path, key);

    HttpResponse::Ok().json(MessageRes::from("The upload has been finished."))
}

// Rename file/folder in vault
#[derive(Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct VaultRenameReq {
    origin: String,
    target: String,
    key: String,
}

/// Move a path in vault
///
/// The new name will be encrypted. The original name must be provided in non-encrypted form.
#[utoipa::path(
    post,
    path = "/private/vault/move",
    request_body = VaultRenameReq,
    responses((status = 404, description = "Origin not found."), (status = 400, description = "File can not be moved."), (status = 409, description = "Target path already exists.")),
    tags = ["private", "vault"]
)]
pub async fn rename_file(json: Json<VaultRenameReq>) -> HttpResponse {
    let sent_path = &json.origin;

    if sent_path == "/.vault" {
        return HttpResponse::BadRequest().json(ErrorCode::FileError);
    }

    let path = path::Path::new(&dirs::home_dir().unwrap().to_string_lossy().to_string())
        .join(".local")
        .join("share")
        .join("zentrox")
        .join("vault_directory")
        .join(
            sent_path
                .split("/")
                .filter(|x| !x.is_empty())
                .map(|x| vault::encrypt_string_hash(x.to_string(), &json.key.to_string()).unwrap())
                .collect::<Vec<String>>()
                .join("/"),
        );

    if !path.exists() {
        return HttpResponse::NotFound().json(ErrorCode::FileDoesNotExist.as_error_message());
    }

    let sent_new_path = &json.target;
    let new_path = path::Path::new(&dirs::home_dir().unwrap().to_string_lossy().to_string())
        .join(".local")
        .join("share")
        .join("zentrox")
        .join("vault_directory")
        .join(
            sent_new_path
                .split("/")
                .filter(|x| !x.is_empty())
                .map(|x| vault::encrypt_string_hash(x.to_string(), &json.key.to_string()).unwrap())
                .collect::<Vec<String>>()
                .join("/"),
        );

    if new_path.exists() {
        HttpResponse::Conflict().json(ErrorCode::FileError);
    }

    let _ = fs::rename(&path, &new_path);

    if new_path.metadata().unwrap().is_file() {
        let file_size = fs::metadata(&new_path).unwrap().len();
        let mut i = 0;

        while i != 5 {
            let random_data = (0..file_size)
                .map(|_| rand::random::<u8>())
                .collect::<Vec<u8>>();
            let _ = fs::write(&path, random_data);
            i += 1;
        }

        let _ = fs::remove_file(path);
    }

    HttpResponse::Ok().json(MessageRes::from("The file has been moved."))
}

// Download vault file
#[derive(Deserialize, ToSchema)]
pub struct VaultFileDownloadReq {
    key: String,
    path: String,
}

fn append_file_extension(p: PathBuf, extension: &str) -> PathBuf {
    let mut stringified = p.to_string_lossy().to_string();
    stringified.push_str(extension);
    PathBuf::from(stringified)
}

/// Download vault file.
///
/// This requires a password from the user in order to decrypt the file server side.
/// The file will not be provided in encrypted form.
#[utoipa::path(
    post,
    path = "/private/vault/download",
    request_body = VaultFileDownloadReq,
    responses((status = 200, content_type = "application/octet-stream"), (status = 400, description = "File can not be read."), (status = 404, description = "File does not exist.")),
    tags = ["private", "vault"]
)]
pub async fn download_file(json: Json<VaultFileDownloadReq>) -> HttpResponse {
    let sent_path = &json.path;
    let key = &json.key;

    if sent_path == "/.vault" {
        HttpResponse::BadRequest().json(ErrorCode::FileError);
    }

    let path = dirs::home_dir()
        .unwrap()
        .join(".local")
        .join("share")
        .join("zentrox")
        .join("vault_directory")
        .join(
            sent_path
                .split("/")
                .filter(|x| !x.is_empty())
                .map(|x| vault::encrypt_string_hash(x.to_string(), &json.key.to_string()).unwrap())
                .collect::<Vec<String>>()
                .join("/"),
        );

    let temporary_decrypted_file = append_file_extension(path.clone(), ".dec");

    let _ = fs::copy(path, temporary_decrypted_file.clone());

    vault::decrypt_file(temporary_decrypted_file.clone(), key);
    if temporary_decrypted_file.exists() {
        let f = fs::read(&temporary_decrypted_file);
        match f {
            Ok(fh) => {
                let data = fh.bytes().map(|x| x.unwrap_or(0_u8)).collect::<Vec<u8>>();
                let _ = vault::burn_file(temporary_decrypted_file.clone());
                let _ = fs::remove_file(temporary_decrypted_file.clone());
                HttpResponse::Ok().body(data)
            }
            Err(_) => {
                HttpResponse::InternalServerError().json(ErrorCode::FileError.as_error_message())
            }
        }
    } else {
        HttpResponse::NotFound().json(ErrorCode::FileDoesNotExist.as_error_message())
    }
}
