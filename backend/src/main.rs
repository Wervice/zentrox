use actix_cors::Cors;
use actix_files as afs;
use actix_multipart::form::{tempfile::TempFile, text::Text, MultipartForm};
use actix_rt::time::interval;
use actix_session::{storage::CookieSessionStore, Session, SessionMiddleware};
use actix_web::cookie::Key;
use actix_web::{error, HttpRequest};
use actix_web::{
    get, http::header, http::StatusCode, middleware, post, web, web::Data, App, HttpResponse,
    HttpServer,
};
use aes_gcm::aead::Buffer;
use base64::{engine::general_purpose::STANDARD as b64, Engine as _};
use dirs::{self, home_dir};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::HashMap,
    env,
    fs::{self, File},
    io::{BufReader, Read, Seek, SeekFrom},
    path::{self, Path, PathBuf},
    process::Command,
    sync::{Arc, Mutex},
    time::{SystemTime, UNIX_EPOCH},
};
use sysinfo::System as SysInfoSystem;
use systemstat::{Platform, System};
use tokio::task;
extern crate inflector;
use inflector::Inflector;

mod config_file;
mod crypto_utils;
mod drives;
mod is_admin;
mod logs;
mod mime;
mod otp;
mod packages;
mod setup;
mod sudo;
mod ufw;
mod url_decode;
mod vault;
mod video;
mod visit_dirs;

use is_admin::is_admin_state;
use visit_dirs::visit_dirs;

#[allow(non_snake_case)]
#[derive(Clone)]
/// Current state of the application used to keep track of the logged in users, DoS/Brute force
/// attack requests and sharing a instance of the System struct.
struct AppState {
    login_requests: Arc<
        Mutex<
            HashMap<
                String, /* IP Adress of caller */
                (
                    u128, /* Unix Timestamp of last request */
                    u64,  /* Number of requests since last reset */
                ),
            >,
        >,
    >,
    login_token: Arc<Mutex<String>>,
    system: Arc<Mutex<System>>,
    username: Arc<Mutex<String>>,
}

impl AppState {
    /// Initiate a new AppState
    fn new() -> Self {
        let random_string: Vec<u8> = (0..128).map(|_| rand::random::<u8>()).collect();
        AppState {
            login_requests: Arc::new(Mutex::new(HashMap::new())),
            login_token: Arc::new(Mutex::new(
                String::from_utf8_lossy(&random_string).to_string(),
            )),
            system: Arc::new(Mutex::new(System::new())),
            username: Arc::new(Mutex::new(String::new())),
        }
    }

    /// Remove old IP adresses from the AppState login_requests.
    /// The is required to be lighther on memory and be GDPR compliant.
    fn cleanup_old_ips(&self) {
        let mut login_requests = self.login_requests.lock().unwrap();
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_millis();

        // Retain only entries where the timestamp is within the last 15 minutes (900 seconds)
        login_requests.retain(|_ip, (timestamp, _)| {
            let ip_age_seconds = current_time - *timestamp;
            ip_age_seconds <= 900_000 // 900 seconds = 15 minutes
        });
    }

    /// Initiate a loop that periodically cleans up the login_requests of the current AppState.
    fn start_cleanup_task(self) {
        let cleanup_interval = std::time::Duration::from_secs(60); // Every 60 seconds
        task::spawn(async move {
            let mut interval = interval(cleanup_interval);

            loop {
                interval.tick().await;
                self.cleanup_old_ips();
            }
        });
    }
}

/// Root of the server.
///
/// If the user is logged in, they get redireced to /dashboard, otherwise the login is shown.
#[get("/")]
async fn index(session: Session, state: Data<AppState>) -> HttpResponse {
    // is_admin session value is != true (None or false), the user is served the login screen
    // otherwise, the user is redirected to /
    if is_admin_state(&session, state) {
        HttpResponse::Found()
            .append_header(("Location", "/dashboard"))
            .finish()
    } else {
        HttpResponse::build(StatusCode::OK)
            .body(std::fs::read_to_string("static/index.html").expect("Failed to read file"))
    }
}

#[get("/alerts")]
async fn alerts(session: Session, state: Data<AppState>) -> HttpResponse {
    // is_admin session value is != true (None or false), the user is served the alerts screen
    // otherwise, the user is redirected to /
    if is_admin_state(&session, state) {
        HttpResponse::build(StatusCode::OK).body(
            std::fs::read_to_string("static/alerts.html").expect("Failed to read alerts page"),
        )
    } else {
        HttpResponse::Found()
            .append_header(("Location", "/?app=true"))
            .finish()
    }
}

#[get("/media")]
async fn media(session: Session, state: Data<AppState>) -> HttpResponse {
    // is_admin session value is != true (None or false), the user is served the media screen
    // otherwise, the user is redirected to /
    if is_admin_state(&session, state) {
        if config_file::read("media_enabled") == "0" {
            return HttpResponse::Forbidden().body("Media center has been disabled");
        }
        HttpResponse::build(StatusCode::OK)
            .body(std::fs::read_to_string("static/media.html").expect("Failed to read alerts page"))
    } else {
        HttpResponse::Found()
            .append_header(("Location", "/"))
            .finish()
    }
}

#[get("/alerts/manifest.json")]
async fn alerts_manifest() -> HttpResponse {
    HttpResponse::Ok().body(include_str!("../manifest.json"))
}

/// The dashboard route.
///
/// If the user is logged in, the dashboard is shown, otherwise they get redirected to root.
#[get("/dashboard")]
async fn dashboard(session: Session, state: Data<AppState>) -> HttpResponse {
    // is_admin session value is != true (None or false), the user is redirected to /
    // otherwise, the user is served the dashboard.html file
    if is_admin_state(&session, state) {
        HttpResponse::build(StatusCode::OK)
            .body(std::fs::read_to_string("static/dashboard.html").expect("Failed to read file"))
    } else {
        HttpResponse::Found()
            .append_header(("Location", "/"))
            .finish()
    }
}

// API (Actuall API calls)

/// Empty Json Response
///
/// This struct implements serde::Serialize. It can be used to responde with an empty Json
/// response.
#[derive(Serialize)]
struct EmptyJson {}

/// Request that only contains a sudo password from the backend.
///
/// This struct implements serde::Derserialize. It can be used to parse a single sudoPassword from
/// the user. It only has the String filed sudoPassword.
#[derive(Deserialize)]
#[allow(non_snake_case)]
struct SudoPasswordOnlyRequest {
    sudoPassword: String,
}

// Login

#[derive(Deserialize)]
#[allow(non_snake_case)]
struct Login {
    username: String,
    password: String,
    userOtp: String,
}

/// Route that loggs a user in.
///
/// First the users name is check in the users database. If it is not in there, the user is
/// rejected.
/// Next, the user password is hashed and compared to the password corresponding to the stored
/// hash.
/// If the user has enabled 2FA via OTP, the provided token is compared to the one that can be
/// calculated using the stored OTP secret.
/// The function keeps track on how often a user attempted to login. If they tried to login more
/// than 5 times in 10 seconds, they are automatically being rejected for the next 10 seconds, even
/// if the credentials are correct.
#[post("/login")]
async fn login(
    session: Session,
    json: web::Json<Login>,
    req: actix_web::HttpRequest,
    state: Data<AppState>,
) -> HttpResponse {
    let username = &json.username;
    let password = &json.password;
    let otp_code = &json.userOtp;

    let ip: String;
    if let Some(peer_addr) = req.peer_addr() {
        let mut hasher = Sha256::new();
        hasher.update(peer_addr.ip().to_string());
        ip = hex::encode(hasher.finalize()).to_string();
    } else {
        eprintln!("Failed to retrieve IP address during login. Early return.");
        return HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR).body("");
    }

    let mut requests = match state.login_requests.lock() {
        Ok(guard) => guard,
        Err(v) => v.into_inner(), // Recover the lock even if it's poisoned
    };

    let current_request_entry = &mut requests.get(&ip.to_string()).unwrap_or(&(0, 0));
    let current_request_last_request_time = current_request_entry.0;
    let current_request_counter = current_request_entry.1;
    let current_unix_timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis();

    if current_request_counter > 10
        && (current_unix_timestamp - current_request_last_request_time) < 10000
    {
        let _ = &mut requests.insert(
            ip.to_string(),
            (current_unix_timestamp, current_request_counter + 1),
        );
        return HttpResponse::TooManyRequests().body("You were rate-limited.");
    } else if current_request_counter > 5 {
        // Implementing exponential back off
        let penalty_time = 2_u64.pow(current_request_counter.saturating_sub(5) as u32) * 1000; // Exponential back off in milliseconds

        if (current_unix_timestamp - current_request_last_request_time) < penalty_time.into() {
            let _ = &mut requests.insert(
                ip.to_string(),
                (current_unix_timestamp, current_request_counter + 1),
            );
            return HttpResponse::TooManyRequests().body("You were rate-limited.");
        } else {
            // Reset the counter after the penalty period has passed
            let _ = &mut requests.insert(ip.to_string(), (current_unix_timestamp, 1));
        }
    } else {
        // Increment the request counter and update the last request timestamp
        let _ = &mut requests.insert(
            ip.to_string(),
            (current_unix_timestamp, current_request_counter + 1),
        );
    }

    // Check if username exists
    let zentrox_installation_path = path::Path::new("")
        .join(dirs::home_dir().unwrap())
        .join(".local")
        .join("share")
        .join("zentrox");

    for line in fs::read_to_string(zentrox_installation_path.join("users"))
        .unwrap()
        .split("\n")
    {
        let line_username_entry = line.split(": ").next().expect("Failed to get username");
        let line_username = String::from_utf8(b64.decode(line_username_entry).unwrap())
            .expect("Failed to decode username");
        let mut found_user: bool = false;
        if &line_username == username {
            found_user = true;

            let stored_password = line.split(": ").nth(1).expect("Failed to get password");
            let hashes_correct =
                is_admin::password_hash(password.to_string(), stored_password.to_string());
            if hashes_correct {
                let login_token: Vec<u8> = is_admin::generate_random_token();
                if config_file::read("use_otp") == "1" {
                    let otp_secret = config_file::read("otp_secret");
                    if otp::calculate_current_otp(&otp_secret) == *otp_code {
                        let _ =
                            session.insert("login_token", hex::encode(&login_token).to_string());

                        *state.login_token.lock().unwrap() = hex::encode(&login_token).to_string();
                        *state.username.lock().unwrap() = username.to_string();

                        return HttpResponse::build(StatusCode::OK).json(web::Json(EmptyJson {}));
                    } else {
                        println!("Wrong OTP while authenticating");
                    }
                } else {
                    // for hashes
                    let _ = session.insert("login_token", hex::encode(&login_token).to_string());

                    *state.login_token.lock().unwrap() = hex::encode(&login_token).to_string();
                    *state.username.lock().unwrap() = username.to_string();

                    return HttpResponse::build(StatusCode::OK).json(web::Json(EmptyJson {}));
                }
            } else {
                println!("Wrong Password while authenticating");
                return HttpResponse::build(StatusCode::FORBIDDEN).body("Missing permissions");
            }
        }
        if !found_user {
            println!("User not found while authenticating");
            return HttpResponse::build(StatusCode::FORBIDDEN).body("Missing permissions");
        }
    }
    println!("Drop Thru while authenticating");
    HttpResponse::build(StatusCode::FORBIDDEN).body("Missing permissions")
}

