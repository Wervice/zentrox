use std::fmt::Display;
use std::fs;
use std::path::Path;

use aes_gcm::aead::Aead;
use aes_gcm::{AeadCore, Aes256Gcm, Key, KeyInit};
use argon2::{
    Argon2,
    password_hash::{SaltString, rand_core::OsRng},
};
use subtle::ConstantTimeEq;

#[derive(Debug, PartialEq, Eq)]
pub struct SaltedHash {
    pub bytes: [u8; 32],
    pub salt: SaltString,
}

impl Display for SaltedHash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let _ = f.write_str(self.salt.as_ref());
        f.write_str(hex::encode(self.bytes).as_str())
    }
}

impl From<String> for SaltedHash {
    fn from(value: String) -> Self {
        let salt = &value[..22];
        let mut bytes = [0u8; 32];
        let _ = hex::decode_to_slice(&value[22..], &mut bytes);
        SaltedHash {
            bytes,
            salt: SaltString::from_b64(salt).unwrap(),
        }
    }
}

/// Derive a key using Argon2. The key length is set to 32 bytes.
pub fn argon2_derive_key(password: &str, salt: SaltString) -> SaltedHash {
    let params = argon2::Params::new(2097152, 1, 4, Some(32)).unwrap(); // As per RFC 9106 as "FIRST RECOMMENDED"
    let argon2 = Argon2::new(
        argon2::Algorithm::default(),
        argon2::Version::default(),
        params,
    );

    let mut output = [0u8; 32];
    let _ = argon2.hash_password_into(
        password.as_bytes(),
        salt.to_string().as_bytes(),
        &mut output,
    );

    SaltedHash {
        bytes: output,
        salt,
    }
}

/// Give an original hash in text form as well as a plain password, this function verifies if the
/// password matches the hashes' original password.
/// This function is necessary as the salt for a hash is randomly generated for [`argon2_derive_key`].
///
/// ```rust
/// use utils::crypto_utils::verify_with_hash;
///
/// assert!(verify_with_hash("/+A+9O4k1rhIW4r0FJh11gd8f802b90edbc5cb2740a84aeeb963b381e54c192915f205889c9fd5b572a053", "Hello World"))
/// ```
pub fn verify_with_hash(original: &str, new: &str) -> bool {
    let original_des = SaltedHash::from(original.to_string());
    let original_salt = original_des.salt.clone();
    let new_hashed = argon2_derive_key(new, original_salt);
    new_hashed.bytes.ct_eq(&original_des.bytes).into()
}

/// A struct, used to represent encrypted data.
/// It implements [`Display`], [`From<String>`] and [`Into<Vec<u8>>`]. Thus, it can be used to
/// store and retrieve encrypted data in form of bytes or Strings.
#[derive(Debug, PartialEq, Clone)]
pub struct Ciphertext {
    pub bytes: Vec<u8>,
    pub nonce: Vec<u8>,
    pub salt: SaltString,
}

impl Display for Ciphertext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Nonce is known to be 96 bits long which equals to 12 bytes
        let combination = [self.nonce.clone(), self.bytes.clone()].concat();
        let _ = f.write_str(self.salt.to_string().as_str()); // Known length: 22 single-byte characters
        f.write_str(hex::encode(combination).as_str())
    }
}

impl From<String> for Ciphertext {
    fn from(value: String) -> Self {
        let split = value.split_at(22);
        let raw_bytes = hex::decode(split.1).unwrap();

        let nonce = &raw_bytes[0..12];
        let bytes = &raw_bytes[12..];
        Ciphertext {
            bytes: bytes.to_vec(),
            nonce: nonce.to_vec(),
            salt: SaltString::from_b64(split.0).expect("Invalid salt"),
        }
    }
}

impl Into<Vec<u8>> for Ciphertext {
    fn into(self) -> Vec<u8> {
        [
            self.salt.to_string().as_bytes().to_vec(),
            self.nonce,
            self.bytes,
        ]
        .concat()
    }
}

impl From<Vec<u8>> for Ciphertext {
    fn from(value: Vec<u8>) -> Self {
        let split = value.split_at(22);
        let raw_bytes = split.1;

        let nonce = &raw_bytes[0..12];
        let bytes = &raw_bytes[12..];
        Ciphertext {
            bytes: bytes.to_vec(),
            nonce: nonce.to_vec(),
            salt: SaltString::from_b64(String::from_utf8(split.0.to_vec()).unwrap().as_str())
                .expect("Invalid salt"),
        }
    }
}

/// Given a slice of bytes and a password string, this function will encrypt the bytes using
/// Aes256Gcm for encryption and argon2 for key derivation.
/// The function returns a [`Ciphertext`] struct that includes the 22 byte salt used for hashing, the encrypted
/// bytes and the 12 byte nonce.
/// [`Ciphertext`] implements [`Display`]
///
/// # Example
/// The following will encrypt the string "Hello World" with the password "password" and serialize
/// the output into a string.
/// ```rust
/// use utils::crypto_utils::encrypt_bytes;
///
/// dbg!(encrypt_bytes(b"Hello World", "password").to_string())
/// ```
pub fn encrypt_bytes(plaintext: &[u8], password: &str) -> Ciphertext {
    let salt = SaltString::generate(&mut OsRng);
    let derived_key = argon2_derive_key(password, salt);
    let key = Key::<Aes256Gcm>::from_slice(&derived_key.bytes);

    let cipher = Aes256Gcm::new(key);
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
    let bytes = cipher
        .encrypt(&nonce, plaintext)
        .expect("Encryption failed");

    Ciphertext {
        bytes,
        nonce: nonce.to_vec(),
        salt: derived_key.salt,
    }
}

