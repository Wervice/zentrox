use crate::crypto_utils::argon2_derive_key;
use crate::database;
use crate::database::InsertValue as SQLInsertValue;
use crate::sudo::{SudoExecutionOutput, SudoExecutionResult, SwitchedUserCommand};
use dirs::{self, home_dir};
use rand::distributions::DistString;
use rcgen::{generate_simple_self_signed, CertifiedKey};
use rpassword::prompt_password;
use sha2::{Digest, Sha512};
use std::io::{self, BufRead, Write};
use std::{fs, path};

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
    let hd = home_dir().unwrap();
    let mut vd: Option<path::PathBuf> = None;
    let options = ["Video", "video", "Videos", "videos", "Movie", "Movies"].into_iter(); // Guess vid dir
    options.for_each(|o| {
        if hd.join(o).exists() {
            vd = Some(hd.join(o));
        }
    });
    let _ = fs::write(
        &data_path.join("zentrox_media_locations.txt"),
        if vd.is_some() {
            format!(
                "{};Video Directory;true",
                vd.unwrap().to_string_lossy().to_string()
            )
            .to_string()
        } else {
            "".to_string()
        },
    );

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
    println!("Setting up zentrox backend database");
    let setup_database = database::setup_database();
    match setup_database {
        Ok(_) => {
            println!("Table structure configured")
        }
        Err(e) => {
            eprintln!("Setting up the database failed with error: {e}")
        }
    };

    fn random_string() -> String {
        rand::distributions::Alphanumeric.sample_string(&mut rand::thread_rng(), 8)
    }

    let set_a = database::write_kv("Settings", "server_name", SQLInsertValue::from(servername));
    let set_b = database::write_kv(
        "Settings",
        "media_enabled",
        SQLInsertValue::from(database::ST_BOOL_FALSE.to_string()),
    );
    let set_c = database::write_kv(
        "Settings",
        "vault_enabled",
        SQLInsertValue::Text(database::ST_BOOL_FALSE.to_string()),
    );
    let set_d = database::insert(
        "Ftp",
        &["key", "running", "pid", "username", "local_root"],
        &[
            SQLInsertValue::Int32(0),
            SQLInsertValue::from(false),
            SQLInsertValue::Null(),
            SQLInsertValue::from(random_string()),
            SQLInsertValue::from(
                dirs::home_dir()
                    .unwrap_or(std::path::PathBuf::from("/home"))
                    .to_string_lossy()
                    .to_string(),
            ),
        ],
    );

    let password_hash = argon2_derive_key(&password.unwrap());
    let password_hash_hex = hex::encode(password_hash.unwrap()).to_string();

    let set_e = database::write_kv(
        "Settings",
        "tls_cert",
        SQLInsertValue::Text("selfsigned.pem".to_string()),
    );

    let set_f = database::write_kv(
        "Secrets",
        "admin_password",
        SQLInsertValue::from(password_hash_hex),
    );
    let set_g = database::write_kv(
        "Secrets",
        "ftp_password",
        SQLInsertValue::from({
            let mut hasher = Sha512::new();
            sha2::Digest::update(&mut hasher, b"CHANGE_ME");
            let result = hasher.finalize();
            hex::encode(&result).to_string()
        }),
    );
    let set_h = database::insert(
        "Admin",
        &["key", "username", "use_otp", "knows_otp"],
        &[
            SQLInsertValue::Int32(0),
            SQLInsertValue::from(username),
            SQLInsertValue::from(enable_otp),
            SQLInsertValue::from(false),
        ],
    );

    if set_a.is_ok()
        && set_b.is_ok()
        && set_c.is_ok()
        && set_d.is_ok()
        && set_e.is_ok()
        && set_f.is_ok()
        && set_g.is_ok()
        && set_h.is_ok()
    {
        println!("Database settings written")
    } else {
        println!("Failed to write to database")
    }

    // let _ = config_file::write("media_enabled", "0"); // Media center is disabled by default
    // let _ = config_file::write("ftp_pid", ""); // No ftp PID is set
    // let _ = config_file::write("ftp_running", "0"); // FTP is not running
    // let _ = config_file::write(
    //    "ftp_username",
    //    (0..16)
    //        .map(|_| (rand::random::<u8>() * (90 - 65) + 65) as char)
    //        .collect::<String>()
    //        .as_str(),
    // );

    //let _ = config_file::write("knows_otp_secret", "0");
    //let _ = config_file::write("tls_cert", "selfsigned.pem");
    //let _ = config_file::write("vault_enabled", "0");
    //let _ = config_file::write("use_otp", {
    //    if enable_otp {
    //        "1"
    //    } else {
    //        "0"
    //    }
    //});

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
