use actix_cors::Cors;
use actix_files as afs;
use actix_multipart::form::MultipartFormConfig;
use actix_multipart::form::{tempfile::TempFile, text::Text, MultipartForm};
use actix_session::config::CookieContentSecurity;
use actix_session::{storage::CookieSessionStore, Session, SessionMiddleware};
use actix_web::cookie::Key;
use actix_web::web::Path;
use actix_web::HttpRequest;
use actix_web::{
    get, http::header, http::StatusCode, middleware, post, web, web::Data, App, HttpResponse,
    HttpServer,
};
use core::panic;
use futures::FutureExt;
use serde::{Deserialize, Serialize};
use std::ffi::OsString;
use std::net::IpAddr;
use std::ops::Div;
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::process::{Command, Stdio};
use std::str::FromStr;
use std::time::Duration;
use std::{
    collections::HashMap,
    env,
    fs::{self, File},
    io::{BufReader, Read, Seek, SeekFrom},
    path::{self, PathBuf},
    sync::{Arc, Mutex},
    time::UNIX_EPOCH,
};
use sysinfo::{Components, MemoryRefreshKind, Pid, ProcessRefreshKind, RefreshKind, UpdateKind};
use uuid::Uuid;
extern crate inflector;
use actix_governor::{self, Governor, GovernorConfigBuilder};
use database::InsertValue;
use inflector::Inflector;
use log::{debug, error, warn};

mod cron;
mod crypto_utils;
mod database;
mod drives;
mod is_admin;
mod logs;
mod mime;
mod net_data;
mod otp;
mod packages;
mod setup;
mod sudo;
mod ufw;
mod uptime;
mod url_decode;
mod users;
mod vault;
mod video;
mod visit_dirs;

use is_admin::is_admin_state;
use net_data::private_ip;
use users::convert_uid_to_name;
use visit_dirs::visit_dirs;

use self::cron::{
    delete_interval_cronjob, delete_specific_cronjob, IntervalCronJob, SpecificCronJob, User,
};
use self::net_data::{DeletionRoute, Destination, IpAddrWithSubnet, OperationalState, Route};

#[derive(Debug, Clone)]
#[allow(unused)]
enum BackgroundTaskState {
    Success,
    Fail,
    SuccessOutput(String),
    FailOutput(String),
    Pending,
}

#[derive(Debug, Clone, Serialize)]
#[allow(unused)]
struct MeasuredInterface {
    pub ifindex: i64,
    pub ifname: String,
    pub flags: Vec<String>,
    pub mtu: i64,
    pub qdisc: String,
    pub operstate: OperationalState,
    pub linkmode: String,
    pub group: String,
    pub txqlen: Option<i64>,
    pub link_type: String,
    pub address: String,
    pub broadcast: String,
    pub up: f64,
    pub down: f64,
    pub altnames: Option<Vec<String>>,
}

#[allow(non_snake_case)]
#[derive(Clone)]
/// Current state of the application used to keep track of the logged in users, DoS/Brute force
/// attack requests and sharing a instance of the System struct.
struct AppState {
    login_token: Arc<Mutex<String>>,
    system: Arc<Mutex<sysinfo::System>>,
    username: Arc<Mutex<String>>,
    cpu_usage: Arc<Mutex<f32>>,
    network_interfaces: Arc<Mutex<Vec<MeasuredInterface>>>,
    background_jobs: Arc<Mutex<HashMap<Uuid, BackgroundTaskState>>>,
}

