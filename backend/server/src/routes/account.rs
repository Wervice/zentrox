use crate::AppState;
use actix_multipart::form::{MultipartForm, tempfile::TempFile};
use actix_web::{
    HttpResponse,
    web::{Data, Json},
};
use diesel::prelude::*;
use log::error;
use serde::{Deserialize, Serialize};
use std::{fs, io::Read, path, time::UNIX_EPOCH};
use utils::{
    database::establish_connection,
    schema,
    status_com::{ErrorCode, MessageRes},
};
use utoipa::ToSchema;

#[derive(Serialize, ToSchema)]
struct AccountDetailsRes {
    username: String,
}

/// Admin account details
#[utoipa::path(
    get,
    path = "/private/account/details",
    responses((status = 200, body = AccountDetailsRes)),
    tags = ["private", "account"]
)]
pub async fn details(state: Data<AppState>) -> HttpResponse {
    let state_username = match state.username.lock() {
        Ok(v) => v,
        Err(e) => e.into_inner(),
    };

    HttpResponse::Ok().json(AccountDetailsRes {
        username: state_username.to_string(),
    })
}

#[derive(Deserialize, ToSchema)]
pub struct UpdateAccountReq {
    password: String,
    username: String,
}

/// Update account details
#[utoipa::path(
    post,
    path = "/private/account/details",
    request_body = UpdateAccountReq,
    responses((status = 200)),
    tags = ["private", "account"]
)]
pub async fn update_details(json: Json<UpdateAccountReq>) -> HttpResponse {
    use schema::Admin::dsl::*;
    let connection = &mut establish_connection();

    let request_password = &json.password;
    let request_username = &json.username;

    let current_ts = std::time::SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64;

    if !request_password.is_empty() {
        let hashed_request_password =
            hex::encode(utils::crypto_utils::argon2_derive_key(request_password).unwrap())
                .to_string();

        let password_execution = diesel::update(Admin)
            .set((
                password_hash.eq(hashed_request_password),
                updated_at.eq(current_ts),
            ))
            .execute(connection);

        if let Err(database_error) = password_execution {
            return HttpResponse::InternalServerError().json(
                ErrorCode::DatabaseUpdateFailed(database_error.to_string()).as_error_message(),
            );
        }
    }

    if !request_username.is_empty() {
        let username_execution = diesel::update(Admin)
            .set(username.eq(request_username))
            .execute(connection);

        if let Err(database_error) = username_execution {
            return HttpResponse::InternalServerError().json(
                ErrorCode::DatabaseUpdateFailed(database_error.to_string()).as_error_message(),
            );
        }
    }

    HttpResponse::Ok().json(MessageRes::from("Account details have been updated."))
}

/// Admin profile picture
#[utoipa::path(
    get,
    path = "/private/account/profilePicture",
    responses((status = 200, body = &[u8], content_type = "image/")),
    tags = ["private", "account"]
)]
pub async fn picture() -> HttpResponse {
    let f = fs::read(
        path::Path::new(&dirs::home_dir().unwrap())
            .join(".local")
            .join("share")
            .join("zentrox")
            .join("profile.png"),
    );

    match f {
        Ok(fh) => {
            HttpResponse::Ok().body(fh.bytes().map(|x| x.unwrap_or(0_u8)).collect::<Vec<u8>>())
        }
        Err(_) => HttpResponse::NotFound().json(ErrorCode::FileDoesNotExist.as_error_message()),
    }
}

#[derive(MultipartForm, ToSchema)]
pub struct ProfilePictureUploadForm {
    #[multipart(limit = "2MB")]
    #[schema(value_type = Vec<u8>)]
    file: TempFile,
}

/// Upload admin profile picture.
///
/// The picture may not be larger than 2MB in order to keep loading time down.
#[utoipa::path(
    post,
    path = "/private/account/profilePicture",
    request_body = ProfilePictureUploadForm,
    responses((status = 200)),
    tags = ["private", "account"]
)]
pub async fn upload_picture(
    MultipartForm(form): MultipartForm<ProfilePictureUploadForm>,
) -> HttpResponse {
    let profile_picture_path = path::Path::new(&dirs::home_dir().unwrap())
        .join(".local")
        .join("share")
        .join("zentrox")
        .join("profile.png");

    let tmp_file_path = form.file.file.path().to_owned();
    let _ = fs::copy(&tmp_file_path, &profile_picture_path);

    match fs::remove_file(&tmp_file_path) {
        Ok(_) => HttpResponse::Ok().json(MessageRes::from("The profile picture has been updated.")),
        Err(_) => {
            error!(
                "Remove a temporary file at {} failed.",
                tmp_file_path.to_string_lossy()
            );
            HttpResponse::InternalServerError().json(ErrorCode::FileError.as_error_message())
        }
    }
}
