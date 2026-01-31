use log::debug;
use rand::Rng;
use totp_rs::{Algorithm, Secret, TOTP};

/// Generates a random OTP secret.
///
/// The secret consists of 16 random bytes that are encoded with unpadded base32 and the Rfc4648
/// alphabet.
pub fn generate_otp_secret() -> String {
    let mut random_number_generator = rand::thread_rng();
    let secret_bytes: Vec<u8> = (0..20).map(|_| random_number_generator.r#gen()).collect();

    debug!("OTP secret with the bytes {secret_bytes:?} has been generated.");

    base32::encode(base32::Alphabet::Rfc4648 { padding: false }, &secret_bytes)
}

/// Given the username and the plaintext secret from the users entry in the database,
/// this function will derive a URL that can be used to add the entry to 2FA management apps.
pub fn derive_otp_url(otp_secret: String, username: String) -> String {
    let totp = TOTP::new(
        Algorithm::SHA1,
        8,
        1,
        30,
        Secret::Encoded(otp_secret.to_string()).to_bytes().unwrap(),
        Some("Zentrox".to_string()),
        username,
    )
    .unwrap();
    totp.get_url()
}

/// Calculates the current 6 digit OTP token from a given secret.
///
/// * `otp_secret` The OTP secret that is used for calculation.
///
/// The function uses SHA-1 and a 30 second interval.
pub fn verify_current_otp<T: AsRef<str>>(
    secret: String,
    token: T,
) -> Result<bool, Box<dyn std::error::Error>> {
    let validator = TOTP::new(
        Algorithm::SHA1,
        8,
        1,
        30,
        Secret::Encoded(secret.to_string()).to_bytes().unwrap(),
        Some("Zentrox".to_string()),
        String::default(),
    )
    .unwrap();
    Ok(validator.check_current(token.as_ref())?)
}
