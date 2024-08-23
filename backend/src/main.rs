extern crate systemstat;
use actix_files as afs;
use actix_session::{storage::CookieSessionStore, Session, SessionMiddleware};
use actix_web::cookie::Key;
use actix_web::{get, http::StatusCode, middleware, post, web, App, HttpResponse, HttpServer};
use base64::{engine::general_purpose::STANDARD as b64, Engine as _};
use hex;
use hmac_sha512::Hash;
use serde::{Deserialize, Serialize};
mod is_admin;
use is_admin::is_admin;
mod config_file;
mod otp;
mod sudo;
use actix_cors::Cors;
use sysinfo::System as SysInfoSystem;
use systemstat::{Platform, System};
use whoami;

use std::{
    collections::HashMap,
    fs, path,
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

#[allow(non_snake_case)]

// General App Code

struct AppState {
    login_requests: std::sync::Mutex<
        HashMap<
            String, /* IP Adress of caller */
            (
                u128, /* Unix Timestamp of last request */
                u64,  /* Number of requests since last reset */
            ),
        >,
    >,
}

// Page Routes (Routes that lead to the display of a static page)
#[get("/")]
async fn index(session: Session) -> HttpResponse {
    // is_admin session value is != true (None or false), the user is served the login screen
    // otherwise, the user is redirected to /
    if is_admin(&session) {
        HttpResponse::Found()
            .append_header(("Location", "/dashboard"))
            .finish()
    } else {
        return HttpResponse::build(StatusCode::OK)
            .body(std::fs::read_to_string("static/index.html").expect("Failed to read file"));
    }
}

#[get("/dashboard")]
async fn dashboard(session: Session) -> HttpResponse {
    // is_admin session value is != true (None or false), the user is redirected to /
    // otherwise, the user is served the dashboard.html file
    if is_admin(&session) {
        return HttpResponse::build(StatusCode::OK)
            .body(std::fs::read_to_string("static/dashboard.html").expect("Failed to read file"));
    } else {
        HttpResponse::Found()
            .append_header(("Location", "/"))
            .finish()
    }
}

// API (Actuall API calls)
#[derive(Serialize)]
struct EmptyJson {}

// Login

#[derive(Deserialize)]
#[allow(non_snake_case)]
struct Login {
    username: String,
    password: String,
    userOtp: String,
}

#[post("/login")]
async fn login(
    session: Session,
    json: web::Json<Login>,
    req: actix_web::HttpRequest,
    state: web::Data<AppState>,
) -> HttpResponse {
    let username = &json.username;
    let password = &json.password;
    let otp_code = &json.userOtp;

    let ip: std::net::IpAddr;
    if let Some(peer_addr) = req.peer_addr() {
        ip = peer_addr.ip();
    } else {
        eprintln!("Failed to retrieve IP address during login. Early return.");
        return HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR).body("");
    }

    let mut requests = state.login_requests.lock().unwrap();
    let current_request_entry = &mut requests.get(&ip.to_string()).unwrap_or(&(0, 0));
    let current_request_last_request_time = current_request_entry.0;
    let current_request_counter = current_request_entry.1;
    let current_unix_timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis();

    if current_request_counter > 5
        && (current_unix_timestamp - current_request_last_request_time) < 10000
    {
        let _ = &mut requests.insert(
            ip.to_string(),
            (current_unix_timestamp, current_request_counter + 1),
        );
        return HttpResponse::build(StatusCode::BAD_REQUEST).finish();
    } else if current_request_counter > 5
        && (current_unix_timestamp - current_request_last_request_time) > 10000
    {
        let _ = &mut requests.insert(ip.to_string(), (current_unix_timestamp, 0));
    } else {
        let _ = &mut requests.insert(
            ip.to_string(),
            (current_unix_timestamp, current_request_counter + 1),
        );
    }

    // Check if username exists
    let zentrox_installation_path = path::Path::new("")
        .join(dirs::home_dir().unwrap())
        .join("zentrox_data");

    for line in fs::read_to_string(zentrox_installation_path.join("users.txt"))
        .unwrap()
        .split("\n")
    {
        let line_username_entry = line.split(": ").nth(0).expect("Failed to get username");
        let line_username = String::from_utf8(b64.decode(&line_username_entry).unwrap())
            .expect("Failed to decode username");
        let mut found_user: bool = false;
        if &line_username == username {
            found_user = true;
            let mut hasher = Hash::new();
            hasher.update(password);
            let hash = hex::encode(hasher.finalize());
            if hash == line.split(": ").nth(1).expect("Failed to get password") {
                if config_file::read("use_otp") == "1" {
                    let otp_secret = config_file::read("otp_secret");
                    if otp::calculate_current_otp(&otp_secret) == *otp_code {
                        let _ = session.insert("is_admin", true);
                        let _ = session
                            .insert("zentrox_admin_password", sudo::admin_password(&password));
                        return HttpResponse::build(StatusCode::OK).json(web::Json(EmptyJson {}));
                    } else {
                        println!("❌ Wrong OTP while authenticating {}", &username);
                    }
                } else {
                    let _ = session.insert("is_admin", true);
                    let _ =
                        session.insert("zentrox_admin_password", sudo::admin_password(&password));
                    return HttpResponse::build(StatusCode::OK).json(web::Json(EmptyJson {}));
                }
            } else {
                println!("❌ Wrong Password while authenticating {}", &username);
                return HttpResponse::build(StatusCode::FORBIDDEN).body("Missing permissions");
            }
        }
        if !found_user {
            println!("❌ User not found while authenticating {}", &username);
            return HttpResponse::build(StatusCode::FORBIDDEN).body("Missing permissions");
        }
    }
    println!("❌ Drop Thru while authenticating {}", &username);
    HttpResponse::build(StatusCode::FORBIDDEN).body("Missing permissions")
}

