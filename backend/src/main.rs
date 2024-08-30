extern crate systemstat;
use actix_files as afs;
use actix_session::{storage::CookieSessionStore, Session, SessionMiddleware};
use actix_web::cookie::Key;
use actix_web::{get, http::StatusCode, middleware, post, web, App, HttpResponse, HttpServer};
use base64::{engine::general_purpose::STANDARD as b64, Engine as _};
use hmac_sha512::Hash;
use serde::{Deserialize, Serialize};
mod is_admin;
use is_admin::is_admin_state;
mod config_file;
mod otp;
mod sudo;
use actix_cors::Cors;
use std::{
    collections::HashMap,
    fs, path,
    process::Command,
    sync::{Arc, Mutex},
    time::{SystemTime, UNIX_EPOCH},
};
use sysinfo::System as SysInfoSystem;
use systemstat::{Platform, System};
mod packages;
mod ufw;
mod url_decode;
use std::io::Read;

#[allow(non_snake_case)]
// General App Code
#[derive(Clone)]
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
}

impl AppState {
    fn new() -> Self {
        AppState {
            login_requests: Arc::new(Mutex::new(HashMap::new())),
            login_token: Arc::new(Mutex::new(String::from(""))),
        }
    }
}

#[get("/")]
async fn index(session: Session, state: web::Data<AppState>) -> HttpResponse {
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

#[get("/dashboard")]
async fn dashboard(session: Session, state: web::Data<AppState>) -> HttpResponse {
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
#[derive(Serialize)]
struct EmptyJson {}

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
        let line_username_entry = line.split(": ").next().expect("Failed to get username");
        let line_username = String::from_utf8(b64.decode(line_username_entry).unwrap())
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
                        let login_token: Vec<u8> = (0..4).map(|_| rand::random::<u8>()).collect();
                        let _ =
                            session.insert("login_token", hex::encode(&login_token).to_string());

                        *state.login_token.lock().unwrap() = hex::encode(&login_token).to_string();

                        return HttpResponse::build(StatusCode::OK).json(web::Json(EmptyJson {}));
                    } else {
                        println!("❌ Wrong OTP while authenticating {}", &username);
                    }
                } else {
                    let login_token: Vec<u8> = (0..16).map(|_| rand::random::<u8>()).collect();
                    // for hashes
                    let _ = session.insert("login_token", hex::encode(&login_token).to_string());

                    *state.login_token.lock().unwrap() = hex::encode(&login_token).to_string();

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
async fn logout(session: Session, _state: web::Data<AppState>) -> HttpResponse {
    session.remove("login_token");
    let _ = config_file::write("login_token", "");
    session.remove("zentrox_admin_password");
    HttpResponse::Found()
        .append_header(("Location", "/"))
        .finish()
}

// Ask for Otp Secret
#[get("/login/otpSecret")]
async fn otp_secret_request(_state: web::Data<AppState>) -> HttpResponse {
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
        HttpResponse::Forbidden().finish()
    }
}

