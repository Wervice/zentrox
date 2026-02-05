use std::net::IpAddr;
use std::time::SystemTime;

use actix_session::Session;
use actix_web::HttpRequest;
use actix_web::web::Json;
use actix_web::{HttpResponse, web::Data};
use diesel::prelude::*;
use log::{error, info, warn};
use serde::{Deserialize, Serialize};
use utils::crypto_utils::{self, Ciphertext, decrypt_bytes};
use utils::models::LoginRequest;
use utils::status_com::{ErrorCode, MessageRes};
use utoipa::ToSchema;

use crate::permissions::{
    ACCOUNT_REQUEST_LIMIT, ACCOUNT_TIME_WINDOW, BASE_REQUEST_LIMIT, BASE_TIME_WINDOW,
    LARGE_REQUEST_LIMIT, LARGE_TIME_WINDOW, LoginAction, locate_session, register_session,
    remove_session,
};
use crate::{AppState, SudoPasswordReq, permissions};

#[derive(Deserialize, ToSchema)]
pub struct LoginReq {
    username: String,
    password: String,
    otp: Option<String>,
}

fn since_now(earlier: SystemTime) -> u64 {
    SystemTime::now().duration_since(earlier).unwrap().as_secs()
}

/// Simultaneously stores request in database (unless the request is blocked) and in the AppState.
fn store_request(req_username: String, req_ip: IpAddr, req_action: LoginAction, state: &AppState) {
    use utils::models::LoginRequest;
    use utils::schema::LoginRequestHistory::dsl::*;

    // To prevent flooding the database, blocked requests are not stored
    if req_action != LoginAction::Blocked {
        let _ = diesel::insert_into(LoginRequestHistory)
            .values(LoginRequest {
                action: req_action.to_string(),
                id: uuid::Uuid::new_v4().to_string(),
                time: utils::time::current_timestamp_unix() as i64,
                username: req_username.clone(),
                ip: req_ip.to_string(),
            })
            .execute(&mut state.db_pool.lock().unwrap().get().unwrap());
    }

    state
        .login_requests
        .lock()
        .unwrap()
        .push(permissions::LoginRequest {
            time: SystemTime::now(),
            action: req_action,
            ip: req_ip,
            username: req_username,
        });
}