impl AppState {
    /// Initiate a new AppState
    fn new() -> Self {
        let random_string: Arc<[u8]> = (0..128).map(|_| rand::random::<u8>()).collect();
        AppState {
            login_token: Arc::new(Mutex::new(
                String::from_utf8_lossy(&random_string).to_string(),
            )),
            system: Arc::new(Mutex::new(sysinfo::System::new())),
            username: Arc::new(Mutex::new(String::new())),
            network_interfaces: Arc::new(Mutex::new(Vec::new())),
            cpu_usage: Arc::new(Mutex::new(0_f32)),
            background_jobs: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    fn update_network_statistics(&self) {
        if (*self.username.lock().unwrap()).to_string().is_empty() {
            return;
        }
        let devices_a = net_data::get_network_interfaces().unwrap();
        std::thread::sleep(std::time::Duration::from_millis(1000));
        let devices_b = net_data::get_network_interfaces().unwrap();
        let devices_b_hashmap: HashMap<String, &net_data::Interface> =
            devices_b.iter().map(|d| (d.ifname.clone(), d)).collect();
        let mut result: Vec<MeasuredInterface> = Vec::new();
        for device in devices_a {
            match devices_b_hashmap.get(&device.ifname) {
                Some(v) => {
                    let a_up = device.stats64.get("tx").unwrap().bytes;
                    let a_down = device.stats64.get("rx").unwrap().bytes;
                    let b_up = v.stats64.get("tx").unwrap().bytes;
                    let b_down = v.stats64.get("rx").unwrap().bytes;
                    result.push(MeasuredInterface {
                        ifname: device.ifname,
                        ifindex: device.ifindex,
                        flags: device.flags,
                        mtu: device.mtu,
                        qdisc: device.qdisc,
                        operstate: device.operstate,
                        linkmode: device.linkmode,
                        address: device.address,
                        altnames: device.altnames,
                        broadcast: device.broadcast,
                        down: (b_down - a_down) / 5_f64,
                        up: (b_up - a_up) / 5_f64,
                        group: device.group,
                        link_type: device.link_type,
                        txqlen: device.txqlen,
                    });
                }
                None => {}
            }
        }
        *self.network_interfaces.lock().unwrap() = result;
    }

    /// Update CPU statistics
    fn update_cpu_statistics(&self) {
        if (*self.username.lock().unwrap()).to_string().is_empty() {
            return;
        }
        self.system.lock().unwrap().refresh_cpu_usage();
        std::thread::sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL);
        self.system.lock().unwrap().refresh_cpu_usage();
        *self.cpu_usage.lock().unwrap() = self.system.lock().unwrap().global_cpu_usage();
    }

    fn start_interval_tasks(self) {
        let network_clone = self.clone();
        let cpu_clone = self.clone();
        std::thread::spawn(move || loop {
            std::thread::sleep(std::time::Duration::from_millis(5 * 1000));
            network_clone.update_network_statistics();
        });
        std::thread::spawn(move || loop {
            std::thread::sleep(std::time::Duration::from_millis(5 * 1000));
            cpu_clone.update_cpu_statistics();
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
            .body("You will soon be redirected")
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
            .body("You will soon be redirected")
    }
}

#[get("/media")]
async fn media(session: Session, state: Data<AppState>) -> HttpResponse {
    // is_admin session value is != true (None or false), the user is served the media screen
    // otherwise, the user is redirected to /
    if is_admin_state(&session, state) {
        if database::read_kv("Settings", "media_enabled").unwrap() == database::ST_BOOL_FALSE {
            return HttpResponse::Forbidden().body("Media center has been disabled");
        }
        HttpResponse::build(StatusCode::OK)
            .body(std::fs::read_to_string("static/media.html").expect("Failed to read alerts page"))
    } else {
        HttpResponse::Found()
            .append_header(("Location", "/"))
            .body("You will soon be redirected")
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
            .body("You will soon be redirected")
    }
}

// API (Actuall API calls)

/// Empty Json Response
///
/// This struct implements serde::Serialize. It can be used to respond with an empty Json
/// response.
#[derive(Serialize)]
struct EmptyJson {}

/// Error JSON
///
/// This struct implements serder::Serialize. It can be used to respond with an error message.
#[derive(Serialize)]
struct ErrorJson {
    error: String,
}

/// Request that only contains a sudo password from the backend.
///
/// This struct implements serde::Derserialize. It can be used to parse a single sudoPassword from
/// the user. It only has the String filed sudoPassword.
#[derive(Deserialize, Debug)]
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
async fn login(session: Session, json: web::Json<Login>, state: Data<AppState>) -> HttpResponse {
    let username = &json.username;
    let password = &json.password;
    let otp_code = &json.userOtp;

    if &database::read_cols::<&str, (String,)>("Admin", &["username"]).unwrap()[0].0 == username {
        let stored_password = database::read_kv("Secrets", "admin_password").unwrap();
        let hashes_correct =
            is_admin::password_hash(password.to_string(), stored_password.to_string());
        if hashes_correct {
            let login_token: Vec<u8> = is_admin::generate_random_token();
            if database::read_cols::<&str, (bool,)>("Admin", &["use_otp"]).unwrap()[0].0 {
                let otp_secret = database::read_kv("Secrets", "otp_secret").unwrap();
                if otp::calculate_current_otp(&otp_secret) == *otp_code {
                    // User has logged in successfully using password and 2FA
                    let _ = session.insert("login_token", hex::encode(&login_token).to_string());

                    *state.login_token.lock().unwrap() = hex::encode(&login_token).to_string();
                    *state.username.lock().unwrap() = username.to_string();
                    let state_copy = state.clone();
                    std::thread::spawn(move || {
                        state_copy.update_network_statistics();
                        state_copy.update_cpu_statistics();
                    });

                    return HttpResponse::build(StatusCode::OK).json(web::Json(EmptyJson {}));
                }
            } else {
                // User has logged in successfully using password
                let _ = session.insert("login_token", hex::encode(&login_token).to_string());

                *state.login_token.lock().unwrap() = hex::encode(&login_token).to_string();
                *state.username.lock().unwrap() = username.to_string();

                let state_copy = state.clone();
                std::thread::spawn(move || {
                    state_copy.update_network_statistics();
                    state_copy.update_cpu_statistics();
                });

                return HttpResponse::build(StatusCode::OK).json(web::Json(EmptyJson {}));
            }
        } else {
            return HttpResponse::build(StatusCode::FORBIDDEN).body("Missing permissions");
        }
    } else {
        return HttpResponse::build(StatusCode::FORBIDDEN).body("Missing permissions");
    }
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

        *state.username.lock().unwrap() = "".to_string();
        *state.login_token.lock().unwrap() =
            hex::encode((0..64).map(|_| rand::random::<u8>()).collect::<Vec<u8>>()).to_string();
        HttpResponse::Found()
            .append_header(("Location", "/"))
            .body("You will soon be redirected")
    } else {
        HttpResponse::BadRequest().body("You are not logged in.")
    }
}

/// Retrieve OTP secret on first login.
///
/// This function will only return the users OTP secret when the web page is viewed for the first
/// time. To keep track of this status, a key knows_otp_secret is used.
#[get("/api/otpSecret")]
async fn otp_secret_request(state: Data<AppState>, session: Session) -> HttpResponse {
    #[derive(Serialize)]
    struct SecretJsonResponse {
        secret: String,
    }

    if (!database::read_cols::<&str, (bool,)>("Admin", &["knows_otp"]).unwrap()[0].0
        && database::read_cols::<&str, (bool,)>("Admin", &["use_otp"]).unwrap()[0].0)
        || is_admin_state(&session, state.clone())
    {
        let _u = database::update_where(
            "Admin",
            &["knows_otp"],
            &[database::InsertValue::Bool(true)],
            "key",
            "0",
        );
        HttpResponse::build(StatusCode::OK).json(SecretJsonResponse {
            secret: database::read_kv("Secrets", "otp_secret")
                .unwrap_or("Secret not found".to_string()),
        })
    } else {
        HttpResponse::Forbidden().body("You can not access this value anymore.")
    }
}

#[get("/api/updateOtp/{status}")]
async fn update_otp_status(
    state: Data<AppState>,
    session: Session,
    path: Path<bool>,
) -> HttpResponse {
    if !is_admin_state(&session, state.clone()) {
        return HttpResponse::Forbidden().body("This resource is blocked.");
    };

    if path.into_inner() {
        let _a = database::write(
            "Admin",
            &["use_otp"],
            &[InsertValue::Bool(true)],
            "key",
            "0",
        );
        let secret = otp::generate_otp_secret();
        let _b = database::write_kv("Secrets", "otp_secret", InsertValue::Text(secret.clone()));
        let _ = database::write(
            "Admin",
            &["knows_otp"],
            &[InsertValue::Bool(true)],
            "key",
            "0",
        );
        return HttpResponse::Ok().body(secret);
    } else {
        let _a = database::write(
            "Admin",
            &["use_otp"],
            &[InsertValue::Bool(false)],
            "key",
            "0",
        );
        let _b = database::delete_row("Secrets", "name", "otp_secret");
        let _ = database::write(
            "Admin",
            &["knows_otp"],
            &[InsertValue::Bool(false)],
            "key",
            "0",
        );
        return HttpResponse::Ok().body("OTP secret has been updated");
    }
}

/// Check if the users uses OTP.
///
/// This function returns a boolean depending on the user using OTP or not.
#[post("/api/useOtp")]
async fn use_otp(_state: Data<AppState>) -> HttpResponse {
    #[derive(Serialize)]
    struct JsonResponse {
        used: bool,
    }

    HttpResponse::Ok().json(JsonResponse {
        used: database::read_cols::<&str, (bool,)>("Admin", &["use_otp"]).unwrap()[0].0,
    })
}

/// Verifies a given sudo password
#[post("/api/verifySudoPassword")]
async fn verify_sudo_password(
    session: Session,
    state: Data<AppState>,
    json: web::Json<SudoPasswordOnlyRequest>,
) -> HttpResponse {
    if !is_admin_state(&session, state.clone()) {
        return HttpResponse::Forbidden().body("This resource is blocked.");
    }

    if !sudo::verify_password(json.sudoPassword.clone()) {
        return HttpResponse::BadRequest().body("Wrong sudo password");
    }

    return HttpResponse::Ok().body("Sudo password has been verified");
}

// Functional Requests

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
    if !is_admin_state(&session, state.clone()) {
        return HttpResponse::Forbidden().body("This resource is blocked.");
    }

    #[derive(Serialize)]
    struct JsonResponse {
        hostname: String,
        ip: String,
        uptime: u128,
        temperature: f32,
        zentrox_pid: u16,
        net_up: f64,
        net_down: f64,
        net_interface: String,
        net_connected_interfaces: i32,
        memory_total: u64,
        memory_free: u64,
        cpu_usage: f32,
        os_name: String,
    }

    // Current machines hostname. i.e.: debian_pc or 192.168.1.3
    let hostname = fs::read_to_string("/etc/hostname")
        .unwrap_or("Unknown hostname".to_string())
        .replace("\n", "")
        .to_string();

    let uptime = uptime::get().unwrap().as_millis();

    let mut temperature = -300_f32;
    let c = Components::new_with_refreshed_list();
    for comp in &c {
        temperature = comp.temperature().unwrap_or(-300_f32);
    }

    state.system.lock().unwrap().refresh_memory();
    state.system.lock().unwrap().refresh_cpu_usage();
    let cpu_usage = state.system.lock().unwrap().global_cpu_usage();
    let memory_total: u64 = state.system.lock().unwrap().total_memory();
    let memory_free: u64 = state.system.lock().unwrap().available_memory();

    let mut net_down = 0.0;
    let mut net_up = 0.0;
    let mut net_interface = None;
    let mut net_interface_name = "MISSING_INTERFACE".to_string();
    let mut interfaces_i = 0;
    let interfaces = state.network_interfaces.lock().unwrap();
    let interfaces_count = interfaces.iter().len();
    while interfaces_i != interfaces_count {
        if interfaces[interfaces_i].up != 0.0 && interfaces[interfaces_i].down != 0.0 {
            net_interface = Some(interfaces[interfaces_i].clone());
        }
        interfaces_i += 1;
    }
    if net_interface.is_some() {
        let u = net_interface.unwrap();
        net_interface_name = u.ifname;
        net_down = u.down;
        net_up = u.up;
    } else if interfaces.len() > 0 {
        let u = interfaces.iter().nth(0).unwrap().clone();
        net_interface_name = u.ifname;
        net_down = u.down;
        net_up = u.up;
    }

    let os_release = fs::read_to_string("/etc/os-release");
    let mut os_name = String::new();
    match os_release {
        Ok(s) => {
            s.lines().for_each(|l| {
                if l.starts_with("PRETTY_NAME") {
                    os_name = l.split("=").nth(1).unwrap_or("").replace("\"", "");
                }
            });
        }
        Err(_) => {}
    }

    HttpResponse::Ok().json(JsonResponse {
        zentrox_pid: std::process::id() as u16,
        hostname,
        uptime,
        temperature,
        net_down,
        net_up,
        net_interface: net_interface_name,
        net_connected_interfaces: interfaces_count as i32,
        ip: match private_ip() {
            Ok(v) => v.to_string(),
            Err(_) => "No route".to_string(),
        },
        memory_free,
        memory_total,
        cpu_usage,
        os_name,
    })
}

// Package API

#[derive(Serialize)]
#[allow(non_snake_case)]
struct PackageResponseJson {
    packages: Vec<String>, // Any package the supported package managers (apt, pacman and dnf) say
    // would be installed on the system (names only)
    others: Vec<String>, // Not installed and not a .desktop file
    packageManager: String,
    canProvideUpdates: bool,
    updates: Vec<String>,
    lastDatabaseUpdate: u64,
}

#[derive(Serialize)]
#[allow(non_snake_case)]
struct PackageResponseJsonCounts {
    packages: usize, // Any package the supported package managers (apt, pacman and dnf) say
    // would be installed on the system (names only)
    others: usize, // Not installed and not a .desktop file
    packageManager: String,
    canProvideUpdates: bool,
    updates: usize,
    lastDatabaseUpdate: u64,
}

/// Return the current package database.
///
/// This returns a list of every installed packages, every app the has a .desktop file and all
/// available packages that are listed in the package manager.
#[get("/api/packageDatabase/{count_only}")]
async fn package_database(
    session: Session,
    state: Data<AppState>,
    path: Path<bool>,
) -> HttpResponse {
    if !is_admin_state(&session, state) {
        return HttpResponse::Forbidden().body("This resource is blocked.");
    }

    let query =
        database::read_cols::<&str, (u64,)>("PackageActions", &["last_database_update"]).unwrap();
    let last_database_update;
    if query.len() != 1 {
        last_database_update = 0
    } else {
        last_database_update = query[0].0
    }

    if path.into_inner() {
        let installed = match packages::list_installed_packages() {
            Ok(packages) => packages.len(),
            Err(_err) => {
                eprintln!("Listing installed packages failed");
                0
            }
        };

        let available = packages::list_available_packages().unwrap().len();

        let updates_execution = packages::list_updates();
        let (can_provide_updates, updates) = {
            if let Ok(u) = updates_execution {
                (true, u.len())
            } else {
                (false, 0)
            }
        };

        HttpResponse::Ok().json(PackageResponseJsonCounts {
            packages: installed,
            others: available,
            packageManager: packages::get_package_manager().unwrap_or("".to_string()),
            canProvideUpdates: can_provide_updates,
            updates,
            lastDatabaseUpdate: last_database_update,
        })
    } else {
        let installed = match packages::list_installed_packages() {
            Ok(packages) => packages,
            Err(_err) => {
                eprintln!("Listing installed packages failed");
                Vec::new()
            }
        };

        let available = packages::list_available_packages().unwrap();

        let updates_execution = packages::list_updates();
        let (can_provide_updates, updates) = {
            if let Ok(u) = updates_execution {
                (true, u)
            } else {
                (false, Vec::new())
            }
        };

        HttpResponse::Ok().json(PackageResponseJson {
            packages: installed,
            others: available,
            packageManager: packages::get_package_manager().unwrap_or("".to_string()),
            canProvideUpdates: can_provide_updates,
            updates,
            lastDatabaseUpdate: last_database_update,
        })
    }
}

// Packages that would be affected by an autoremove

#[derive(Serialize)]
struct PackageDatabaseAutoremoveJson {
    packages: Vec<String>,
}

/// Return a list of all packages that would be affected by an autoremove.
#[get("/api/listOrphanedPackages")]
async fn orphaned_packages(session: Session, state: Data<AppState>) -> HttpResponse {
    if !is_admin_state(&session, state) {
        return HttpResponse::Forbidden().body("This resource is blocked.");
    }

    let packages = packages::list_orphaned_packages().unwrap();

    HttpResponse::Ok().json(PackageDatabaseAutoremoveJson { packages })
}

/// Update package database
#[post("/api/updatePackageDatabase")]
async fn update_package_database(
    session: Session,
    state: Data<AppState>,
    json: web::Json<SudoPasswordOnlyRequest>,
) -> HttpResponse {
    if !is_admin_state(&session, state.clone()) {
        return HttpResponse::Forbidden().body("This resource is blocked.");
    }

    let job_id = Uuid::new_v4();

    state
        .background_jobs
        .lock()
        .unwrap()
        .insert(job_id, BackgroundTaskState::Pending);

    let _ = actix_web::web::block(move || {
        match packages::update_database(json.into_inner().sudoPassword) {
            Ok(_) => {
                let x = database::write(
                    "PackageActions",
                    &["last_database_update", "key"],
                    &[
                        InsertValue::UnsignedInt64(
                            std::time::SystemTime::now()
                                .duration_since(UNIX_EPOCH)
                                .unwrap()
                                .as_secs(),
                        ),
                        InsertValue::UnsignedInt64(0_u64),
                    ],
                    "key",
                    "0",
                );
                if x.is_ok() {
                    state
                        .background_jobs
                        .lock()
                        .unwrap()
                        .insert(job_id, BackgroundTaskState::Success)
                } else {
                    state.background_jobs.lock().unwrap().insert(
                        job_id,
                        BackgroundTaskState::FailOutput(
                            "Zentrox was unable to update package actions database".to_string(),
                        ),
                    )
                }
            }
            Err(_) => state
                .background_jobs
                .lock()
                .unwrap()
                .insert(job_id, BackgroundTaskState::Fail),
        }
    });

    HttpResponse::Ok().body(job_id.to_string())
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
    if !is_admin_state(&session, state.clone()) {
        return HttpResponse::Forbidden().body("This resource is blocked.");
    }

    let job_id = Uuid::new_v4();

    let _ = actix_web::web::block(move || {
        match packages::install_package(json.packageName.to_string(), json.sudoPassword.to_string())
        {
            Ok(_) => state
                .background_jobs
                .lock()
                .unwrap()
                .insert(job_id, BackgroundTaskState::Success),
            Err(_) => state
                .background_jobs
                .lock()
                .unwrap()
                .insert(job_id, BackgroundTaskState::Fail),
        }
    });

    HttpResponse::Ok().body(job_id.to_string())
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
    if !is_admin_state(&session, state.clone()) {
        return HttpResponse::Forbidden().body("This resource is blocked.");
    }

    let job_id = Uuid::new_v4();

    let _ = actix_web::web::block(move || {
        match packages::remove_package(json.packageName.to_string(), json.sudoPassword.to_string())
        {
            Ok(_) => state
                .background_jobs
                .lock()
                .unwrap()
                .insert(job_id, BackgroundTaskState::Success),
            Err(_) => state
                .background_jobs
                .lock()
                .unwrap()
                .insert(job_id, BackgroundTaskState::Fail),
        }
    });

    HttpResponse::Ok().body(job_id.to_string())
}

#[post("/api/updatePackage")]
/// Remove a package from the users system.
///
/// It requires the package name along side the sudo password in the request body.
/// This only works under apt, dnf and pacman.
async fn update_package(
    session: Session,
    json: web::Json<PackageActionRequest>,
    state: Data<AppState>,
) -> HttpResponse {
    if !is_admin_state(&session, state.clone()) {
        return HttpResponse::Forbidden().body("This resource is blocked.");
    }

    let job_id = Uuid::new_v4();

    let _ = actix_web::web::block(move || {
        match packages::update_package(json.packageName.to_string(), json.sudoPassword.to_string())
        {
            Ok(_) => state
                .background_jobs
                .lock()
                .unwrap()
                .insert(job_id, BackgroundTaskState::Success),
            Err(_) => state
                .background_jobs
                .lock()
                .unwrap()
                .insert(job_id, BackgroundTaskState::Fail),
        }
    });

    HttpResponse::Ok().body(job_id.to_string())
}

#[post("/api/updateAllPackages")]
/// Remove a package from the users system.
///
/// It requires the package name along side the sudo password in the request body.
/// This only works under apt, dnf and pacman.
async fn update_all_packages(
    session: Session,
    json: web::Json<SudoPasswordOnlyRequest>,
    state: Data<AppState>,
) -> HttpResponse {
    if !is_admin_state(&session, state.clone()) {
        return HttpResponse::Forbidden().body("This resource is blocked.");
    }

    let sudo_password = json.sudoPassword.clone();
    let job_id = Uuid::new_v4();

    state
        .background_jobs
        .lock()
        .unwrap()
        .insert(job_id, BackgroundTaskState::Pending);

    let _ = actix_web::web::block(move || {
        match packages::update_all_packages(sudo_password.to_string()) {
            Ok(_) => state
                .background_jobs
                .lock()
                .unwrap()
                .insert(job_id, BackgroundTaskState::Success),
            Err(_) => state
                .background_jobs
                .lock()
                .unwrap()
                .insert(job_id, BackgroundTaskState::Fail),
        }
    });

    HttpResponse::Ok().body(job_id.to_string())
}

#[get("/api/fetchJobStatus/{jobId}")]
async fn fetch_job_status(
    session: Session,
    state: Data<AppState>,
    path: web::Path<String>,
) -> HttpResponse {
    if !is_admin_state(&session, state.clone()) {
        return HttpResponse::Forbidden().body("This resource is blocked.");
    }

    let requested_id = path.into_inner().to_string();
    let jobs = state.background_jobs.lock().unwrap().clone();
    let background_state = jobs.get(&uuid::Uuid::parse_str(&requested_id).unwrap());

    match background_state {
        Some(bs) => match bs {
            BackgroundTaskState::Success => HttpResponse::Ok().body("Task finished successfully"),
            BackgroundTaskState::Fail => HttpResponse::UnprocessableEntity().body("Task failed"),
            BackgroundTaskState::SuccessOutput(s) => HttpResponse::Ok().body(s.clone()),
            BackgroundTaskState::FailOutput(f) => {
                HttpResponse::UnprocessableEntity().body(f.clone())
            }
            BackgroundTaskState::Pending => HttpResponse::Accepted().body("Task is pending"),
        },
        None => HttpResponse::Accepted().body("Task does not exist"),
    }
}

#[post("/api/removeOrphanedPackages")]
/// Run an autoremove command on the users computer.
async fn remove_orphaned_packages(
    session: Session,
    json: web::Json<SudoPasswordOnlyRequest>,
    state: Data<AppState>,
) -> HttpResponse {
    if !is_admin_state(&session, state.clone()) {
        return HttpResponse::Forbidden().body("This resource is blocked.");
    }

    let sudo_password = json.sudoPassword.clone();
    let job_id = Uuid::new_v4();

    state
        .background_jobs
        .lock()
        .unwrap()
        .insert(job_id, BackgroundTaskState::Pending);

    let _ = actix_web::web::block(move || {
        match packages::remove_orphaned_packages(sudo_password.to_string()) {
            Ok(_) => state
                .background_jobs
                .lock()
                .unwrap()
                .insert(job_id, BackgroundTaskState::Success),
            Err(_) => state
                .background_jobs
                .lock()
                .unwrap()
                .insert(job_id, BackgroundTaskState::Fail),
        }
    });

    return HttpResponse::Ok().body(job_id.to_string());
}

// Firewall API

#[get("/api/hasUfw")]
async fn firewall_has_ufw(session: Session, state: Data<AppState>) -> HttpResponse {
    if !is_admin_state(&session, state) {
        return HttpResponse::Forbidden().body("This resource is blocked.");
    }

    let installed_packages = packages::list_installed_packages();

    match installed_packages {
        Ok(v) => {
            if v.contains(&String::from("ufw")) {
                return HttpResponse::Ok().body("true");
            } else {
                return HttpResponse::Ok().body("false");
            }
        }
        Err(_) => {
            if Command::new("ufw").spawn().is_ok() {
                return HttpResponse::Ok().body("true");
            } else {
                return HttpResponse::Ok().body("false");
            }
        }
    }
}

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
            sudo::SudoExecutionResult::Success(status) => {
                if status == 0 {
                    return HttpResponse::Ok().body("UFW has been started");
                } else {
                    return HttpResponse::InternalServerError()
                        .body("Failed to start UFW (Return value unequal 0)");
                }
            }
            sudo::SudoExecutionResult::ExecutionError(_) => {
                return HttpResponse::InternalServerError()
                    .body("Failed to start UFW because to command error");
            }
            _ => {
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
            sudo::SudoExecutionResult::Success(status) => {
                if status == 0 {
                    return HttpResponse::Ok().body("UFW has been stopped");
                } else {
                    return HttpResponse::InternalServerError()
                        .body("Failed to stop UFW (Return value unequal 0)");
                }
            }
            sudo::SudoExecutionResult::ExecutionError(_) => {
                return HttpResponse::InternalServerError()
                    .body("Failed to stop UFW because of command error");
            }
            _ => {
                return HttpResponse::InternalServerError()
                    .body("Failed to start UFW because to command error");
            }
        }
    }

    HttpResponse::Ok().body("UFW has been configured")
}

