use crate::crypto_utils;
use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    Aes256Gcm,
};
use sha2::{Digest, Sha256};
use std::fs;

/// Decrypts a file with a specefied key and writes the cleartext back to the file.
/// * `file` - Path to the file to decrypt
/// * `key` - Key to decrypt file
pub fn decrypt_file(file: String, key: &str) -> Option<()> {
    let key_hashed = crypto_utils::argon2_derive_key(key);

    let value = fs::read(&file).unwrap();

    // Extract the salt and nonce from the file data
    let nonce = &value[0..12]; // Assuming 12-byte nonce

    let cipher = Aes256Gcm::new(&key_hashed.unwrap().into());
    let ciphertext = &value[12..];

    match cipher.decrypt(nonce.into(), ciphertext) {
        Ok(p) => {
            fs::write(&file, p).unwrap();
            Some(())
        }
        Err(e) => {
            eprintln!("❌ Failed to decrypt file\n{e}");
            None
        }
    }
}

#[allow(dead_code)]
/// Decrypts a string with a given key and decodes the input from hexadecimal.
/// * `string` - Path to the file to decrypt
/// * `key` - Key to decrypt file
pub fn decrypt_string(string: String, key: &str) -> Option<String> {
    let key_hashed = crypto_utils::argon2_derive_key(key);

    let value = &hex::decode(&string).unwrap();

    // Extract the salt and nonce from the file data
    let nonce = &value[0..12]; // Assuming 12-byte nonce

    let cipher = Aes256Gcm::new(&key_hashed.unwrap().into());
    let ciphertext = &value[12..];

    match cipher.decrypt(nonce.into(), ciphertext) {
        Ok(v) => Some(String::from_utf8_lossy(&v).to_string()),
        Err(_) => None,
    }
}

#[allow(dead_code)]
/// Encrypt a string with a given key and encodes the output to hexadecimal.
/// * `string` - Path to the file to decrypt
/// * `key` - Key to decrypt file
pub fn encrypt_string(string: String, key: &str) -> Option<String> {
    let key_hashed = crypto_utils::argon2_derive_key(key);

    // Extract the salt and nonce from the file data
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);

    let cipher = Aes256Gcm::new(&key_hashed.unwrap().into());

    match cipher.encrypt(&nonce, string.as_bytes()) {
        Ok(v) => Some(format!("{}{}", hex::encode(nonce), hex::encode(&v))),
        Err(_) => None,
    }
}

fn generate_nonce(input: &[u8]) -> [u8; 12] {
    // Create a SHA-256 hash of the input
    let mut hasher = Sha256::new();
    hasher.update(input);
    let result = hasher.finalize();

    // Take the first 12 bytes for the nonce
    let mut nonce = [0u8; 12];
    nonce.copy_from_slice(&result[..12]);
    nonce
}

/// Decrypts a string with a given key and decodes the input from hexadecimal.
/// The IV for this encryption is derived using a hash of the filename.
/// * `string` - Path to the file to decrypt
/// * `key` - Key to decrypt file
pub fn decrypt_string_hash(string: String, key: &str) -> Option<String> {
    let key_hashed = crypto_utils::argon2_derive_key(key);

    let value = &hex::decode(&string).unwrap();

    // Extract the salt and nonce from the file data
    let nonce = &value[0..12]; // Assuming 12-byte nonce

    let cipher = Aes256Gcm::new(&key_hashed.unwrap().into());
    let ciphertext = &value[12..];

    match cipher.decrypt(nonce.into(), ciphertext) {
        Ok(v) => Some(String::from_utf8_lossy(&v).to_string()),
        Err(_) => None,
    }
}

/// Encrypt a string with a given key and encodes the output to hexadecimal.
/// The IV for this encryption is derived using a hash of the filename.
/// * `string` - Path to the file to decrypt
/// * `key` - Key to decrypt file
pub fn encrypt_string_hash(string: String, key: &str) -> Option<String> {
    let key_hashed = crypto_utils::argon2_derive_key(key);

    // Extract the salt and nonce from the file data
    let nonce = generate_nonce(string.as_bytes());

    let cipher = Aes256Gcm::new(&key_hashed.unwrap().into());

    match cipher.encrypt(&nonce.into(), string.as_bytes()) {
        Ok(v) => Some(format!("{}{}", hex::encode(nonce), hex::encode(&v))),
        Err(_) => None,
    }
}

