use base32;
use rand::Rng;
use totp_rs::{Algorithm, Secret, TOTP};

pub fn generate_otp_secret() -> String {
    let mut random_number_generator = rand::thread_rng();
    let secret_bytes: Vec<u8> = (0..16).map(|_| random_number_generator.gen()).collect();

    base32::encode(base32::Alphabet::Rfc4648 { padding: false }, &secret_bytes)
}

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