// Logout
#[get("/logout")]
async fn logout(session: Session) -> HttpResponse {
    session.remove("is_admin");
    session.remove("zentrox_admin_password");
    HttpResponse::Found()
        .append_header(("Location", "/"))
        .finish()
}

// Ask for Otp Secret
#[get("/login/otpSecret")]
async fn otp_secret_request() -> HttpResponse {
    #[derive(Serialize)]
    struct SecretJsonResponse {
        secret: String,
    }

    if "1" != config_file::read("knows_otp_secret") && "0" != config_file::read("use_otp") {
        config_file::write("knows_otp_secret", "1");
        return HttpResponse::build(StatusCode::OK).json(SecretJsonResponse {
            secret: config_file::read("otp_secret"),
        });
    } else {
        HttpResponse::Forbidden().finish()
    }
}

#[get("/login/useOtp")]
async fn use_otp() -> HttpResponse {
    #[derive(Serialize)]
    struct JsonResponse {
        used: bool,
    }

    HttpResponse::Ok().json(JsonResponse {
        used: config_file::read("use_otp") != "0",
    })
}

// Functional Requests
#[get("/api/cpuPercent")]
async fn cpu_percent(session: Session) -> HttpResponse {
    if !is_admin(&session) {
        return HttpResponse::Forbidden().finish();
    };

    #[derive(Serialize)]
    struct JsonResponse {
        p: f32,
    }

    let sys = System::new();
    let cpu_ussage = match sys.cpu_load_aggregate() {
        Ok(cpu) => {
            std::thread::sleep(std::time::Duration::from_secs(1)); // wait a second
            let cpu = cpu.done().unwrap();
            cpu.user * 100.0
        }
        Err(err) => {
            eprintln!("❌ CPU Ussage Error (Returned f32 0.0) {}", err);
            0.0
        }
    };

    HttpResponse::Ok().json(JsonResponse { p: cpu_ussage })
}

#[get("/api/ramPercent")]
async fn ram_percent(session: Session) -> HttpResponse {
    if !is_admin(&session) {
        return HttpResponse::Forbidden().finish();
    };

    #[derive(Serialize)]
    struct JsonResponse {
        p: f64,
    }

    let sys = System::new();
    let memory_ussage = match sys.memory() {
        Ok(memory) => {
            (memory.total.as_u64() as f64 - memory.free.as_u64() as f64)
                / memory.total.as_u64() as f64
        }
        Err(err) => {
            eprintln!("❌ Memory Ussage Error (Returned f64 0.0) {}", err);
            0.0
        }
    };

    HttpResponse::Ok().json(JsonResponse {
        p: memory_ussage * 100.0,
    })
}

