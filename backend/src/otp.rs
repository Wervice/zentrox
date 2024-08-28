use rand::Rng;
use totp_rs::{Algorithm, Secret, TOTP};

/// Generates a random otp secret.
/// The secret is generated from 16 random u8 numbers that are than base32 encoded without padding
/// using Rfc4648
pub fn generate_otp_secret() -> String {
    let mut random_number_generator = rand::thread_rng();
    let secret_bytes: Vec<u8> = (0..16).map(|_| random_number_generator.gen()).collect();

    base32::encode(base32::Alphabet::Rfc4648 { padding: false }, &secret_bytes)
}

/// Calculates the current 6 digit OTP token from a given secret.
/// * `otp_secret` The OTP secret that is used for calculation.
///
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
    validator.generate_current().unwrap()
}