/// Log out a user.
///
/// This function removes the users login token from the cookie as well as the
/// zentrox_admin_password. This invalidates the user and they are logged out.
/// To prevent the user from re-using the current cookie, the state is replaced by a new random
/// token that is longer than the one that would normally be used to log in.
#[post("/logout")]
async fn logout(session: Session, state: Data<AppState>) -> HttpResponse {
    if is_admin_state(&session, state.clone()) {
        session.purge();
        *state.login_token.lock().unwrap() =
            hex::encode((0..64).map(|_| rand::random::<u8>()).collect::<Vec<u8>>()).to_string();
        HttpResponse::Found()
            .append_header(("Location", "/"))
            .finish()
    } else {
        HttpResponse::BadRequest().body("You are not logged in.")
    }
}

/// Retrieve OTP secret on first login.
///
/// This function will only return the users OTP secret when the web page is viewed for the first
/// time. To keep track of this status, a key knows_otp_secret is used.
#[post("/login/otpSecret")]
async fn otp_secret_request(_state: Data<AppState>) -> HttpResponse {
    #[derive(Serialize)]
    struct SecretJsonResponse {
        secret: String,
    }

    if "1" != config_file::read("knows_otp_secret") && "0" != config_file::read("use_otp") {
        let _ = config_file::write("knows_otp_secret", "1");
        HttpResponse::build(StatusCode::OK).json(SecretJsonResponse {
            secret: config_file::read("otp_secret"),
        })
    } else {
        HttpResponse::Forbidden().body("You can not access this value anymore.")
    }
}

/// Check if the users uses OTP.
///
/// This function returns a boolean depending on the user using OTP or not.
#[post("/login/useOtp")]
async fn use_otp(_state: Data<AppState>) -> HttpResponse {
    #[derive(Serialize)]
    struct JsonResponse {
        used: bool,
    }

    HttpResponse::Ok().json(JsonResponse {
        used: config_file::read("use_otp") != "0",
    })
}

// Functional Requests

/// Return the CPU ussage percentage.
#[get("/api/cpuPercent")]
async fn cpu_percent(session: Session, state: Data<AppState>) -> HttpResponse {
    if !is_admin_state(&session, state.clone()) {
        return HttpResponse::Forbidden().body("This resource is blocked.");
    };

    #[derive(Serialize)]
    struct JsonResponse {
        p: f32,
    }

    let cpu_ussage = match state.system.lock().unwrap().cpu_load_aggregate() {
        Ok(cpu) => {
            std::thread::sleep(std::time::Duration::from_secs(1));
            let cpu = cpu.done().unwrap();
            cpu.user * 100.0
        }
        Err(err) => {
            eprintln!("CPU Ussage Error (Returned f32 0.0) {}", err);
            0.0
        }
    };

    HttpResponse::Ok().json(JsonResponse { p: cpu_ussage })
}

/// Return the the RAM ussage percentage.
#[get("/api/ramPercent")]
async fn ram_percent(session: Session, state: Data<AppState>) -> HttpResponse {
    if !is_admin_state(&session, state.clone()) {
        return HttpResponse::Forbidden().body("This resource is blocked.");
    };

    #[derive(Serialize)]
    struct JsonResponse {
        p: f64,
    }

    let memory_ussage = match state.system.lock().unwrap().memory() {
        Ok(memory) => {
            (memory.total.as_u64() as f64 - memory.free.as_u64() as f64)
                / memory.total.as_u64() as f64
        }
        Err(err) => {
            eprintln!("Memory Ussage Error (Returned f64 0.0) {}", err);
            0.0
        }
    };

    HttpResponse::Ok().json(JsonResponse {
        p: memory_ussage * 100.0,
    })
}

/// Return the main disk ussage percentage.
#[get("/api/diskPercent")]
async fn disk_percent(session: Session, state: Data<AppState>) -> HttpResponse {
    if !is_admin_state(&session, state.clone()) {
        return HttpResponse::Forbidden().body("This resource is blocked.");
    };

    #[derive(Serialize)]
    struct JsonResponse {
        p: f64,
    }

    let disk_ussage = match state.system.lock().unwrap().mount_at("/") {
        Ok(disk) => {
            let total = disk.total.as_u64() as f64;
            let free = disk.free.as_u64() as f64;
            if total > 0.0 {
                (total - free) / total
            } else {
                0.0
            }
        }
        Err(err) => {
            eprintln!("Disk Ussage Error (Returned f64 0.0) {}", err);
            0.0
        }
    };

    HttpResponse::Ok().json(JsonResponse {
        p: disk_ussage * 100.0,
    })
}

