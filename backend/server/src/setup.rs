use argon2::password_hash::SaltString;
use diesel::RunQueryDsl;
use dirs::{self, home_dir};
use rand::rngs::OsRng;
use rcgen::{CertifiedKey, generate_simple_self_signed};
use rpassword::prompt_password;
use std::fs;
use std::io::{self, BufRead, Write};
use std::time::UNIX_EPOCH;
use utils::crypto_utils::{argon2_derive_key, encrypt_bytes};
use utils::database::{self, establish_direct_connection};
use utils::otp::{derive_otp_url, generate_otp_secret};
use utils::sudo::SudoCommand;

fn flush() {
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
    print!("{msg}");
    flush();
    read().replace("\n", "")
}

fn hostname() -> Option<String> {
    let r = fs::read_to_string("/etc/hostname");

    r.ok()
}

pub fn run_setup() -> Result<(), String> {
    // NOTE Prettier TUI would be reasonable

    use utils::models::Account;
    use utils::models::Configurations;
    use utils::models::PackageAction;
    use utils::schema::Configuration::dsl::*;
    use utils::schema::PackageActions::dsl::*;
    use utils::schema::Users::dsl::*;

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

    let _system_username = whoami::username_os().to_string_lossy().to_string();
    println!("Installing Zentrox...");
    println!("Configuring admin account: ");
    let input_username = prompt(" | Username: ");
    let input_password = prompt_password(" | Password: ");
    let enable_otp: bool = {
        let p = prompt(" | Use with OTP [y/n]: ");
        p.to_lowercase() == "y"
    };
    let servername = prompt(" | Server Name: ");
    println!("Setting up zentrox backend database");
    let setup_database = database::base_database_setup();
    match setup_database {
        Ok(_) => {
            println!("Table structure configured")
        }
        Err(e) => {
            eprintln!("Setting up the database failed with error: {e}")
        }
    };

    let connection = &mut establish_direct_connection();

    if let Err(e) = diesel::insert_into(Configuration)
        .values(Configurations {
            media_enabled: false,
            vault_enabled: false,
            server_name: servername,
            tls_cert: "selfsigned.pem".to_string(),
            id: 0,
        })
        .execute(connection)
    {
        eprintln!("Datebase insertion failed with error: {e}");
    }

    if let Err(e) = diesel::insert_into(PackageActions)
        .values(PackageAction {
            key: 0_i32,
            last_database_update: None,
        })
        .execute(connection)
    {
        eprintln!("Datebase insertion failed with error: {e}");
    }

    let random_salt = SaltString::generate(&mut OsRng);
    let new_password_hash =
        argon2_derive_key(input_password.as_ref().unwrap(), random_salt).to_string();
    
    let mut encrypted_generated_otp_secret: Option<String> = None;
    if enable_otp {
        let generated_otp_secret = generate_otp_secret();
        encrypted_generated_otp_secret = Some(
            encrypt_bytes(generated_otp_secret.as_bytes(), &input_password.unwrap()).to_string(),
        );
        println!("Your OTP secret is: {generated_otp_secret}");
        println!(
            "Store it in a secure location, ideally a 2FA App and keep it to yourself. You can not view this secret again.\n"
        );
        let qr_x = qr2term::print_qr(derive_otp_url(generated_otp_secret, input_username.clone()));
        if let Err(e) = qr_x {
            eprintln!("Failed to show OTP QR code due to error: {e}");
        }
        println!("\n");
    }

    let current_ts = std::time::SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64;

    if let Err(e) = diesel::insert_into(Users)
        .values(Account {
            username: input_username,
            use_otp: enable_otp,
            otp_secret: { encrypted_generated_otp_secret },
            password_hash: new_password_hash.to_string(),
            created_at: current_ts,
            updated_at: current_ts,
            id: 0_i32,
        })
        .execute(connection)
    {
        eprintln!("Datebase insertion failed with error: {e}");
    }

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
        let sudo_password =
            rpassword::prompt_password("Please enter your sudo password to run UFW: ");
        let ufw_command =
            SudoCommand::new(sudo_password.unwrap().to_string(), "/sbin/ufw".to_string())
                .args(if ip_addr.is_empty() {
                    vec!["allow", "8080"]
                } else {
                    vec!["allow", "from", &ip_addr, "to", "8080"]
                })
                .output();

        match ufw_command {
            Ok(_) => {
                println!("New rule created");
            }
            _ => {
                eprintln!("Failed to create new rule")
            }
        }
    }

    println!("Installation finished successfully.");
    println!("Starting Zentrox now...");

    Ok(())
}
