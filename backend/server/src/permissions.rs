use actix_session::Session;
use actix_web::web;
use log::{error, info, warn};
use rand::Rng;
use std::thread;
use utils::crypto_utils;

use crate::AppState;

pub const SESSION_TIMEOUT_SECS: u64 = 60 * 60 * 12; // = 12h

/// Checks if a user is admin.
///
/// The function requires two arguments:
/// * `session` - The current session from the handler
/// * `state` - The current server state in form of the AppState struct.
///
/// The function will then compare the states login token to the token provided by the session.
/// In case no token is provided by the sessions, false is returned as default.
///
/// The variable disable_auth_for_development can be used during development to disable
/// authentication.
/// If the function is left enabled a warning will be printed in the terminal. Otherwise,
/// nothing is shown.
pub fn is_admin_state(session: &Session, state: web::Data<AppState>) -> bool {
    let mut vars = std::env::vars();
    let disable_auth_for_development: bool = vars.any(|x| {
        x == ("ZENTROX_MODE".to_string(), "NO_AUTH".to_string())
            || x == ("ZENTROX_MODE".to_string(), "DEV".to_string())
    });
    if disable_auth_for_development {
        warn!("Authentication has been disabled for development. This is a massive security risk!");
        return true;
    }

    info!("Verifying request for permissions.");

    if let Some(t) = *state.last_login.lock().unwrap() {
        let time_delta_seconds = utils::time::current().duration_since(t).unwrap().as_secs();
        info!("User time delta was {}s.", time_delta_seconds);
        if time_delta_seconds >= SESSION_TIMEOUT_SECS {
            warn!("Session reached time-out.");
            reset_session(&state, session);
            info!("Session defaults restored.");
            return false;
        } else {
            info!("Time delta was in range.");
        }
    } else {
        warn!("No active session.");
        return false;
    }

    match session.get::<String>("login_token") {
        Ok(value) => {
            match value {
                Some(value) => match hex::decode(&value) {
                    Ok(decoded) => {
                        if decoded.len() != 16 {
                            warn!(
                                "The authentication token was not 16 characters long and was deemed invalid"
                            );
                            return false;
                        }

                        if (*state).login_token.lock().unwrap().is_none() {
                            warn!(
                                "No user was logged in at the time of an attempted private request."
                            );
                            return false;
                        }

                        return value == (*state).login_token.lock().unwrap().clone().unwrap();
                    }
                    Err(_) => {
                        error!(
                            "The authentication token could not be decoded and was deemed invalid"
                        );
                        false
                    }
                },
                None => {
                    error!("There was no authentication token.");
                    false
                }
            }
            // Handle invalid hex decode safely
        }
        Err(_err) => {
            error!("Login token field was missing.");
            false
        }
    }
}

pub fn generate_random_token() -> Vec<u8> {
    let mut rng = rand::rngs::OsRng;
    let token: [u8; 16] = rng.r#gen(); // Generate 16 random bytes
    token.to_vec()
}

/// Check if the password provided during login matches with the password stored on the server
/// side. It uses Argon2.
pub fn password_hash(login_password: String, stored_hash: String) -> bool {
    let hash = hex::encode(crypto_utils::argon2_derive_key(&login_password).unwrap());

    hash == stored_hash
}

/// Resets the user session to its default values, logging the active user out in the process.
pub fn reset_session(state: &web::Data<AppState>, session: &Session) {
    session.purge();
    warn!("Session was purged.");
    let un_cpy = state.clone();
    let lt_cpy = state.clone();
    let ll_cpy = state.clone();
    thread::spawn(move || {
        let mut v = un_cpy.username.lock().unwrap();
        *v = None
    });
    thread::spawn(move || {
        let mut v = lt_cpy.login_token.lock().unwrap();
        *v = None
    });
    thread::spawn(move || {
        let mut v = ll_cpy.last_login.lock().unwrap();
        *v = None
    });
}
