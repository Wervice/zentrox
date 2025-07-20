use rand::Rng;
use totp_rs::{Algorithm, Secret, TOTP};

const BASE32_CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";

fn base32_encode(bytes: &[u8]) -> String {
    let mut result = String::new();
    let mut buffer = 0;
    let mut bits_left = 0;

    for &byte in bytes {
        buffer = (buffer << 8) | (byte as u32);
        bits_left += 8;

        while bits_left >= 5 {
            bits_left -= 5;
            let index = ((buffer >> bits_left) & 0x1F) as usize;
            result.push(BASE32_CHARS[index] as char);
        }
    }

    if bits_left > 0 {
        buffer <<= 5 - bits_left;
        let index = (buffer & 0x1F) as usize;
        result.push(BASE32_CHARS[index] as char);
    }

    result
}

/// Generates a random otp secret.
///
/// The secret is generated from 16 random u8 numbers that are then base32 encoded without padding
/// using Rfc4648
pub fn generate_otp_secret() -> String {
    let mut random_number_generator = rand::thread_rng();
    let secret_bytes: Vec<u8> = (0..16).map(|_| random_number_generator.gen()).collect();

    base32_encode(&secret_bytes)
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