/// Encrypts a file with a specefied key and writes the ciphertext back to the file.
/// * `file` - Path to the file to decrypt
/// * `key` - Key to encrypt file
pub fn encrypt_file(file: String, key: &str) {
    let key_hashed = crypto_utils::argon2_derive_key(key);

    let value = fs::read(&file).unwrap();

    let cipher = Aes256Gcm::new(&key_hashed.unwrap().into());
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);

    let ciphertext = cipher.encrypt(&nonce, value.as_ref()).unwrap();

    // Store the salt and nonce with the ciphertext

    let mut enc_data = Vec::new();
    enc_data.extend_from_slice(&nonce);
    enc_data.extend_from_slice(&ciphertext);

    fs::write(&file, enc_data).unwrap();
}

pub fn encrypt_directory(directory: &str, key: &str) -> Result<(), String> {
    let dir_contents = fs::read_dir(directory).unwrap();

    for entry in dir_contents {
        let e_u = entry.unwrap();
        if e_u.metadata().unwrap().is_file() {
            encrypt_file(e_u.path().to_string_lossy().to_string(), key);
            let filename = e_u.file_name().to_string_lossy().to_string();
            if filename != ".vault" {
                let mut encrypted_path = e_u.path();
                encrypted_path.pop();
                let _ = fs::rename(
                    e_u.path(),
                    encrypted_path.join(encrypt_string_hash(filename, key).unwrap()),
                );
            }
        } else {
            let _ = encrypt_directory(e_u.path().to_string_lossy().as_ref(), key);
            let filename = e_u.file_name().to_string_lossy().to_string();
            let mut encrypted_path = e_u.path();
            encrypted_path.pop();
            let _ = fs::rename(
                e_u.path(),
                encrypted_path.join(encrypt_string_hash(filename, key).unwrap()),
            );
        }
    }

    Ok(())
}

pub fn decrypt_directory(directory: &str, key: &str) -> Result<(), String> {
    let dir_contents = fs::read_dir(directory).unwrap();

    for entry in dir_contents {
        let e_u = entry.unwrap();
        if e_u.metadata().unwrap().is_file() {
            decrypt_file(e_u.path().to_string_lossy().to_string(), key);

            let filename = e_u.file_name().to_string_lossy().to_string();

            if filename != ".vault" {
                let mut encrypted_path = e_u.path();
                encrypted_path.pop();
                let _ = fs::rename(
                    e_u.path(),
                    encrypted_path.join(decrypt_string_hash(filename, key).unwrap()),
                );
            }
        } else {
            let _ = decrypt_directory(e_u.path().to_string_lossy().as_ref(), key);
            let filename = e_u.file_name().to_string_lossy().to_string();
            let mut encrypted_path = e_u.path();
            encrypted_path.pop();
            let _ = fs::rename(
                e_u.path(),
                encrypted_path.join(decrypt_string_hash(filename, key).unwrap()),
            );
        }
    }

    Ok(())
}

pub fn burn_file(path: String) -> Result<(), String> {
    let mut i = 0;
    let file_size = fs::metadata(&path).unwrap().len();

    while i != 5 {
        let random_data = (0..file_size)
            .map(|_| rand::random::<u8>())
            .collect::<Vec<u8>>();
        match fs::write(&path, random_data) {
            Ok(_) => {}
            Err(e) => {
                eprintln!("❌ Failed to burn file\n{e}");
                return Err("Failed to burn file".to_string());
            }
        };

        i += 1;
    }

    Ok(())
}

/// Recursively overwrites the contents of a directory with random data.
pub fn burn_directory(path: String) -> Result<(), String> {
    let dir_contents = fs::read_dir(&path);

    match dir_contents {
        Ok(contents) => {
            for entry in contents {
                let entry_unwrap = entry.unwrap();
                let entry_metadata = entry_unwrap.metadata().unwrap();
                let is_file = entry_metadata.is_file() || entry_metadata.is_symlink();

                if is_file {
                    burn_file(entry_unwrap.path().to_string_lossy().to_string())?
                } else {
                    match burn_directory(entry_unwrap.path().to_string_lossy().to_string()) {
                        Ok(_) => return Ok(()),
                        Err(e) => return Err(e),
                    }
                }
            }
            Ok(())
        }
        Err(e) => {
            eprintln!("❌ Failed to read directory\n{e}");
            Err(e.to_string())
        }
    }
}
