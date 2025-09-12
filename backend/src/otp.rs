use log::debug;
use rand::Rng;
use totp_rs::{Algorithm, Secret, TOTP};

/// Generates a random OTP secret.
///
/// The secret consists of 16 random bytes that are encoded with unpadded base32 and the Rfc4648
/// alphabet.
pub fn generate_otp_secret() -> String {
    let mut random_number_generator = rand::thread_rng();
    let secret_bytes: Vec<u8> = (0..16).map(|_| random_number_generator.r#gen()).collect();

    debug!("OTP secret with the bytes {secret_bytes:?} has been generated.");

    base32::encode(base32::Alphabet::Rfc4648 { padding: false }, &secret_bytes)
}

/// Calculates the current 6 digit OTP token from a given secret.
///
/// * `otp_secret` The OTP secret that is used for calculation.
/// The function uses Sha1 and a 30 second interval.
pub fn calculate_current_otp(otp_secret: &String) -> String {
    let validator = TOTP::new(
        Algorithm::SHA1,
        6,
        1,
        30,
        Secret::Encoded(otp_secret.to_string()).to_bytes().unwrap(),
    )
    .unwrap();

    debug!("The OTP secret is {otp_secret}.");
    debug!(
        "The current calculated OTP code for {validator:?} is {}.",
        validator.generate_current().unwrap()
    );

    validator.generate_current().unwrap()
}