#[post(
    "/api/newFireWallRule/{mode}/{destination}/{protocol}/{sender_mode}/{sender_adress}/{action}"
)]
/// Create a new firewall rule.
///
/// This request takes three URL parameters.
/// This requires a sudo password.
async fn new_firewall_rule(
    session: Session,
    state: Data<AppState>,
    path: web::Path<(String, String, String, String, String, String)>,
    json: web::Json<SudoPasswordOnlyRequest>,
) -> HttpResponse {
    if !is_admin_state(&session, state) {
        return HttpResponse::Forbidden().body("This resource is blocked.");
    }

    let password = &json.sudoPassword;
    let (mode, destination, protocol, sender_mode, sender_adress, action) = path.into_inner();

    let decoded_sender_adress = url_decode::url_decode(sender_adress);

    if mode == "p" {
        let destination_parsed: u64;
        match destination.parse::<u64>() {
            Ok(d) => destination_parsed = d,
            Err(_) => return HttpResponse::BadRequest().body("Malformed port"),
        };
        let x = ufw::new_rule_port(
            &password,
            {
                if protocol == "tcp" {
                    ufw::NetworkProtocol::Tcp(destination_parsed)
                } else if protocol == "udp" {
                    ufw::NetworkProtocol::Udp(destination_parsed)
                } else {
                    ufw::NetworkProtocol::Tcp(destination_parsed)
                }
            },
            {
                if sender_mode == "any" {
                    ufw::FirewallSender::Any
                } else {
                    ufw::FirewallSender::Specific(decoded_sender_adress)
                }
            },
            {
                if action == "allow" {
                    ufw::FirewallAction::Allow
                } else {
                    ufw::FirewallAction::Deny
                }
            },
        );

        match x {
            Ok(_) => HttpResponse::Ok().body("Added new rule"),
            Err(_e) => HttpResponse::InternalServerError().body("Failed to create new rule"),
        }
    } else if mode == "r" {
        let lr: Vec<&str> = destination.split(":").collect();
        let range_left: u64;
        let range_right: u64;

        if lr.len() != 2 {
            return HttpResponse::BadRequest().body("Malformed port range");
        }

        match lr[0].parse::<u64>() {
            Ok(v) => range_left = v,
            Err(_) => return HttpResponse::BadRequest().body("Malformed left side port"),
        }

        match lr[1].parse::<u64>() {
            Ok(v) => range_right = v,
            Err(_) => return HttpResponse::BadRequest().body("Malformed right side port"),
        }

        let x = ufw::new_rule_range(
            &password,
            {
                if protocol == "tcp" {
                    ufw::PortRange::Tcp(range_left, range_right)
                } else if protocol == "udp" {
                    ufw::PortRange::Udp(range_left, range_right)
                } else {
                    ufw::PortRange::Tcp(range_left, range_right)
                }
            },
            {
                if sender_mode == "any" {
                    ufw::FirewallSender::Any
                } else {
                    ufw::FirewallSender::Specific(decoded_sender_adress)
                }
            },
            {
                if action == "allow" {
                    ufw::FirewallAction::Allow
                } else {
                    ufw::FirewallAction::Deny
                }
            },
        );

        match x {
            Ok(_) => HttpResponse::Ok().body("Added new rule"),
            Err(e) => {
                eprintln!("Failed to create firewall rule: {e}");
                HttpResponse::InternalServerError().body("Failed to create new rule")
            }
        }
    } else {
        return HttpResponse::BadRequest().body("Malformed mode parameter");
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
        Ok(_) => HttpResponse::Ok().body("The rule has been deleted"),
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
    content: Vec<(String, DirectoryEntryType)>,
}

#[derive(Serialize)]
enum DirectoryEntryType {
    File,
    Directory,
    InsufficientPermissions,
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
            let mut result: Vec<(String, DirectoryEntryType)> = Vec::new();
            for e in contents {
                let e_unwrap = &e.unwrap();
                let file_name = &e_unwrap.file_name().into_string().unwrap();
                let is_file = e_unwrap.metadata().unwrap().is_file();
                let is_dir = e_unwrap.metadata().unwrap().is_dir();
                let is_symlink = e_unwrap.metadata().unwrap().is_symlink();

                if is_file {
                    result.push((file_name.to_string(), DirectoryEntryType::File))
                } else if is_dir || is_symlink {
                    result.push((file_name.to_string(), DirectoryEntryType::Directory))
                } else {
                    result.push((
                        file_name.to_string(),
                        DirectoryEntryType::InsufficientPermissions,
                    ))
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
                Ok(_) => HttpResponse::Ok().body("The file has been deleted"),
                Err(_) => HttpResponse::InternalServerError().body("Failed to delete file."),
            }
        } else if is_dir && has_permissions {
            match fs::remove_dir_all(&file_path) {
                Ok(_) => HttpResponse::Ok().body("The directory has been deleted"),
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
                Ok(_) => HttpResponse::Ok().body("The file has been renamed"),
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
    let path = std::path::Path::new(&file_path);

    if path.is_dir() {
        return HttpResponse::BadRequest().body("This is a directory. It can not be burned.");
    }

    if !path.exists() {
        return HttpResponse::BadRequest().body("This path does not exist");
    }

    let metadata = fs::metadata(&file_path).unwrap();
    let has_permissions = !metadata.permissions().readonly();

    if !has_permissions {
        return HttpResponse::Forbidden().body("Missing file permissions. Can not burn.");
    }

    let size = metadata.len();
    let r_s = (0..size).map(|_| rand::random::<u8>()).collect::<Vec<u8>>();
    match fs::write(file_path.clone(), r_s) {
        Ok(_) => {
            let _ = fs::remove_file(&file_path);
            HttpResponse::Ok().body("The file has been burned")
        }
        Err(_) => HttpResponse::InternalServerError().body("Failed to destroy file."),
    }
}

#[derive(Serialize)]
enum FileSystemEntry {
    File,
    Directory,
    Symlink,
    Unknown,
}

#[derive(Serialize)]
struct GetMetadataResponseJson {
    permissions: i32,
    owner_username: String,
    owner_uid: u32,
    owner_gid: u32,
    size: u64,
    entry_type: FileSystemEntry,
    created: Option<u64>,
    modified: Option<u64>,
    filename: String,
    absolute_path: String,
}

fn recursive_size_of_directory(path: &PathBuf) -> u64 {
    let mut size = 0_u64;
    match fs::read_dir(path) {
        Ok(contents) => {
            contents.for_each(|f| {
                if f.is_err() { return; }
                match f.as_ref().unwrap().metadata() {
                    Ok(metadata) => {
                if metadata.is_symlink() || metadata.is_file() {
                    size += metadata.size();
                } else {
                    size += recursive_size_of_directory(&f.unwrap().path());
                }
                    },
                    _ => {}
                }
            });
        }
        Err(_) => {}
    }
    return size;
}

#[get("/api/getMetadata/{path}")]
async fn get_metadata(
    session: Session,
    state: Data<AppState>,
    path: web::Path<String>,
) -> HttpResponse {
    if !is_admin_state(&session, state) {
        return HttpResponse::Forbidden().body("This resource is blocked.");
    }

    let from_url_path = url_decode::url_decode(&path);
    let constructed_path = std::path::PathBuf::from(from_url_path);
    let metadata =
        fs::metadata(constructed_path.clone()).expect("Failed to retrieve metadata for file");

    let permissions = metadata.permissions().mode() & 0o777;
    let permissions_octal_string = format!("{:o}", permissions);
    let permissions_octal: i32 = permissions_octal_string.parse().unwrap();

    // UNIX timestamp in seconds at which this file was created
    let created_stamp = match metadata.created() {
        Ok(v) => Some(
            v.duration_since(UNIX_EPOCH)
                .expect("Time went backwards.")
                .as_secs(),
        ),
        Err(_) => None,
    };

    // UNIX timestamp in seconds at which this file was last modified
    let modified_stamp = match metadata.modified() {
        Ok(v) => Some(
            v.duration_since(UNIX_EPOCH)
                .expect("Time went backwards.")
                .as_secs(),
        ),
        Err(_) => None,
    };

    let size;
    let is_file = metadata.is_file();
    let is_dir = metadata.is_dir();
    let is_symlink = metadata.is_symlink();

    if is_file || is_symlink {
        size = metadata.size();
    } else {
        size = recursive_size_of_directory(&constructed_path)
    }

    let owner_uid = metadata.uid();
    let owner_username =
        convert_uid_to_name(owner_uid as usize).unwrap_or("Unknown owner username".to_string());
    let owner_gid = metadata.gid();

    return HttpResponse::Ok().json(GetMetadataResponseJson {
        permissions: permissions_octal,
        size: size,
        owner_uid,
        owner_gid,
        owner_username,
        modified: modified_stamp,
        created: created_stamp,
        entry_type: {
            if is_dir {
                FileSystemEntry::Directory
            } else if is_file {
                FileSystemEntry::File
            } else if is_symlink {
                FileSystemEntry::Symlink
            } else {
                FileSystemEntry::Unknown
            }
        },
        absolute_path: constructed_path.to_str().unwrap().to_string(),
        filename: constructed_path
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string(),
    });
}

#[derive(Debug, MultipartForm)]
struct UploadFileForm {
    #[multipart(limit = "32 GiB")]
    file: TempFile,
    path: Text<String>,
}

#[post("/upload/file")]
async fn upload_file(
    session: Session,
    state: Data<AppState>,
    MultipartForm(form): MultipartForm<UploadFileForm>,
) -> HttpResponse {
    if !is_admin_state(&session, state) {
        return HttpResponse::Forbidden().body("This resource is blocked.");
    }

    let intended_path = PathBuf::from(form.path.to_string());
    let filename: String;
    match form.file.file_name {
        Some(v) => filename = v,
        None => {
            return HttpResponse::BadRequest().body("No filename specified");
        }
    };
    let intended_path = intended_path.join(filename);
    let temp_path = form.file.file.path();
    let cpy = fs::copy(temp_path, &intended_path);
    let unk = fs::remove_file(temp_path);

    if !cpy.is_ok() {
        HttpResponse::InternalServerError()
            .body("Unable to copy temporary file to intended destination");
    }

    if !unk.is_ok() {
        HttpResponse::InternalServerError().body("Unable to delete temporary file");
    }

    HttpResponse::Ok().body("The upload has been finished")
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

    if database::read_kv("Settings", "vault_enabled").unwrap() == *database::ST_BOOL_FALSE
        && !vault_path.exists()
    {
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
        let _ = database::write_kv(
            "Settings",
            "vault_enabled",
            database::InsertValue::Text(database::ST_BOOL_TRUE.into()),
        );
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

    if database::read_kv("Settings", "vault_enabled").unwrap() == database::ST_BOOL_TRUE {
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
    if database::read_kv("Settings", "vault_enabled").unwrap() == database::ST_BOOL_TRUE {
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
    if !is_admin_state(&session, state.clone()) {
        return HttpResponse::Forbidden().body("This resource is blocked.");
    }

    let uuid = Uuid::new_v4();
    state
        .background_jobs
        .lock()
        .unwrap()
        .insert(uuid, BackgroundTaskState::Pending);

    let _ = actix_web::web::block(move || {
        let sent_path = &json.deletePath;
        if sent_path == ".vault" {
            error!(".vault file may never be deleted.");
            state.background_jobs.lock().unwrap().insert(
                uuid,
                BackgroundTaskState::FailOutput(".vault file may never be deleted.".to_string()),
            );
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
                    .map(|x| {
                        vault::encrypt_string_hash(x.to_string(), &json.key.to_string()).unwrap()
                    })
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
                Err(_e) => {
                    error!("Failed to remove vault file.");
                    state.background_jobs.lock().unwrap().insert(
                        uuid,
                        BackgroundTaskState::FailOutput("Failed to remove vault file.".to_string()),
                    );
                }
            };
        } else {
            let _ = vault::burn_directory(path.to_string_lossy().to_string());
            match fs::remove_dir_all(path) {
                Ok(_) => {}
                Err(_e) => {
                    error!("Failed to remove vault directory.");
                    state.background_jobs.lock().unwrap().insert(
                        uuid,
                        BackgroundTaskState::FailOutput(
                            "Failed to remove vault directory.".to_string(),
                        ),
                    );
                }
            };
        }
        state
            .background_jobs
            .lock()
            .unwrap()
            .insert(uuid, BackgroundTaskState::Success);
    });

    HttpResponse::Ok().body(uuid.to_string())
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
        return HttpResponse::InternalServerError().body("The path is invalid");
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
    key: Text<String>,
    path: Text<String>,

    #[multipart(limit = "32 GiB")]
    file: TempFile,
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

    let _ = tokio::fs::copy(&tmp_file_path, &in_vault_path).await;

    println!("{:?}", &tmp_file_path);

    let _ = actix_web::web::block(|| {
        let _ = tokio::fs::metadata(&tmp_file_path.clone()).then(async move |v| {
            let file_size = v.unwrap().len();
            let mut i = 0;
            while i != 5 {
                let random_data = (0..file_size)
                    .map(|_| rand::random::<u8>())
                    .collect::<Vec<u8>>();
                let _ = tokio::fs::write(&tmp_file_path, random_data);
                i += 1;
            }
            let _ = tokio::fs::remove_file(&tmp_file_path);
        });
    });

    vault::encrypt_file(in_vault_path.to_string_lossy().to_string(), key);

    HttpResponse::Ok().body("The upload has been finished")
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
        HttpResponse::BadRequest().body("This file can never be deleted");
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

    let _ = database::write_kv(
        "Settings",
        "tls_cert",
        database::InsertValue::Text(file_name.to_string()),
    );

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

    let tls = database::read_kv("Settings", "tls_cert").unwrap();

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

    let e =
        sudo::SwitchedUserCommand::new(json.sudoPassword.clone(), "poweroff".to_string()).spawn();

    if let sudo::SudoExecutionResult::Success(_) = e {
        HttpResponse::Ok().body("Shutting down.")
    } else {
        HttpResponse::InternalServerError().body("Failed to execute poweroff as super user.")
    }
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

    let password = &json.password;
    let new_username = &json.username;

    if !password.is_empty() {
        let hashed_password =
            hex::encode(crypto_utils::argon2_derive_key(password).unwrap()).to_string();

        let a = database::write_kv(
            "Secrets",
            "admin_password",
            database::InsertValue::Text(hashed_password),
        );

        if a.is_err() {
            return HttpResponse::InternalServerError().body("Failed to update admin_password.");
        }
    }

    if !new_username.is_empty() {
        let b = database::update_where(
            "Admin",
            &["username"],
            &[database::InsertValue::from(new_username)],
            "key",
            "0",
        );

        if b.is_err() {
            return HttpResponse::InternalServerError().body("Failed to update username.");
        }
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

// Media center endpoints

fn parse_range(range: actix_web::http::header::HeaderValue) -> (usize, Option<usize>) {
    let range_str = range.to_str().ok().unwrap(); // Safely convert to str, return None if failed
    let range_separated_clear = range_str.replace("bytes=", "");
    let range_separated: Vec<&str> = range_separated_clear.split('-').collect(); // Split the range

    // Parse the start and end values safely
    let start = range_separated.get(0).unwrap().parse::<usize>().unwrap();

    (
        start,
        match range_separated.get(1) {
            Some(v) => {
                if v == &"" {
                    None
                } else {
                    Some(v.parse::<usize>().unwrap())
                }
            }
            None => None,
        },
    )
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

    if database::read_kv("Settings", "media_enabled").unwrap() == database::ST_BOOL_FALSE {
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
        let whitelist_vector: Vec<(bool, PathBuf)> =
            database::read_cols::<&str, (bool, String)>("MediaSources", &["enabled", "folderpath"])
                .unwrap()
                .into_iter()
                .map(|e| (e.0, PathBuf::from(e.1)))
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
                let filesize: usize = reader
                    .get_ref()
                    .metadata()
                    .unwrap()
                    .len()
                    .try_into()
                    .unwrap_or(0);
                if byte_range.0 > filesize {
                    return HttpResponse::RangeNotSatisfiable()
                        .body("The requested range can not be satisfied.");
                }
                if byte_range.1.is_some() && (byte_range.1.unwrap() > filesize) {
                    return HttpResponse::RangeNotSatisfiable()
                        .body("The requested range can not be satisfied.");
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
                        format!(
                            "bytes {}-{}/{}",
                            byte_range.0,
                            byte_range.1.unwrap_or(filesize - 1),
                            filesize
                        ), // We HAVE to subtract 1 from the actual file size
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

    let locations = &json.locations;

    // The frontend only sends an updated array of all resources.
    // It is easier to truncate the entire table and then rewrite its' contents.

    if let Err(_e) = database::truncate_table("MediaSources") {
        return HttpResponse::InternalServerError().body("Failed to truncate sources table.");
    }

    for l in locations {
        if l.0.exists() {
            let clean_path = fs::canonicalize(l.0.clone())
                .unwrap()
                .to_string_lossy()
                .to_string();

            let s = database::insert(
                "MediaSources",
                &["folderpath", "alias", "enabled"],
                &[
                    InsertValue::from(clean_path),
                    InsertValue::from(l.1.to_string()),
                    InsertValue::from(l.2),
                ],
            );

            if let Err(_e) = s {
                return HttpResponse::InternalServerError().body("Failed to write source entry.");
            }
        }
    }

    HttpResponse::Ok().body("Updated video sources.")
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

    let rows = database::read_cols::<&str, (String, String, bool)>(
        "MediaSources",
        &["folderpath", "alias", "enabled"],
    )
    .unwrap();
    let mut s = Vec::new();
    for r in rows {
        s.push((r.0.clone(), r.1.clone(), r.2));
    }

    HttpResponse::Ok().json(VideoSourcesListResponseJson { locations: s })
}

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

    if database::read_kv("Settings", "media_enabled").unwrap() == database::ST_BOOL_FALSE {
        return HttpResponse::Forbidden().body("Media center has been disabled");
    }

    let sources: Vec<PathBuf> =
        database::read_cols::<&str, (String, bool)>("MediaSources", &["folderpath", "enabled"])
            .unwrap()
            .into_iter()
            .filter(|e| e.1)
            .map(|e| PathBuf::from(e.0.clone()))
            .collect();

    let media_files_vector: Vec<Vec<PathBuf>> = sources
        .clone()
        .into_iter()
        .filter(|p| p.exists())
        .map(|p| {
            visit_dirs(p)
                .unwrap()
                .map(|x| x.path())
                .filter(|x| x.is_file())
                // Remove directories and metadata files.
                .collect()
        })
        .collect();

    let mut all_media_files: Vec<PathBuf> = Vec::new();

    for mut paths in media_files_vector {
        all_media_files.append(&mut paths);
    }

    // Create hashmap of all metadata files

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

    let metadata_rows = database::read_cols::<&str, (String, String, String, String, String)>(
        "Media",
        &["filepath", "genre", "name", "artist", "cover"],
    )
    .unwrap();

    for row in metadata_rows {
        let filepath = PathBuf::from(row.0.clone());
        if !filepath.exists() {
            let x = database::delete_row(
                "Media",
                "filepath",
                filepath
                    .to_str()
                    .unwrap_or(filepath.to_string_lossy().to_string().as_str()),
            );

            if let Err(_e) = x {
                return HttpResponse::InternalServerError()
                    .body("Failed to delete row of removed video file.");
            }
        } else {
            let mut a = false;
            for path in &sources {
                if a {
                    continue;
                }
                if filepath.starts_with(path) {
                    a = true;
                }
            }

            if !a {
                continue;
            }

            media_info_hashmap.insert(
                filepath,
                (row.2.clone(), row.4.clone(), row.1.clone(), row.3.clone()),
            );
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

    if database::read_kv("Settings", "media_enabled").unwrap()
        == database::ST_BOOL_FALSE.to_string()
    {
        return HttpResponse::Forbidden().body("Media center has been disabled");
    }

    let sources: Vec<(String, bool)> =
        database::read_cols::<&str, (String, bool)>("MediaSources", &["folderpath", "enabled"])
            .unwrap();

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
        if !sources.contains(&(parent.unwrap().to_string_lossy().to_string(), true)) {
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
        enabled: database::read_kv("Settings", "media_enabled").unwrap() == database::ST_BOOL_TRUE,
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
        let _ = database::write_kv(
            "Settings",
            "media_enabled",
            database::InsertValue::Text(database::ST_BOOL_TRUE.to_string()),
        );
    } else {
        let _ = database::write_kv(
            "Settings",
            "media_enabled",
            database::InsertValue::Text(database::ST_BOOL_FALSE.to_string()),
        );
    }

    return HttpResponse::Ok().body("Updated media center status");
}

#[get("/api/rememberMusic/{songPath}")]
async fn remember_music(
    session: Session,
    state: Data<AppState>,
    e: web::Path<String>,
) -> HttpResponse {
    if !is_admin_state(&session, state) {
        return HttpResponse::Forbidden().body("This resource is blocked.");
    }

    if database::read_kv("Settings", "media_enabled").unwrap()
        == database::ST_BOOL_FALSE.to_string()
    {
        return HttpResponse::Forbidden().body("Media center has been disabled");
    }

    let current_ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("Time does not appear to be correct. Is your system's time configuration correct?");

    let fp = &e.into_inner();

    let x = database::write(
        "RecommendedMedia",
        &["filepath", "lastview", "category"],
        &[
            database::InsertValue::from(fp),
            database::InsertValue::from(current_ts.as_millis()),
            database::InsertValue::from("music"),
        ],
        "filepath",
        fp,
    );

    database::read_cols::<&str, (String,)>("RecommendedMedia", &["filepath"])
        .unwrap()
        .into_iter()
        .for_each(|e| {
            let pb = PathBuf::from(&e.0);

            if !pb.exists() {
                let _x = database::delete_row("RecommendedMedia", "filepath", e.0.as_str());
            }
        });

    match x {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(_e) => HttpResponse::InternalServerError().body("Failed to write database."),
    }
}

#[get("/api/rememberVideo/{videoPath}")]
async fn remember_video(
    session: Session,
    state: Data<AppState>,
    e: web::Path<String>,
) -> HttpResponse {
    if !is_admin_state(&session, state) {
        return HttpResponse::Forbidden().body("This resource is blocked.");
    }

    if database::read_kv("Settings", "media_enabled").unwrap()
        == database::ST_BOOL_FALSE.to_string()
    {
        return HttpResponse::Forbidden().body("Media center has been disabled");
    }

    let current_ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("Time does not appear to be correct. Is your system's time configuration correct?");

    let fp = &e.into_inner();

    let x = database::write(
        "RecommendedMedia",
        &["filepath", "lastview", "category"],
        &[
            database::InsertValue::from(fp),
            database::InsertValue::from(current_ts.as_millis()),
            database::InsertValue::from("video"),
        ],
        "filepath",
        fp,
    );

    database::read_cols::<&str, (String,)>("RecommendedMedia", &["filepath"])
        .unwrap()
        .into_iter()
        .for_each(|e| {
            let pb = PathBuf::from(&e.0);

            if !pb.exists() {
                let _x = database::delete_row("RecommendedMedia", "filepath", e.0.as_str());
            }
        });

    match x {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(_e) => HttpResponse::InternalServerError().body("Failed to write database."),
    }
}

#[derive(Serialize)]
struct Recommendations {
    rec: Vec<(String, i64)>,
}

#[get("/api/recommendedMusic")]
async fn get_recomended_music(session: Session, state: Data<AppState>) -> HttpResponse {
    if !is_admin_state(&session, state) {
        return HttpResponse::Forbidden().body("This resource is blocked.");
    }

    if database::read_kv("Settings", "media_enabled").unwrap() == *database::ST_BOOL_FALSE {
        return HttpResponse::Forbidden().body("Media center has been disabled");
    }

    let entries: Vec<(String, i64)> = database::read_cols::<&str, (String, i64, String)>(
        "RecommendedMedia",
        &["filepath", "lastview", "category"],
    )
    .unwrap()
    .into_iter()
    .filter(|e| e.2 == "music")
    .filter(|e| PathBuf::from(&e.0).exists())
    .map(|e| (e.0, e.1))
    .collect();

    return HttpResponse::Ok().json(Recommendations { rec: entries });
}

#[get("/api/recommendedVideos")]
async fn get_recomended_videos(session: Session, state: Data<AppState>) -> HttpResponse {
    if !is_admin_state(&session, state) {
        return HttpResponse::Forbidden().body("This resource is blocked.");
    }

    if database::read_kv("Settings", "media_enabled").unwrap() == *database::ST_BOOL_FALSE {
        return HttpResponse::Forbidden().body("Media center has been disabled");
    }

    let entries: Vec<(String, i64)> = database::read_cols::<&str, (String, i64, String)>(
        "RecommendedMedia",
        &["filepath", "lastview", "category"],
    )
    .unwrap()
    .into_iter()
    .filter(|e| e.2 == "video")
    .filter(|e| PathBuf::from(&e.0).exists())
    .map(|e| (e.0, e.1))
    .collect();

    return HttpResponse::Ok().json(Recommendations { rec: entries });
}

#[derive(Deserialize)]
struct MetadataJson {
    name: String,
    genre: String,
    cover: String,
    artist: String,
    filename: String,
}

#[post("/api/updateMetadata")]
async fn update_metadata(
    session: Session,
    state: Data<AppState>,
    data: web::Json<MetadataJson>,
) -> HttpResponse {
    if !is_admin_state(&session, state) {
        return HttpResponse::Forbidden().body("This resource is blocked.");
    }

    let wx = database::write(
        "Media",
        &["filepath", "name", "genre", "cover", "artist"],
        &[
            InsertValue::from(&data.filename),
            InsertValue::from({
                let name = &data.name;
                if name.is_empty() {
                    "UNKNOWN_NAME"
                } else {
                    name
                }
            }),
            InsertValue::from({
                let genre = &data.genre;
                if genre.is_empty() {
                    "UNKNOWN_GENRE"
                } else {
                    genre
                }
            }),
            InsertValue::from({
                let cover = &data.cover;
                if cover.is_empty() {
                    "UNKNOWN_COVER"
                } else {
                    cover
                }
            }),
            InsertValue::from({
                let artist = &data.artist;
                if artist.is_empty() {
                    "UNKNOWN_ARTIST"
                } else {
                    artist
                }
            }),
        ],
        "filepath",
        &data.filename,
    );

    if let Err(_e) = wx {
        return HttpResponse::InternalServerError()
            .body("Failed to write to database.".to_string());
    } else {
        return HttpResponse::Ok().body("Wrote data.");
    }
}

// Networking

#[derive(Serialize)]
struct NetworkInterfacesResponseJson {
    interfaces: Vec<MeasuredInterface>,
}

#[derive(Serialize)]
struct NetworkRoutesResponseJson {
    routes: Vec<Route>,
}

#[get("/api/networkInterfaces")]
async fn network_interfaces(session: Session, state: Data<AppState>) -> HttpResponse {
    if !is_admin_state(&session, state.clone()) {
        return HttpResponse::Forbidden().body("This resource is blocked.");
    }

    let interfaces = state.network_interfaces.lock().unwrap().clone();

    return HttpResponse::Ok().json(NetworkInterfacesResponseJson { interfaces });
}

#[get("/api/networkRoutes")]
async fn network_routes(session: Session, state: Data<AppState>) -> HttpResponse {
    if !is_admin_state(&session, state) {
        return HttpResponse::Forbidden().body("This resource is blocked.");
    }

    let routes = net_data::get_routes();

    return HttpResponse::Ok().json(NetworkRoutesResponseJson {
        routes: routes.unwrap(),
    });
}

#[derive(Deserialize, Debug)]
struct DeleteNetworkRoutesJson {
    device: String,
    destination: (bool, Option<String>, Option<i32>),
    gateway: (bool, Option<String>, Option<i32>),
    sudo_password: String,
}

#[post("/api/deleteNetworkRoute")]
async fn delete_network_route(
    session: Session,
    state: Data<AppState>,
    json: web::Json<DeleteNetworkRoutesJson>,
) -> HttpResponse {
    if !is_admin_state(&session, state) {
        return HttpResponse::Forbidden().body("This resource is blocked.");
    }

    let del_route = DeletionRoute {
        device: json.device.clone(),
        nexthop: None,
        gateway: {
            if json.gateway.0 {
                None
            } else {
                Some(IpAddrWithSubnet {
                    address: IpAddr::from_str(&json.gateway.1.clone().unwrap()).unwrap(),
                    subnet: json.gateway.2,
                })
            }
        },
        destination: {
            if json.destination.0 {
                Destination::Default
            } else {
                Destination::Prefix(IpAddrWithSubnet {
                    address: IpAddr::from_str(&json.destination.1.clone().unwrap()).unwrap(),
                    subnet: json.destination.2,
                })
            }
        },
    };

    net_data::delete_route(del_route, json.sudo_password.clone());

    HttpResponse::Ok().finish()
}

#[derive(Deserialize)]
struct NetworkingInterfaceActivityJson {
    activity: bool,
    interface: String,
    sudoPassword: String,
}

#[post("/api/networkingInterfaceActive")]
async fn networking_interface_active(
    session: Session,
    state: Data<AppState>,
    json: web::Json<NetworkingInterfaceActivityJson>,
) -> HttpResponse {
    if !is_admin_state(&session, state) {
        return HttpResponse::Forbidden().body("This resource is blocked.");
    }

    if json.activity {
        net_data::enable_interface(json.sudoPassword.clone(), json.interface.clone());
    } else {
        net_data::disable_interface(json.sudoPassword.clone(), json.interface.clone());
    }
    return HttpResponse::Ok().finish();
}

#[derive(Serialize)]
struct ListProcessesResponseJson {
    processes: Vec<SerializableProcess>,
}

#[derive(Serialize)]
struct SerializableProcess {
    name: Option<String>,
    cpu_usage: f32,
    memory_usage_bytes: u64,
    username: Option<String>,
    uid: Option<u32>,
    executable_path: Option<String>,
    pid: u32,
}

fn os_string_array_to_string_vector(s: &[OsString]) -> Vec<String> {
    s.iter()
        .map(|x| {
            let b = x.as_encoded_bytes();
            String::from_utf8(b.to_vec()).unwrap()
        })
        .collect::<Vec<String>>()
}

#[get("/api/listProcesses")]
async fn list_processes(session: Session, state: Data<AppState>) -> HttpResponse {
    if !is_admin_state(&session, state.clone()) {
        return HttpResponse::Forbidden().body("This resource is blocked.");
    }

    let process_refresh = ProcessRefreshKind::nothing()
        .without_tasks()
        .with_cpu()
        .with_memory()
        .with_user(UpdateKind::OnlyIfNotSet)
        .with_exe(UpdateKind::OnlyIfNotSet);

    let mut system = sysinfo::System::new_with_specifics(
        RefreshKind::nothing()
            .with_memory(MemoryRefreshKind::everything())
            .with_processes(process_refresh),
    );
    std::thread::sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL);
    system.refresh_processes_specifics(sysinfo::ProcessesToUpdate::All, false, process_refresh);
    let processes = system.processes();
    let mut processes_for_response = Vec::new();

    processes.iter().for_each(|x| {
        let memory_usage_bytes = x.1.memory();
        let cpu_usage = x.1.cpu_usage();

        let executable_path: Option<String>;
        match x.1.exe() {
            Some(v) => {
                executable_path = Some(v.to_str().unwrap().to_string());
            }
            None => executable_path = None,
        }

        let mut username: Option<String> = None;
        let mut uid_true: Option<u32> = None;
        let uid = x.1.user_id();
        match uid {
            Some(uid) => {
                username = Some(
                    convert_uid_to_name(uid.div(1) as usize)
                        .unwrap_or(format!("Missing username ({})", uid.div(1)).to_string()),
                );
                uid_true = Some(uid.div(1));
            }
            None => {}
        }

        let pid = x.1.pid().as_u32();
        let name = Some(x.1.name().to_string_lossy().to_string());
        processes_for_response.push(SerializableProcess {
            cpu_usage,
            memory_usage_bytes,
            executable_path,
            name,
            pid,
            uid: uid_true,
            username,
        });
    });

    return HttpResponse::Ok().json(ListProcessesResponseJson {
        processes: processes_for_response,
    });
}

#[get("/api/killProcess/{pid}")]
async fn kill_process(session: Session, state: Data<AppState>, path: Path<u32>) -> HttpResponse {
    if !is_admin_state(&session, state) {
        return HttpResponse::Forbidden().body("This resource is blocked.");
    }

    let process_refresh = ProcessRefreshKind::nothing()
        .without_tasks()
        .with_cpu()
        .with_memory()
        .with_user(UpdateKind::OnlyIfNotSet)
        .with_exe(UpdateKind::OnlyIfNotSet);
    let mut system = sysinfo::System::new_with_specifics(
        RefreshKind::nothing()
            .with_memory(MemoryRefreshKind::everything())
            .with_processes(process_refresh),
    );
    std::thread::sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL);
    system.refresh_processes_specifics(sysinfo::ProcessesToUpdate::All, false, process_refresh);
    let processes = system.processes();

    match processes.get(&Pid::from_u32(path.into_inner())) {
        Some(p) => {
            if p.kill() {
                return HttpResponse::Ok().body("Signal sent successfully");
            } else {
                return HttpResponse::InternalServerError().json(ErrorJson {
                    error: "SignalError".to_string(),
                });
            }
        }
        None => {
            return HttpResponse::InternalServerError().json(ErrorJson {
                error: "WrongPID".to_string(),
            })
        }
    }
}

#[derive(Serialize)]
struct ProcessDetailsResponseJson {
    name: String,
    pid: u32,
    username: String,
    uid: u32,
    memory_usage_bytes: u64,
    cpu_usage: f32,
    run_time: u64,
    command_line: Vec<String>,
    executable_path: String,
    priority: isize,
    threads: isize,
    parent: String,
}

#[get("/api/detailsProcess/{pid}")]
async fn details_process(session: Session, state: Data<AppState>, path: Path<u32>) -> HttpResponse {
    if !is_admin_state(&session, state) {
        return HttpResponse::Forbidden().body("This resource is blocked.");
    }

    let process_refresh = ProcessRefreshKind::nothing()
        .without_tasks()
        .with_cpu()
        .with_memory()
        .with_cmd(UpdateKind::OnlyIfNotSet)
        .with_user(UpdateKind::OnlyIfNotSet)
        .with_exe(UpdateKind::OnlyIfNotSet);
    let mut system = sysinfo::System::new_with_specifics(
        RefreshKind::nothing()
            .with_memory(MemoryRefreshKind::everything())
            .with_processes(process_refresh),
    );
    std::thread::sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL);
    system.refresh_processes_specifics(sysinfo::ProcessesToUpdate::All, false, process_refresh);
    let processes = system.processes();
    let selected_process = processes.get(&Pid::from_u32(path.into_inner())).unwrap();

    let name = selected_process.name();
    let pid = selected_process.pid();
    let uid = selected_process.user_id().unwrap();
    let username = convert_uid_to_name(uid.div(1) as usize);
    let memory_usage_bytes = selected_process.memory();
    let cpu_usage = selected_process.cpu_usage();
    let run_time = selected_process.run_time();
    let command_line = os_string_array_to_string_vector(selected_process.cmd());
    let executable_path_determination = selected_process.exe();

    let parent = selected_process
        .parent()
        .expect("The process has no parrent id.");
    let parent_name = system
        .process(parent)
        .expect("The process can not be accessed.")
        .name()
        .to_str()
        .unwrap()
        .to_string();

    let mut executable_path = String::from("Unknown");
    if executable_path_determination.is_some() {
        executable_path = executable_path_determination
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();
    }
    let stat_file = fs::read_to_string(
        PathBuf::from("/")
            .join("proc")
            .join(pid.to_string())
            .join("stat"),
    )
    .unwrap();

    let stat_file_split = stat_file.split(" ").collect::<Vec<&str>>();
    let priority = stat_file_split[18].parse::<isize>().unwrap_or(-1);

    let thread_count = stat_file_split[19].parse::<isize>().unwrap_or(-1);

    return HttpResponse::Ok().json(ProcessDetailsResponseJson {
        name: name.to_str().unwrap().to_string(),
        pid: pid.as_u32(),
        username: username.unwrap_or(String::from("Unknown")),
        uid: uid.div(1),
        memory_usage_bytes,
        cpu_usage,
        command_line,
        threads: thread_count,
        priority,
        run_time,
        executable_path,
        parent: parent_name,
    });
}

#[derive(Serialize)]
struct ListCronjobsReturnJson {
    specific_jobs: Vec<SpecificCronJob>,
    interval_jobs: Vec<IntervalCronJob>,
    crontab_exists: bool,
}

#[get("/api/listCronjobs/{current}/{specific}")]
async fn list_cronjobs(
    session: Session,
    state: Data<AppState>,
    path: Path<(String, Option<String>)>,
) -> HttpResponse {
    if !is_admin_state(&session, state.clone()) {
        return HttpResponse::Forbidden().body("This resource is blocked.");
    }

    let path_segs = path.into_inner();
    let user: User;
    if path_segs.0 == "current" {
        user = User::Current
    } else if path_segs.0 == "specific" {
        user = User::Specific(path_segs.1.unwrap())
    } else {
        return HttpResponse::BadRequest().body(format!("Unknown variant {}", path_segs.0));
    }

    let crons = cron::list_cronjobs(user);
    let mut interval_cronjobs: Vec<IntervalCronJob> = Vec::new();
    let mut specific_cronjobs: Vec<SpecificCronJob> = Vec::new();
    match crons {
        Ok(crons_unwrapped) => {
            for ele in crons_unwrapped {
                match ele {
                    cron::CronJob::Specific(spec) => specific_cronjobs.push(spec),
                    cron::CronJob::Interval(inter) => interval_cronjobs.push(inter),
                }
            }
        }
        Err(e) => match e {
            cron::CronListingError::NoCronFile => {
                return HttpResponse::Ok().json(ListCronjobsReturnJson {
                    specific_jobs: specific_cronjobs,
                    interval_jobs: interval_cronjobs,
                    crontab_exists: false,
                })
            }
            _ => return HttpResponse::InternalServerError().body("Failed to retrieve cronjobs"),
        },
    }

    return HttpResponse::Ok().json(ListCronjobsReturnJson {
        specific_jobs: specific_cronjobs,
        interval_jobs: interval_cronjobs,
        crontab_exists: true,
    });
}

#[derive(serde::Deserialize)]
struct CronjobCommandJson {
    command: String,
}

#[post("/api/runCronjobCommand")]
async fn run_cronjob_command(
    session: Session,
    state: Data<AppState>,
    json: web::Json<CronjobCommandJson>,
) -> HttpResponse {
    if !is_admin_state(&session, state.clone()) {
        return HttpResponse::Forbidden().body("This resource is blocked.");
    }

    let uuid = Uuid::new_v4();
    state
        .background_jobs
        .lock()
        .unwrap()
        .insert(uuid, BackgroundTaskState::Pending);

    let _ = actix_web::web::block(move || {
        match Command::new("sh")
            .arg("-c")
            .arg(&json.command)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .stdin(Stdio::null())
            .spawn()
        {
            Ok(mut h) => {
                let _ = h.wait();
                state
                    .background_jobs
                    .lock()
                    .unwrap()
                    .insert(uuid, BackgroundTaskState::Success);
            }
            Err(_) => {
                state
                    .background_jobs
                    .lock()
                    .unwrap()
                    .insert(uuid, BackgroundTaskState::Fail);
            }
        }
    });

    return HttpResponse::Ok().body(uuid.to_string());
}

#[derive(serde::Deserialize)]
struct DeleteCronjobJson {
    index: u32,
    variant: String,
}

#[post("/api/deleteCronjob")]
async fn delete_cronjob(
    session: Session,
    state: Data<AppState>,
    json: web::Json<DeleteCronjobJson>,
) -> HttpResponse {
    if !is_admin_state(&session, state.clone()) {
        return HttpResponse::Forbidden().body("This resource is blocked.");
    }

    if &json.variant == "specific" {
        let _ = delete_specific_cronjob(json.index, User::Current);
    } else if &json.variant == "interval" {
        let _ = delete_interval_cronjob(json.index, User::Current);
    } else {
        return HttpResponse::BadRequest()
            .body(format!("Unknown cronjob variant {}", &json.variant));
    }

    HttpResponse::Ok().finish()
}

#[derive(serde::Deserialize)]
struct CreateCronjobJson {
    variant: String,
    command: String,
    interval: Option<String>,
    minute: Option<String>,
    hour: Option<String>,
    day_of_month: Option<String>,
    day_of_week: Option<String>,
    month: Option<String>,
}

#[post("/api/createCronjob")]
async fn create_cronjob(
    session: Session,
    state: Data<AppState>,
    json: web::Json<CreateCronjobJson>,
) -> HttpResponse {
    if !is_admin_state(&session, state.clone()) {
        return HttpResponse::Forbidden().body("This resource is blocked.");
    }

    let variant = &json.variant;

    if variant == "interval" {
        let json_interval = &json.interval.clone().unwrap();

        let interval = match json_interval.as_str() {
            "daily" => cron::Interval::Daily,
            "weekly" => cron::Interval::Weekly,
            "monthly" => cron::Interval::Monthly,
            "yearly" => cron::Interval::Yearly,
            "hourly" => cron::Interval::Hourly,
            "reboot" => cron::Interval::Reboot,
            _ => panic!("Unknown interval type {}", json_interval),
        };

        match cron::create_new_interval_cronjob(
            cron::IntervalCronJob {
                interval,
                command: json.command.clone(),
            },
            User::Current,
        ) {
            Ok(_) => HttpResponse::Ok().finish(),
            Err(_) => {
                error!("Failed to create interval cronjob");
                HttpResponse::InternalServerError().body("Failed to create new interval cronjob")
            }
        }
    } else if variant == "specific" {
        let day_of_month = cron::Digit::from(json.day_of_month.clone().unwrap().as_str());
        let day_of_week = cron::DayOfWeek::from(json.day_of_week.clone().unwrap().as_str());
        let month = cron::Month::from(json.month.clone().unwrap().as_str());
        let minute = cron::Digit::from(json.minute.clone().unwrap().as_str());
        let hour = cron::Digit::from(json.hour.clone().unwrap().as_str());
        match cron::create_new_specific_cronjob(
            cron::SpecificCronJob {
                command: json.command.clone(),
                day_of_week,
                day_of_month,
                minute,
                hour,
                month,
            },
            User::Current,
        ) {
            Ok(_) => HttpResponse::Ok().finish(),
            Err(_) => {
                error!("Failed to create specific cronjob");
                HttpResponse::InternalServerError().body("Failed to create new specific cronjob")
            }
        }
    } else {
        HttpResponse::BadRequest().body("Unknown cronjob variant")
    }
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

fn configure_multipart(cfg: &mut web::ServiceConfig) {
    // Configure multipart form settings
    let multipart_config = MultipartFormConfig::default().total_limit(1024 * 1024 * 1024 * 32);

    // Store the configuration in app data
    cfg.app_data(multipart_config);
}

#[actix_web::main]
/// Prepares Zentrox and starts the server.
async fn main() -> std::io::Result<()> {
    if !env::current_dir().unwrap().join("static").exists() {
        let _ = env::set_current_dir(dirs::home_dir().unwrap().join("zentrox"));
    }

    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    if !dirs::home_dir()
        .unwrap()
        .join(".local")
        .join("share")
        .join("zentrox")
        .exists()
    {
        let _ = setup::run_setup();
    } else {
        debug!("Found configurations in ~/.local/share/zentrox/")
    }

    let secret_session_key = Key::try_generate().expect("Failed to generate session key");
    let app_state = AppState::new();
    app_state.clone().start_interval_tasks();
    debug!("Started interval tasks");

    if !database::exists("Secrets", "name", "otp_secret").unwrap()
        && database::read_cols::<&str, (bool,)>("Admin", &["use_otp"]).unwrap()[0].0
    {
        let new_otp_secret = otp::generate_otp_secret();
        if let Err(e) = database::write_kv(
            "Secrets",
            "otp_secret",
            database::InsertValue::from(&new_otp_secret),
        ) {
            eprintln!("Failed to write OTP secret: {}", e.to_string());
        }
        println!(
            "{}",
            include_str!("../notes/otp_note.txt").replace("SECRET", &new_otp_secret)
        );
    }

    if database::read_kv("Settings", "tls_cert").unwrap() == "selfsigned.pem" {
        warn!("Using a self singed certificate");
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
                .join(database::read_kv("Settings", "tls_cert").unwrap()),
        )
        .unwrap(),
    );
    debug!(
        "Using certificate file from {}",
        data_path
            .join("certificates")
            .join(database::read_kv("Settings", "tls_cert").unwrap())
            .to_str()
            .unwrap()
    );

    let mut key_file = BufReader::new(
        File::open(
            data_path
                .join("certificates")
                .join(database::read_kv("Settings", "tls_cert").unwrap()),
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

    let mut gov_vars = std::env::vars();
    let governor_strict: bool = !gov_vars.any(|x| {
        x == ("ZENTROX_MODE".to_string(), "NO_LIMITING".to_string())
            || x == ("ZENTROX_MODE".to_string(), "DEV".to_string())
    });

    let governor_conf = if governor_strict {
        GovernorConfigBuilder::default()
            .burst_size(100)
            .period(Duration::from_millis(250))
            .finish()
            .unwrap()
    } else {
        warn!("Using permissive governor configuration");
        GovernorConfigBuilder::default()
            .permissive(true)
            .finish()
            .unwrap()
    };

    let harsh_governor_conf = if governor_strict {
        GovernorConfigBuilder::default()
            .requests_per_minute(5)
            .finish()
            .unwrap()
    } else {
        warn!("Using permissive governor configuration");
        GovernorConfigBuilder::default()
            .permissive(true)
            .finish()
            .unwrap()
    };

    println!("🚀 Serving Zentrox on Port 8080");

    HttpServer::new(move || {
        let mut cors_vars = std::env::vars();
        let cors_permissive: bool = cors_vars.any(|x| {
            x == ("ZENTROX_MODE".to_string(), "NO_CORS".to_string())
                || x == ("ZENTROX_MODE".to_string(), "DEV".to_string())
        });
        if cors_permissive {
            warn!("CORS policy is set to permissive! This poses a high security risk.")
        }

        App::new()
            .app_data(Data::new(app_state.clone()))
            .configure(configure_multipart)
            .wrap(middleware::Logger::new("%a %U %s"))
            .wrap(
                SessionMiddleware::builder(
                    CookieSessionStore::default(),
                    secret_session_key.clone(),
                )
                .cookie_content_security(CookieContentSecurity::Private)
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
            .wrap(Governor::new(&governor_conf))
            // Landing
            .service(dashboard)
            .service(index)
            .service(alerts)
            .service(alerts_manifest)
            // Login, OTP and Logout
            .service(
                web::scope("/login")
                    .wrap(Governor::new(&harsh_governor_conf))
                    .route("/verify", web::post().to(login)),
            )
            .service(use_otp)
            .service(otp_secret_request)
            .service(logout) // Remove admin status and redirect to /
            .service(update_otp_status)
            // Sudo
            .service(verify_sudo_password)
            // API Device Stats
            .service(device_information) // General device information
            // API Packages
            .service(package_database)
            .service(orphaned_packages)
            .service(remove_orphaned_packages)
            .service(install_package)
            .service(remove_package)
            .service(update_package)
            .service(update_all_packages)
            .service(update_package_database)
            .service(fetch_job_status)
            // API Firewall
            .service(firewall_information)
            .service(switch_ufw)
            .service(new_firewall_rule)
            .service(delete_firewall_rule)
            .service(firewall_has_ufw)
            // API Files
            .service(call_file)
            .service(files_list)
            .service(delete_file)
            .service(rename_file)
            .service(burn_file)
            .service(upload_file)
            .service(get_metadata)
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
            .service(remember_music)
            .service(remember_video)
            .service(get_recomended_music)
            .service(get_recomended_videos)
            .service(update_metadata)
            // Networking
            .service(network_interfaces)
            .service(network_routes)
            .service(delete_network_route)
            .service(networking_interface_active)
            // Process manager
            .service(list_processes)
            .service(list_cronjobs)
            .service(kill_process)
            .service(details_process)
            .service(delete_cronjob)
            .service(run_cronjob_command)
            .service(create_cronjob)
            // General services and blocks
            .service(dashboard_asset_block)
            .service(robots_txt)
            .service(afs::Files::new("/", "static/"))
    })
    .workers(16)
    .keep_alive(std::time::Duration::from_secs(60 * 6))
    .bind_rustls_0_23(("0.0.0.0", 8080), tls_config)?
    .run()
    .await
}
