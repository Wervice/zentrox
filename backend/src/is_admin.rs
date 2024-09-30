use actix_session::Session;
use actix_web::web;
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
    let disable_auth_for_development = false; // 🚨 DO NOT LEAVE THIS ON DURING RELEASE / PROD
    if disable_auth_for_development {
        println!(include_str!("../notes/auth_note.txt"));
        return true;
    }

    match session.get::<String>("login_token") {
        Ok(value) => {
            match value {
                Some(value) => {
                    match hex::decode(&value) {
                        Ok(decoded) => {
                            if decoded.len() != 16 {
                                return false;
                            }
                            return value == *state.login_token.lock().unwrap();
                        }
                        Err(_) => {
                            // If decoding fails, consider the token invalid
                            false
                        }
                    }
                }
                None => false,
            }
            // Handle invalid hex decode safely
        }
        Err(err) => {
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
/// side. It uses SHA 512 PBKDF2.
pub fn password_hash(login_password: String, stored_hash: String) -> bool {
    let hash = hex::encode(&crypto_utils::argon2_derive_key(&login_password).unwrap()); 

    hash == stored_hash
}
