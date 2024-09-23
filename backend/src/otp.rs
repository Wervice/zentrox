use hex;
use rand::Rng;
use totp_rs::{Algorithm, Secret, TOTP};

/// Generates a random otp secret.
///
/// The secret is generated from 16 random u8 numbers that are than base32 encoded without padding
/// using Rfc4648
pub fn generate_otp_secret() -> String {
    let mut random_number_generator = rand::thread_rng();
    let secret_bytes: Vec<u8> = (0..16).map(|_| random_number_generator.gen()).collect();

    hex::encode(&secret_bytes)
        .to_string()
        .chars()
        .map(|c| match c {
            '0' => 'g',
            '1' => 'h',
            '2' => 'i',
            '3' => 'j',
            '4' => 'k',
            '5' => 'l',
            '6' => 'm',
            '7' => 'n',
            '8' => 'o',
            '9' => 'p',
            _ => c,
        })
        .collect()
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
    validator.generate_current().unwrap()
}