#[get("/api/diskPercent")]
async fn disk_percent(session: Session) -> HttpResponse {
    if !is_admin(&session) {
        return HttpResponse::Forbidden().finish();
    };

    #[derive(Serialize)]
    struct JsonResponse {
        p: f64,
    }

    let sys = System::new();
    let disk_ussage = match sys.mount_at("/") {
        Ok(disk) => {
            (disk.total.as_u64() as f64 - disk.free.as_u64() as f64) / disk.total.as_u64() as f64
        }
        Err(err) => {
            eprintln!("❌ Disk Ussage Error (Returned f64 0.0) {}", err);
            0.0
        }
    };

    HttpResponse::Ok().json(JsonResponse {
        p: disk_ussage * 100.0,
    })
}

#[get("/api/deviceInformation")]
async fn device_information(session: Session) -> HttpResponse {
    if !is_admin(&session) {
        return HttpResponse::Forbidden().finish();
    };

    #[derive(Serialize)]
    struct JsonResponse {
        os_name: String,
        power_supply: String,
        hostname: String,
        uptime: String,
        temperature: String,
        zentrox_pid: u16,
        process_number: u32,
    }

    let os_name = String::from_utf8_lossy(
        &Command::new("lsb_release")
            .arg("-d")
            .output()
            .unwrap()
            .stdout,
    )
    .to_string()
    .replace("Description:", ""); // Operating System Name Like Fedora or Debian

    let power_supply = match fs::read_to_string("/sys/class/power_supply/BAT0/status") {
        Ok(value) => {
            if value.replace("\n", "") == "Discharging" {
                format!(
                    "Discharging {}%",
                    fs::read_to_string("/sys/class/power_supply/BAT0/capacity")
                        .expect("Failed to get Bat 0 capacity")
                )
                .to_string()
            } else if value.replace("\n", "") != "Full" {
                format!(
                    "Charging {}%",
                    fs::read_to_string("/sys/class/power_supply/BAT0/capacity")
                        .expect("Failed to get Bat 0 capacity")
                )
                .to_string()
            } else {
                value.replace("\n", "").to_string()
            }
        }

        Err(_err) => String::from("No Battery"),
    };

    // Current machines hostname. i.e.: debian_pc or 192.168.1.3
    let hostname =
        String::from_utf8_lossy(&Command::new("hostname").output().unwrap().stdout).to_string();

    let uptime =
        String::from_utf8_lossy(&Command::new("uptime").arg("-p").output().unwrap().stdout)
            .to_string()
            .replace("up ", "");
    let temperature: String = match System::new().cpu_temp() {
        Ok(value) => format!("{}°C", value as u16).to_string(),
        Err(_) => "No data".to_string(),
    };

    let mut process_number_system = SysInfoSystem::new_all();
    process_number_system.refresh_processes(sysinfo::ProcessesToUpdate::All);
    let process_number = process_number_system.processes().len() as u32;

    HttpResponse::Ok().json(JsonResponse {
        zentrox_pid: std::process::id() as u16,
        os_name,
        power_supply,
        hostname,
        uptime,
        temperature,
        process_number,
    })
}

// FTP Config
#[get("/api/fetchFTPconfig")]
async fn fetch_ftp_config(session: Session) -> HttpResponse {
    if !is_admin(&session) {
        return HttpResponse::Forbidden().finish();
    };

    #[derive(Serialize)]
    #[allow(non_snake_case)]
    struct JsonResponse {
        ftpUserUsername: String,
        ftpLocalRoot: String,
        enabled: bool,
    }

    let ftp_username = config_file::read("ftp_username");
    let ftp_local_root = config_file::read("ftp_local_root");
    let ftp_running: bool = config_file::read("ftp_running") == "1";

    HttpResponse::Ok().json(JsonResponse {
        ftpUserUsername: ftp_username,
        ftpLocalRoot: ftp_local_root,
        enabled: ftp_running,
    })
}

#[allow(non_snake_case)]
#[derive(Deserialize)]
struct JsonRequest {
    enableDisable: Option<bool>,
    enableFTP: Option<bool>,
    ftpUserUsername: Option<String>,
    ftpUserPassword: Option<String>,
    ftpLocalRoot: Option<String>,
}

