use actix_session::Session;
use actix_web::web;

use crate::AppState;

/// Checks if a user is admin.
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
        println!(
            "
            ⚠️ AUTH IS DISABLED
            ----------------------------------------------------------------
            🛑 Auth is disabled for development.
            🛑 Stop the program immediately if you are running this in prod!
            🛑 This is intended for development only!
            ----------------------------------------------------------------
            "
        );
        return true;
    }

    match session
        .get::<String>("login_token")
        .expect("Failed to get login_token")
    {
        Some(value) => value == *state.login_token.lock().unwrap(),
        None => false,
    }
}