/// Given a [`Ciphertext`] struct and a password this function decrypts the ciphertext and returns
/// the decrypted bytes.
/// The function may fail, for example due to a wrong password and will return an
/// [`aes_gcm::Error`].
pub fn decrypt_bytes(ciphertext: Ciphertext, password: &str) -> Result<Vec<u8>, aes_gcm::Error> {
    let derived_key = argon2_derive_key(password, ciphertext.salt);
    let key = Key::<Aes256Gcm>::from_slice(&derived_key.bytes);

    let cipher = Aes256Gcm::new(key);
    cipher.decrypt(
        ciphertext.nonce.as_slice().into(),
        ciphertext.bytes.as_slice(),
    )
}

/// Given the path to a target file and a password, this function will encrypt the target file
/// using the same mechanism as [`encrypt_bytes`] with serialized bytes.
pub fn encrypt_file<P: AsRef<Path>>(path: P, password: &str) -> Result<(), std::io::Error> {
    let contents = fs::read(&path)?;
    let encrypted_bytes: Vec<u8> = encrypt_bytes(&contents, password).into();
    fs::write(path, encrypted_bytes)
}

pub enum FileDecryptionError {
    IoError(std::io::Error),
    CryptographyError(aes_gcm::Error),
}

/// Given the path to a target file and a password, this function will decrypt the bytes in a
/// target file using the same mechanism as in [`decrypt_bytes`].
/// The function may fail due to IO errors or encryption errors, differentiated using a
/// [`FileDecryptionError`].
pub fn decrypt_file<P: AsRef<Path>>(path: P, password: &str) -> Result<(), FileDecryptionError> {
    let read = fs::read(&path);
    if let Ok(contents) = read {
        match decrypt_bytes(Ciphertext::from(contents), password) {
            Ok(bytes) => {
                if let Err(e) = fs::write(path, bytes) {
                    return Err(FileDecryptionError::IoError(e));
                }
            }
            Err(e) => return Err(FileDecryptionError::CryptographyError(e)),
        }
    } else if let Err(e) = read {
        return Err(FileDecryptionError::IoError(e));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_hello_world() {
        let salt = SaltString::generate(&mut OsRng);
        let hash = argon2_derive_key("Hello World", salt.clone());
        assert_eq!(hash.salt, salt);
        assert_eq!(hash.bytes.len(), 32);
    }

    #[test]
    fn serialized_hash_hello_world() {
        let salt = SaltString::generate(&mut OsRng);
        let original = argon2_derive_key("Hello World", salt.clone());
        let ser = original.to_string();
        let des = SaltedHash::from(ser);
        assert_eq!(original, des);
    }

    #[test]
    fn encrypt_decrypt_hello_world() {
        let e = encrypt_bytes(b"Hello World!", "password");
        let d = decrypt_bytes(e, "password");
        assert_eq!(d, Ok(b"Hello World!".to_vec()));
    }

    #[test]
    fn check_serialization() {
        let original = encrypt_bytes(b"Hello World!", "password");
        let serialized = original.to_string();
        let deserialized = Ciphertext::from(serialized);
        assert_eq!(original, deserialized);
    }

    #[test]
    fn check_bytes_serialization() {
        let original = encrypt_bytes(b"Hello World!", "password");
        let serialized: Vec<u8> = original.clone().into();
        let deserialized = Ciphertext::from(serialized);
        assert_eq!(original, deserialized);
    }

    #[test]
    fn check_hash_verification() {
        assert!(verify_with_hash(
            argon2_derive_key("password", SaltString::generate(&mut OsRng))
                .to_string()
                .as_str(),
            "password"
        ));
        assert!(!verify_with_hash(
            argon2_derive_key("password", SaltString::generate(&mut OsRng))
                .to_string()
                .as_str(),
            "wrong password"
        ));
    }

    #[test]
    fn verify_randomness() {
        let e_a = encrypt_bytes(b"Hello World!", "password");
        let e_b = encrypt_bytes(b"Hello World!", "password");
        let e_c = encrypt_bytes(b"Hello World!", "password");
        assert_ne!(e_a, e_b);
        assert_ne!(e_a, e_c);
        assert_ne!(e_b, e_c);
    }

    #[test]
    fn verify_wrong_password() {
        let e = encrypt_bytes(b"Hello World", "Correct password");
        let dec = decrypt_bytes(e, "Wrong password");
        assert!(dec.is_err())
    }

    #[test]
    fn encrypt_decrypt_file() {
        let original = b"Hello World\nNew lines are great";
        let password = "password";
        let p = "encryption_test.txt";
        if fs::exists(p).unwrap() {
            panic!("encryption_test.txt file still exists.");
        }
        assert!(fs::write(p, original).is_ok());
        assert!(encrypt_file(p, password).is_ok());
        assert_ne!(fs::read(p).unwrap(), original);
        assert!(decrypt_file(p, password).is_ok());
        assert_eq!(fs::read(p).unwrap(), original);
        fs::remove_file(p).expect("Failed to remove test file for encryption.");
    }
}
