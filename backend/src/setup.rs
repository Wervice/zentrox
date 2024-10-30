use crate::config_file;
use crate::crypto_utils::argon2_derive_key;
use crate::sudo::SwitchedUserCommand;
use dirs::{self, home_dir};
use rcgen::{generate_simple_self_signed, CertifiedKey};
use rpassword::prompt_password;
use sha2::{Digest, Sha512};
use std::fs;
use std::io::{self, BufRead, Write};

fn f() {
    let _ = io::stdout().flush();
}

fn read() -> String {
    let mut buffer = String::new();
    let stdin = io::stdin();
    let mut handle = stdin.lock();

    handle.read_line(&mut buffer).unwrap().to_string();
    buffer
}

fn prompt(msg: &str) -> String {
    print!("{}", msg);
    f();
    read().replace("\n", "")
}

fn hostname() -> Option<String> {
    let r = fs::read_to_string("/etc/hostname");

    match r {
        Ok(v) => return Some(v),
        Err(_e) => return None,
    }
}

pub fn run_setup() -> Result<(), String> {
    let _installation_path = home_dir()
        .unwrap()
        .join(".local")
        .join("bin")
        .join("zentrox");
    let data_path = home_dir()
        .unwrap()
        .join(".local")
        .join("share")
        .join("zentrox");

    let _ = fs::create_dir_all(&data_path);
    let _ = fs::write(&data_path.join("zentrox_store.toml"), "");

    let _system_username = whoami::username_os().to_string_lossy().to_string();
    println!("Installing Zentrox...");
    println!("");
    println!("Configuring admin account: ");
    let username = prompt(" | Username: ");
    let password = prompt_password(" | Password: ");
    let enable_otp: bool = {
        let p = prompt(" | Use with OTP [y/n]: ");
        p.to_lowercase() == "y"
    };
    let servername = prompt(" | Server Name: ");
    println!("Setting configuration files");
    let _ = config_file::write("server_name", &servername);
    let _ = config_file::write("ftp_pid", "");
    let _ = config_file::write("ftp_running", "0");
    let _ = config_file::write("ftp_username", "lorem");
    let _ = config_file::write("ftp_password", {
        let mut hasher = Sha512::new();
        sha2::Digest::update(&mut hasher, b"ipsum");
        let result = hasher.finalize();
        &hex::encode(&result).to_string()
    });
    let _ = config_file::write("ftp_running", "0");
    let _ = config_file::write(
        "ftp_local_root",
        &dirs::home_dir().unwrap().to_string_lossy().to_string(),
    );
    let _ = config_file::write("knows_otp_secret", "0");
    let _ = config_file::write("tls_cert", "selfsigned.pem");
    let _ = config_file::write("vault_enabled", "0");
    let _ = config_file::write("use_otp", {
        if enable_otp {
            "1"
        } else {
            "0"
        }
    });
    let admin_account_string = {
        let password_hash = argon2_derive_key(&password.unwrap());
        let password_hash_hex = hex::encode(password_hash.unwrap()).to_string();
        let username_b64 = base64::encode(username.as_bytes());
        format!("{}: {}: admin\n", username_b64, password_hash_hex)
    };
    let _ = fs::write(data_path.join("users"), admin_account_string);

    let subject_alt_names = vec![
        "localhost".to_string(),
        hostname().unwrap_or("localhost".to_string()),
    ];

    println!("Generating SSL/TLS certificate");

    let CertifiedKey { cert, key_pair } = generate_simple_self_signed(subject_alt_names).unwrap();

    let _ = fs::create_dir_all(data_path.join("certificates"));
    let _ = fs::write(
        data_path.join("certificates").join("selfsigned.pem"),
        format!("{}{}", key_pair.serialize_pem(), cert.pem()),
    );

    println!("System settings");
    let allow_8080 =
        { prompt("Add UFW rule to allow port 8080 for Zentrox [y/n]: ").to_lowercase() == "y" };
    if allow_8080 {
        let ip_addr = prompt("Only allow port 8080 for specific IP [enter ip/leave empty]: ");
        let ip_arg;
        if ip_addr.len() < 5 {
            ip_arg = "any"
        } else {
            ip_arg = &ip_addr
        }
        let sudo_password =
            rpassword::prompt_password("Please enter your sudo password to run UFW: ");
        let ufw_command =
            SwitchedUserCommand::new(sudo_password.unwrap().to_string(), "/sbin/ufw".to_string())
                .args(vec!["allow", "from", ip_arg, "to", "8080"])
                .spawn();

        match ufw_command {
            Ok(_sc) => {
                println!("New rule created");
            }
            Err(_) => {
                eprintln!("Failed to create new rule")
            }
        }
    }

    println!("Installation finished successfully.");
    println!("Starting Zentrox now...");

    Ok(())
}