fn detects_spamming(state: &AppState, req_username: &String, req_ip: IpAddr) -> bool {
    let lock = state.login_requests.lock().unwrap();
    let past_requests = lock.clone();
    drop(lock);

    // Get the total number of all requests to the specified account that failed in the
    // ACCOUNT_TIME_WINDOW
    let total_denying_request = past_requests
        .iter()
        .filter(|r| {
            r.username == *req_username
                && r.action.is_denying()
                && since_now(r.time) < ACCOUNT_TIME_WINDOW
        })
        .count();

    if total_denying_request > ACCOUNT_REQUEST_LIMIT {
        // It appears, someone is trying to brute-force the credentials through multiple peers.
        store_request(
            req_username.to_string(),
            req_ip,
            LoginAction::Limited,
            state,
        );
        warn!("Too many failed attempts were performed to login to {req_username}.");
        return true;
    }

    let ip_matching_denying_request = past_requests
        .iter()
        .filter(|r| r.username == *req_username && r.action.is_denying() && r.ip == req_ip);

    // Get the number of all requests by this IP to the specified account in the BASE_TIME_WINDOW
    let ip_base_window_reqs = ip_matching_denying_request
        .clone()
        .filter(|r| since_now(r.time) < BASE_TIME_WINDOW)
        .count();
    // Get the number of all requests by this IP to the specified account in the LARGE_TIME_WINDOW
    let ip_large_window_reqs = ip_matching_denying_request
        .clone()
        .filter(|r| since_now(r.time) < LARGE_TIME_WINDOW)
        .count();

    if ip_large_window_reqs > LARGE_REQUEST_LIMIT {
        permissions::block_ip(state, req_ip);
        store_request(
            req_username.to_string(),
            req_ip,
            LoginAction::Blocked,
            state,
        );
        warn!("Login requests seem to be persistent brute-forcing and will be blocked.");
        return true;
    }

    if ip_base_window_reqs > BASE_REQUEST_LIMIT {
        store_request(req_username.clone(), req_ip, LoginAction::Limited, state);
        warn!("Login requests seem suspicious and will be blocked for this cycle.");
        return true;
    }

    false
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
pub async fn login(
    session: Session,
    json: Json<LoginReq>,
    state: Data<AppState>,
    req: HttpRequest,
) -> HttpResponse {
    use utils::models::Account;
    use utils::schema::Users::dsl::*;

    let req_ip = req.peer_addr().unwrap().ip();
    let req_username = &json.username;
    let req_password = &json.password;
    let req_otp_token = &json.otp;

    if detects_spamming(&state, req_username, req_ip) {
        return HttpResponse::TooManyRequests()
            .body("Your request was identified as spam and has been rejected.");
    }
    info!("No spamming detected.");

    // Find account entry for specified user, reject request if no such account exists
    let user_query: Result<Account, _> = Users
        .select(Account::as_select())
        .filter(username.eq(req_username))
        .get_result(&mut state.db_pool.lock().unwrap().get().unwrap());

    let correct_user: Account = match user_query {
        Ok(v) => v,
        Err(diesel::NotFound) => {
            store_request(
                req_username.to_string(),
                req_ip,
                LoginAction::Rejected,
                &state,
            );
            warn!("Unknown username detected.");
            return HttpResponse::Forbidden().json(ErrorCode::UnkownUsername);
        }
        Err(e) => {
            error!("Failed to read user list with error: {e}");
            return HttpResponse::InternalServerError().json(
                ErrorCode::DatabaseReadFailed(
                    "Reading the user list from the database failed.".to_string(),
                )
                .as_error_message(),
            );
        }
    };

    let stored_password = correct_user.password_hash.clone();

    if !crypto_utils::verify_with_hash(&stored_password, req_password) {
        store_request(
            req_username.to_string(),
            req_ip,
            LoginAction::Rejected,
            &state,
        );
        warn!("Wrong password detected.");
        return HttpResponse::Forbidden().json(ErrorCode::WrongPassword);
    }

    if !correct_user.use_otp {
        store_request(
            req_username.to_string(),
            req_ip,
            LoginAction::Approved,
            state.as_ref(),
        );
        info!("New login as {req_username} from {req_ip}");
        register_session(session, state.as_ref(), req_username.to_string(), req_ip);
        return HttpResponse::Ok().json(MessageRes::from(format!(
            "You have been logged in as {req_username}."
        )));
    }

    if let Some(token) = req_otp_token
        && token.len() == 8
    {
        let enc_otp_secret = correct_user.otp_secret.clone().unwrap();
        let dec_otp_secret = decrypt_bytes(Ciphertext::from(enc_otp_secret), req_password);
        let token_check = utils::otp::verify_current_otp(
            String::from_utf8(dec_otp_secret.unwrap()).unwrap(),
            token,
        );
        if let Ok(correct_token) = token_check
            && correct_token
        {
            store_request(
                req_username.to_string(),
                req_ip,
                LoginAction::Approved,
                state.as_ref(),
            );
            register_session(session, state.as_ref(), req_username.to_string(), req_ip);
            info!("New login as {req_username} from {req_ip}");
            HttpResponse::Ok().json(MessageRes::from(format!(
                "You have been logged in as {req_username}."
            )))
        } else {
            store_request(
                req_username.to_string(),
                req_ip,
                LoginAction::Rejected,
                &state,
            );
            warn!("Login request has a wrong otp code.");
            HttpResponse::BadRequest().json(ErrorCode::WrongOtpCode)
        }
    } else {
        store_request(
            req_username.to_string(),
            req_ip,
            LoginAction::Rejected,
            &state,
        );
        warn!("Login request was missing an OTP code.");
        HttpResponse::UnprocessableEntity().json(ErrorCode::MissingOtpCode)
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

    HttpResponse::Ok().json(MessageRes::from("The provided sudo password is correct."))
}

/// Logs a user out.
#[utoipa::path(
    post,
    path = "/private/auth/logout",
    responses((status = 301, description = "User has been logged out successfully and will be redirected.")),
    tags = ["private", "authentication"]
)]
pub async fn logout(session: Session, state: Data<AppState>) -> HttpResponse {
    let current = locate_session(&session, &state).expect("No login session found.");
    remove_session(current, &state);
    session.purge();
    HttpResponse::Found()
        .append_header(("Location", "/"))
        .body("Redirecting...")
}

#[derive(Serialize)]
pub struct RequestHistoryRes {
    history: Vec<LoginRequest>,
}

/// Logs a user out.
#[utoipa::path(
    post,
    path = "/private/auth/requestHistory",
    responses((status = 301, description = "History of all attempts to login")),
    tags = ["private", "authentication"]
)]
pub async fn request_history(state: actix_web::web::Data<AppState>) -> HttpResponse {
    use utils::schema::LoginRequestHistory::dsl::*;

    let exec = LoginRequestHistory
        .select(LoginRequest::as_select())
        .get_results(&mut state.db_pool.lock().unwrap().get().unwrap());

    match exec {
        Ok(v) => HttpResponse::Ok().json(RequestHistoryRes { history: v }),
        Err(e) => {
            HttpResponse::InternalServerError().json(ErrorCode::DatabaseReadFailed(e.to_string()))
        }
    }
}
