use crate::schema::Secrets::dsl::*;
use crate::{database::establish_connection, models::Secret};
use argon2::{
    password_hash::{rand_core::OsRng, SaltString},
    Argon2,
};
use diesel::{self, RunQueryDsl};

use crate::database::get_secret_by_name;

/// Derive a key using Argon2. The keylength is set to 32 bytes.
/// It uses the salt stored in the config file under a2_salt.
pub fn argon2_derive_key(password: &str) -> Option<[u8; 32]> {
    let salt: SaltString;
    if get_secret_by_name("a2_salt").unwrap_or_default().is_empty() {
        salt = SaltString::generate(&mut OsRng);
        let new_secret = Secret {
            name: "a2_salt".to_string(),
            value: Some(salt.as_ref().to_string()),
        };

        let connection = &mut establish_connection();

        let w = diesel::insert_into(Secrets)
            .values(&new_secret)
            .on_conflict(name)
            .do_update()
            .set(&new_secret)
            .execute(connection);

        match w {
            Ok(_) => {}
            Err(e) => {
                eprintln!("{e}");
                return None;
            }
        }
    } else {
        salt = SaltString::from_b64(get_secret_by_name("a2_salt").unwrap_or_default().as_ref())
            .unwrap();
    }

    let params = argon2::Params::new(4096, 3, 1, Some(32)).unwrap();
    let argon2 = Argon2::new(argon2::Algorithm::Argon2id, argon2::Version::V0x13, params);

    let mut output = [0u8; 32];
    let _ = argon2.hash_password_into(
        password.as_bytes(),
        salt.to_string().as_bytes(),
        &mut output,
    );

    Some(output)
}
