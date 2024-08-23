use actix_session::Session;

pub fn is_admin(session: &Session) -> bool {
    match session
        .get::<bool>("is_admin")
        .expect("Failed to get is_admin")
    {
        Some(value) => return value == true,
        None => return false,
    }
}
