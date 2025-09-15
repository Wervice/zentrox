use utils::crypto_utils::argon2_derive_key;
use utils::database::{self, establish_connection};
use utils::otp::generate_otp_secret;
use utils::sudo::{SudoExecutionResult, SwitchedUserCommand};
use diesel::RunQueryDsl;
use dirs::{self, home_dir};
use rcgen::{CertifiedKey, generate_simple_self_signed};
use rpassword::prompt_password;
use std::fs;
use std::io::{self, BufRead, Write};
use std::time::UNIX_EPOCH;

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

    use utils::models::AdminAccount;
    use utils::models::Configurations;
    use utils::models::PackageAction;
    use utils::schema::Admin::dsl::*;
    use utils::schema::Configuration::dsl::*;
    use utils::schema::PackageActions::dsl::*;

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

    let connection = &mut establish_connection();

    // NOTE Handle these errors

    diesel::insert_into(Configuration)
        .values(Configurations {
            media_enabled: false,
            vault_enabled: false,
            server_name: servername,
            tls_cert: "selfsigned.pem".to_string(),
            id: 0,
        })
        .execute(connection);

    diesel::insert_into(PackageActions)
        .values(PackageAction {
            key: 0_i32,
            last_database_update: None,
        })
        .execute(connection);

    let new_password_hash = argon2_derive_key(&input_password.unwrap());
    let new_password_hash_hex = hex::encode(new_password_hash.unwrap()).to_string();

    let generated_otp_secret = generate_otp_secret();

    let current_ts = std::time::SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64;

    diesel::insert_into(Admin)
        .values(AdminAccount {
            username: input_username,
            use_otp: enable_otp,
            otp_secret: {
                if enable_otp {
                    Some(generated_otp_secret.clone())
                } else {
                    None
                }
            },
            password_hash: new_password_hash_hex.to_string(),
            created_at: current_ts,
            updated_at: current_ts,
            id: 0_i32,
        })
        .execute(connection);

    println!(
        "Your OTP secret is: {generated_otp_secret}\nStore it in a secure location, ideally a 2FA App and keep it to yourself. You can not view this secret again."
    );

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
            SwitchedUserCommand::new(sudo_password.unwrap().to_string(), "/sbin/ufw".to_string())
                .args(if ip_addr.is_empty() {
                    vec!["allow", "8080"]
                } else {
                    vec!["allow", "from", &ip_addr, "to", "8080"]
                })
                .spawn();

        match ufw_command {
            SudoExecutionResult::Success(_sc) => {
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
