use ring::pbkdf2;
use std::num::NonZeroU32;
use argon2::{
    password_hash::{
        rand_core::OsRng, SaltString
    },
    Argon2
};


use crate::config_file;


/// Performs hashing on a password with a given salt.
/// It uses HMAC_SHA_512 with PBKDF2, 210_000 iterations and 64 bytes keylength.
pub fn hmac_sha_512_pbkdf2_hash(password: &str, salt: &str) -> Option<Vec<u8>> {
    let mut password_hash = vec![0_u8; 64];

    if password.is_empty() {
        return None
    }

    pbkdf2::derive(
        pbkdf2::PBKDF2_HMAC_SHA512,
        NonZeroU32::new(210_000).unwrap(),
        salt.as_bytes(),
        password.as_bytes(),
        &mut password_hash,
    );

    Some(password_hash)
}

/// Performs hashing on a password with a given salt.
/// It uses HMAC_SHA_256 with PBKDF2, 600_000 iterations and 32 bytes keylength.
pub fn hmac_sha_256_pbkdf2_hash(password: &str, salt: &str) -> Option<Vec<u8>> {
    let mut password_hash = vec![0_u8; 32];

     if password.is_empty() {
        return None
    }

    pbkdf2::derive(
        pbkdf2::PBKDF2_HMAC_SHA256,
        NonZeroU32::new(600_000).unwrap(),
        salt.as_bytes(),
        password.as_bytes(),
        &mut password_hash,
    );

    Some(password_hash)
}

/// Performs hashing on a password with a given salt.
/// It uses HMAC_SHA_1 with PBKDF2, 1_300_000 iterations and 20 bytes keylength.
pub fn hmac_sha_1_pbkdf2_hash(password: &str, salt: &str) -> Option<Vec<u8>> {
    let mut password_hash = vec![0_u8; 20];

    if password.is_empty() {
        return None
    }

    pbkdf2::derive(
        pbkdf2::PBKDF2_HMAC_SHA1,
        NonZeroU32::new(1_300_000).unwrap(),
        salt.as_bytes(),
        password.as_bytes(),
        &mut password_hash,
    );

    Some(password_hash)
}

/// Derive a key using Argon2. The keylength is set to 32 bytes.
/// It uses the salt stored in the config file under vault_key_salt.
pub fn argon2_derive_key(password: &str) -> Option<[u8; 32]> {
    
    let salt: SaltString;
    if config_file::read("vault_key_salt").is_empty() {
        salt = SaltString::generate(&mut OsRng);
        let _ = config_file::write("vault_key_salt", salt.as_ref());
    } else {
        salt = SaltString::from_b64(&config_file::read("vault_key_salt")).unwrap();
    }


    let params = argon2::Params::new(4096, 3, 1, Some(32)).unwrap();
    let argon2 = Argon2::new(argon2::Algorithm::Argon2id, argon2::Version::V0x13, params);

    let mut output = [0u8; 32];
    let _ = argon2.hash_password_into(password.as_bytes(), salt.to_string().as_bytes(), &mut output);

    Some(output)
}
