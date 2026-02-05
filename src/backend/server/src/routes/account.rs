use crate::{
    AppState,
    permissions::{self, locate_session},
};
use actix_multipart::form::{MultipartForm, tempfile::TempFile};
use actix_session::Session;
use actix_web::{
    HttpResponse,
    web::{Data, Json},
};
use argon2::password_hash::SaltString;
use diesel::prelude::*;
use log::error;
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};
use std::{fs, io::Read, path, time::UNIX_EPOCH};
use utils::{
    crypto_utils::encrypt_bytes,
    otp::{derive_otp_url, generate_otp_secret},
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
pub async fn details(state: Data<AppState>, session: Session) -> HttpResponse {
    let find_current_session = locate_session(&session, &state);

    if let Ok(current_session) = find_current_session {
        HttpResponse::Ok().json(AccountDetailsRes {
            username: current_session.username.clone(),
        })
    } else {
        HttpResponse::NotFound().json(ErrorCode::InsufficientData)
    }
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
pub async fn update_details(
    json: Json<UpdateAccountReq>,
    session: Session,
    state: Data<AppState>,
) -> HttpResponse {
    use schema::Users::dsl::*;
    let connection = &mut state.db_pool.lock().unwrap().get().unwrap();

    let request_password = &json.password;
    let request_username = &json.username;

    let current_session =
        permissions::locate_session(&session, &state).expect("User session does not exist.");

    let current_ts = std::time::SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64;

    if !request_password.is_empty() {
        let hashed_request_password = utils::crypto_utils::argon2_derive_key(
            request_password,
            SaltString::generate(&mut OsRng),
        )
        .to_string()
        .to_string();

        let password_execution = diesel::update(Users)
            .filter(username.eq(&current_session.username))
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
        let username_execution = diesel::update(Users)
            .filter(username.eq(current_session.username.clone()))
            .set(username.eq(request_username))
            .execute(connection);

        if let Err(database_error) = username_execution {
            return HttpResponse::InternalServerError().json(
                ErrorCode::DatabaseUpdateFailed(database_error.to_string()).as_error_message(),
            );
        }
    }

    HttpResponse::Ok().json(MessageRes::from(
        "Account details have been updated. Start a new session to see changes apply.",
    ))
}

#[derive(Deserialize, ToSchema)]
pub struct OtpActivationReq {
    active: bool,
    password: Option<String>,
}

#[utoipa::path(
    put,
    path = "/private/account/enableOtp",
    responses(
            (status = 200, description = "Status updated."),
    ),
    request_body = OtpActivationReq,
    tags = ["authentication", "private"]
)]
pub async fn enable_otp(
    json: Json<OtpActivationReq>,
    session: Session,
    state: actix_web::web::Data<AppState>,
) -> HttpResponse {
    use utils::schema::Users::dsl::*;
    let connection = &mut state.db_pool.lock().unwrap().get().unwrap();

    let status: bool = json.active;

    let current_username = locate_session(&session, &state).unwrap().username;

    let status_update_execution = diesel::update(Users)
        .filter(username.eq(current_username.clone()))
        .set(use_otp.eq(status))
        .execute(connection);

    if let Err(update_error) = status_update_execution {
        return HttpResponse::InternalServerError()
            .json(ErrorCode::DatabaseReadFailed(update_error.to_string()).as_error_message());
    }

    if status {
        let secret = generate_otp_secret();
        let encrypted_secret =
            encrypt_bytes(secret.as_bytes(), &json.password.clone().unwrap()).to_string();

        let secret_update_execution = diesel::update(Users)
            .filter(username.eq(&current_username))
            .set(otp_secret.eq(Some(encrypted_secret.clone())))
            .execute(connection);

        if let Err(update_error) = secret_update_execution {
            return HttpResponse::InternalServerError()
                .json(ErrorCode::DatabaseReadFailed(update_error.to_string()).as_error_message());
        }

        HttpResponse::Ok().body(derive_otp_url(secret, current_username))
    } else {
        let secret_reset_execution = diesel::update(Users)
            .filter(username.eq(current_username))
            .set(otp_secret.eq(None::<String>))
            .execute(connection);

        if let Err(update_error) = secret_reset_execution {
            return HttpResponse::InternalServerError()
                .json(ErrorCode::DatabaseReadFailed(update_error.to_string()).as_error_message());
        }

        HttpResponse::Ok().json(MessageRes::from("Updated OTP activation."))
    }
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
