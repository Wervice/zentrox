use actix_session::Session;
use actix_web::web;
use log::{debug, warn};
use rand::Rng;

use crate::{crypto_utils, AppState};

/// Checks if a user is admin.
///
/// The function requires two arguments:
/// * `session` - The current session from the handler
/// * `state` - The current server state in form of the AppState struct.
///
/// The function willthen compare the states login token to the token provided by the session.
/// In case no token is provided by the sessions, false is returned as default.
///
/// The variable disable_auth_for_development can be used during development to disable
/// authentication.
/// If the function is left enabled a warning will be printend in the terminal. Otherwise
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

    match session.get::<String>("login_token") {
        Ok(value) => {
            match value {
                Some(value) => {
                    match hex::decode(&value) {
                        Ok(decoded) => {
                            if decoded.len() != 16 {
                                debug!("The authentication token was not 16 characters long and was deemed invalid");
                                return false;
                            }
                            if value != *state.login_token.lock().unwrap() {
                                debug!("The authentication token was unequal to the correct token");
                            }
                            return value == *state.login_token.lock().unwrap();
                        }
                        Err(_) => {
                            debug!("The authentication token could not be decoded and was deemed invalid");
                            false
                        }
                    }
                }
                None => {
                    debug!("There was no authentication token");
                    false
                }
            }
            // Handle invalid hex decode safely
        }
        Err(err) => {
            debug!("There was no authentication token");
            eprintln!("❌ Failed to get login_token from session: {err}");
            false
        }
    }
}

pub fn generate_random_token() -> Vec<u8> {
    let mut rng = rand::rngs::OsRng;
    let token: [u8; 16] = rng.gen(); // Generate 16 random bytes
    token.to_vec()
}

/// Check if the password provided during login matches with the password stored on the server
/// side. It uses Argon2.
pub fn password_hash(login_password: String, stored_hash: String) -> bool {
    let hash = hex::encode(crypto_utils::argon2_derive_key(&login_password).unwrap());

    hash == stored_hash
}
