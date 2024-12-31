use argon2::{
    password_hash::{rand_core::OsRng, SaltString},
    Argon2,
};

use crate::database;

/// Derive a key using Argon2. The keylength is set to 32 bytes.
/// It uses the salt stored in the config file under a2_salt.
pub fn argon2_derive_key(password: &str) -> Option<[u8; 32]> {
    let salt: SaltString;
    if database::read_kv("Secrets", "a2_salt")
        .unwrap_or(String::new())
        .is_empty()
    {
        salt = SaltString::generate(&mut OsRng);
        let w = database::write_kv(
            "Secrets",
            "a2_salt",
            database::InsertValue::from(salt.as_ref()),
        );
        match w {
            Ok(_) => {}
            Err(e) => {
                eprintln!("{e}");
                return None;
            }
        }
    } else {
        salt = SaltString::from_b64(
            database::read_kv("Secrets", "a2_salt")
                .unwrap_or(String::new())
                .as_ref(),
        )
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