#[post("/api/updateFTPConfig")]
async fn update_ftp_config(session: Session, json: web::Json<JsonRequest>) -> HttpResponse {
    if !is_admin(&session) {
        return HttpResponse::Forbidden().finish();
    };

    if !json.enableFTP.expect("Failed to get enableFTP") {
        // Kill FTP server
        let ftp_server_pid = config_file::read("ftp_pid");
        sudo::spawn(
            "zentrox".to_string(),
            session
                .get::<String>("zentrox_admin_password")
                .unwrap()
                .expect("Failed to get zentrox admin password"),
            ("kill ".to_string() + &ftp_server_pid.to_string()).to_string(),
        );
        config_file::write("ftp_running", "0");
    } else {
        config_file::write("ftp_running", "1");
        let _server = sudo::spawn(
            "zentrox".to_string(),
            session
                .get::<String>("zentrox_admin_password")
                .unwrap()
                .expect("Failed to get zentrox admin password"),
            ("python3 ftp.py ".to_string() + &whoami::username()).to_string(),
        );
    }

    if !json.enableDisable.unwrap_or(false) {
        let username = json.ftpUserUsername.clone().unwrap_or(String::from(""));
        let password = json.ftpUserPassword.clone().unwrap_or(String::from(""));
        let local_root = json.ftpLocalRoot.clone().unwrap_or(String::from(""));

        if password != "" && password.len() != 0 {
            let hasher = &mut Hash::new();
            hasher.update(&password);
            let hashed_password = hex::encode(hasher.finalize());
            config_file::write("ftp_password", &hashed_password);
        }

        if username != "" && username.len() != 0 {
            config_file::write("ftp_username", &username);
        }

        if local_root != "" && local_root.len() != 0 {
            config_file::write("ftp_local_root", &local_root);
        }
    }

    HttpResponse::Ok().json(EmptyJson {})
}

// ======================================================================
// Blocks (Used to prevent users from accessing certain static resources)

#[get("/dashboard.html")]
async fn dashboard_asset_block(session: Session) -> HttpResponse {
    if !is_admin(&session) {
        HttpResponse::build(StatusCode::FORBIDDEN).finish()
    } else {
        HttpResponse::build(StatusCode::OK)
            .body(std::fs::read_to_string("static/dashboard.html").expect("Failed to read file"))
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("🚀 Serving Zentrox on Port 8080");

    // Resetting variables to default state
    config_file::write("ftp_pid", "");
    config_file::write("ftp_running", "0");

    let secret_session_key = Key::try_generate().unwrap();
    let app_state = web::Data::new(AppState {
        login_requests: std::sync::Mutex::new(HashMap::new()),
    });

    if config_file::read("otp_secret") == "" && config_file::read("use_otp") == "1" {
        let new_otp_secret = otp::generate_otp_secret();
        config_file::write("otp_secret", &new_otp_secret);
        println!("🔒 Your One-Time-Pad Secret is: {}\n🔒 Store this in a secure location and add it to your 2FA app.\n🔒 If you lose this key, you will need physical access to this device.\n🔒 From there, you can find it in ~/zentrox_data/config.toml", new_otp_secret)
    }
    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .wrap(
                SessionMiddleware::builder(
                    CookieSessionStore::default(),
                    secret_session_key.clone(),
                )
                .cookie_secure(false)
                .build(),
            )
            .wrap(Cors::permissive())
            .wrap(middleware::Compress::default())
            // Landing
            .service(dashboard)
            .service(index)
            // Login, OTP and Logout
            .service(login) // Login user using username, password and otp token if enabled
            .service(logout) // Remove admin status and redirect to /
            .service(otp_secret_request) // Return OTP secret, if this is the first time viewing
            // the secret
            .service(use_otp) // Return if OTP is enabled
            // API
            // API Device Stats
            .service(cpu_percent) // CPU Ussage as f64
            .service(ram_percent) // RAM Ussage as f64
            .service(disk_percent) // Disk (/) as f64
            .service(device_information) // General device information
            // API FTP
            .service(fetch_ftp_config)
            .service(update_ftp_config)
            // General services and blocks
            .service(dashboard_asset_block)
            .service(afs::Files::new("/", "static/"))
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
