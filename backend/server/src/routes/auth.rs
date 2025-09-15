use actix_session::Session;
use actix_web::web::Json;
use actix_web::{HttpResponse, web::Data};
use diesel::prelude::*;
use log::info;
use serde::{Deserialize, Serialize};
use utils::database;
use utils::{
    database::get_administrator_account,
    otp,
    status_com::{ErrorCode, MessageRes},
};
use utoipa::ToSchema;

use crate::{AppState, SudoPasswordReq, is_admin};

#[derive(Deserialize, ToSchema)]
pub struct LoginReq {
    username: String,
    password: String,
    otp: Option<String>,
}

fn setup_login_state(session: Session, state: Data<AppState>, provided_username: String) {
    let login_token: Vec<u8> = is_admin::generate_random_token();
    let _ = session.insert("login_token", hex::encode(&login_token).to_string());

    *state.login_token.lock().unwrap() = hex::encode(&login_token).to_string();
    *state.username.lock().unwrap() = provided_username;

    let state_copy = state.clone();
    std::thread::spawn(move || {
        state_copy.update_network_statistics();
    });
}

/// Verify user and log in
#[utoipa::path(
    post,
    path = "/public/auth/login",
    responses(
        (status = 200, description = "The login was successful"),
        (status = 403, description = "A wrong password was provided."),
        (status = 401, description = "The username does not exist."),
        (status = 400, description = "Not enough information was provided.")
    ),
    request_body = LoginReq,
    tags = ["public", "authentication"]
)]
pub async fn login(session: Session, json: Json<LoginReq>, state: Data<AppState>) -> HttpResponse {
    let request_username = &json.username;
    let request_password = &json.password;
    let request_otp_code = &json.otp;

    let database_admin_entry = get_administrator_account();

    if &database_admin_entry.username != request_username {
        info!("A login with a wrong username will be denied.");
        return HttpResponse::Unauthorized().json(ErrorCode::UnkownUsername.as_error_message());
    }
    let stored_password: String = database_admin_entry.password_hash;
    let hashes_correct =
        is_admin::password_hash(request_password.to_string(), stored_password.to_string());

    if !hashes_correct {
        info!("A login with a wrong password will be denied.");
        return HttpResponse::Forbidden().json(ErrorCode::WrongPassword.as_error_message());
    }
    if database_admin_entry.use_otp {
        if json.otp.is_none() {
            info!("The user is missing an otp code.");
            return HttpResponse::BadRequest().json(ErrorCode::MissingOtpCode.as_error_message());
        }

        let stored_otp_secret = database_admin_entry.otp_secret.unwrap();

        if otp::calculate_current_otp(&stored_otp_secret) != request_otp_code.clone().unwrap() {
            info!("A login with a wrong OTP code will be denied.");
            return HttpResponse::Forbidden().json(ErrorCode::WrongOtpCode.as_error_message());
        }
        setup_login_state(session, state, database_admin_entry.username);
        HttpResponse::Ok().json(MessageRes::from("The login was successful."))
    } else {
        // User has logged in successfully using password
        setup_login_state(session, state, database_admin_entry.username);
        HttpResponse::Ok().json(MessageRes::from("The login was successful."))
    }
}

#[derive(Serialize, ToSchema)]
pub struct UseOtpRes {
    used: bool,
}

#[utoipa::path(
    get,
    path = "/public/auth/useOtp",
    responses((
            status = 200,
            body = UseOtpRes)),
    tags=["public", "authentication"]
)]
/// Does the user use OTP?
pub async fn use_otp(_state: Data<AppState>) -> HttpResponse {
    HttpResponse::Ok().json(UseOtpRes {
        used: get_administrator_account().use_otp,
    })
}

/// Logs a user out.
#[utoipa::path(
    get,
    path = "/private/logout",
    responses((status = 301, description = "User has been logged out successfully and will be redirected.")),
    tags = ["private", "authentication"]
)]
pub async fn logout(session: Session, state: Data<AppState>) -> HttpResponse {
    session.purge();
    *state.username.lock().unwrap() = "".to_string();
    // TODO Login token should be Option<String> and set to None if user is logged out
    *state.login_token.lock().unwrap() =
        hex::encode((0..64).map(|_| rand::random::<u8>()).collect::<Vec<u8>>()).to_string();
    HttpResponse::Found()
        .append_header(("Location", "/"))
        .body("You will soon be redirected")
}

#[derive(Deserialize, ToSchema)]
pub struct OtpActivationReq {
    active: bool,
}

/// Disable or enable 2FA using OTP
#[utoipa::path(
    put,
    path = "/private/auth/useOtp",
    responses(
            (status = 200, description = "Status updated."),
    ),
    request_body = OtpActivationReq,
    tags = ["authentication", "private"]
)]
pub async fn activate_otp(json: Json<OtpActivationReq>) -> HttpResponse {
    use utils::schema::Admin::dsl::*;
    let connection = &mut database::establish_connection();

    let status: bool = json.active;

    let status_update_execution = diesel::update(Admin)
        .set(use_otp.eq(status))
        .execute(connection);

    if let Err(update_error) = status_update_execution {
        return HttpResponse::InternalServerError()
            .json(ErrorCode::DatabaseReadFailed(update_error.to_string()).as_error_message());
    }

    if status {
        let secret = otp::generate_otp_secret();

        let secret_update_execution = diesel::update(Admin)
            .set(otp_secret.eq(Some(secret.clone())))
            .execute(connection);

        if let Err(update_error) = secret_update_execution {
            return HttpResponse::InternalServerError()
                .json(ErrorCode::DatabaseReadFailed(update_error.to_string()).as_error_message());
        }

        HttpResponse::Ok().body(secret)
    } else {
        let secret_reset_execution = diesel::update(Admin)
            .set(otp_secret.eq(None::<String>))
            .execute(connection);

        if let Err(update_error) = secret_reset_execution {
            return HttpResponse::InternalServerError()
                .json(ErrorCode::DatabaseReadFailed(update_error.to_string()).as_error_message());
        }

        HttpResponse::Ok().json(MessageRes::from("Updated OTP activation."))
    }
}

/// Verifies a given sudo password
#[utoipa::path(
    post,
    path = "/private/auth/sudo/verify",
    request_body = SudoPasswordReq,
    responses((status = 401, description = "Wrong sudo password"), (status = 200, description = "Correct sudo password")),
    tags=["private", "authentication"])]
pub async fn verify_sudo_password(json: Json<SudoPasswordReq>) -> HttpResponse {
    if !utils::sudo::verify_password(json.sudo_password.clone()) {
        return HttpResponse::Unauthorized().json(ErrorCode::BadSudoPassword.as_error_message());
    }

    return HttpResponse::Ok().json(MessageRes::from("Sudo password is correct"));
}