/// Return general information about the system. This includes:
/// * `os_name` {string} - The name of your operating system. i.e.: Debian Bookworm 12
/// * `power_supply` {string} - Does you PC get AC power of battery? Is it charging?
/// * `hostname` {string} - The hostname of your computer.
/// * `uptime ` {string} - How long is your computer running since the last boot.
/// * `temperature` {string} - Your computer CPU temperature in celcius.
/// * `zentrox_pid` {u16} - The PID of the current running Zentrox instance.
/// * `process_number` {u32} - The number of active running processes
#[get("/api/deviceInformation")]
async fn device_information(session: Session, state: Data<AppState>) -> HttpResponse {
    if !is_admin_state(&session, state) {
        return HttpResponse::Forbidden().body("This resource is blocked.");
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

    let os_name = match Command::new("lsb_release").arg("-d").output() {
        Ok(output_value) => {
            String::from_utf8_lossy(&output_value.stdout)
                .to_string()
                .replace("Description:", "") // Operating System Name Like Fedora or Debian
        }
        Err(_) => {
            let data = fs::read_to_string("/etc/os-release").unwrap();
            let lines = data.lines();
            let mut rv = "Unknown OS".to_string();
            for line in lines {
                let line_split = line.split("=").collect::<Vec<&str>>();
                if line_split.len() == 2 && line_split[0] == "NAME" {
                    rv = line_split[1].replace("\"", "").to_string();
                    break;
                }
            }
            rv
        }
    };

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
    let hostname = {
        match Command::new("hostname").output() {
            Ok(v) => String::from_utf8_lossy(&v.stdout).replace("\n", ""),
            Err(_) => fs::read_to_string("/etc/hostname")
                .unwrap_or("No Hostname".to_string())
                .to_string(),
        }
    };

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

// FTP API

/// Return the current FTP config.
///
/// This includes the ftp username, password and status (is the server on or off)
#[get("/api/fetchFTPconfig")]
async fn fetch_ftp_config(session: Session, state: Data<AppState>) -> HttpResponse {
    if !is_admin_state(&session, state) {
        return HttpResponse::Forbidden().body("This resource is blocked.");
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
    sudoPassword: String,
}

/// Update the FTP config.
///
/// This function updates the FTP config. For this to work, Zentrox needs the sudo password to
/// enable or disable the FTP server, depending on the users choice. The request can also only
/// contain the desired status instead of username, password or other information. In this case,
/// the value enableDisable has to be true.
#[post("/api/updateFTPConfig")]
async fn update_ftp_config(
    session: Session,
    json: web::Json<JsonRequest>,
    state: Data<AppState>,
) -> HttpResponse {
    if !is_admin_state(&session, state) {
        return HttpResponse::Forbidden().body("This resource is blocked.");
    };

    if !json.enableFTP.expect("Failed to get enableFTP") {
        // Kill FTP server
        let sudo_password = json.sudoPassword.to_string();
        let ftp_server_pid = config_file::read("ftp_pid");
        if !ftp_server_pid.is_empty() {
            std::thread::spawn(move || {
                let _ = sudo::SwitchedUserCommand::new(sudo_password, String::from("kill"))
                    .arg(ftp_server_pid.to_string())
                    .spawn();
            });
            let _ = config_file::write("ftp_running", "0");
        }
    } else {
        let sudo_password = json.sudoPassword.to_string();

        std::thread::spawn(move || {
            let _ = sudo::SwitchedUserCommand::new(sudo_password, String::from("python3"))
                .arg("ftp.py".to_string())
                .arg(whoami::username_os().into_string().unwrap())
                .spawn();
        });
    }

    if !json.enableDisable.unwrap_or(false) {
        let username = json.ftpUserUsername.clone().unwrap_or(String::from(""));
        let password = json.ftpUserPassword.clone().unwrap_or(String::from(""));
        let local_root = json.ftpLocalRoot.clone().unwrap_or(String::from(""));

        if !password.is_empty() {
            let hasher = &mut Sha256::new();
            hasher.update(&password);
            let hashed_password = hex::encode(hasher.clone().finalize());
            let _ = config_file::write("ftp_password", &hashed_password);
        }

        if !username.is_empty() {
            let _ = config_file::write("ftp_username", &username);
        }

        if !local_root.is_empty() {
            let _ = config_file::write("ftp_local_root", &local_root);
        }
    }

    HttpResponse::Ok().json(EmptyJson {})
}

// Package API

#[derive(Serialize)]
struct PackageResponseJson {
    apps: Vec<Vec<std::string::String>>, // Any "package" that has a .desktop file
    packages: Vec<String>, // Any package the supported package managers (apt, pacman and dnf) say
    // would be installed on the system (names only)
    others: Vec<String>, // Not installed and not a .desktop file
}

/// Return the current package database.
///
/// This returns a list of every installed packages, every app the has a .desktop file and all
/// available packages that are listed in the package manager.
#[get("/api/packageDatabase")]
async fn package_database(session: Session, state: Data<AppState>) -> HttpResponse {
    if !is_admin_state(&session, state) {
        return HttpResponse::Forbidden().body("This resource is blocked.");
    }

    let installed = match packages::list_installed_packages() {
        Ok(packages) => packages,
        Err(err) => {
            eprintln!("Listing installed packages failed: {}", err);
            Vec::new()
        }
    };

    let desktops = match packages::list_desktop_applications() {
        Ok(v) => v,
        Err(_) => {
            return HttpResponse::InternalServerError().body("Failed to list desktop applications")
        }
    };

    let desktops_clear = {
        let mut clear: Vec<Vec<String>> = Vec::new();

        for entry in desktops {
            clear.push(vec![entry.name, entry.exec_name]);
        }

        clear
    };

    let available = packages::list_available_packages().unwrap();

    HttpResponse::Ok().json(PackageResponseJson {
        apps: desktops_clear, // Placeholder
        packages: installed,
        others: available, // Placeholder
    })
}

// Packages that would be affected by an autoremove

#[derive(Serialize)]
struct PackageDatabaseAutoremoveJson {
    packages: Vec<String>,
}

/// Return a list of all packages that would be affected by an autoremove.
#[get("/api/packageDatabaseAutoremove")]
async fn package_database_autoremove(session: Session, state: Data<AppState>) -> HttpResponse {
    if !is_admin_state(&session, state) {
        return HttpResponse::Forbidden().body("This resource is blocked.");
    }

    let packages = packages::list_autoremoveable_packages().unwrap();

    HttpResponse::Ok().json(PackageDatabaseAutoremoveJson { packages })
}

#[derive(Deserialize)]
#[allow(non_snake_case)]
struct PackageActionRequest {
    packageName: String,
    sudoPassword: String,
}

#[post("/api/installPackage")]
/// Install a package on the users system.
///
/// It requires the package name along side the sudo password in the request body.
/// This only works under apt, dnf and pacman.
async fn install_package(
    session: Session,
    json: web::Json<PackageActionRequest>,
    state: Data<AppState>,
) -> HttpResponse {
    if !is_admin_state(&session, state) {
        return HttpResponse::Forbidden().body("This resource is blocked.");
    }

    let package_mame = &json.packageName;
    let sudo_password = &json.sudoPassword;

    match packages::install_package(package_mame.to_string(), sudo_password.to_string()) {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(_) => HttpResponse::InternalServerError().body("Failed to install package."),
    }
}

#[post("/api/removePackage")]
/// Remove a package from the users system.
///
/// It requires the package name along side the sudo password in the request body.
/// This only works under apt, dnf and pacman.
async fn remove_package(
    session: Session,
    json: web::Json<PackageActionRequest>,
    state: Data<AppState>,
) -> HttpResponse {
    if !is_admin_state(&session, state) {
        return HttpResponse::Forbidden().body("This resource is blocked.");
    }

    let package_mame = &json.packageName;
    let sudo_password = &json.sudoPassword;

    match packages::remove_package(package_mame.to_string(), sudo_password.to_string()) {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(_) => HttpResponse::InternalServerError().body("Failed to remove package."),
    }
}

#[post("/api/clearAutoRemove")]
/// Run an autoremove command on the users computer.
async fn clear_auto_remove(
    session: Session,
    json: web::Json<SudoPasswordOnlyRequest>,
    state: Data<AppState>,
) -> HttpResponse {
    if !is_admin_state(&session, state) {
        return HttpResponse::Forbidden().body("This resource is blocked.");
    }

    let sudo_password = &json.sudoPassword;

    match packages::auto_remove(sudo_password.to_string()) {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(_) => HttpResponse::InternalServerError().body("Failed to autoremove package."),
    }
}

// Firewall API

#[derive(Serialize)]
struct FireWallInformationResponseJson {
    enabled: bool,
    rules: Vec<ufw::UfwRule>,
}

#[post("/api/fireWallInformation")]
/// Returns general information about the current UFW firewall configuration.
async fn firewall_information(
    session: Session,
    state: Data<AppState>,
    json: web::Json<SudoPasswordOnlyRequest>,
) -> HttpResponse {
    if !is_admin_state(&session, state) {
        return HttpResponse::Forbidden().body("This resource is blocked.");
    }

    let password = &json.sudoPassword;

    match ufw::ufw_status(password.to_string()) {
        Ok(ufw_status) => {
            let enabled = ufw_status.0;
            let rules = ufw_status.1;

            HttpResponse::Ok().json(FireWallInformationResponseJson { enabled, rules })
        }
        Err(err) => {
            eprintln!("UFW Status error {err}");
            HttpResponse::BadRequest().body(err)
        }
    }
}

#[post("/api/switchUfw/{value}")]
/// Enable or disable the UFW firewall.
///
/// This requires a url parameter. It can either be "true" or "false".
/// In addtion to that the request has to server the user with a sudo password.
async fn switch_ufw(
    session: Session,
    state: Data<AppState>,
    path: web::Path<String>,
    json: web::Json<SudoPasswordOnlyRequest>,
) -> HttpResponse {
    if !is_admin_state(&session, state) {
        return HttpResponse::Forbidden().body("This resource is blocked.");
    }

    let v: String = path.into_inner();

    if v == *"true" {
        let password = &json.sudoPassword;
        match sudo::SwitchedUserCommand::new(
            password.to_string(),
            "/usr/sbin/ufw --force enable".to_string(),
        )
        .spawn()
        {
            Ok(status) => {
                if status == 0 {
                    println!("✅ Started UFW");
                    return HttpResponse::Ok().finish();
                } else {
                    println!("Failed to start UFW (Status != 0)");
                    return HttpResponse::InternalServerError()
                        .body("Failed to start UFW (Return value unequal 0)");
                }
            }
            Err(_) => {
                println!("Failed to start UFW (Err)");
                return HttpResponse::InternalServerError()
                    .body("Failed to start UFW because to command error");
            }
        }
    } else if v == *"false" {
        let password = &json.sudoPassword;
        match sudo::SwitchedUserCommand::new(
            password.to_string(),
            "/usr/sbin/ufw --force disable".to_string(),
        )
        .spawn()
        {
            Ok(status) => {
                if status == 0 {
                    println!("✅ Stopped UFW");
                    return HttpResponse::Ok().finish();
                } else {
                    println!("Failed to stop UFW (Status != 0)");
                    return HttpResponse::InternalServerError()
                        .body("Failed to stop UFW (Return value unequal 0)");
                }
            }
            Err(_) => {
                println!("Failed to stop UFW (Err)");
                return HttpResponse::InternalServerError()
                    .body("Failed to stop UFW because of command error");
            }
        }
    }

    HttpResponse::Ok().finish()
}

#[post("/api/newFireWallRule/{from}/{to}/{action}")]
/// Create a new firewall rule.
///
/// This request takes three URL parameters.
/// * `from` - The IP adress the request comes from (can be "any" as well).
/// * `to` - The port the request goes to.
/// * `action` - The action (allow / deny) that is taken.
/// This requires a sudo password.
async fn new_firewall_rule(
    session: Session,
    state: Data<AppState>,
    json: web::Json<SudoPasswordOnlyRequest>,
    path: web::Path<(String, String, String)>,
) -> HttpResponse {
    if !is_admin_state(&session, state) {
        return HttpResponse::Forbidden().body("This resource is blocked.");
    }

    let password = &json.sudoPassword;
    let (mut from, mut to, action) = path.into_inner();

    if action.is_empty() {
        println!("User provided insufficent firewall rule settings");
        return HttpResponse::BadRequest()
            .body("The UFW configuration provided by the user was insufficent.");
    }

    if from.is_empty() {
        from = "any".to_string()
    }

    if to.is_empty() {
        to = "any".to_string()
    }

    match ufw::new_rule(String::from(password), from, to, action) {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(_) => HttpResponse::InternalServerError()
            .body("Failed to create new rule because of command error"),
    }
}

#[post("/api/deleteFireWallRule/{index}")]
/// Delete a firewall rule by its index.
async fn delete_firewall_rule(
    session: Session,
    state: Data<AppState>,
    json: web::Json<SudoPasswordOnlyRequest>,
    path: web::Path<i32>,
) -> HttpResponse {
    if !is_admin_state(&session, state) {
        return HttpResponse::Forbidden().body("This resource is blocked.");
    }

    let i = path.into_inner();
    let password = &json.sudoPassword;

    match ufw::delete_rule(password.to_string(), i as u32) {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(_) => HttpResponse::InternalServerError()
            .body("Failed to remove rule because of command error"),
    }
}

// File API
#[get("/api/callFile/{file_name}")]
/// Download a file from the machines file system.
/// This does not work for files that can not be read by the current user.
async fn call_file(
    session: Session,
    state: Data<AppState>,
    path: web::Path<String>,
) -> HttpResponse {
    if !is_admin_state(&session, state) {
        return HttpResponse::Forbidden().body("This resource is blocked.");
    }

    let file_path = url_decode::url_decode(&path);

    if path::Path::new(&file_path).exists() {
        let f = fs::read(&file_path);
        match f {
            Ok(fh) => {
                HttpResponse::Ok().body(fh.bytes().map(|x| x.unwrap_or(0_u8)).collect::<Vec<u8>>())
            }
            Err(_) => HttpResponse::InternalServerError()
                .body(format!("Failed to read file {}", &file_path)),
        }
    } else {
        HttpResponse::BadRequest().body("This file does not exist.")
    }
}

#[derive(Serialize)]
struct FilesListJson {
    content: Vec<(String, String)>,
}

#[get("/api/filesList/{path}")]
/// List all the files and folders in a current path.
/// The path provided in the URL has to be url encoded.
async fn files_list(
    session: Session,
    state: Data<AppState>,
    path: web::Path<String>,
) -> HttpResponse {
    if !is_admin_state(&session, state) {
        return HttpResponse::Forbidden().body("This resource is blocked.");
    }

    let dir_path = url_decode::url_decode(&path);

    match fs::read_dir(dir_path) {
        Ok(contents) => {
            let mut result: Vec<(String, String)> = Vec::new();
            for e in contents {
                let e_unwrap = &e.unwrap();
                let file_name = &e_unwrap.file_name().into_string().unwrap();
                let is_file = e_unwrap.metadata().unwrap().is_file();
                let is_dir = e_unwrap.metadata().unwrap().is_dir();
                let is_symlink = e_unwrap.metadata().unwrap().is_symlink();

                if is_file {
                    result.push((file_name.to_string(), "f".to_string()))
                } else if is_dir || is_symlink {
                    result.push((file_name.to_string(), "d".to_string()))
                } else {
                    result.push((file_name.to_string(), "a".to_string()))
                }
            }
            HttpResponse::Ok().json(FilesListJson { content: result })
        }
        Err(_) => HttpResponse::InternalServerError().body("Failed to read directory."),
    }
}

#[get("/api/deleteFile/{path}")]
/// Delete a path from the machines file system.
/// This does not work with files that can not be written by the current user.
/// The path provided in the requests URL has to be url encoded.
async fn delete_file(
    session: Session,
    state: Data<AppState>,
    path: web::Path<String>,
) -> HttpResponse {
    if !is_admin_state(&session, state) {
        return HttpResponse::Forbidden().body("This resource is blocked.");
    }

    let file_path = url_decode::url_decode(&path);

    if std::path::Path::new(&file_path).exists() {
        let metadata = fs::metadata(&file_path).unwrap();
        let is_file = metadata.is_file();
        let is_dir = metadata.is_dir();
        let is_link = metadata.is_symlink();
        let has_permissions = !metadata.permissions().readonly();

        if (is_file || is_link) && has_permissions {
            match fs::remove_file(&file_path) {
                Ok(_) => HttpResponse::Ok().finish(),
                Err(_) => HttpResponse::InternalServerError().body("Failed to delete file."),
            }
        } else if is_dir && has_permissions {
            match fs::remove_dir_all(&file_path) {
                Ok(_) => HttpResponse::Ok().finish(),
                Err(_) => HttpResponse::InternalServerError().body("Failed to delete directory."),
            }
        } else {
            HttpResponse::Forbidden().body("Missing file permissions. File is readonly.")
        }
    } else {
        HttpResponse::BadRequest().body("This path does not exist")
    }
}

#[get("/api/renameFile/{old_path}/{new_path}")]
/// Rename/move a file or folder on the machines file system.
/// This does not work with files that can not be written/read by the current user.
/// The paths provided in the URL have to be url encoded.
async fn rename_file(
    session: Session,
    state: Data<AppState>,
    path: web::Path<(String, String)>,
) -> HttpResponse {
    if !is_admin_state(&session, state) {
        return HttpResponse::Forbidden().body("This resource is blocked.");
    }

    let (old_path_e, new_path_e) = &path.into_inner();
    let old_path = url_decode::url_decode(old_path_e);
    let new_path = url_decode::url_decode(new_path_e);

    if std::path::Path::new(&old_path).exists() {
        let metadata = fs::metadata(&old_path).unwrap();
        let has_permissions = !metadata.permissions().readonly();
        if has_permissions && !std::path::Path::new(&new_path).exists() {
            match fs::rename(old_path, new_path) {
                Ok(_) => HttpResponse::Ok().finish(),
                Err(_) => HttpResponse::InternalServerError().body("Failed to rename file"),
            }
        } else {
            HttpResponse::Forbidden().body("Missing file permissions. Can not rename.")
        }
    } else {
        HttpResponse::BadRequest().body("This path does not exist")
    }
}

#[get("/api/burnFile/{path}")]
/// Overwrite a file with random data and then delete it.
/// This does not work with files that can not be written by the current user.
/// The path provided in the URL has to be url encoded.
async fn burn_file(
    session: Session,
    state: Data<AppState>,
    path: web::Path<String>,
) -> HttpResponse {
    if !is_admin_state(&session, state) {
        return HttpResponse::Forbidden().body("This resource is blocked.");
    }

    let file_path = url_decode::url_decode(&path);

    if std::path::Path::new(&file_path).exists() {
        let metadata = fs::metadata(&file_path).unwrap();
        let has_permissions = !metadata.permissions().readonly();
        if has_permissions {
            let size = metadata.len();
            let r_s = (0..size).map(|_| rand::random::<u8>()).collect::<Vec<u8>>();
            match fs::write(file_path.clone(), r_s) {
                Ok(_) => {
                    let _ = fs::remove_file(&file_path);
                    HttpResponse::Ok().finish()
                }
                Err(_) => HttpResponse::InternalServerError().body("Failed to destroy file."),
            }
        } else {
            HttpResponse::Forbidden().body("Missing file permissions. Can not burn.")
        }
    } else {
        HttpResponse::BadRequest().body("This path does not exist")
    }
}

// Block Device API
#[derive(Serialize)]
struct DriveListJson {
    drives: Vec<drives::BlockDevice>,
}

#[get("/api/driveList")]
/// List all block devices connected to the current machine including partition and virtual block
/// devices.
async fn list_drives(session: Session, state: Data<AppState>) -> HttpResponse {
    if !is_admin_state(&session, state) {
        return HttpResponse::Forbidden().body("This resource is blocked.");
    }

    let drives_out = drives::device_list();

    let drives_out_blkdv = drives_out
        .expect("Failed to get block devices.")
        .blockdevices;

    HttpResponse::Ok().json(DriveListJson {
        drives: drives_out_blkdv,
    })
}

#[derive(Serialize)]
struct DriveInformationJson {
    drives: drives::Drive,
    ussage: Vec<(String, u64, u64, u64, f64, String)>,
}

#[get("/api/driveInformation/{drive}")]
/// Return information about a block device specified in the URL.
/// The provided information includes (but is not limited to) the block devices mountpoint, file
/// size, path, owner...
async fn drive_information(
    session: Session,
    state: Data<AppState>,
    path: web::Path<String>,
) -> HttpResponse {
    if !is_admin_state(&session, state) {
        return HttpResponse::Forbidden().body("This resource is blocked.");
    }

    let drive = path;

    let info = drives::drive_information(drive.to_string());

    HttpResponse::Ok().json(DriveInformationJson {
        drives: info.unwrap(),
        ussage: drives::drive_statistics(drive.to_string())
            .expect("Failed to get drive statistics"),
    })
}

// Vault API

#[derive(Deserialize)]
#[allow(non_snake_case)]
struct VaultConfigurationJson {
    key: Option<String>,
    oldKey: Option<String>,
    newKey: Option<String>,
}

#[derive(Serialize)]
struct VaultConfigurationCodeResponseJson {
    code: String,
}

#[derive(Serialize)]
struct VaultConfigurationMessageResponseJson {
    message: String,
}

#[post("/api/vaultConfigure")]
/// Configure Vault by providing a password.
/// If no instance of vault has yet been created, it will be. If there already is an instance of
/// vault on the current machine, the password is changed, by decrypting the .vault file in the
/// vault folder with the old password and then encrypting it with the new password. Thus, Zentrox
/// also requires the current password.
async fn vault_configure(
    session: Session,
    state: Data<AppState>,
    json: web::Json<VaultConfigurationJson>,
) -> HttpResponse {
    if !is_admin_state(&session, state) {
        return HttpResponse::Forbidden().body("This resource is blocked.");
    }

    let vault_path = path::Path::new(&dirs::home_dir().unwrap())
        .join(".local")
        .join("share")
        .join("zentrox")
        .join("vault_directory");

    if config_file::read("vault_enabled") == "0" && !vault_path.exists() {
        if json.key.is_none() {
            return HttpResponse::BadRequest().body("This request is malformed");
        }

        let key = &json.key.clone().unwrap();

        match fs::create_dir_all(&vault_path) {
            Ok(_) => {}
            Err(e) => {
                eprintln!("Failed to create vault_directory.\n{}", e);
                return HttpResponse::InternalServerError()
                    .body("Failed to create vault_directory.");
            }
        };

        let vault_file_contents = format!(
            "Vault created by {} at UNIX {}.",
            whoami::username(),
            match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
                Ok(v) => v,
                Err(_) =>
                    return HttpResponse::InternalServerError()
                        .body("System time before UNIX epoch (1/1/1970)"),
            }
            .as_millis()
        );

        match fs::write(vault_path.join(".vault"), vault_file_contents) {
            Ok(_) => {}
            Err(e) => {
                eprintln!("Failed to write vault file.\n{}", e);
                return HttpResponse::InternalServerError().body(e.to_string());
            }
        }

        vault::encrypt_file(vault_path.join(".vault").to_string_lossy().to_string(), key);
        let _ = config_file::write("vault_enabled", "1");
    } else if json.oldKey.is_some() && json.newKey.is_some() {
        let old_key = json.oldKey.clone().unwrap();
        let new_key = json.newKey.clone().unwrap();
        match vault::decrypt_file(
            std::path::Path::new(&dirs::home_dir().unwrap())
                .join(".local")
                .join("share")
                .join("zentrox")
                .join("vault_directory")
                .join(".vault")
                .to_string_lossy()
                .to_string(),
            &old_key.to_string(),
        ) {
            Some(_) => vault::encrypt_file(
                std::path::Path::new(&dirs::home_dir().unwrap())
                    .join(".local")
                    .join("share")
                    .join("zentrox")
                    .join("vault_directory")
                    .join(".vault")
                    .to_string_lossy()
                    .to_string(),
                &old_key.to_string(),
            ),
            None => {
                return HttpResponse::Forbidden().json(VaultConfigurationMessageResponseJson {
                    message: "auth_failed".to_string(),
                })
            }
        };

        match vault::decrypt_directory(
            path::Path::new(&dirs::home_dir().unwrap())
                .join(".local")
                .join("share")
                .join("zentrox")
                .join("vault_directory")
                .to_string_lossy()
                .as_ref(),
            &old_key,
        ) {
            Ok(_) => {}
            Err(_) => {
                return HttpResponse::Forbidden().json(VaultConfigurationMessageResponseJson {
                    message: "auth_failed".to_string(),
                })
            }
        };

        match vault::encrypt_directory(
            path::Path::new(&dirs::home_dir().unwrap())
                .join(".local")
                .join("share")
                .join("zentrox")
                .join("vault_directory")
                .to_string_lossy()
                .as_ref(),
            &new_key,
        ) {
            Ok(_) => {}
            Err(e) => return HttpResponse::InternalServerError().body(e),
        };
    } else {
        return HttpResponse::Ok().json(VaultConfigurationCodeResponseJson {
            code: "no_decrypt_key".to_string(),
        });
    }

    HttpResponse::Ok().json(EmptyJson {})
}

#[get("/api/isVaultConfigured")]
async fn is_vault_configured(session: Session, state: Data<AppState>) -> HttpResponse {
    if !is_admin_state(&session, state) {
        return HttpResponse::Forbidden().body("This resource is blocked.");
    }

    if config_file::read("vault_enabled") == "1" {
        return HttpResponse::Ok().body("1");
    } else {
        return HttpResponse::Ok().body("0");
    }
}

// Vault Tree

#[derive(Serialize)]
struct VaultFsPathJson {
    fs: Vec<String>,
}

#[derive(Deserialize)]
struct VaultKeyRequest {
    key: String,
}

/// List all paths in Zentrox Vault in a one-dimensional vector. This also decrypts the file names
/// and folder names of the entries. A directory path always ends with /. The path never starts
/// with /, thus root is "".
/// Example: abc.txt; def/; def/gh.i; dev/jkl/mno
fn list_paths(directory: String, key: String) -> Vec<String> {
    let read = fs::read_dir(directory).unwrap();
    let mut paths: Vec<String> = Vec::new();

    for entry in read {
        let entry_unwrap = &entry.unwrap();
        let entry_metadata = &entry_unwrap.metadata().unwrap();
        let is_file = entry_metadata.is_file() || entry_metadata.is_symlink();
        let path = &entry_unwrap.path().to_string_lossy().to_string().replace(
            &path::Path::new(&dirs::home_dir().unwrap())
                .join(".local")
                .join("share")
                .join("zentrox")
                .join("vault_directory")
                .to_string_lossy()
                .to_string(),
            "",
        );

        if is_file {
            paths.push(
                path.to_string()
                    .split("/")
                    .filter(|x| !x.is_empty())
                    .map(|x| {
                        if x != ".vault" && !x.is_empty() {
                            match vault::decrypt_string_hash(x.to_string(), &key) {
                                Some(v) => v,
                                None => "Decryption Error".to_string(),
                            }
                        } else {
                            ".vault".to_string()
                        }
                    })
                    .collect::<Vec<String>>()
                    .join("/"),
            ); // Path of the file, while ignoring the path until (but still including) vault_directory.
        } else {
            paths.push(
                path.to_string()
                    .split("/")
                    .filter(|x| !x.is_empty())
                    .map(|x| {
                        if x != ".vault" && !x.is_empty() {
                            match vault::decrypt_string_hash(x.to_string(), &key) {
                                Some(v) => v,
                                None => "Decryption Error".to_string(),
                            }
                        } else {
                            ".vault".to_string()
                        }
                    })
                    .collect::<Vec<String>>()
                    .join("/")
                    .to_string()
                    + "/",
            ); // Path of the file, while ignoring the path until (but still including) vault_directory.
            for e in list_paths(
                entry_unwrap.path().to_string_lossy().to_string(),
                key.clone(),
            ) {
                paths.push(e); // Path of the file, while ignoring the path until (but still including) vault_directory.
            }
        }
    }
    paths
}

#[post("/api/vaultTree")]
/// Calls list_paths with the password provided by the user. This does not work if the password is
/// wrong.
async fn vault_tree(
    session: Session,
    state: Data<AppState>,
    json: web::Json<VaultKeyRequest>,
) -> HttpResponse {
    if !is_admin_state(&session, state) {
        return HttpResponse::Forbidden().body("This resource is blocked.");
    }
    if config_file::read("vault_enabled") == "1" {
        let key = &json.key;

        match vault::decrypt_file(
            std::path::Path::new(&dirs::home_dir().unwrap())
                .join(".local")
                .join("share")
                .join("zentrox")
                .join("vault_directory")
                .join(".vault")
                .to_string_lossy()
                .to_string(),
            &key.to_string(),
        ) {
            Some(_) => vault::encrypt_file(
                std::path::Path::new(&dirs::home_dir().unwrap())
                    .join(".local")
                    .join("share")
                    .join("zentrox")
                    .join("vault_directory")
                    .join(".vault")
                    .to_string_lossy()
                    .to_string(),
                &key.to_string(),
            ),
            None => {
                return HttpResponse::Forbidden().json(VaultConfigurationMessageResponseJson {
                    message: "auth_failed".to_string(),
                })
            }
        };

        let paths = list_paths(
            std::path::Path::new(&dirs::home_dir().unwrap())
                .join(".local")
                .join("share")
                .join("zentrox")
                .join("vault_directory")
                .to_string_lossy()
                .to_string(),
            key.to_string(),
        );

        HttpResponse::Ok().json(VaultFsPathJson { fs: paths })
    } else {
        HttpResponse::Ok().json(VaultConfigurationMessageResponseJson {
            message: "vault_not_configured".to_string(),
        })
    }
}

// Delete vault file
#[derive(Deserialize)]
#[allow(non_snake_case)]
struct VaultDeleteRequest {
    deletePath: String,
    key: String,
}

#[post("/api/deleteVaultFile")]
/// Overwrites a file in vault with random data and then removes it from the users file system.
/// This does not work for the .vault file. This request also does not check for the password
/// provided by the user to be correct.
async fn delete_vault_file(
    session: Session,
    state: Data<AppState>,
    json: web::Json<VaultDeleteRequest>,
) -> HttpResponse {
    if !is_admin_state(&session, state) {
        return HttpResponse::Forbidden().body("This resource is blocked.");
    }

    let sent_path = &json.deletePath;

    if sent_path == ".vault" {
        HttpResponse::BadRequest().finish();
    }

    let path = path::Path::new(&dirs::home_dir().unwrap().to_string_lossy().to_string())
        .join(".local")
        .join("share")
        .join("zentrox")
        .join("vault_directory")
        .join(
            sent_path
                .split("/")
                .filter(|x| !x.is_empty())
                .map(|x| vault::encrypt_string_hash(x.to_string(), &json.key.to_string()).unwrap())
                .collect::<Vec<String>>()
                .join("/"),
        );

    if path.metadata().unwrap().is_file() {
        let file_size = fs::metadata(&path).unwrap().len();
        let mut i = 0;

        while i != 5 {
            let random_data = (0..file_size)
                .map(|_| rand::random::<u8>())
                .collect::<Vec<u8>>();
            let _ = fs::write(&path, random_data);
            i += 1;
        }

        match fs::remove_file(path) {
            Ok(_) => {}
            Err(e) => {
                eprintln!("Failed to remove vault file.\n{}", e);
                return HttpResponse::InternalServerError().finish();
            }
        };
    } else {
        let _ = vault::burn_directory(path.to_string_lossy().to_string());
        match fs::remove_dir_all(path) {
            Ok(_) => {}
            Err(e) => {
                eprintln!("Failed to remove vault directory.\n{}", e);
                return HttpResponse::InternalServerError().finish();
            }
        };
    }

    HttpResponse::Ok().json(EmptyJson {})
}

// Create new folder in vault
#[derive(Deserialize)]
#[allow(non_snake_case)]
struct VaultNewFolderRequest {
    folder_name: String,
    key: String,
}

#[post("/api/vaultNewFolder")]
/// This creates a new folder/directory in the vault directory. The name of the directory is
/// encrypted. This does not check if the password provided by the user is correct. This wont work
/// if the directory name is longer than 64 characters.
async fn vault_new_folder(
    session: Session,
    state: Data<AppState>,
    json: web::Json<VaultNewFolderRequest>,
) -> HttpResponse {
    if !is_admin_state(&session, state) {
        return HttpResponse::Forbidden().body("This resource is blocked.");
    }

    let sent_path = &json.folder_name;

    if sent_path.split("/").last().unwrap().len() > 64 {
        return HttpResponse::InternalServerError().finish();
    }

    let path = path::Path::new(&dirs::home_dir().unwrap().to_string_lossy().to_string())
        .join(".local")
        .join("share")
        .join("zentrox")
        .join("vault_directory")
        .join(
            sent_path
                .split("/")
                .filter(|x| !x.is_empty())
                .map(|x| vault::encrypt_string_hash(x.to_string(), &json.key.to_string()).unwrap())
                .collect::<Vec<String>>()
                .join("/"),
        );

    if path.exists() {
        return HttpResponse::BadRequest().body("This file already exists.");
    }

    let _ = fs::create_dir(&path);
    HttpResponse::Ok().json(EmptyJson {})
}

// Upload vault file

#[derive(Debug, MultipartForm)]
struct VaultUploadForm {
    #[multipart(limit = "10GB")]
    file: TempFile,
    key: Text<String>,
    path: Text<String>,
}

#[post("/upload/vault")]
/// Upload a new file to vault. This requires a multipart form to be the requests body. The file is
/// then extracted and stored in the users file system.
async fn upload_vault(
    session: Session,
    state: Data<AppState>,
    MultipartForm(form): MultipartForm<VaultUploadForm>,
) -> HttpResponse {
    if !is_admin_state(&session, state) {
        return HttpResponse::Forbidden().body("This resource is blocked.");
    }

    let file_name = form
        .file
        .file_name
        .unwrap_or_else(|| "vault_default_file".to_string())
        .replace("..", "")
        .replace("/", "");
    let key = &form.key;

    if file_name == ".vault" {
        return HttpResponse::BadRequest().body("A file can not be named .vault");
    }

    let base_path = path::Path::new(&dirs::home_dir().unwrap())
        .join(".local")
        .join("share")
        .join("zentrox")
        .join("vault_directory");

    let encrypted_path = form
        .path
        .to_string()
        .split('/')
        .filter(|x| !x.is_empty()) // Filter out empty path entries
        .map(|x| vault::encrypt_string_hash(x.to_string(), key).unwrap())
        .collect::<Vec<String>>()
        .join("/");

    let in_vault_path = base_path
        .join(encrypted_path)
        .join(vault::encrypt_string_hash(file_name.to_string(), key).unwrap());

    if in_vault_path.exists() {
        return HttpResponse::BadRequest().body("This file already exists.");
    }

    let tmp_file_path = form.file.file.path().to_owned();
    let _ = fs::copy(&tmp_file_path, &in_vault_path);

    let _ = tokio::fs::copy(&tmp_file_path, &in_vault_path);

    let file_size = fs::metadata(&tmp_file_path).unwrap().len();
    let mut i = 0;

    while i != 5 {
        let random_data = (0..file_size)
            .map(|_| rand::random::<u8>())
            .collect::<Vec<u8>>();
        let _ = fs::write(&tmp_file_path, random_data);
        i += 1;
    }

    let _ = tokio::fs::remove_file(&tmp_file_path).await;

    vault::encrypt_file(in_vault_path.to_string_lossy().to_string(), key);

    HttpResponse::Ok().finish()
}

// Rename file/folder in vault
#[derive(Deserialize)]
#[allow(non_snake_case)]
struct VaultRenameRequest {
    path: String,
    newName: String,
    key: String,
}

#[post("/api/renameVaultFile")]
/// This renames/moves a directory or file in the vault directory.
/// The new name will be encrypted. The original name must be provided in non-encrypted form.
async fn rename_vault_file(
    session: Session,
    state: Data<AppState>,
    json: web::Json<VaultRenameRequest>,
) -> HttpResponse {
    if !is_admin_state(&session, state) {
        return HttpResponse::Forbidden().body("This resource is blocked.");
    }

    let sent_path = &json.path;

    if sent_path == "/.vault" {
        HttpResponse::BadRequest().finish();
    }

    let path = path::Path::new(&dirs::home_dir().unwrap().to_string_lossy().to_string())
        .join(".local")
        .join("share")
        .join("zentrox")
        .join("vault_directory")
        .join(
            sent_path
                .split("/")
                .filter(|x| !x.is_empty())
                .map(|x| vault::encrypt_string_hash(x.to_string(), &json.key.to_string()).unwrap())
                .collect::<Vec<String>>()
                .join("/"),
        );

    let sent_new_path = &json.newName;
    let new_path = path::Path::new(&dirs::home_dir().unwrap().to_string_lossy().to_string())
        .join(".local")
        .join("share")
        .join("zentrox")
        .join("vault_directory")
        .join(
            sent_new_path
                .split("/")
                .filter(|x| !x.is_empty())
                .map(|x| vault::encrypt_string_hash(x.to_string(), &json.key.to_string()).unwrap())
                .collect::<Vec<String>>()
                .join("/"),
        );

    if new_path.exists() {
        return HttpResponse::BadRequest().body("This file already exists.");
    }

    let _ = fs::rename(&path, &new_path);

    if new_path.metadata().unwrap().is_file() {
        let file_size = fs::metadata(&new_path).unwrap().len();
        let mut i = 0;

        while i != 5 {
            let random_data = (0..file_size)
                .map(|_| rand::random::<u8>())
                .collect::<Vec<u8>>();
            let _ = fs::write(&path, random_data);
            i += 1;
        }

        let _ = fs::remove_file(path);
    }

    HttpResponse::Ok().json(EmptyJson {})
}

// Download vault file
#[derive(Deserialize)]
struct VaultFileDownloadJson {
    key: String,
    path: String,
}

#[post("/api/vaultFileDownload")]
/// Download a file from Zentrox Vault.
/// This requires a password from the user in order to decrypt the file server side.
/// The file will not be provided in encrypted form.
async fn vault_file_download(
    session: Session,
    state: Data<AppState>,
    json: web::Json<VaultFileDownloadJson>,
) -> HttpResponse {
    if !is_admin_state(&session, state) {
        return HttpResponse::Forbidden().body("This resource is blocked.");
    }

    let sent_path = &json.path;
    let key = &json.key;

    if sent_path == "/.vault" {
        HttpResponse::BadRequest().finish();
    }

    let path = path::Path::new(&dirs::home_dir().unwrap().to_string_lossy().to_string())
        .join(".local")
        .join("share")
        .join("zentrox")
        .join("vault_directory")
        .join(
            sent_path
                .split("/")
                .filter(|x| !x.is_empty())
                .map(|x| vault::encrypt_string_hash(x.to_string(), &json.key.to_string()).unwrap())
                .collect::<Vec<String>>()
                .join("/"),
        )
        .to_string_lossy()
        .to_string();

    let _ = fs::copy(&path, format!("{}.dec", path).to_string());

    vault::decrypt_file(format!("{}.dec", path).to_string(), key);
    if path::Path::new(&format!("{}.dec", path).to_string()).exists() {
        let f = fs::read(format!("{}.dec", path).to_string());
        match f {
            Ok(fh) => {
                let data = fh.bytes().map(|x| x.unwrap_or(0_u8)).collect::<Vec<u8>>();
                let _ = vault::burn_file(format!("{}.dec", path).to_string());
                let _ = fs::remove_file(format!("{}.dec", path).to_string());
                HttpResponse::Ok().body(data)
            }
            Err(_) => HttpResponse::InternalServerError()
                .body("Failed to read decrypted file".to_string()),
        }
    } else {
        HttpResponse::BadRequest().body("This file does not exist.")
    }
}

// Show Robots.txt
#[get("/robots.txt")]
/// Return the robots.txt file to prevent search engines from indexing this server.
async fn robots_txt() -> HttpResponse {
    HttpResponse::Ok().body(include_str!("../robots.txt"))
}

// Upload tls cert

#[derive(Debug, MultipartForm)]
struct TlsUploadForm {
    #[multipart(limit = "1GB")]
    file: TempFile,
}

#[post("/upload/tls")]
/// Upload and store a new tls certificate used for TLS protection in HTTPS and FTPS.
/// The name of then new certificate is stored in the configuration file.
async fn upload_tls(
    session: Session,
    state: Data<AppState>,
    MultipartForm(form): MultipartForm<TlsUploadForm>,
) -> HttpResponse {
    if !is_admin_state(&session, state) {
        return HttpResponse::Forbidden().body("This resource is blocked.");
    }

    let file_name = form
        .file
        .file_name
        .unwrap_or_else(|| "tls".to_string())
        .replace("..", "")
        .replace("/", "");

    let base_path = path::Path::new(&dirs::home_dir().unwrap())
        .join(".local")
        .join("share")
        .join("zentrox")
        .join(&file_name);

    let _ = config_file::write("tls_cert", &file_name.to_string());

    let tmp_file_path = form.file.file.path().to_owned();
    let _ = fs::copy(&tmp_file_path, &base_path);

    match fs::remove_file(&tmp_file_path) {
        Ok(_) => {}
        Err(e) => {
            eprintln!("Failed to remove temp file.\n{}", e);
            return HttpResponse::InternalServerError().finish();
        }
    };

    HttpResponse::Ok().finish()
}

#[derive(Serialize)]
struct CertNamesJson {
    tls: String,
}

#[get("/api/certNames")]
/// Return the name of the current certificate.
async fn cert_names(session: Session, state: Data<AppState>) -> HttpResponse {
    if !is_admin_state(&session, state) {
        return HttpResponse::Forbidden().body("This resource is blocked.");
    }

    let tls = config_file::read("tls_cert");

    HttpResponse::Ok().json(CertNamesJson { tls })
}

// Power Off System
#[post("/api/powerOff")]
/// Powers off the system.
async fn power_off(
    session: Session,
    state: Data<AppState>,
    json: web::Json<SudoPasswordOnlyRequest>,
) -> HttpResponse {
    if !is_admin_state(&session, state) {
        return HttpResponse::Forbidden().body("This resource is blocked.");
    }

    let _ =
        sudo::SwitchedUserCommand::new(json.sudoPassword.clone(), "poweroff".to_string()).spawn();

    HttpResponse::Ok().finish()
}

// Account Details
#[derive(Serialize)]
struct AccountDetailsJson {
    username: String,
}

#[post("/api/accountDetails")]
/// Return the users account details, which is currently limited to the users username.
async fn account_details(session: Session, state: Data<AppState>) -> HttpResponse {
    if !is_admin_state(&session, state.clone()) {
        return HttpResponse::Forbidden().body("This resource is blocked.");
    }

    let username = match state.username.lock() {
        Ok(v) => v,
        Err(e) => e.into_inner(),
    };

    HttpResponse::Ok().json(AccountDetailsJson {
        username: username.to_string(),
    })
}

#[derive(Deserialize)]
struct UpdateAccountJson {
    password: String,
    username: String,
}

#[post("/api/updateAccountDetails")]
/// Update the users username and password.
async fn update_account_details(
    session: Session,
    state: Data<AppState>,
    json: web::Json<UpdateAccountJson>,
) -> HttpResponse {
    if !is_admin_state(&session, state.clone()) {
        return HttpResponse::Forbidden().body("This resource is blocked.");
    }

    let username = match state.username.lock() {
        Ok(v) => v,
        Err(e) => e.into_inner(),
    };

    let password = &json.password;
    let new_username = &json.username;

    let users_txt_path = path::Path::new(&dirs::home_dir().unwrap())
        .join(".local")
        .join("share")
        .join("zentrox")
        .join("users");
    let users_txt_contents = match fs::read_to_string(&users_txt_path) {
        Ok(v) => v.to_string(),
        Err(err) => {
            eprintln!("Can't read users {err}");
            return HttpResponse::InternalServerError().finish();
        }
    };

    if !password.is_empty() || *new_username != *username {
        let users_lines: Vec<String> = users_txt_contents.lines().map(|x| x.to_string()).collect();
        let mut new_lines: Vec<String> = Vec::new();
        for line in users_lines {
            let dec_username = b64.decode(line.split(": ").next().unwrap());
            if String::from_utf8(dec_username.unwrap()).unwrap() == username.to_string() {
                new_lines.push(
                    [b64.encode(new_username), {
                        if !password.is_empty() {
                            hex::encode(crypto_utils::argon2_derive_key(password).unwrap())
                                .to_string()
                        } else {
                            line.split(": ").nth(1).unwrap().to_string()
                        }
                    }]
                    .join(": "),
                )
            } else {
                new_lines.push(line)
            }
        }

        let _ = fs::write(users_txt_path, new_lines.join("\n"));
    }

    HttpResponse::Ok().finish()
}

#[get("/api/profilePicture")]
/// Return the current profile picture.
async fn profile_picture(session: Session, state: Data<AppState>) -> HttpResponse {
    if !is_admin_state(&session, state) {
        return HttpResponse::Forbidden().body("This resource is blocked.");
    }

    let f = fs::read(
        path::Path::new(&dirs::home_dir().unwrap())
            .join(".local")
            .join("share")
            .join("zentrox")
            .join("profile.png"),
    );

    match f {
        Ok(fh) => {
            HttpResponse::Ok().body(fh.bytes().map(|x| x.unwrap_or(0_u8)).collect::<Vec<u8>>())
        }
        Err(_) => HttpResponse::NotFound().body("Failed to find account picture".to_string()),
    }
}

#[derive(Debug, MultipartForm)]
struct ProfilePictureUploadForm {
    #[multipart(limit = "2MB")]
    file: TempFile,
}

#[post("/api/uploadProfilePicture")]
/// Upload a new profile picture for the users account.
/// The picture may not be larger than 2MB in order to keep loading time down.
async fn upload_profile_picture(
    session: Session,
    state: Data<AppState>,
    MultipartForm(form): MultipartForm<ProfilePictureUploadForm>,
) -> HttpResponse {
    if !is_admin_state(&session, state) {
        return HttpResponse::Forbidden().body("This resource is blocked.");
    }

    let profile_picture_path = path::Path::new(&dirs::home_dir().unwrap())
        .join(".local")
        .join("share")
        .join("zentrox")
        .join("profile.png");

    let tmp_file_path = form.file.file.path().to_owned();
    let _ = fs::copy(&tmp_file_path, &profile_picture_path);

    match fs::remove_file(&tmp_file_path) {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(e) => {
            eprintln!("Failed to remove temp file.\n{}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

#[derive(Serialize)]
struct MessagesLog {
    logs: Vec<(String, String, String, String, String)>,
}

#[post("/api/logs/{log}/{since}/{until}")]
async fn logs_request(
    session: Session,
    state: Data<AppState>,
    json: web::Json<SudoPasswordOnlyRequest>,
    path: web::Path<(String, u64, u64)>,
) -> HttpResponse {
    if !is_admin_state(&session, state) {
        return HttpResponse::Forbidden().body("This resource is blocked.");
    }

    let pii = path.into_inner();
    let log = pii.0;
    let since = pii.1;
    let until = pii.2;

    if log == "messages" {
        let messages = logs::log_messages(json.sudoPassword.clone(), since / 1000, until / 1000);

        match messages {
            Ok(v) => return HttpResponse::Ok().json(MessagesLog { logs: v }),
            Err(e) => {
                eprintln!("Failed to fetch for logs");
                return HttpResponse::InternalServerError()
                    .body(format!("Failed to get message logs {}", e));
            }
        }
    } else {
        return HttpResponse::NotFound().body("The requested logs do not exist");
    }
}

/// Parse a GET RANGE Header Parameter into a Rust byte range
fn parse_range(range: actix_web::http::header::HeaderValue) -> (usize, Option<usize>) {
    let range_str = range.to_str().ok().unwrap(); // Safely convert to str, return None if failed
    let range_separated_clear = range_str.replace("bytes=", "");
    let range_separated: Vec<&str> = range_separated_clear.split('-').collect(); // Split the range

    // Parse the start and end values safely
    let start = range_separated.get(0).unwrap().parse::<usize>().unwrap();

    (start, match range_separated.get(1) {
        Some(v) => {
            if v == &"" { None } else {
            Some(v.parse::<usize>().unwrap())}
        },
        None => None
    })
}

fn is_whitelisted(l: Vec<(bool, PathBuf)>, p: PathBuf) -> bool {
    let mut r = false;
    l.iter().for_each(|le| {
        if !r && p.starts_with(&le.1) && le.0 {
            r = true
        }
    });
    r
}

#[get("/api/getMedia/{path}")]
async fn media_request(
    session: Session,
    state: Data<AppState>,
    path: web::Path<String>,
    req: HttpRequest,
) -> HttpResponse {
    if !is_admin_state(&session, state) {
        return HttpResponse::Forbidden().body("This resource is blocked.");
    }

    if config_file::read("media_enabled") == "0" {
        return HttpResponse::Forbidden().body("Media center has been disabled");
    }

    // Implement HTTP Ranges
    let headers = req.headers();
    let range = headers.get(actix_web::http::header::RANGE);

    // Determine the requested file path
    let pii = path.into_inner();
    let file_path_url = url_decode::url_decode(&pii);
    let file_path = PathBuf::from(&file_path_url);

    let mime = mime::guess_mime(file_path.to_path_buf());

    if file_path.exists() {
        let whitelist = fs::read_to_string(
            home_dir()
                .unwrap()
                .join(".local")
                .join("share")
                .join("zentrox")
                .join("zentrox_media_locations.txt"),
        );
        let whitelist_vector: Vec<(bool, PathBuf)> = whitelist
            .unwrap_or("".to_string())
            .to_string()
            .lines()
            .map(|x| {
                (
                    x.split(";").nth(2).unwrap() == "true",
                    PathBuf::from(x.split(";").nth(0).unwrap().to_string()),
                )
            })
            .collect();

        if !is_whitelisted(
            whitelist_vector,
            fs::canonicalize(file_path.clone()).unwrap(),
        ) {
            return HttpResponse::Forbidden().body("This is not a white-listed location.");
        }

        if file_path.is_dir() {
            return HttpResponse::BadRequest().body("A file can not be a directory.");
        }

        match range {
            None => {
                // Does the file even exist
                HttpResponse::Ok()
                    .insert_header((
                        header::CONTENT_TYPE,
                        mime.unwrap_or("application/octet-stream".to_string()),
                    ))
                    .insert_header(header::ContentEncoding::Identity)
                    .insert_header((header::ACCEPT_RANGES, "bytes"))
                    .body(fs::read(file_path).unwrap())
            }
            Some(e) => {
                let byte_range = parse_range(e.clone());
                let file = File::open(&file_path).unwrap();
                let mut reader = BufReader::new(file);
                let filesize: usize = reader.get_ref().metadata().unwrap().len().try_into().unwrap_or(0);
                if byte_range.0 > filesize {
                    return HttpResponse::RangeNotSatisfiable().body("The requested range can not be satisfied.");
                }
                if byte_range.1.is_some() {
                    if byte_range.1.unwrap() > filesize {
                        return HttpResponse::RangeNotSatisfiable().body("The requested range can not be satisfied.");
                    }
                }
                let buffer_length = byte_range.1.unwrap_or(filesize) - byte_range.0;               
                let _ = reader.seek(SeekFrom::Start(byte_range.0 as u64));
                let mut buf = vec![0; buffer_length]; // A buffer with the length buffer_length
                reader.read_exact(&mut buf).unwrap();

                return HttpResponse::PartialContent()
                    .insert_header(header::ContentEncoding::Identity)
                    .insert_header((header::ACCEPT_RANGES, "bytes"))
                    .insert_header((
                        header::CONTENT_DISPOSITION,
                        format!(
                            "inline; filename=\"{}\"",
                            &file_path.file_name().unwrap().to_str().unwrap()
                        ),
                    ))
                    .insert_header((
                        header::CONTENT_RANGE,
                        format!("bytes {}-{}/{}", byte_range.0, byte_range.1.unwrap_or(filesize - 1), filesize), // We HAVE to subtract 1 from the actual file size
                    ))
                    .insert_header((header::VARY, "*"))
                    .insert_header((header::ACCESS_CONTROL_ALLOW_ORIGIN, "*"))
                    .insert_header((header::ACCESS_CONTROL_ALLOW_METHODS, "GET, HEAD, OPTIONS"))
                    .insert_header((header::ACCESS_CONTROL_ALLOW_HEADERS, "Range"))
                    .insert_header((header::CONTENT_LENGTH, buf.len()))
                    .insert_header((
                        header::CONTENT_TYPE,
                        mime.unwrap_or("application/octet-stream".to_string()),
                    ))
                    .body(buf);
            }
        }
    } else {
        return HttpResponse::NotFound().body("The requested audio file is not on the server.");
    }
}

#[derive(Deserialize)]
struct VideoSourceJson {
    locations: Vec<(PathBuf, String, bool)>,
}

#[post("/api/updateVideoSourceList")]
async fn update_video_source_list(
    session: Session,
    state: Data<AppState>,
    json: web::Json<VideoSourceJson>,
) -> HttpResponse {
    if !is_admin_state(&session, state) {
        return HttpResponse::Forbidden().body("This resource is blocked.");
    }

    let sources_file = Path::new("")
        .join(home_dir().unwrap())
        .join(".local")
        .join("share")
        .join("zentrox")
        .join("zentrox_media_locations.txt");

    let sources_file_swap = Path::new("")
        .join(home_dir().unwrap())
        .join(".local")
        .join("share")
        .join("zentrox")
        .join("zentrox_media_locations_swap.txt");

    let mut sources_list_content = String::new();

    let locations = &json.locations;

    for l in locations {
        let line = format!(
            "{};{};{}\n",
            fs::canonicalize(l.0.to_str().unwrap_or("/this_path_does_not_exist"))
                .unwrap_or("/this_path_does_not_exist".into())
                .to_string_lossy()
                .to_string(),
            l.1.replace(";", "&semi;"),
            l.2
        )
        .to_string();
        sources_list_content = sources_list_content + &line;
    }

    let _ = fs::write(&sources_file_swap, sources_list_content);
    let _ = fs::rename(sources_file_swap, sources_file);

    HttpResponse::Ok().body("")
}

#[derive(Serialize)]
struct VideoSourcesListResponseJson {
    locations: Vec<(String, String, bool)>,
}

#[get("/api/getVideoSourceList")]
async fn get_video_source_list(session: Session, state: Data<AppState>) -> HttpResponse {
    if !is_admin_state(&session, state) {
        return HttpResponse::Forbidden().body("This resource is blocked.");
    }

    let sources_file = Path::new("")
        .join(home_dir().unwrap())
        .join(".local")
        .join("share")
        .join("zentrox")
        .join("zentrox_media_locations.txt");

    let sources_file_read = fs::read_to_string(sources_file);

    let mut sources: Vec<(String, String, bool)> = Vec::new();

    match sources_file_read {
        Ok(v) => {
            let lines = v.lines();

            for l in lines {
                let l_split: Vec<&str> = l.split(";").collect();

                if l_split.len() != 3 {
                    return HttpResponse::InternalServerError().body("Malformed entry");
                }

                let path = String::from(l_split[0]);
                let name = String::from(l_split[1]).replace("&semi;", ";");
                let enabled: bool = l_split[2] == "true";

                sources.push((path, name.into(), enabled));
            }
        }
        Err(e) => {
            eprintln!("Failed to read video sources: {}", e);
            return HttpResponse::InternalServerError().body("Faild to read video sources");
        }
    }

    HttpResponse::Ok().json(VideoSourcesListResponseJson { locations: sources })
}

// 1. Video & Media list and metadata
// 2. G̶e̶n̶r̶e̶ ̶l̶i̶s̶t̶
// 3. M̶a̶k̶e̶ ̶g̶e̶n̶r̶e̶
// 4. Change metadata

#[derive(Serialize)]
struct MediaListResponseJson {
    media: HashMap<PathBuf, (String, String, String, String)>,
}

/// HashMap all media files including name, filename, cover and genre.
/// A metadata file is used to keep track of values configured by the user.
/// If no name is configured in the metadata file, the name is generated automatically.
/// If no cover is configured, a default cover is sent.
#[get("/api/getMediaList")]
async fn get_media_list(session: Session, state: Data<AppState>) -> HttpResponse {
    if !is_admin_state(&session, state) {
        return HttpResponse::Forbidden().body("This resource is blocked.");
    }

    if config_file::read("media_enabled") == "0" {
        return HttpResponse::Forbidden().body("Media center has been disabled");
    }

    let sources_file = Path::new("")
        .join(home_dir().unwrap())
        .join(".local")
        .join("share")
        .join("zentrox")
        .join("zentrox_media_locations.txt");

    let media_directory_vector: Vec<PathBuf> = fs::read_to_string(sources_file)
        .unwrap_or(String::new())
        .lines()
        .filter(|x| !x.is_empty())
        .map(|x| {
            let p = x.split(";").nth(0).unwrap_or("");
            PathBuf::from(p)
        })
        .collect();

    let media_files_vector: Vec<Vec<PathBuf>> = media_directory_vector
        .clone()
        .into_iter()
        .filter(|p| p.exists())
        .map(|p| {
            visit_dirs(p)
                .unwrap()
                .map(|x| x.path())
                .filter(|x| {
                    x.is_file() && x.file_name().expect("Failed to get filename.") != ".mcmetadata"
                })
                // Remove directories and metadata files.
                .collect()
        })
        .collect();

    let mut all_media_files: Vec<PathBuf> = Vec::new();

    for mut paths in media_files_vector {
        all_media_files.append(&mut paths);
    }

    // Create hashmap of all metadata files

    let metadata_files = media_directory_vector
        .into_iter()
        .map(|p| p.join(".mcmetadata"));
    // Assume every directory contains a metadata file. If the file does not exists later on, the
    // code will act as if it is empty "".

    let mut media_info_hashmap: HashMap<PathBuf, (String, String, String, String)> = HashMap::new(); // Make empty hashmap
                                                                                                     // Every media files' path is asigned the information from the metadata files.

    // The metadata file is a file designed the same way as the source directory file.
    // It is a line-separated file where every line corresponds to a media file.
    // The individual segments are as follows:
    // - path: The path in the server filesystem.
    // - name: The corresponding name of the video (e.g., Big Buck Bunny).
    // - path: The path the frontend has to ask for to get the media file.
    // - cover: The filename the frontend has to ask for to get the cover image.
    // - genre: The genre the media file belongs to (e.g., Animation).
    // - artist: The name of the artist
    // These segments are separated using semicolons.

    for file in metadata_files {
        let file_contents = fs::read_to_string(file)
            .unwrap_or("".to_string())
            .to_string();
        let lines = file_contents.lines().filter(|x| !x.is_empty());
        // Get line of file and ignore files that do not exist.
        for l in lines {
            let segments: Vec<&str> = l.split(";").collect();
            if segments.len() != 5 {
                continue;
            }
            let internal_path: PathBuf = segments[0].into(); // The path on the system
            let name = segments[1].to_string(); // The name configured by the user
            let cover = segments[2].to_string(); // The file name of the cover image
            let genre = segments[3].to_string(); // The name of the genre
            let artist = segments[4].to_string(); // The artist/band/... name
            media_info_hashmap.insert(internal_path.clone(), (name, cover, genre, artist));
        }
    }

    // For every file, check if it is in the hashmap, if it isn't add it by guessing a name, adding
    // a blank cover, generating the source path and adding a blank genre.

    for f in all_media_files {
        if !media_info_hashmap.contains_key(&f) {
            let name = f
                .file_name()
                .unwrap()
                .to_string_lossy()
                .to_string()
                .split(".")
                .nth(0)
                .unwrap_or("")
                .to_string()
                .replace("_", " ")
                .replace("-", " ")
                .replace("HD", "")
                .replace("4K", "")
                .to_title_case();
            // Automatically generates a name

            media_info_hashmap.insert(
                f.clone().into(),
                // Using playceholders.
                // The cover can not actually exist in that way, so no user could accidentally
                // create this cover name. The genre is possible, but it will not really do
                // anything except be ignored on the frontend side.
                (
                    name,
                    "UNKNOWN_COVER".to_string(),
                    "UNKNOWN_GENRE".to_string(),
                    "UNKNOWN_ARTIST".to_string(),
                ),
            );
        }
    }

    return HttpResponse::Ok().json(MediaListResponseJson {
        media: media_info_hashmap,
        // The frontend is passed a hashmap with the percise path, name, cover and genre.
    });
}

#[get("/api/cover/{cover_uri}")]
async fn get_cover(
    session: Session,
    state: Data<AppState>,
    path: web::Path<String>,
) -> HttpResponse {
    if !is_admin_state(&session, state) {
        return HttpResponse::Forbidden().body("This resource is blocked.");
    }

    if config_file::read("media_enabled") == "0" {
        return HttpResponse::Forbidden().body("Media center has been disabled");
    }

    let sources_file = Path::new("")
        .join(home_dir().unwrap())
        .join(".local")
        .join("share")
        .join("zentrox")
        .join("zentrox_media_locations.txt");

    let sources_file_contents = fs::read_to_string(sources_file)
        .unwrap_or(String::new())
        .to_string();
    let sources: Vec<String> = sources_file_contents
        .lines()
        .filter(|l| !l.is_empty())
        .map(|l| {
            let mut line_segments = l.split(";");
            line_segments.nth(0).unwrap().to_string()
        })
        .collect();

    let cover_uri = &url_decode::url_decode(&path);

    if cover_uri == "music" {
        let cover = include_str!("../music_default.svg");
        HttpResponse::Ok()
            .insert_header((header::CONTENT_TYPE, "image/svg+xml".to_string()))
            .body(cover.bytes().collect::<Vec<u8>>())
    } else if cover_uri == "video" {
        let cover = include_str!("../video_default.svg");
        HttpResponse::Ok()
            .insert_header((header::CONTENT_TYPE, "image/svg+xml".to_string()))
            .body(cover.bytes().collect::<Vec<u8>>())
    } else if cover_uri == "badtype" {
        let cover = include_str!("../unknown_default.svg");
        HttpResponse::Ok()
            .insert_header((header::CONTENT_TYPE, "image/svg+xml".to_string()))
            .body(cover.bytes().collect::<Vec<u8>>())
    } else {
        let cover_path = PathBuf::from(cover_uri)
            .canonicalize()
            .unwrap_or(PathBuf::from(cover_uri));
        let parent = cover_path.parent();
        if !sources.contains(&parent.unwrap().to_string_lossy().to_string()) {
            HttpResponse::Forbidden().body("This cover is not in a source folder.")
        } else {
            let fh = fs::read(cover_path).unwrap_or("".into());
            HttpResponse::Ok().body(fh.bytes().map(|x| x.unwrap_or(0_u8)).collect::<Vec<u8>>())
        }
    }
}

#[derive(Serialize)]
struct MediaEnabledResponseJson {
    enabled: bool,
}

#[get("/api/getEnableMedia")]
async fn get_enable_media(session: Session, state: Data<AppState>) -> HttpResponse {
    if !is_admin_state(&session, state) {
        return HttpResponse::Forbidden().body("This resource is blocked.");
    }

    return HttpResponse::Ok().json(MediaEnabledResponseJson {
        enabled: config_file::read("media_enabled") == "1",
    });
}

#[get("/api/setEnableMedia/{value}")]
async fn set_enable_media(
    session: Session,
    state: Data<AppState>,
    e: web::Path<bool>,
) -> HttpResponse {
    if !is_admin_state(&session, state) {
        return HttpResponse::Forbidden().body("This resource is blocked.");
    }

    if e.into_inner() {
        let _ = config_file::write("media_enabled", "1");
    } else {
        let _ = config_file::write("media_enabled", "0");
    }

    return HttpResponse::Ok().body("Updated media center status");
}

// ======================================================================
// Blocks (Used to prevent users from accessing certain static resources)

#[get("/dashboard.html")]
async fn dashboard_asset_block(session: Session, state: Data<AppState>) -> HttpResponse {
    if !is_admin_state(&session, state) {
        HttpResponse::Forbidden().body("This resource is blocked.")
    } else {
        HttpResponse::build(StatusCode::OK)
            .body(std::fs::read_to_string("static/dashboard.html").expect("Failed to read file"))
    }
}

// The main function

#[actix_web::main]
/// Prepares Zentrox and starts the server.
async fn main() -> std::io::Result<()> {
    if !env::current_dir().unwrap().join("static").exists() {
        let _ = env::set_current_dir(dirs::home_dir().unwrap().join("zentrox"));
    }

    if !dirs::home_dir()
        .unwrap()
        .join(".local")
        .join("share")
        .join("zentrox")
        .exists()
    {
        let _ = setup::run_setup();
    }

    // Resetting variables to default state
    if let Err(e) = config_file::write("ftp_pid", "") {
        eprintln!("Failed to reset ftp_pid: {}", e);
    }
    if let Err(e) = config_file::write("ftp_running", "0") {
        eprintln!("Failed to reset ftp_running: {}", e);
    }

    let secret_session_key = Key::try_generate().expect("Failed to generate session key");
    let app_state = AppState::new();
    app_state.clone().start_cleanup_task(); // Start periodic cleanup

    if config_file::read("otp_secret").is_empty() && config_file::read("use_otp") == "1" {
        let new_otp_secret = otp::generate_otp_secret();
        if let Err(e) = config_file::write("otp_secret", &new_otp_secret) {
            eprintln!("Failed to write OTP secret: {}", e);
        }
        println!(
            "{}",
            include_str!("../notes/otp_note.txt").replace("SECRET", &new_otp_secret)
        );
    }

    if config_file::read("tls_cert") == "selfsigned.pem" {
        println!(include_str!("../notes/cert_note.txt"));
    }

    rustls::crypto::aws_lc_rs::default_provider()
        .install_default()
        .unwrap();
    let data_path = dirs::home_dir()
        .unwrap()
        .join(".local")
        .join("share")
        .join("zentrox");

    let mut certs_file = BufReader::new(
        File::open(
            data_path
                .join("certificates")
                .join(config_file::read("tls_cert")),
        )
        .unwrap(),
    );
    let mut key_file = BufReader::new(
        File::open(
            data_path
                .join("certificates")
                .join(config_file::read("tls_cert")),
        )
        .unwrap(),
    );

    // load TLS certs and key
    // to create a self-signed temporary cert for testing:
    // `openssl req -x509 -newkey rsa:4096 -nodes -keyout key.pem -out cert.pem -days 365 -subj '/CN=localhost'`
    let tls_certs = rustls_pemfile::certs(&mut certs_file)
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    let tls_key = rustls_pemfile::pkcs8_private_keys(&mut key_file)
        .next()
        .unwrap()
        .unwrap();

    // set up TLS config options
    let tls_config = rustls::ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(tls_certs, rustls::pki_types::PrivateKeyDer::Pkcs8(tls_key))
        .unwrap();

    println!("🚀 Serving Zentrox on Port 8080");

    HttpServer::new(move || {
        let cors_permissive: bool = true; // Enable or disable strict cors for developement. Leaving this
                               // enabled poses a grave security vulnerability and shows a
                               // disclaimer.
        if cors_permissive {
            print!(include_str!("../notes/cors_note.txt"))
        }

        App::new()
            .app_data(Data::new(app_state.clone()))
            .wrap(
                SessionMiddleware::builder(
                    CookieSessionStore::default(),
                    secret_session_key.clone(),
                )
                .session_lifecycle(
                    actix_session::config::PersistentSession::default()
                        .session_ttl(actix_web::cookie::time::Duration::seconds(24 * 60 * 60)),
                )
                .cookie_secure(true)
                .cookie_name("session".to_string())
                .build(),
            )
            .wrap(if cors_permissive {
                Cors::permissive()
            } else {
                Cors::default()
            })
            .wrap(middleware::Compress::default())
            // Landing
            .service(dashboard)
            .service(index)
            .service(alerts)
            .service(alerts_manifest)
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
            // API Packages
            .service(package_database)
            .service(package_database_autoremove)
            .service(install_package)
            .service(remove_package)
            .service(clear_auto_remove)
            // API Firewall
            .service(firewall_information)
            .service(switch_ufw)
            .service(new_firewall_rule)
            .service(delete_firewall_rule)
            // API Files
            .service(call_file)
            .service(files_list)
            .service(delete_file)
            .service(rename_file)
            .service(burn_file)
            // Block Device API
            .service(list_drives)
            .service(drive_information)
            // Vault API
            .service(vault_configure)
            .service(is_vault_configured)
            .service(vault_tree)
            .service(vault_new_folder)
            .service(upload_vault)
            .service(delete_vault_file)
            .service(rename_vault_file)
            .service(vault_file_download)
            // Power Off System
            .service(power_off)
            // Certificates
            .service(upload_tls)
            .service(cert_names)
            // Account Details
            .service(account_details)
            .service(update_account_details)
            .service(profile_picture)
            .service(upload_profile_picture)
            // Logs
            .service(logs_request)
            // Video
            .service(media)
            .service(media_request)
            .service(get_enable_media)
            .service(set_enable_media)
            .service(get_video_source_list)
            .service(update_video_source_list)
            .service(get_media_list)
            .service(get_cover)
            // General services and blocks
            .service(dashboard_asset_block)
            .service(robots_txt)
            .service(afs::Files::new("/", "static/"))
    })
    .bind_rustls_0_23(("0.0.0.0", 8080), tls_config)?
    .run()
    .await
}

// Thank you for reading through all of this 😄