#[get("/login/useOtp")]
async fn use_otp(_state: web::Data<AppState>) -> HttpResponse {
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
async fn cpu_percent(session: Session, state: web::Data<AppState>) -> HttpResponse {
    if !is_admin_state(&session, state) {
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
async fn ram_percent(session: Session, state: web::Data<AppState>) -> HttpResponse {
    if !is_admin_state(&session, state) {
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
async fn disk_percent(session: Session, state: web::Data<AppState>) -> HttpResponse {
    if !is_admin_state(&session, state) {
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
async fn device_information(session: Session, state: web::Data<AppState>) -> HttpResponse {
    if !is_admin_state(&session, state) {
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

// FTP API
#[get("/api/fetchFTPconfig")]
async fn fetch_ftp_config(session: Session, state: web::Data<AppState>) -> HttpResponse {
    if !is_admin_state(&session, state) {
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
    sudoPassword: String,
}

#[post("/api/updateFTPConfig")]
async fn update_ftp_config(
    session: Session,
    json: web::Json<JsonRequest>,
    state: web::Data<AppState>,
) -> HttpResponse {
    if !is_admin_state(&session, state) {
        return HttpResponse::Forbidden().finish();
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

        let _ = config_file::write("ftp_running", "1");
    }

    if !json.enableDisable.unwrap_or(false) {
        let username = json.ftpUserUsername.clone().unwrap_or(String::from(""));
        let password = json.ftpUserPassword.clone().unwrap_or(String::from(""));
        let local_root = json.ftpLocalRoot.clone().unwrap_or(String::from(""));

        if !password.is_empty() {
            let hasher = &mut Hash::new();
            hasher.update(&password);
            let hashed_password = hex::encode(hasher.finalize());
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

#[get("/api/packageDatabase")]
async fn package_database(session: Session, state: web::Data<AppState>) -> HttpResponse {
    if !is_admin_state(&session, state) {
        return HttpResponse::Forbidden().finish();
    }

    let installed = match packages::list_installed_packages() {
        Ok(packages) => packages,
        Err(err) => {
            eprintln!("❌ Listing installed packages failed: {}", err);
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

#[get("/api/packageDatabaseAutoremove")]
async fn package_database_autoremove(session: Session, state: web::Data<AppState>) -> HttpResponse {
    if !is_admin_state(&session, state) {
        return HttpResponse::Forbidden().finish();
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
async fn install_package(
    session: Session,
    json: web::Json<PackageActionRequest>,
    state: web::Data<AppState>,
) -> HttpResponse {
    if !is_admin_state(&session, state) {
        return HttpResponse::Forbidden().finish();
    }

    let package_mame = &json.packageName;
    let sudo_password = &json.sudoPassword;

    match packages::install_package(package_mame.to_string(), sudo_password.to_string()) {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[post("/api/removePackage")]
async fn remove_package(
    session: Session,
    json: web::Json<PackageActionRequest>,
    state: web::Data<AppState>,
) -> HttpResponse {
    if !is_admin_state(&session, state) {
        return HttpResponse::Forbidden().finish();
    }

    let package_mame = &json.packageName;
    let sudo_password = &json.sudoPassword;

    match packages::remove_package(package_mame.to_string(), sudo_password.to_string()) {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[post("/api/clearAutoRemove")]
async fn clear_auto_remove(
    session: Session,
    json: web::Json<SudoPasswordOnlyRequest>,
    state: web::Data<AppState>,
) -> HttpResponse {
    if !is_admin_state(&session, state) {
        return HttpResponse::Forbidden().finish();
    }

    let sudo_password = &json.sudoPassword;

    match packages::auto_remove(sudo_password.to_string()) {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

// Firewall API

#[derive(Serialize)]
struct FireWallInformationResponseJson {
    enabled: bool,
    rules: Vec<ufw::UfwRule>,
}

#[post("/api/fireWallInformation")]
async fn firewall_information(
    session: Session,
    state: web::Data<AppState>,
    json: web::Json<SudoPasswordOnlyRequest>,
) -> HttpResponse {
    if !is_admin_state(&session, state) {
        return HttpResponse::Forbidden().finish();
    }

    let password = &json.sudoPassword;

    let ufw_stauts = ufw::ufw_status(password.to_string());

    let enabled = ufw_stauts.0;
    let rules = ufw_stauts.1;

    HttpResponse::Ok().json(FireWallInformationResponseJson { enabled, rules })
}

#[post("/api/switchUfw/{value}")]
async fn switch_ufw(
    session: Session,
    state: web::Data<AppState>,
    path: web::Path<String>,
    json: web::Json<SudoPasswordOnlyRequest>,
) -> HttpResponse {
    if !is_admin_state(&session, state) {
        return HttpResponse::Forbidden().finish();
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
                    println!("❌ Failed to start UFW (Status != 0)");
                    return HttpResponse::InternalServerError().finish();
                }
            }
            Err(_) => {
                println!("❌ Failed to start UFW (Err)");
                return HttpResponse::InternalServerError().finish();
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
                    println!("❌ Failed to stop UFW (Status != 0)");
                    return HttpResponse::InternalServerError().finish();
                }
            }
            Err(_) => {
                println!("❌ Failed to stop UFW (Err)");
                return HttpResponse::InternalServerError().finish();
            }
        }
    }

    HttpResponse::Ok().finish()
}

#[post("/api/newFireWallRule/{from}/{to}/{action}")]
async fn new_firewall_rule(
    session: Session,
    state: web::Data<AppState>,
    json: web::Json<SudoPasswordOnlyRequest>,
    path: web::Path<(String, String, String)>,
) -> HttpResponse {
    if !is_admin_state(&session, state) {
        return HttpResponse::Forbidden().finish();
    }

    let password = &json.sudoPassword;
    let (mut from, mut to, action) = path.into_inner();

    if action.is_empty() {
        println!("❌ User provided insufficent firewall rule settings");
        return HttpResponse::BadRequest().finish();
    }

    if from.is_empty() {
        from = "any".to_string()
    }

    if to.is_empty() {
        to = "any".to_string()
    }

    match ufw::new_rule(String::from(password), from, to, action) {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[post("/api/deleteFireWallRule/{index}")]
async fn delete_firewall_rule(
    session: Session,
    state: web::Data<AppState>,
    json: web::Json<SudoPasswordOnlyRequest>,
    path: web::Path<i32>,
) -> HttpResponse {
    if !is_admin_state(&session, state) {
        return HttpResponse::Forbidden().finish();
    }

    let i = path.into_inner();
    let password = &json.sudoPassword;

    match ufw::delete_rule(password.to_string(), i as u32) {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

// File API
#[get("/api/callFile/{file_name}")]
async fn call_file(
    session: Session,
    state: web::Data<AppState>,
    path: web::Path<String>,
) -> HttpResponse {
    if !is_admin_state(&session, state) {
        return HttpResponse::Forbidden().finish();
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
async fn files_list(
    session: Session,
    state: web::Data<AppState>,
    path: web::Path<String>,
) -> HttpResponse {
    if !is_admin_state(&session, state) {
        return HttpResponse::Forbidden().finish();
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
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[get("/api/deleteFile/{path}")]
async fn delete_file(
    session: Session,
    state: web::Data<AppState>,
    path: web::Path<String>,
) -> HttpResponse {
    if !is_admin_state(&session, state) {
        return HttpResponse::Forbidden().finish();
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
                Err(_) => HttpResponse::InternalServerError().finish(),
            }
        } else if is_dir && has_permissions {
            match fs::remove_dir_all(&file_path) {
                Ok(_) => HttpResponse::Ok().finish(),
                Err(_) => HttpResponse::InternalServerError().finish(),
            }
        } else {
            HttpResponse::Forbidden().body("Missing file permissions. File is readonly.")
        }
    } else {
        HttpResponse::BadRequest().body("This path does not exist")
    }
}

#[get("/api/renameFile/{old_path}/{new_path}")]
async fn rename_file(
    session: Session,
    state: web::Data<AppState>,
    path: web::Path<(String, String)>,
) -> HttpResponse {
    if !is_admin_state(&session, state) {
        return HttpResponse::Forbidden().finish();
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
async fn burn_file(
    session: Session,
    state: web::Data<AppState>,
    path: web::Path<String>,
) -> HttpResponse {
    if !is_admin_state(&session, state) {
        return HttpResponse::Forbidden().finish();
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

// ======================================================================
// Blocks (Used to prevent users from accessing certain static resources)

#[get("/dashboard.html")]
async fn dashboard_asset_block(session: Session, state: web::Data<AppState>) -> HttpResponse {
    if !is_admin_state(&session, state) {
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
    let _ = config_file::write("ftp_pid", "");
    let _ = config_file::write("ftp_running", "0");

    let secret_session_key = Key::try_generate().unwrap();
    let app_state = web::Data::new(AppState::new());

    if config_file::read("otp_secret").is_empty() && config_file::read("use_otp") == "1" {
        let new_otp_secret = otp::generate_otp_secret();
        let _ = config_file::write("otp_secret", &new_otp_secret);
        println!(
            "🔒 Your One-Time-Pad Secret is: {}
            🔒 Store this in a secure location and add it to your 2FA app.
            🔒 If you lose this key, you will need physical access to this device.
            🔒 From there, you can find it in ~/zentrox_data/config.toml",
            new_otp_secret
        )
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
                .cookie_name("session".to_string())
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
            // General services and blocks
            .service(dashboard_asset_block)
            .service(afs::Files::new("/", "static/"))
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
