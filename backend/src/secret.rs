use aes_gcm;
use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    Aes256Gcm, Key // Or `Aes128Gcm`
};
use hex;
use std::path;
use std::fs;

// The encryption key can be generated randomly:

// Crate to manage secrets like:
// Session Key
// Login Token
// Zentrox Admin Password
pub fn write(toml_key: &str, toml_value: String, crypto_key: String) -> bool {
    let zentrox_installation_path = path::Path::new("")
        .join(dirs::home_dir().unwrap())
        .join("zentrox_data");
    let secret_file = zentrox_installation_path.join("secret.toml");

    let key = crypto_key.as_bytes();
    let key: &Key<Aes256Gcm> = key.into();

    let cipher = Aes256Gcm::new(&key);
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng); // 96-bits; unique per message
    let enc_toml_key = cipher.encrypt(&nonce, toml_key.as_bytes().as_ref()).unwrap();
    let enc_toml_value = cipher.encrypt(&nonce, toml_value.as_bytes().as_ref()).unwrap();
    let hex_enc_toml_key = hex::encode(enc_toml_key).to_string();
    let hex_enc_toml_value = hex::encode(enc_toml_value).to_string();

    let mut secret_file_parsed = fs::read_to_string(&secret_file)
        .expect("Failed to read config file")
        .to_string()
        .parse::<toml_edit::DocumentMut>()
        .expect("Failed to parse config file");
    secret_file_parsed[&hex_enc_toml_key] = toml_edit::value(hex_enc_toml_value);
    match fs::write(secret_file, secret_file_parsed.to_string()) {
        Ok(_) => true,
        Err(_) => false
    }
}


pub fn read(toml_key: &str, crypto_key: String) -> String {
    let zentrox_installation_path = path::Path::new("")
        .join(dirs::home_dir().unwrap())
        .join("zentrox_data");
    let secret_file = zentrox_installation_path.join("secret.toml");

    let key = crypto_key.as_bytes();
    let key: &Key<Aes256Gcm> = key.into();

    let cipher = Aes256Gcm::new(&key);
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng); // 96-bits; unique per message
    let enc_toml_key = cipher.encrypt(&nonce, toml_key.as_bytes().as_ref()).unwrap();
    let hex_enc_toml_key = hex::encode(enc_toml_key).to_string();

    match fs::read_to_string(secret_file)
        .expect("Failed to read config file")
        .parse::<toml::Table>()
        .unwrap()
        .get(&hex_enc_toml_key)
    {
        Some(value) => return String::from_utf8_lossy(&cipher.decrypt(&nonce, value.to_string().as_bytes()).unwrap()).to_string(),
        None => return "".to_string(),
    };
}
