use std::fs;
use std::path::PathBuf;
use std::time::UNIX_EPOCH;

use actix_web::HttpResponse;
use actix_web::web::{Data, Json, Path};
use argon2::password_hash::SaltString;
use diesel::prelude::*;
use log::warn;
use rand::Rng;
use rand::distributions::Alphanumeric;
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};
use utils::crypto_utils::{argon2_derive_key, verify_with_hash};
use utils::models::SharedFile;
use utils::status_com::{ErrorCode, MessageRes};
use utils::{models, schema};
use utoipa::ToSchema;

use crate::AppState;

#[derive(Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct FileSharingReq {
    #[schema(value_type = String)]
    file_path: PathBuf,
    password: Option<String>,
}

#[utoipa::path(post, path = "/private/sharing/new", request_body = FileSharingReq, responses((status = 200)), tags = ["private", "sharing"])]
/// Create new file sharing
pub async fn share(json: Json<FileSharingReq>, state: Data<AppState>) -> HttpResponse {
    use models::SharedFile;
    use schema::FileSharing;

    let code: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(64)
        .map(char::from)
        .collect();

    let file_path = &json.file_path.clone();
    let password = &json.password.clone();

    if !file_path.exists() {
        return HttpResponse::NotFound().json(ErrorCode::FileDoesNotExist.as_error_message());
    }

    let password_insert_value = match password {
        Some(v) => {
            let hashed_password = argon2_derive_key(v, SaltString::generate(&mut OsRng));
            Some(hashed_password.to_string())
        }
        None => None,
    };

    let new_shared_file = SharedFile {
        code: code.clone(),
        file_path: file_path.to_str().unwrap().to_string(),
        use_password: password.is_some(),
        password: password_insert_value,
        shared_since: std::time::SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as i64,
    };

    let sharing_creation_database_execution = diesel::insert_into(FileSharing::dsl::FileSharing)
        .values(&new_shared_file)
        .on_conflict(FileSharing::dsl::code)
        .do_update()
        .set(&new_shared_file)
        .execute(&mut state.db_pool.lock().unwrap().get().unwrap());

    if let Err(database_error) = sharing_creation_database_execution {
        return HttpResponse::InternalServerError()
            .json(ErrorCode::DatabaseInsertFailed(database_error.to_string()));
    }

    HttpResponse::Ok().body(code)
}

#[derive(Serialize, ToSchema)]
struct SharedFilesListRes {
    files: Vec<SharedFile>,
}

#[utoipa::path(get, path = "/private/sharing/list", responses((status = 200, body = SharedFilesListRes)), tags = ["private", "sharing"])]
/// List of shared files
pub async fn list(state: Data<AppState>) -> HttpResponse {
    use models::SharedFile;
    use schema::FileSharing::dsl::*;

    let files: Vec<SharedFile> = FileSharing
        .select(SharedFile::as_select())
        .get_results(&mut state.db_pool.lock().unwrap().get().unwrap())
        .unwrap();

    HttpResponse::Ok().json(SharedFilesListRes { files })
}

#[derive(Deserialize, ToSchema)]
pub struct SharedFileReq {
    code: String,
    password: Option<String>,
}

#[utoipa::path(post, path = "/public/shared/get", request_body = SharedFileReq, responses((status = 200, content_type = "application/octet-stream")), tags = ["public", "sharing"])]
/// Selects the file with the specified code, verifying if the password is correct, if a password
/// was required and returns its contents.
pub async fn download_file(json: Json<SharedFileReq>, state: Data<AppState>) -> HttpResponse {
    use models::SharedFile;
    use schema::FileSharing;

    let request_password = &json.password;

    let connection = &mut state.db_pool.lock().unwrap().get().unwrap();

    let shared_files: Vec<SharedFile> = FileSharing::dsl::FileSharing
        .select(SharedFile::as_select())
        .get_results(connection)
        .unwrap();

    let file_query = shared_files.into_iter().find(|e| e.code == json.code);

    // Check if the requested code even exists
    if let Some(correct_file) = file_query {
        let password_checking = correct_file.use_password;
        let database_hash = correct_file.password.clone();

        if password_checking && request_password.is_none() {
            return HttpResponse::Forbidden()
                .json(ErrorCode::MissingSharedFilePermissions.as_error_message());
        }

        let file_path = correct_file.file_path.clone();
        let file_contents = fs::read(file_path).unwrap();

        if password_checking {
            if verify_with_hash(
                &database_hash.expect("Missing file sharing password hash."),
                request_password.as_ref().unwrap(),
            ) {
                return HttpResponse::Ok().body(file_contents);
            } else {
                warn!("User entered wrong file sharing password.");
                return HttpResponse::Forbidden()
                    .json(ErrorCode::MissingSharedFilePermissions.as_error_message());
            }
        }

        return HttpResponse::Ok().body(file_contents);
    }
    HttpResponse::NotFound().json(ErrorCode::NoSuchSharedFile.as_error_message())
}

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
struct SharedFileMetadataRes {
    file_path: String,
    use_password: bool,
    size: u32,
}

#[utoipa::path(get, path = "/private/shared/getMetadata", request_body = SharedFileReq, responses((status = 200, body = SharedFileMetadataRes)), tags = ["public", "sharing"])]
/// Metadata of shared file
pub async fn get_metadata(json: Json<SharedFileReq>, state: Data<AppState>) -> HttpResponse {
    use models::SharedFile;
    use schema::FileSharing::dsl::*;

    let f: Vec<SharedFile> = FileSharing
        .select(SharedFile::as_select())
        .filter(code.eq(&json.code))
        .get_results(&mut state.db_pool.lock().unwrap().get().unwrap())
        .unwrap();

    if f.is_empty() {
        return HttpResponse::BadRequest().json(ErrorCode::InsufficientData.as_error_message());
    }

    let p = PathBuf::from(f[0].file_path.clone());

    if !p.exists() {
        return HttpResponse::NotFound().json(ErrorCode::NoSuchSharedFile.as_error_message());
    }

    let size = p.metadata().unwrap().len();

    HttpResponse::Ok().json(SharedFileMetadataRes {
        file_path: f[0].file_path.clone(),
        use_password: f[0].use_password,
        size: size as u32,
    })
}

#[utoipa::path(post, path = "/private/sharing/delete/{code}", params(("code" = String, Path)), responses((status = 200)), tags = ["private", "sharing"])]
/// Delete file sharing
pub async fn unshare(request_code: Path<String>, state: Data<AppState>) -> HttpResponse {
    use schema::FileSharing::dsl::*;

    let delete_file_sharing_database_execution = diesel::delete(FileSharing)
        .filter(code.eq(request_code.into_inner()))
        .execute(&mut state.db_pool.lock().unwrap().get().unwrap());

    if let Err(database_error) = delete_file_sharing_database_execution {
        return HttpResponse::InternalServerError().json(ErrorCode::DatabaseDeletionFailed(
            database_error.to_string(),
        ));
    }

    HttpResponse::Ok().json(MessageRes::from("The file is no longer being shared."))
}
