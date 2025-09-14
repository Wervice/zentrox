//! Zentrox is a Linux server administration application.
//! It support various tasks ranging from sharing and managing files to installing system packages
//! and basic network configuration tasks.
//!
//! The project uses actix_web together with serde_json for API communication.
//! To provide a session a CookieSession is used. Authentication is handled by Zentrox and does not
//! use a dedicated library.
//!
//! Interactions with the SQLite database are handled through diesel.rs.
//!
//! Most interactions between Zentrox and the operating system are handled through commands or files.
//!
//! Documentation for the API can be obtained by running the executable with the {`--docs`} flag.
//! This will produce an OpenAPI documentation in JSON format.

// NOTE ~/Documents/Zentrox_Rust_Structure.drawio

use actix_cors::Cors;
use actix_files as afs;
use actix_governor::{self, Governor, GovernorConfigBuilder};
use actix_multipart::form::MultipartFormConfig;
use actix_multipart::form::{MultipartForm, tempfile::TempFile, text::Text};
use actix_session::SessionExt;
use actix_session::config::CookieContentSecurity;
use actix_session::{Session, SessionMiddleware, storage::CookieSessionStore};
use actix_web::HttpRequest;
use actix_web::body::{BoxBody, MessageBody};
use actix_web::cookie::Key;
use actix_web::dev::{ServiceRequest, ServiceResponse};
use actix_web::middleware::{Next, from_fn};
use actix_web::web::{Path, Query};
use actix_web::{App, HttpResponse, HttpServer, get, http::header, middleware, web, web::Data};
use futures::FutureExt;
use rand::Rng;
use rand::distributions::Alphanumeric;
use serde::{Deserialize, Serialize};
use std::ffi::OsString;
use std::net::IpAddr;
use std::ops::Div;
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::process::{Command, Stdio, exit};
use std::str::FromStr;
use std::time::{Duration, SystemTime};
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
use utoipa::openapi::ServerBuilder;
use uuid::Uuid;
extern crate inflector;
use diesel::prelude::*;
use log::{debug, error, info, warn};
use utoipa::{OpenApi, ToSchema};

mod routes;

use utils::cron;
use utils::crypto_utils;
use utils::database;
use utils::drives;
mod is_admin;
mod setup;
use utils::logs;
use utils::mime;
use utils::models;
use utils::net_data;
use utils::otp;
use utils::packages;
use utils::schema;
use utils::status_com;
use utils::sudo;
use utils::ufw;
use utils::uptime;
use utils::users;
use utils::vault;
use utils::visit_dirs;

use is_admin::is_admin_state;
use net_data::private_ip;
use status_com::{ErrorCode, MessageRes};
use visit_dirs::visit_dirs;

use crate::crypto_utils::argon2_derive_key;
use crate::database::{establish_connection, get_administrator_account};
use crate::drives::DriveUsageStatistics;
use crate::logs::QuickJournalEntry;
use crate::models::{MediaSource, RecommendedMediaEntry, SharedFile};
use crate::ufw::FirewallAction;
use crate::users::NativeUser;

use self::cron::{
    IntervalCronJob, SpecificCronJob, User, delete_interval_cronjob, delete_specific_cronjob,
};
use self::net_data::{DeletionRoute, Destination, IpAddrWithSubnet, OperationalState, Route};

#[derive(Clone)]
#[allow(unused)]
enum BackgroundTaskState {
    Success,
    Fail,
    SuccessOutput(String),
    FailOutput(String),
    Pending,
}

/// A network interface with an up/down data transfer rate relative to a time.
/// The struct implements Serialize.
#[derive(Clone, Serialize)]
#[allow(unused)]
#[serde(rename_all = "camelCase")]
struct MeasuredInterface {
    pub index: i64,
    pub name: String,
    pub flags: Vec<String>,
    pub max_tranmission_unit: u64,
    pub queuing_discipline: String,
    pub operational_state: OperationalState,
    pub link_mode: String,
    pub group: String,
    pub transmit_queue: Option<i64>,
    pub link_type: String,
    pub address: String,
    pub broadcast: String,
    pub up: f64,
    pub down: f64,
    pub alternative_names: Option<Vec<String>>,
}

/// Current state of the application
/// This AppState is meant to be accessible for every route in the system
#[derive(Clone)]
struct AppState {
    login_token: Arc<Mutex<String>>, // TODO Use Option
    // TODO Make the token be invalidated after some time
    system: Arc<Mutex<sysinfo::System>>,
    username: Arc<Mutex<String>>, // TODO Use Option
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
            background_jobs: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    fn update_network_statistics(&self) {
        if (*self).username.lock().unwrap().is_empty() {
            return;
        }
        let devices_a = net_data::get_network_interfaces().unwrap();
        std::thread::sleep(std::time::Duration::from_millis(1000));
        let devices_b = net_data::get_network_interfaces().unwrap();
        let devices_b_hashmap: HashMap<String, &net_data::Interface> =
            devices_b.iter().map(|d| (d.name.clone(), d)).collect();
        let mut result: Vec<MeasuredInterface> = Vec::new();
        for device in devices_a {
            if let Some(v) = devices_b_hashmap.get(&device.name) {
                let a_up = device.statistics.get("tx").unwrap().bytes;
                let a_down = device.statistics.get("rx").unwrap().bytes;
                let b_up = v.statistics.get("tx").unwrap().bytes;
                let b_down = v.statistics.get("rx").unwrap().bytes;

                result.push(MeasuredInterface {
                    name: device.name,
                    index: device.index,
                    flags: device.flags,
                    max_tranmission_unit: device.max_transmission_unit,
                    queuing_discipline: device.queueing_discipline,
                    operational_state: device.operational_state,
                    link_mode: device.link_mode,
                    address: device.address,
                    alternative_names: device.alternative_names,
                    broadcast: device.broadcast,
                    down: (b_down - a_down) / 5_f64,
                    up: (b_up - a_up) / 5_f64,
                    group: device.group,
                    link_type: device.link_type,
                    transmit_queue: device.transmit_queue,
                })
            }
        }
        *self.network_interfaces.lock().unwrap() = result;
    }

    fn start_interval_tasks(self) {
        let network_clone = self.clone();
        std::thread::spawn(move || {
            loop {
                std::thread::sleep(std::time::Duration::from_millis(5 * 1000));
                network_clone.update_network_statistics();
            }
        });
    }
}

/// Root of the server.
///
/// If the user is logged in, they get redirected to /dashboard, otherwise the login is shown.
async fn index(session: Session, state: Data<AppState>) -> HttpResponse {
    // is_admin session value is != true (None or false), the user is served the login screen
    // otherwise, the user is redirected to /
    if is_admin_state(&session, state) {
        HttpResponse::Found()
            .append_header(("Location", "/dashboard"))
            .body("You will soon be redirected.")
    } else {
        HttpResponse::Ok()
            .body(std::fs::read_to_string("static/index.html").expect("Failed to read file"))
    }
}

async fn alerts_page(session: Session, state: Data<AppState>) -> HttpResponse {
    // is_admin session value is != true (None or false), the user is served the alerts screen
    // otherwise, the user is redirected to /
    if is_admin_state(&session, state) {
        HttpResponse::Ok().body(
            std::fs::read_to_string("static/alerts.html").expect("Failed to read alerts page"),
        )
    } else {
        HttpResponse::Found()
            .append_header(("Location", "/?app=true"))
            .body("You will soon be redirected")
    }
}

async fn media_page() -> HttpResponse {
    // is_admin session value is != true (None or false), the user is served the media screen
    // otherwise, the user is redirected to /
    if !get_media_enabled_database() {
        return HttpResponse::Forbidden().json(ErrorCode::MediaCenterDisabled.as_error_message());
    }
    HttpResponse::Ok()
        .body(std::fs::read_to_string("static/media.html").expect("Failed to read alerts page"))
}

async fn shared_page() -> HttpResponse {
    HttpResponse::Ok()
        .body(std::fs::read_to_string("static/shared.html").expect("Failed to read shared page"))
}

async fn alerts_manifest() -> HttpResponse {
    HttpResponse::Ok().body(include_str!("../../assets/manifest.json"))
}

/// The dashboard route.
///
/// If the user is logged in, the dashboard is shown, otherwise they get redirected to root.
async fn dashboard(session: Session, state: Data<AppState>) -> HttpResponse {
    // is_admin session value is != true (None or false), the user is redirected to /
    // otherwise, the user is served the dashboard.html file
    if is_admin_state(&session, state) {
        HttpResponse::Ok()
            .body(std::fs::read_to_string("static/dashboard.html").expect("Failed to read file"))
    } else {
        HttpResponse::Found()
            .append_header(("Location", "/"))
            .body("You will soon be redirected")
    }
}

// API (Actual API calls)

/// Single path schema
///
/// This struct implements serde::Serialize and Deserialize. It is intended for handling query
/// parameters, request bodies or response bodies that only contain a single file path that can be
/// expressed as a PathBuf.
#[derive(Deserialize, Serialize)]
struct SinglePath {
    path: PathBuf,
}

/// Request that only contains a sudo password for the backend.
///
/// This struct implements serde::Deserialize. It can be used to parse a single sudoPassword from
/// the user. It only has the String filed sudoPassword.
#[derive(Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
struct SudoPasswordReq {
    sudo_password: String,
}

// Login

#[derive(Deserialize, ToSchema)]
struct LoginReq {
    username: String,
    password: String,
    otp: Option<String>,
}

fn setup_login_state(session: Session, state: Data<AppState>, provided_username: String) {
    let login_token: Vec<u8> = is_admin::generate_random_token();
    let _ = session.insert("login_token", hex::encode(&login_token).to_string());

    *state.login_token.lock().unwrap() = hex::encode(&login_token).to_string();
    *state.username.lock().unwrap() = provided_username;

    let state_copy = state.clone();
    std::thread::spawn(move || {
        state_copy.update_network_statistics();
    });
}

/// Verify user and log in
#[utoipa::path(
    post,
    path = "/public/auth/login",
    responses(
        (status = 200, description = "The login was successful"),
        (status = 403, description = "A wrong password was provided."),
        (status = 401, description = "The username does not exist."),
        (status = 400, description = "Not enough information was provided.")
    ),
    request_body = LoginReq,
    tags = ["public", "authentication"]
)]
async fn verification(
    session: Session,
    json: web::Json<LoginReq>,
    state: Data<AppState>,
) -> HttpResponse {
    let request_username = &json.username;
    let request_password = &json.password;
    let request_otp_code = &json.otp;

    let database_admin_entry = get_administrator_account();

    if &database_admin_entry.username != request_username {
        info!("A login with a wrong username will be denied.");
        return HttpResponse::Unauthorized().json(ErrorCode::UnkownUsername.as_error_message());
    }
    let stored_password: String = database_admin_entry.password_hash;
    let hashes_correct =
        is_admin::password_hash(request_password.to_string(), stored_password.to_string());

    if !hashes_correct {
        info!("A login with a wrong password will be denied.");
        return HttpResponse::Forbidden().json(ErrorCode::WrongPassword.as_error_message());
    }
    if database_admin_entry.use_otp {
        if json.otp.is_none() {
            info!("The user is missing an otp code.");
            return HttpResponse::BadRequest().json(ErrorCode::MissingOtpCode.as_error_message());
        }

        let stored_otp_secret = database_admin_entry.otp_secret.unwrap();

        if otp::calculate_current_otp(&stored_otp_secret) != request_otp_code.clone().unwrap() {
            info!("A login with a wrong OTP code will be denied.");
            return HttpResponse::Forbidden().json(ErrorCode::WrongOtpCode.as_error_message());
        }
        setup_login_state(session, state, database_admin_entry.username);
        HttpResponse::Ok().json(MessageRes::from("The login was successful."))
    } else {
        // User has logged in successfully using password
        setup_login_state(session, state, database_admin_entry.username);
        HttpResponse::Ok().json(MessageRes::from("The login was successful."))
    }
}

/// Logs a user out.
#[utoipa::path(
    get,
    path = "/private/logout",
    responses((status = 301, description = "User has been logged out successfully and will be redirected.")),
    tags = ["private", "authentication"]
)]
async fn logout(session: Session, state: Data<AppState>) -> HttpResponse {
    session.purge();
    *state.username.lock().unwrap() = "".to_string();
    // TODO Login token should be Option<String> and set to None if user is logged out
    *state.login_token.lock().unwrap() =
        hex::encode((0..64).map(|_| rand::random::<u8>()).collect::<Vec<u8>>()).to_string();
    HttpResponse::Found()
        .append_header(("Location", "/"))
        .body("You will soon be redirected")
}

#[derive(Deserialize, ToSchema)]
struct OtpActivationReq {
    active: bool,
}

/// Disable or enable 2FA using OTP
#[utoipa::path(
    put,
    path = "/private/auth/useOtp",
    responses(
            (status = 200, description = "Status updated."),
    ),
    request_body = OtpActivationReq,
    tags = ["authentication", "private"]
)]
async fn otp_activation(json: web::Json<OtpActivationReq>) -> HttpResponse {
    use schema::Admin::dsl::*;
    let connection = &mut database::establish_connection();

    let status: bool = json.active;

    let status_update_execution = diesel::update(Admin)
        .set(use_otp.eq(status))
        .execute(connection);

    if let Err(update_error) = status_update_execution {
        return HttpResponse::InternalServerError()
            .json(ErrorCode::DatabaseReadFailed(update_error.to_string()).as_error_message());
    }

    if status {
        let secret = otp::generate_otp_secret();

        let secret_update_execution = diesel::update(Admin)
            .set(otp_secret.eq(Some(secret.clone())))
            .execute(connection);

        if let Err(update_error) = secret_update_execution {
            return HttpResponse::InternalServerError()
                .json(ErrorCode::DatabaseReadFailed(update_error.to_string()).as_error_message());
        }

        HttpResponse::Ok().body(secret)
    } else {
        let secret_reset_execution = diesel::update(Admin)
            .set(otp_secret.eq(None::<String>))
            .execute(connection);

        if let Err(update_error) = secret_reset_execution {
            return HttpResponse::InternalServerError()
                .json(ErrorCode::DatabaseReadFailed(update_error.to_string()).as_error_message());
        }

        HttpResponse::Ok().json(MessageRes::from("Updated OTP activation."))
    }
}

#[derive(Serialize, ToSchema)]
struct UseOtpRes {
    used: bool,
}

#[utoipa::path(
    get,
    path = "/public/auth/useOtp",
    responses((
            status = 200,
            body = UseOtpRes)),
    tags=["public", "authentication"]
)]
/// Does the user use OTP?
async fn use_otp(_state: Data<AppState>) -> HttpResponse {
    HttpResponse::Ok().json(UseOtpRes {
        used: get_administrator_account().use_otp,
    })
}

/// Verifies a given sudo password
#[utoipa::path(
    post,
    path = "/private/auth/sudo/verify",
    request_body = SudoPasswordReq,
    responses((status = 401, description = "Wrong sudo password"), (status = 200, description = "Correct sudo password")),
    tags=["private", "authentication"])]
async fn verify_sudo_password(json: web::Json<SudoPasswordReq>) -> HttpResponse {
    if !sudo::verify_password(json.sudo_password.clone()) {
        return HttpResponse::Unauthorized().json(ErrorCode::BadSudoPassword.as_error_message());
    }

    return HttpResponse::Ok().json(MessageRes::from("Sudo password is correct"));
}

// Package API

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
struct PackageDatabaseRes {
    installed: Vec<String>,
    available: Vec<String>,
    package_manager: Option<packages::PackageManagers>,
    updates: Option<Vec<String>>,
    last_database_update: Option<i64>, // The last database update expressed as seconds since the
                                       // UNIX epoch
}

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
struct PackageStatisticsRes {
    installed: usize,
    available: usize,
    package_manager: Option<packages::PackageManagers>,
    updates: Option<usize>,
    last_database_update: Option<i64>, // The last database update expressed as seconds since the
                                       // UNIX epoch
}

#[utoipa::path(
    get,
    path = "/private/packages/database",
    responses((status = 200, body = PackageDatabaseRes)),
    tags = ["private", "packages"]
)]
/// Package database
///
/// This includes the full names of installed packages. Updates can not be listed if the package
/// manager is PacMan.
async fn package_database() -> HttpResponse {
    use models::PackageAction;
    use schema::PackageActions::dsl::*;
    let connection = &mut establish_connection();

    let stored_last_database_update = match PackageActions
        .select(PackageAction::as_select())
        .first(connection)
    {
        Ok(v) => v.last_database_update,
        Err(database_error) => {
            return HttpResponse::InternalServerError().json(
                ErrorCode::DatabaseReadFailed(database_error.to_string()).as_error_message(),
            );
        }
    };

    let installed = packages::list_installed_packages();

    let available = match packages::list_available_packages() {
        Ok(packages) => packages,
        Err(_) => {
            return HttpResponse::InternalServerError()
                .json(ErrorCode::PackageManagerFailed.as_error_message());
        }
    };

    let updates = packages::list_updates().ok();

    HttpResponse::Ok().json(PackageDatabaseRes {
        installed,
        available,
        package_manager: packages::get_package_manager().ok(),
        updates,
        last_database_update: stored_last_database_update,
    })
}

#[utoipa::path(
    get,
    path = "/private/packages/statistics",
    responses((status = 200, body = PackageStatisticsRes)),
    tags = ["private", "packages"]
)]
/// Package database counts
async fn package_statistics() -> HttpResponse {
    use models::PackageAction;
    use schema::PackageActions::dsl::*;
    let connection = &mut establish_connection();

    let stored_last_database_update = match PackageActions
        .select(PackageAction::as_select())
        .first(connection)
    {
        Ok(v) => v.last_database_update,
        Err(database_error) => {
            return HttpResponse::InternalServerError().json(
                ErrorCode::DatabaseReadFailed(database_error.to_string()).as_error_message(),
            );
        }
    };

    let installed = packages::list_installed_packages().len();

    let available = match packages::list_available_packages() {
        Ok(packages) => packages.len(),
        Err(_) => {
            return HttpResponse::InternalServerError()
                .json(ErrorCode::PackageManagerFailed.as_error_message());
        }
    };

    let updates = match packages::list_updates() {
        Ok(packages) => Some(packages.len()),
        Err(_) => None,
    };

    HttpResponse::Ok().json(PackageStatisticsRes {
        installed,
        available,
        package_manager: packages::get_package_manager().ok(),
        updates,
        last_database_update: stored_last_database_update,
    })
}

#[derive(Serialize, ToSchema)]
struct OrphanedPackagesRes {
    packages: Vec<String>,
}

/// List of packages affected by auto-remove.
#[utoipa::path(
    get,
    path = "/private/packages/orphaned",
    responses((status = 200, body = OrphanedPackagesRes)),
    tags = ["private", "packages"]
)]
async fn orphaned_packages() -> HttpResponse {
    if let Ok(packages) = packages::list_orphaned_packages() {
        HttpResponse::Ok().json(OrphanedPackagesRes { packages })
    } else {
        HttpResponse::InternalServerError().json(ErrorCode::PackageManagerFailed.as_error_message())
    }
}

#[utoipa::path(
    post,
    path = "/private/packages/updateDatabase",
    request_body = SudoPasswordReq,
    responses((status = 200, description = "The update has been started, a Job ID in UUID format is provided.")),
    tags=["private", "packages", "responding_job"]
)]
/// Update package database
///
/// This action may take several minutes and is useful for discovering outdated packages.
/// The task is ran asynchronous to the rest of the program and a job id is given which can be
/// polled to get the state of the job.
async fn update_package_database(
    state: Data<AppState>,
    json: web::Json<SudoPasswordReq>,
) -> HttpResponse {
    use models::PackageAction;
    use schema::PackageActions::dsl::*;
    let job_id = Uuid::new_v4();

    state
        .background_jobs
        .lock()
        .unwrap()
        .insert(job_id, BackgroundTaskState::Pending);

    let block = actix_web::web::block(move || {
        let connection = &mut establish_connection();
        if let Ok(_) = packages::update_database(json.into_inner().sudo_password) {
            let updated_new_database_update = std::time::SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64;

            let new_package_action_row = PackageAction {
                last_database_update: Some(updated_new_database_update),
                key: 0_i32,
            };

            let package_action_time_update_execution = diesel::insert_into(PackageActions)
                .values(new_package_action_row)
                .on_conflict(key)
                .do_update()
                .set(last_database_update.eq(updated_new_database_update))
                .execute(connection);

            if let Err(database_error) = package_action_time_update_execution {
                state.background_jobs.lock().unwrap().insert(
                    job_id,
                    BackgroundTaskState::FailOutput(database_error.to_string()),
                )
            } else {
                state
                    .background_jobs
                    .lock()
                    .unwrap()
                    .insert(job_id, BackgroundTaskState::Success)
            }
        } else {
            state
                .background_jobs
                .lock()
                .unwrap()
                .insert(job_id, BackgroundTaskState::Fail)
        }
    });

    drop(block);

    HttpResponse::Ok().body(job_id.to_string())
}

/// Struct used for all actions performed on a package (install, remove, update...)
#[derive(Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
struct PackageActionReq {
    package_name: String,
    sudo_password: String,
}

#[utoipa::path(
    post,
    path = "/private/packages/install",
    request_body = PackageActionReq,
    responses((status = 200, description = "Job started with ID")),
    tags = ["private", "packages", "responding_job"]
)]
/// Install package
///
/// It requires the package name along side the sudo password in the request body.
/// This only works under apt, dnf and pacman. This request responds only with a job id.
async fn install_package(json: web::Json<PackageActionReq>, state: Data<AppState>) -> HttpResponse {
    let job_id = Uuid::new_v4();

    drop(actix_web::web::block(
        move || match packages::install_package(
            json.package_name.to_string(),
            json.sudo_password.to_string(),
        ) {
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
        },
    ));

    HttpResponse::Ok().body(job_id.to_string())
}

#[utoipa::path(
    post,
    path = "/private/packages/remove",
    request_body = PackageActionReq,
    responses((status = 200, description = "Job started with ID")),
    tags = ["private", "packages", "responding_job"]
)]
/// Remove package
///
/// It requires the package name along side the sudo password in the request body.
/// This only works under apt, dnf and pacman. This request responds only with a job id.
async fn remove_package(json: web::Json<PackageActionReq>, state: Data<AppState>) -> HttpResponse {
    let job_id = Uuid::new_v4();

    drop(actix_web::web::block(
        move || match packages::remove_package(
            json.package_name.to_string(),
            json.sudo_password.to_string(),
        ) {
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
        },
    ));

    HttpResponse::Ok().body(job_id.to_string())
}

#[utoipa::path(
    post,
    path = "/private/packages/update",
    request_body = PackageActionReq,
    responses((status = 200, description = "Job started with ID")),
    tags = ["private", "packages", "responding_job"]
)]
/// Update packages
///
/// It requires the package name along side the sudo password in the request body.
/// This only works under apt, dnf and pacman. This request responds only with a job id.
async fn update_package(state: Data<AppState>, json: web::Json<PackageActionReq>) -> HttpResponse {
    let job_id = Uuid::new_v4();

    drop(actix_web::web::block(
        move || match packages::update_package(
            json.package_name.to_string(),
            json.sudo_password.to_string(),
        ) {
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
        },
    ));

    HttpResponse::Ok().body(job_id.to_string())
}

#[utoipa::path(
    post,
    path = "/private/packages/updateAll",
    request_body = SudoPasswordReq,
    responses((status = 200, description = "Job started with ID")),
    tags = ["private", "packages", "responding_job"]
)]
/// Update all packages
///
/// It requires the package name along side the sudo password in the request body.
/// This only works under apt, dnf and pacman.
async fn update_all_packages(
    state: Data<AppState>,
    json: web::Json<SudoPasswordReq>,
) -> HttpResponse {
    let sudo_password = json.sudo_password.clone();
    let job_id = Uuid::new_v4();

    state
        .background_jobs
        .lock()
        .unwrap()
        .insert(job_id, BackgroundTaskState::Pending);

    drop(actix_web::web::block(
        move || match packages::update_all_packages(sudo_password.to_string()) {
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
        },
    ));

    HttpResponse::Ok().body(job_id.to_string())
}

#[utoipa::path(
    post,
    path = "/private/packages/removeOrphaned",
    request_body = SudoPasswordReq,
    responses((status = 200, description = "Job started with ID")),
    tags = ["private", "packages", "responding_job"]
)]
/// Auto-remove packages
async fn remove_orphaned_packages(
    json: web::Json<SudoPasswordReq>,
    state: Data<AppState>,
) -> HttpResponse {
    let sudo_password = json.sudo_password.clone();
    let job_id = Uuid::new_v4();

    state
        .background_jobs
        .lock()
        .unwrap()
        .insert(job_id, BackgroundTaskState::Pending);

    drop(actix_web::web::block(move || {
        let status = match packages::remove_orphaned_packages(sudo_password.to_string()) {
            Ok(_) => BackgroundTaskState::Success,
            Err(e) => BackgroundTaskState::FailOutput(e.to_string()),
        };
        state.background_jobs.lock().unwrap().insert(job_id, status)
    }));

    return HttpResponse::Ok().body(job_id.to_string());
}

#[utoipa::path(
    get,
    path = "/private/jobs/status/{id}",
    responses((status = 200, description = "The operation finished and may have provided results."),
    (status = 422, description = "The task failed and may have provided error details."),
    (status = 202, description = "The task is still pending."),
    (status = 404, description = "A job with this ID could not be found.")),
    tags = ["private", "jobs"],
    params(("id" = String, Path))
)]
/// Get the status of a job.
///
/// Jobs are used for tasks that would block the server and take a lot of time to finish, making it
/// unreasonable to keep the connection alive for that long. Some browser may even time out.
async fn fetch_job_status(state: Data<AppState>, path: web::Path<String>) -> HttpResponse {
    let requested_id = path.into_inner().to_string();
    let jobs = state.background_jobs.lock().unwrap().clone();
    let background_state = jobs.get(&uuid::Uuid::parse_str(&requested_id).unwrap());

    match background_state {
        Some(bs) => match bs {
            BackgroundTaskState::Success => {
                HttpResponse::Ok().json(MessageRes::from("Operation finished successfully."))
            }
            BackgroundTaskState::Fail => {
                HttpResponse::UnprocessableEntity().json(ErrorCode::TaskFailed.as_error_message())
            }
            BackgroundTaskState::SuccessOutput(s) => {
                HttpResponse::Ok().json(MessageRes::from(s.clone()))
            }
            BackgroundTaskState::FailOutput(f) => HttpResponse::UnprocessableEntity()
                .json(ErrorCode::TaskFailedWithDescription(f.to_string()).as_error_message()),
            BackgroundTaskState::Pending => {
                HttpResponse::Accepted().json(MessageRes::from("The task is still in work."))
            }
        },
        None => HttpResponse::NotFound().json(ErrorCode::NoSuchTask.as_error_message()),
    }
}

// Firewall API

#[derive(Serialize, ToSchema)]
struct HasUfwReq {
    has: bool,
}

#[utoipa::path(
    get,
    path = "/private/firewall/ufwPresent",
    responses((status = 200, body = HasUfwReq)),
    tags = ["private", "firewall"]
)]
/// Is UFW installed
async fn firewall_has_ufw() -> HttpResponse {
    let check = packages::list_installed_packages().contains(&String::from("ufw"));
    HttpResponse::Ok().json(HasUfwReq { has: check })
}

#[derive(Serialize, ToSchema)]
struct FirewallInformationRes {
    enabled: bool,
    rules: Vec<ufw::UfwRule>,
}

#[utoipa::path(
    post,
    path = "/private/firewall/rules",
    request_body = SudoPasswordReq,
    responses(
        (status = 200, body = FirewallInformationRes),
    ),
    tags = ["firewall", "private"])]
/// UFW status
async fn firewall_information(json: web::Json<SudoPasswordReq>) -> HttpResponse {
    let password = &json.sudo_password;

    match ufw::ufw_status(password.to_string()) {
        Ok(ufw_status) => {
            let enabled = ufw_status.0;
            let rules = ufw_status.1;

            HttpResponse::Ok().json(FirewallInformationRes { enabled, rules })
        }
        Err(err) => {
            error!("Executing UFW failed with error: {err}");
            HttpResponse::InternalServerError()
                .json(ErrorCode::UfwExecutionFailed(err).as_error_message())
        }
    }
}

#[derive(Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
struct SwitchUfwReq {
    sudo_password: String,
    enabled: bool,
}

#[utoipa::path(post,
    path = "/private/firewall/enabled",
    request_body = SwitchUfwReq,
    responses(
        (status = 200),
    ),
    tags = ["private", "firewall"])]
/// Set UFW to enabled or disabled
async fn switch_ufw(json: web::Json<SwitchUfwReq>) -> HttpResponse {
    let password = json.sudo_password.clone();
    let enabled = json.enabled;

    if enabled {
        match ufw::enable(password) {
            sudo::SudoExecutionResult::Success(status) => {
                if status == 0 {
                    return HttpResponse::Ok().json(MessageRes::from("UFW has been started."));
                } else {
                    return HttpResponse::InternalServerError()
                        .json(ErrorCode::UfwExecutionFailedWithStatus(status).as_error_message());
                }
            }
            sudo::SudoExecutionResult::ExecutionError(returned_error) => {
                return HttpResponse::InternalServerError()
                    .json(ErrorCode::UfwExecutionFailed(returned_error).as_error_message());
            }
            _ => {
                return HttpResponse::InternalServerError()
                    .json(ErrorCode::MissingSystemPermissions.as_error_message());
            }
        }
    } else {
        match ufw::disable(password) {
            sudo::SudoExecutionResult::Success(status) => {
                if status == 0 {
                    return HttpResponse::Ok().json(MessageRes::from("UFW has been stopped."));
                } else {
                    return HttpResponse::InternalServerError()
                        .json(ErrorCode::UfwExecutionFailedWithStatus(status).as_error_message());
                }
            }
            sudo::SudoExecutionResult::ExecutionError(returned_error) => {
                return HttpResponse::InternalServerError()
                    .json(ErrorCode::UfwExecutionFailed(returned_error).as_error_message());
            }
            _ => {
                return HttpResponse::InternalServerError()
                    .json(ErrorCode::MissingSystemPermissions.as_error_message());
            }
        }
    }
}

#[derive(Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
enum FirewallRulePortMode {
    Single,
    Range,
}

#[derive(Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
enum NetworkProtocolName {
    Tcp,
    Udp,
}

impl NetworkProtocolName {
    fn to_single_port(&self, port: u64) -> ufw::SinglePortProtocol {
        match *self {
            Self::Tcp => return ufw::SinglePortProtocol::Tcp(port),
            Self::Udp => return ufw::SinglePortProtocol::Udp(port),
        }
    }

    fn to_port_range(&self, left: u64, right: u64) -> ufw::PortRangeProtocol {
        match *self {
            Self::Tcp => return ufw::PortRangeProtocol::Tcp(left, right),
            Self::Udp => return ufw::PortRangeProtocol::Udp(left, right),
        }
    }
}

#[derive(Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
struct NewFirewallRuleReq {
    mode: FirewallRulePortMode,
    port: Option<u32>, // The port the rule applies to, this can be None if a range is used
    range: Option<(u32, u32)>, // The port range the rule applies to, this cane be None in a single
    // port is used
    network_protocol: NetworkProtocolName, // The network protocol used on the request (TCP/UDP)
    sender_address: Option<String>,
    action: FirewallAction,
    sudo_password: String,
}

#[utoipa::path(
    post,
    path = "/private/firewall/rule/new",
    responses((status = 200)),
    tags = ["private", "firewall"],
    request_body = NewFirewallRuleReq
)]
/// Create firewall rule.
async fn new_firewall_rule(json: web::Json<NewFirewallRuleReq>) -> HttpResponse {
    let mode = &json.mode;
    let sender = {
        if let Some(specific_address) = &json.sender_address {
            ufw::FirewallSender::Specific(specific_address.clone())
        } else {
            ufw::FirewallSender::Any
        }
    };

    let execution = match mode {
        FirewallRulePortMode::Single => ufw::new_rule_port(
            &json.sudo_password.clone(),
            json.network_protocol
                .to_single_port(json.port.unwrap() as u64),
            sender,
            json.action,
        ),

        FirewallRulePortMode::Range => {
            let range = &json.range.unwrap();
            ufw::new_rule_range(
                &json.sudo_password.clone(),
                json.network_protocol
                    .to_port_range(range.0 as u64, range.1 as u64),
                sender,
                json.action,
            )
        }
    };

    match execution {
        Ok(_) => HttpResponse::Ok().json(MessageRes::from("A new firewall rule was created.")),
        Err(e) => HttpResponse::InternalServerError()
            .json(ErrorCode::UfwExecutionFailed(e).as_error_message()),
    }
}
#[derive(Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
struct FirewallDeleteRuleReq {
    // The index of the rule to delete. UFW starts counting at 1, but common convention is 0, thus
    // this API also uses 0 to start counting.
    index: u32,
    sudo_password: String,
}

#[utoipa::path(
    post,
    path = "/private/firewall/rule/delete",
    tags = ["firewall", "private"],
    responses((status = 200)),
    request_body = FirewallDeleteRuleReq
)]
/// Delete firewall rule
async fn delete_firewall_rule(json: web::Json<FirewallDeleteRuleReq>) -> HttpResponse {
    let password = &json.sudo_password;

    match ufw::delete_rule(password.to_string(), json.index) {
        Ok(_) => HttpResponse::Ok().json(MessageRes::from("The rule has been deleted.")),
        Err(e) => HttpResponse::InternalServerError()
            .json(ErrorCode::UfwExecutionFailed(e).as_error_message()),
    }
}

// File API

#[utoipa::path(
    get,
    path = "/private/files/download",
    responses(
        (status = 200, body = Vec<u8>),
        (status = 404, description = "File not found")
    ),
    params(("path" = String, Query)),
    tags = ["private", "files"]
)]
/// Read file contents
async fn download_file(info: Query<SinglePath>) -> HttpResponse {
    let path_for_logging = info.path.to_string_lossy();

    info!("File download started for {}", path_for_logging);

    if path::Path::new(&info.path).exists() {
        error!("File {} does not exist.", path_for_logging);
        let f = fs::read(&info.path);
        match f {
            Ok(fh) => {
                HttpResponse::Ok().body(fh.bytes().map(|x| x.unwrap_or(0_u8)).collect::<Vec<u8>>())
            }
            Err(_) => {
                HttpResponse::InternalServerError().json(ErrorCode::FileError.as_error_message())
            }
        }
    } else {
        HttpResponse::NotFound().json(ErrorCode::FileDoesNotExist.as_error_message())
    }
}

#[derive(Serialize, ToSchema)]
struct FilesListRes {
    content: Vec<(String, FileSystemEntry)>,
}

/// List directory contents
#[utoipa::path(
    get,
    path = "/private/files/directoryReading",
    responses(
        (status = 200, body = FilesListRes),
        (status = 404, description = "Directory not found")
    ),
    params(("path" = String, Query)),
    tags = ["private", "files"]
)]
async fn files_list(info: Query<SinglePath>) -> HttpResponse {
    if !&info.path.exists() {
        return HttpResponse::NotFound().json(ErrorCode::DirectoryDoesNotExist.as_error_message());
    }

    match fs::read_dir(&info.path) {
        Ok(contents) => {
            let mut result: Vec<(String, FileSystemEntry)> = Vec::new();
            for e in contents {
                let e_unwrap = &e.unwrap();
                let file_name = &e_unwrap.file_name().into_string().unwrap();
                let is_file = e_unwrap.metadata().unwrap().is_file();
                let is_dir = e_unwrap.metadata().unwrap().is_dir();
                let is_symlink = e_unwrap.metadata().unwrap().is_symlink();

                if is_file {
                    result.push((file_name.to_string(), FileSystemEntry::File))
                } else if is_dir {
                    result.push((file_name.to_string(), FileSystemEntry::Directory))
                } else if is_symlink {
                    result.push((file_name.to_string(), FileSystemEntry::Symlink))
                } else {
                    result.push((file_name.to_string(), FileSystemEntry::Unknown))
                }
            }
            HttpResponse::Ok().json(FilesListRes { content: result })
        }
        Err(_) => {
            HttpResponse::InternalServerError().json(ErrorCode::DirectoryError.as_error_message())
        }
    }
}

/// Delete a file path
#[utoipa::path(
    post,
    path = "/private/files/delete",
    responses(
        (status = 200),
        (status = 404, description = "Path does not exist."),
        (status = 403, description = "The user lacks permissions to delete the file.")
    ),
    params(("path" = String, Query)),
    tags = ["private", "files"]
)]
async fn delete_file(info: Query<SinglePath>) -> HttpResponse {
    let file_path = &info.path;

    if !std::path::Path::new(&file_path).exists() {
        return HttpResponse::NotFound().json(ErrorCode::FileDoesNotExist.as_error_message());
    }

    let metadata = fs::metadata(&file_path).unwrap();
    let is_file = metadata.is_file();
    let is_dir = metadata.is_dir();
    let is_link = metadata.is_symlink();
    let has_permissions = !metadata.permissions().readonly();

    if (is_file || is_link) && has_permissions {
        match fs::remove_file(&file_path) {
            Ok(_) => HttpResponse::Ok().json(MessageRes::from("The file has been deleted.")),
            Err(_) => {
                HttpResponse::InternalServerError().json(ErrorCode::FileError.as_error_message())
            }
        }
    } else if is_dir && has_permissions {
        match fs::remove_dir_all(&file_path) {
            Ok(_) => HttpResponse::Ok().json(MessageRes::from("The directory has been deleted.")),
            Err(_) => HttpResponse::InternalServerError()
                .json(ErrorCode::DirectoryError.as_error_message()),
        }
    } else {
        HttpResponse::Forbidden().json(ErrorCode::MissingSystemPermissions.as_error_message())
    }
}

#[derive(Deserialize, ToSchema)]
struct MovePathReq {
    #[schema(value_type = String)]
    origin: PathBuf,
    #[schema(value_type = String)]
    destination: PathBuf,
}

/// Move file path
#[utoipa::path(
    post,
    path = "/private/files/move",
    responses(
        (status = 200),
        (status = 404, description = "Path does not exist."),
        (status = 403, description = "The user lacks permissions to rename the path.")
    ),
    request_body = inline(MovePathReq),
    tags = ["private", "files"]
)]
async fn move_path(json: web::Json<MovePathReq>) -> HttpResponse {
    if !json.origin.exists() || json.destination.exists() {
        HttpResponse::NotFound().json(ErrorCode::FileDoesNotExist.as_error_message());
    }

    let metadata = fs::metadata(&json.origin).unwrap();
    let has_permissions = !metadata.permissions().readonly();
    if has_permissions && !&json.destination.exists() {
        match fs::rename(&json.origin, &json.destination) {
            Ok(_) => HttpResponse::Ok().json(MessageRes::from("The directory has been renamed.")),
            Err(_) => {
                HttpResponse::InternalServerError().json(ErrorCode::FileError.as_error_message())
            }
        }
    } else {
        HttpResponse::Forbidden().json(ErrorCode::MissingSystemPermissions.as_error_message())
    }
}

/// Delete file
///
/// Overwrites the file with random data and removes it.
#[utoipa::path(
    post,
    path = "/private/files/burn",
    responses(
        (status = 200),
        (status = 404, description = "Path does not exist."),
        (status = 403, description = "The user lacks permissions to burn this file.")
    ),
    params(("path" = String, Query)),
    tags = ["private", "files"]
)]
async fn burn_file(info: Query<SinglePath>) -> HttpResponse {
    let path = &info.path;

    if path.is_dir() {
        return HttpResponse::UnprocessableEntity()
            .json(ErrorCode::DirectoryError.as_error_message());
    }

    if !path.exists() {
        return HttpResponse::NotFound().json(ErrorCode::FileDoesNotExist.as_error_message());
    }

    let metadata = fs::metadata(&path).unwrap();
    let has_permissions = !metadata.permissions().readonly();

    if !has_permissions {
        return HttpResponse::Forbidden()
            .json(ErrorCode::MissingSystemPermissions.as_error_message());
    }

    let size = metadata.len();
    let r_s = (0..size).map(|_| rand::random::<u8>()).collect::<Vec<u8>>();
    match fs::write(path.clone(), r_s) {
        Ok(_) => {
            let _ = fs::remove_file(&path);
            HttpResponse::Ok().json(MessageRes::from("The file has been burned."))
        }
        Err(_) => HttpResponse::InternalServerError().json(ErrorCode::FileError.as_error_message()),
    }
}

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
enum FileSystemEntry {
    File,
    Directory,
    Symlink,
    Unknown,
}

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
struct GetMetadataRes {
    permissions: i32,
    owner: NativeUser,
    size: u64,
    entry_type: FileSystemEntry,
    created: Option<u64>,
    modified: Option<u64>,
    filename: String,
    absolute_path: String,
}

fn recursive_size_of_directory(path: &PathBuf) -> u64 {
    let mut size = 0_u64;
    if let Ok(contents) = fs::read_dir(path) {
        contents.for_each(|f| {
            if f.is_err() {
                return;
            }
            if let Ok(metadata) = f.as_ref().unwrap().metadata() {
                if metadata.is_symlink() || metadata.is_file() {
                    size += metadata.size();
                } else {
                    size += recursive_size_of_directory(&f.unwrap().path());
                }
            }
        });
    }
    size
}

/// Metadata of path
#[utoipa::path(
    get,
    path = "/private/files/metadata",
    responses(
        (status = 200, body = GetMetadataRes),
        (status = 404, description = "Path does not exist."),
    ),
    params(("path" = String, Query)),
    tags = ["private", "files"]
)]
async fn get_file_metadata(info: Query<SinglePath>) -> HttpResponse {
    let constructed_path = &info.path;
    if !constructed_path.exists() {
        return HttpResponse::NotFound().json(ErrorCode::FileDoesNotExist.as_error_message());
    }

    let metadata = match fs::metadata(constructed_path.clone()) {
        Ok(m) => m,
        Err(_) => {
            return HttpResponse::InternalServerError()
                .json(ErrorCode::FileError.as_error_message());
        }
    };

    let permissions = metadata.permissions().mode() & 0o777;
    let permissions_octal_string = format!("{permissions:o}");
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

    let is_file = metadata.is_file();
    let is_dir = metadata.is_dir();
    let is_symlink = metadata.is_symlink();

    let size = if is_file || is_symlink {
        metadata.size()
    } else {
        recursive_size_of_directory(&constructed_path)
    };

    let owner = NativeUser::from_uid(metadata.uid()).unwrap();

    HttpResponse::Ok().json(GetMetadataRes {
        permissions: permissions_octal,
        size,
        owner,
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
    })
}

#[derive(MultipartForm, ToSchema)]
struct UploadFileForm {
    #[multipart(limit = "32 GiB")]
    #[schema(value_type = [u8])]
    file: TempFile,
    #[schema(value_type = String)]
    path: Text<String>,
}

/// Uploads a file to the host system.
#[utoipa::path(
    post,
    path = "/private/files/upload",
    responses((status = 200)),
    request_body(content_type = "mutlipart/form-data", content = UploadFileForm,description = "The path and file to upload."),
    tags = ["private", "files"]
)]
async fn upload_file(MultipartForm(form): MultipartForm<UploadFileForm>) -> HttpResponse {
    let intended_path = PathBuf::from(form.path.to_string());
    let filename = match form.file.file_name {
        Some(v) => v,
        None => {
            return HttpResponse::BadRequest().json(ErrorCode::InsufficientData.as_error_message());
        }
    };
    let intended_path = intended_path.join(filename);
    let temp_path = form.file.file.path();
    let cpy = fs::copy(temp_path, &intended_path);
    let unk = fs::remove_file(temp_path);

    if cpy.is_err() || unk.is_err() {
        HttpResponse::InternalServerError().json(ErrorCode::FileError.as_error_message());
    }

    HttpResponse::Ok().json(MessageRes::from("The upload has been finished."))
}

// Block Device API
#[derive(Serialize, ToSchema)]
struct DriveListRes {
    drives: Vec<drives::BlockDevice>,
}

#[utoipa::path(
    get,
    path = "/private/drives/list",
    responses((status = 200, body = DriveListRes)),
    tags = ["private", "drives"]
)]
/// List of connected block devices
async fn list_drives() -> HttpResponse {
    let drives_out = drives::device_list();

    let drives = match drives_out {
        Some(drives_list) => drives_list.blockdevices,
        None => {
            return HttpResponse::InternalServerError()
                .json(ErrorCode::BlockDeviceListingFailed.as_error_message());
        }
    };

    HttpResponse::Ok().json(DriveListRes { drives })
}

#[derive(Serialize, ToSchema)]
struct DriveInformationRes {
    metadata: drives::Drive,
    statistics: DriveUsageStatistics,
}

#[derive(Deserialize)]
struct DriveStatisticsQuery {
    drive: String,
}

/// Block device statistics
#[utoipa::path(
    get,
    path = "/private/drives/statistics",
    responses((status = 200, body = DriveInformationRes)),
    params(("drive" = String, Query)),
    tags = ["private", "drives"]
)]
async fn drive_information(info: Query<DriveStatisticsQuery>) -> HttpResponse {
    let drive = &info.drive;

    let metadata = match drives::drive_information(drive.clone()) {
        Some(m) => m,
        None => {
            return HttpResponse::InternalServerError()
                .json(ErrorCode::DriveMetadataFailed.as_error_message());
        }
    };

    let statistics = match drives::drive_statistics(drive.clone()) {
        Some(s) => s,
        None => {
            return HttpResponse::InternalServerError()
                .json(ErrorCode::DriveStatisticsFailed.as_error_message());
        }
    };

    HttpResponse::Ok().json(DriveInformationRes {
        metadata,
        statistics,
    })
}

// Vault API

#[derive(Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
struct VaultConfigurationReq {
    key: Option<String>,
    old_key: Option<String>,
    new_key: Option<String>,
}

fn get_vault_enabled() -> bool {
    use models::Configurations;
    use schema::Configuration::dsl::*;

    Configuration
        .select(Configurations::as_select())
        .first(&mut establish_connection())
        .unwrap()
        .vault_enabled
}

/// Configure Vault
///
/// This creates a new vault instance. If one is already found on the system, only the password is
/// updated to a new one.
#[utoipa::path(
    post,
    path = "/private/vault/configuration",
    request_body = VaultConfigurationReq,
    responses((status = 200), (status = 403, description = "The new old key could not be used to decrypt.")),
    tags = ["private", "vault"]
)]
// NOTE The following could use less indentations
async fn vault_configure(json: web::Json<VaultConfigurationReq>) -> HttpResponse {
    use schema::Configuration::dsl::*;
    let vault_path = path::Path::new(&dirs::home_dir().unwrap())
        .join(".local")
        .join("share")
        .join("zentrox")
        .join("vault_directory");

    let connection = &mut establish_connection();

    if !get_vault_enabled() && !vault_path.exists() {
        if json.key.is_none() {
            return HttpResponse::BadRequest().json(ErrorCode::InsufficientData.as_error_message());
        }

        let key = &json.key.clone().unwrap();

        match fs::create_dir_all(&vault_path) {
            Ok(_) => {}
            Err(e) => {
                error!("The vault directory could not be created due to the following error: {e}");
                return HttpResponse::InternalServerError()
                    .json(ErrorCode::DirectoryError.as_error_message());
            }
        };

        let vault_file_contents = format!(
            "Vault created by {} at UNIX {}.",
            whoami::username(),
            SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("Time went backwards.")
                .as_millis()
        );

        match fs::write(vault_path.join(".vault"), vault_file_contents) {
            Ok(_) => {}
            Err(e) => {
                error!("Creating the initial vault file failed with error: {e}");
                return HttpResponse::InternalServerError()
                    .json(ErrorCode::FileError.as_error_message());
            }
        }

        vault::encrypt_file(vault_path.join(".vault"), key);

        let database_vault_enable_update = diesel::update(Configuration)
            .set(vault_enabled.eq(true))
            .execute(connection);

        if let Err(database_error) = database_vault_enable_update {
            return HttpResponse::InternalServerError().json(
                ErrorCode::DatabaseUpdateFailed(database_error.to_string()).as_error_message(),
            );
        }
    } else if json.old_key.is_some() && json.new_key.is_some() {
        let old_key = json.old_key.clone().unwrap();
        let new_key = json.new_key.clone().unwrap();
        match vault::decrypt_file(vault_path.join(".vault"), &old_key.to_string()) {
            Some(_) => vault::encrypt_file(vault_path.join(".vault"), &old_key.to_string()),
            None => {
                return HttpResponse::Forbidden()
                    .json(ErrorCode::MissingVaultPermissions.as_error_message());
            }
        };

        match vault::decrypt_directory(vault_path.clone(), &old_key) {
            Ok(_) => {}
            Err(_) => {
                return HttpResponse::Forbidden()
                    .json(ErrorCode::MissingVaultPermissions.as_error_message());
            }
        };

        if let Err(_) = vault::encrypt_directory(vault_path.clone(), &new_key) {
            return HttpResponse::InternalServerError()
                .json(ErrorCode::EncryptionFailed.as_error_message());
        }
    } else {
        return HttpResponse::BadRequest().json(ErrorCode::InsufficientData.as_error_message());
    }

    HttpResponse::Ok().json(MessageRes::from("The vault has been configured."))
}

#[utoipa::path(get,
    path = "/private/vault/enabled",
    responses((status = 200, body = String)),
    tags = ["private", "vault"]
)]
/// Is vault ready for use
async fn is_vault_configured() -> HttpResponse {
    return HttpResponse::Ok().body(get_vault_enabled().to_string());
}

// Vault Tree

#[derive(Serialize, ToSchema)]
struct VaultFsPathRes {
    tree: Vec<String>,
}

#[derive(Deserialize, ToSchema)]
struct VaultKeyReq {
    key: String,
}

/// List all paths in Zentrox Vault in a one-dimensional vector. This also decrypts the filenames
/// and folder names of the entries. A directory path always ends with /. The path never starts
/// with /, thus root is "".
fn list_paths(directory: PathBuf, key: String) -> Vec<String> {
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
        // TODO ^ Check if this is error prone

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
            for e in list_paths(entry_unwrap.path(), key.clone()) {
                paths.push(e); // Path of the file, while ignoring the path until (but still including) vault_directory.
            }
        }
    }
    paths
}

/// Vault paths as flat path tree
#[utoipa::path(
    post,
    path = "/private/vault/tree",
    request_body = VaultKeyReq,
    responses((status = 200, body = VaultFsPathRes), (status = 403, description = "The new key could not be used to decrypt.")),
    tags = ["private", "vault"]
)]
async fn vault_tree(json: web::Json<VaultKeyReq>) -> HttpResponse {
    if get_vault_enabled() {
        let vault_path = path::Path::new(&dirs::home_dir().unwrap())
            .join(".local")
            .join("share")
            .join("zentrox")
            .join("vault_directory");

        let key = &json.key;

        match vault::decrypt_file(vault_path.join(".vault"), &key.to_string()) {
            Some(_) => vault::encrypt_file(vault_path.join(".vault"), &key.to_string()),
            None => {
                return HttpResponse::Forbidden()
                    .json(ErrorCode::MissingVaultPermissions.as_error_message());
            }
        };

        let paths = list_paths(vault_path, key.to_string());

        HttpResponse::Ok().json(VaultFsPathRes { tree: paths })
    } else {
        HttpResponse::BadRequest().json(ErrorCode::VaultUnconfigured)
    }
}

// Delete vault file
#[derive(Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
struct VaultDeleteReq {
    delete_path: String,
    key: String,
}

/// Burn vault file
///
/// Burns a file in Zentrox vault by overwriting it with random data
#[utoipa::path(
    post,
    path = "/private/vault/delete",
    request_body = VaultDeleteReq,
    responses((status = 200, description = "The file is being deleted. This may take several seconds, thus a Job UUID is provided.")),
    tags = ["private", "vault", "responding_job"]
)]
async fn delete_vault_file(state: Data<AppState>, json: web::Json<VaultDeleteReq>) -> HttpResponse {
    let uuid = Uuid::new_v4();
    state
        .background_jobs
        .lock()
        .unwrap()
        .insert(uuid, BackgroundTaskState::Pending);

    drop(actix_web::web::block(move || {
        let sent_path = &json.delete_path;
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
    }));

    HttpResponse::Ok().body(uuid.to_string())
}

// Create new folder in vault
#[derive(Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
struct VaultNewFolderReq {
    directory_name: String,
    key: String,
}

/// Create new directory in vault.
#[utoipa::path(
    post,
    path = "/private/vault/directory",
    request_body = VaultNewFolderReq,
    responses((status = 200), (status = 500, description = "Directory name too long.")),
    tags = ["private", "vault"]
)]
async fn vault_new_folder(json: web::Json<VaultNewFolderReq>) -> HttpResponse {
    let sent_path = &json.directory_name;

    if sent_path.split("/").last().unwrap().len() > 64 {
        return HttpResponse::InternalServerError()
            .json(ErrorCode::VaultPathTooLong.as_error_message());
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
        return HttpResponse::Conflict().json(ErrorCode::DirectoryError);
    }

    let _ = fs::create_dir(&path);
    HttpResponse::Ok().json(MessageRes::from("A new directory has been created."))
}

// Upload vault file

#[derive(MultipartForm, ToSchema)]
struct VaultUploadForm {
    #[schema(value_type = String)]
    key: Text<String>,
    #[schema(value_type = String)]
    path: Text<String>,

    #[multipart(limit = "32 GiB")]
    #[schema(value_type = Vec<u8>)]
    file: TempFile,
}

/// Upload file to vault.
#[utoipa::path(
    post,
    path = "/private/vault/file",
    request_body(content_type = "mutlipart/form-data", content = UploadFileForm,description = "The path and file to upload."),
    responses((status = 200), (status = 409, description = "File with same path already exists.")),
    tags = ["private", "vault"]
    )]
async fn upload_vault(MultipartForm(form): MultipartForm<VaultUploadForm>) -> HttpResponse {
    let file_name = form
        .file
        .file_name
        .unwrap_or_else(|| "vault_default_file".to_string())
        .replace("..", "")
        .replace("/", "");

    let key = &form.key;

    if file_name == ".vault" {
        return HttpResponse::BadRequest().json(ErrorCode::FileError);
    }

    let base_path = path::Path::new(&dirs::home_dir().unwrap())
        .join(".local")
        .join("share")
        .join("zentrox")
        .join("vault_directory");

    let encrypted_path = form
        .path
        .split('/')
        .filter(|x| !x.is_empty()) // Filter out empty path entries
        .map(|x| vault::encrypt_string_hash(x.to_string(), key).unwrap())
        .collect::<Vec<String>>()
        .join("/");

    let in_vault_path = base_path
        .join(encrypted_path)
        .join(vault::encrypt_string_hash(file_name.to_string(), key).unwrap());

    if in_vault_path.exists() {
        return HttpResponse::Conflict().json(ErrorCode::FileError);
    }

    let tmp_file_path = form.file.file.path().to_owned();
    let _ = fs::copy(&tmp_file_path, &in_vault_path);

    let _ = tokio::fs::copy(&tmp_file_path, &in_vault_path).await;

    let block = actix_web::web::block(|| {
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

    drop(block);

    vault::encrypt_file(in_vault_path, key);

    HttpResponse::Ok().json(MessageRes::from("The upload has been finished."))
}

// Rename file/folder in vault
#[derive(Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
struct VaultRenameReq {
    origin: String,
    target: String,
    key: String,
}

/// Move a path in vault
///
/// The new name will be encrypted. The original name must be provided in non-encrypted form.
#[utoipa::path(
    post,
    path = "/private/vault/move",
    request_body = VaultRenameReq,
    responses((status = 404, description = "Origin not found."), (status = 400, description = "File can not be moved."), (status = 409, description = "Target path already exists.")),
    tags = ["private", "vault"]
)]
async fn rename_vault_file(json: web::Json<VaultRenameReq>) -> HttpResponse {
    let sent_path = &json.origin;

    if sent_path == "/.vault" {
        return HttpResponse::BadRequest().json(ErrorCode::FileError);
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

    if !path.exists() {
        return HttpResponse::NotFound().json(ErrorCode::FileDoesNotExist.as_error_message());
    }

    let sent_new_path = &json.target;
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
        HttpResponse::Conflict().json(ErrorCode::FileError);
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

    HttpResponse::Ok().json(MessageRes::from("The file has been moved."))
}

// Download vault file
#[derive(Deserialize, ToSchema)]
struct VaultFileDownloadReq {
    key: String,
    path: String,
}

fn append_file_extension(p: PathBuf, extension: &str) -> PathBuf {
    let mut stringified = p.to_string_lossy().to_string();
    stringified.push_str(extension);
    PathBuf::from(stringified)
}

/// Download vault file.
///
/// This requires a password from the user in order to decrypt the file server side.
/// The file will not be provided in encrypted form.
#[utoipa::path(
    post,
    path = "/private/vault/download",
    request_body = VaultFileDownloadReq,
    responses((status = 200, content_type = "application/octet-stream"), (status = 400, description = "File can not be read."), (status = 404, description = "File does not exist.")),
    tags = ["private", "vault"]
)]
async fn vault_file_download(json: web::Json<VaultFileDownloadReq>) -> HttpResponse {
    let sent_path = &json.path;
    let key = &json.key;

    if sent_path == "/.vault" {
        HttpResponse::BadRequest().json(ErrorCode::FileError);
    }

    let path = dirs::home_dir()
        .unwrap()
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

    let temporary_decrypted_file = append_file_extension(path.clone(), ".dec");

    let _ = fs::copy(path, temporary_decrypted_file.clone());

    vault::decrypt_file(temporary_decrypted_file.clone(), key);
    if temporary_decrypted_file.exists() {
        let f = fs::read(&temporary_decrypted_file);
        match f {
            Ok(fh) => {
                let data = fh.bytes().map(|x| x.unwrap_or(0_u8)).collect::<Vec<u8>>();
                let _ = vault::burn_file(temporary_decrypted_file.clone());
                let _ = fs::remove_file(temporary_decrypted_file.clone());
                HttpResponse::Ok().body(data)
            }
            Err(_) => {
                HttpResponse::InternalServerError().json(ErrorCode::FileError.as_error_message())
            }
        }
    } else {
        HttpResponse::NotFound().json(ErrorCode::FileDoesNotExist.as_error_message())
    }
}

// Show Robots.txt
#[get("/robots.txt")]
/// Return the robots.txt file to prevent search engines from indexing this server.
async fn robots_txt() -> HttpResponse {
    HttpResponse::Ok().body(include_str!("../../assets/robots.txt"))
}

// Upload TLS cert

#[derive(MultipartForm, ToSchema)]
struct TlsUploadForm {
    #[multipart(limit = "1GB")]
    #[schema(value_type = Vec<u8>)]
    file: TempFile,
}

/// Upload new TLS certificate
///
/// This certificate is used to encrypt the connection between your browser and Zentrox.
/// The name of then new certificate is stored in the configuration file.
#[utoipa::path(
    post,
    path = "/private/tls/upload",
    request_body(content = TlsUploadForm, content_type = "multipart/form-data"),
    responses((status = 200)),
    tags = ["private", "tls"]
)]
async fn upload_tls(MultipartForm(form): MultipartForm<TlsUploadForm>) -> HttpResponse {
    use schema::Configuration::dsl::*;

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

    let database_update_execution = diesel::update(Configuration)
        .set(tls_cert.eq(&file_name))
        .execute(&mut establish_connection());

    if let Err(database_error) = database_update_execution {
        return HttpResponse::InternalServerError()
            .json(ErrorCode::DatabaseUpdateFailed(database_error.to_string()).as_error_message());
    }

    let tmp_file_path = form.file.file.path().to_owned();
    let _ = fs::copy(&tmp_file_path, &base_path);

    match fs::remove_file(&tmp_file_path) {
        Ok(_) => {}
        Err(_) => {
            error!(
                "Unable to remove temporary file at {}",
                tmp_file_path.to_string_lossy()
            );
            return HttpResponse::InternalServerError()
                .json(ErrorCode::FileError.as_error_message());
        }
    };

    HttpResponse::Ok().json(MessageRes::from("Certificate uploaded and stored."))
}

#[derive(Serialize, ToSchema)]
struct CertNameRes {
    name: String,
}

/// Name of active certificate.
#[utoipa::path(
    get,
    path = "/private/tls/name",
    responses((status = 200, body = CertNameRes)),
    tags = ["private", "tls"]
)]
async fn cert_names() -> HttpResponse {
    use models::Configurations;
    use schema::Configuration::dsl::*;
    let name = Configuration
        .select(Configurations::as_select())
        .first(&mut establish_connection())
        .unwrap()
        .tls_cert;

    HttpResponse::Ok().json(CertNameRes { name })
}

// Power Off System
/// Powers off the system.
#[utoipa::path(
    post,
    path = "/private/power/off",
    request_body = SudoPasswordReq,
    responses((status = 200)),
    tags = ["private", "power"]
)]
async fn power_off(json: web::Json<SudoPasswordReq>) -> HttpResponse {
    let e =
        sudo::SwitchedUserCommand::new(json.sudo_password.clone(), "poweroff".to_string()).spawn();

    if let sudo::SudoExecutionResult::Success(_) = e {
        HttpResponse::Ok().json(MessageRes::from("The computer is shutting down."))
    } else {
        HttpResponse::InternalServerError().json(ErrorCode::PowerOffFailed.as_error_message())
    }
}

// Account Details
#[derive(Serialize, ToSchema)]
struct AccountDetailsRes {
    username: String,
}

/// Admin account details
#[utoipa::path(
    get,
    path = "/private/account/details",
    responses((status = 200, body = AccountDetailsRes)),
    tags = ["private", "account"]
)]
async fn account_details(state: Data<AppState>) -> HttpResponse {
    let state_username = match state.username.lock() {
        Ok(v) => v,
        Err(e) => e.into_inner(),
    };

    HttpResponse::Ok().json(AccountDetailsRes {
        username: state_username.to_string(),
    })
}

#[derive(Deserialize, ToSchema)]
struct UpdateAccountReq {
    password: String,
    username: String,
}

/// Update account details
#[utoipa::path(
    post,
    path = "/private/account/details",
    request_body = UpdateAccountReq,
    responses((status = 200)),
    tags = ["private", "account"]
)]
async fn update_account_details(json: web::Json<UpdateAccountReq>) -> HttpResponse {
    use schema::Admin::dsl::*;
    let connection = &mut establish_connection();

    let request_password = &json.password;
    let request_username = &json.username;

    let current_ts = std::time::SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64;

    if !request_password.is_empty() {
        let hashed_request_password =
            hex::encode(crypto_utils::argon2_derive_key(request_password).unwrap()).to_string();

        let password_execution = diesel::update(Admin)
            .set((
                password_hash.eq(hashed_request_password),
                updated_at.eq(current_ts),
            ))
            .execute(connection);

        if let Err(database_error) = password_execution {
            return HttpResponse::InternalServerError().json(
                ErrorCode::DatabaseUpdateFailed(database_error.to_string()).as_error_message(),
            );
        }
    }

    if !request_username.is_empty() {
        let username_execution = diesel::update(Admin)
            .set(username.eq(request_username))
            .execute(connection);

        if let Err(database_error) = username_execution {
            return HttpResponse::InternalServerError().json(
                ErrorCode::DatabaseUpdateFailed(database_error.to_string()).as_error_message(),
            );
        }
    }

    HttpResponse::Ok().json(MessageRes::from("Account details have been updated."))
}

/// Admin profile picture
#[utoipa::path(
    get,
    path = "/private/account/profilePicture",
    responses((status = 200, body = &[u8], content_type = "image/")),
    tags = ["private", "account"]
)]
async fn profile_picture() -> HttpResponse {
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
        Err(_) => HttpResponse::NotFound().json(ErrorCode::FileDoesNotExist.as_error_message()),
    }
}

#[derive(MultipartForm, ToSchema)]
struct ProfilePictureUploadForm {
    #[multipart(limit = "2MB")]
    #[schema(value_type = Vec<u8>)]
    file: TempFile,
}

/// Upload admin profile picture.
///
/// The picture may not be larger than 2MB in order to keep loading time down.
#[utoipa::path(
    post,
    path = "/private/account/profilePicture",
    request_body = ProfilePictureUploadForm,
    responses((status = 200)),
    tags = ["private", "account"]
)]
async fn upload_profile_picture(
    MultipartForm(form): MultipartForm<ProfilePictureUploadForm>,
) -> HttpResponse {
    let profile_picture_path = path::Path::new(&dirs::home_dir().unwrap())
        .join(".local")
        .join("share")
        .join("zentrox")
        .join("profile.png");

    let tmp_file_path = form.file.file.path().to_owned();
    let _ = fs::copy(&tmp_file_path, &profile_picture_path);

    match fs::remove_file(&tmp_file_path) {
        Ok(_) => HttpResponse::Ok().json(MessageRes::from("The profile picture has been updated.")),
        Err(_) => {
            error!(
                "Remove a temporary file at {} failed.",
                tmp_file_path.to_string_lossy()
            );
            HttpResponse::InternalServerError().json(ErrorCode::FileError.as_error_message())
        }
    }
}

#[derive(Serialize, ToSchema)]
struct MessagesLogRes {
    users: Vec<NativeUser>,
    logs: Vec<QuickJournalEntry>,
}

#[derive(Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
struct LogReq {
    sudo_password: String,
    since: u64,
    until: u64,
}

/// Journalctl log in certain time frame
#[utoipa::path(
    post,
    path = "/private/logs",
    responses((status = 200, body = MessagesLogRes)),
    request_body = LogReq,
    tags = ["private", "logs"]
)]
async fn logs_request(json: web::Json<LogReq>) -> HttpResponse {
    let since = &json.since;
    let until = &json.until;

    match logs::log_messages(json.sudo_password.clone(), since / 1000, until / 1000) {
        Ok(messages) => {
            let mut users = vec![];
            let messages_minified: Vec<QuickJournalEntry> = messages
                .iter()
                .map(|m| {
                    let user = &m.user;

                    if let Some(valued_user) = user {
                        if !users.contains(valued_user) {
                            users.push(valued_user.clone())
                        }
                    }

                    m.clone().as_quick_journal_entry()
                })
                .collect();

            return HttpResponse::Ok().json(MessagesLogRes {
                users,
                logs: messages_minified,
            });
        }
        Err(_) => {
            error!("Getting logs failed.");
            HttpResponse::InternalServerError()
                .json(ErrorCode::LogFetchingFailed.as_error_message())
        }
    }
}

// Media center endpoints

fn parse_range(range: actix_web::http::header::HeaderValue) -> (usize, Option<usize>) {
    let range_str = range.to_str().ok().unwrap(); // Safely convert to str, return None if failed
    let range_separated_clear = range_str.replace("bytes=", "");
    let range_separated: Vec<&str> = range_separated_clear.split('-').collect(); // Split the range

    // Parse the start and end values safely
    let start = range_separated.first().unwrap().parse::<usize>().unwrap();

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

/// Checks if a given path can be accessed using any enabled media source.
fn is_media_path_whitelisted(l: Vec<MediaSource>, p: PathBuf) -> bool {
    let mut r = false;

    if !p.exists() {
        return false;
    }

    l.iter().for_each(|le| {
        if !r
            && p.canonicalize()
                .unwrap()
                .starts_with(PathBuf::from(&le.directory_path).canonicalize().unwrap())
            && le.enabled
        {
            r = true
        }
    });
    r
}

/// Media file
#[utoipa::path(
    get,
    path = "/private/media/download",
    responses((status = 200, description = "Binary media file", content_type = "application/octet-stream"), (status = 404, description = "File not found."), (status = 416), (status = 403, description = "Media center may be disabled.")),
    tags = ["media", "private"]
)]
async fn media_request(info: Query<SinglePath>, req: HttpRequest) -> HttpResponse {
    use models::MediaSource;
    use models::RecommendedMediaEntry;
    use schema::MediaSources::dsl::*;
    use schema::RecommendedMedia::dsl::*;

    let connection = &mut establish_connection();

    // Determine the requested file path
    let requested_file_path = &info.path;

    let current_ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("Time went backwards.");

    if !requested_file_path.exists() {
        return HttpResponse::NotFound().json(ErrorCode::FileDoesNotExist.as_error_message());
    }

    let database_insert_execution = diesel::insert_into(RecommendedMedia)
        .values(RecommendedMediaEntry {
            file_path: requested_file_path.to_string_lossy().to_string(),
            last_view: current_ts.as_millis() as i64,
        })
        .on_conflict(file_path)
        .do_update()
        .set(last_view.eq(current_ts.as_millis() as i64))
        .execute(connection);

    if let Err(database_error) = database_insert_execution {
        return HttpResponse::InternalServerError()
            .json(ErrorCode::DatabaseInsertFailed(database_error.to_string()));
    }

    // Implement HTTP Ranges
    let headers = req.headers();
    let range = headers.get(actix_web::http::header::RANGE);

    let mime = mime::guess_mime(requested_file_path.to_path_buf());

    let whitelist_vector: Vec<MediaSource> = MediaSources
        .select(MediaSource::as_select())
        .get_results(connection)
        .unwrap();

    if !is_media_path_whitelisted(
        whitelist_vector,
        fs::canonicalize(requested_file_path.clone()).unwrap(),
    ) {
        return HttpResponse::Forbidden().json(ErrorCode::MissingApiPermissions.as_error_message());
    }

    if requested_file_path.is_dir() {
        return HttpResponse::BadRequest().json(ErrorCode::FileError.as_error_message());
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
                .body(fs::read(requested_file_path).unwrap())
        }
        Some(e) => {
            let byte_range = parse_range(e.clone());
            let file = File::open(&requested_file_path).unwrap();
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
                    .json(ErrorCode::LeftRangeTooHigh.as_error_message());
            }

            if let Some(right_limit) = byte_range.1 {
                if right_limit > filesize {
                    return HttpResponse::RangeNotSatisfiable()
                        .json(ErrorCode::RightRangeTooHigh.as_error_message());
                }
            }

            let buffer_length = byte_range.1.unwrap_or(filesize) - byte_range.0;
            let _ = reader.seek(SeekFrom::Start(byte_range.0 as u64));
            let mut buf = vec![0; buffer_length]; // A buffer with the length buffer_length
            reader.read_exact(&mut buf).unwrap();

            HttpResponse::PartialContent()
                .insert_header(header::ContentEncoding::Identity)
                .insert_header((header::ACCEPT_RANGES, "bytes"))
                .insert_header((
                    header::CONTENT_DISPOSITION,
                    format!(
                        "inline; filename=\"{}\"",
                        &requested_file_path.file_name().unwrap().to_str().unwrap()
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
                .insert_header((header::ACCESS_CONTROL_ALLOW_HEADERS, "Range"))
                .insert_header((header::CONTENT_LENGTH, buf.len()))
                .insert_header((
                    header::CONTENT_TYPE,
                    mime.unwrap_or("application/octet-stream".to_string()),
                ))
                .body(buf)
        }
    }
}

#[derive(Deserialize, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
struct MediaSourcesSchema {
    locations: Vec<models::MediaSource>,
}

#[utoipa::path(
    post,
    path = "/private/media/sources",
    request_body = MediaSourcesSchema,
    responses((status = 200)),
    tags = ["media", "private"]
)]
/// Update media sources
///
/// Media sources control what content is shown to the user in Media Center and to which files the
/// user has access.
async fn update_media_source_list(json: web::Json<MediaSourcesSchema>) -> HttpResponse {
    use models::MediaSource;
    use schema::MediaSources::dsl::*;

    let connection = &mut establish_connection();

    let locations = &json.locations;

    // The frontend only sends an updated array of all resources.
    // It is easier to truncate the entire table and then rewrite its' contents.

    if let Err(database_error) = diesel::delete(MediaSources).execute(connection) {
        return HttpResponse::InternalServerError().json(
            ErrorCode::DatabaseTruncateFailed(database_error.to_string()).as_error_message(),
        );
    }

    let deserialized_locations: Vec<&MediaSource> = locations
        .iter()
        .filter(|&e| PathBuf::from(&e.directory_path).exists())
        .collect();

    for location in deserialized_locations {
        let database_update_execution = diesel::insert_into(MediaSources)
            .values(location)
            .on_conflict(directory_path)
            .do_update()
            .set(location)
            .execute(connection);

        if let Err(database_error) = database_update_execution {
            return HttpResponse::InternalServerError().json(
                ErrorCode::DatabaseUpdateFailed(database_error.to_string()).as_error_message(),
            );
        }
    }

    HttpResponse::Ok().json(MessageRes::from("The media source have been updated."))
}

#[utoipa::path(
    get,
    path = "/private/media/sources",
    responses((status = 200, body = MediaSourcesSchema)),
    tags = ["media", "private"]
)]
/// List of media sources.
///
/// See [`update_media_source_list`] for reference.
async fn get_media_source_list() -> HttpResponse {
    use models::MediaSource;
    use schema::MediaSources::dsl::*;

    let locations: Vec<MediaSource> = MediaSources
        .select(MediaSource::as_select())
        .get_results(&mut establish_connection())
        .unwrap();

    HttpResponse::Ok().json(MediaSourcesSchema { locations })
}

#[derive(Serialize, ToSchema)]
struct MediaListRes {
    media: Vec<models::MediaEntry>,
}

#[utoipa::path(
    get,
    path = "/private/media/files",
    responses((status = 200, body = MediaListRes), (status = 403, description = "Media center may be disabled.")),
    tags = ["media", "private"]
)]
/// List of media files
///
/// This list is controlled by the active media sources.
async fn get_media_list() -> HttpResponse {
    use schema::Media::dsl::*;
    use schema::MediaSources::dsl::*;

    use models::MediaEntry;

    let connection = &mut establish_connection();

    let sources: Vec<PathBuf> = MediaSources
        .select(MediaSource::as_select())
        .get_results(connection)
        .unwrap()
        .into_iter()
        .filter(|source| source.enabled)
        .map(|source| PathBuf::from(source.directory_path))
        .filter(|path| path.exists())
        .collect();

    let mut all_media_file_paths: Vec<PathBuf> = vec![];

    for source in sources {
        let source_specific_contents = visit_dirs(source).unwrap();
        source_specific_contents
            .map(|file| file.path())
            .filter(|path| path.is_file())
            .for_each(|path| all_media_file_paths.push(path));
    }

    let mut media_metadata = Media
        .select(MediaEntry::as_select())
        .get_results(connection)
        .unwrap()
        .into_iter();

    let mut completed_media_entries: Vec<MediaEntry> = Vec::new();

    for media_file_path in all_media_file_paths {
        let search = media_metadata
            .find(|entry: &MediaEntry| PathBuf::from(entry.file_path.clone()) == media_file_path);
        if let Some(defined_metadata) = search {
            completed_media_entries.push(defined_metadata);
        } else {
            completed_media_entries.push(MediaEntry::default_with_file_path(media_file_path));
        }
    }

    return HttpResponse::Ok().json(MediaListRes {
        media: completed_media_entries,
    });
}

/// Media cover
///
/// Only media covers that are in an active media source will be shown.
#[utoipa::path(get, path = "/private/media/cover", responses((status = 200, content_type = "image/"), (status = 404, description = "Media not found.")), tags = ["media", "private"], params(("path" = String, Query)))]
async fn get_cover(info: Query<SinglePath>) -> HttpResponse {
    use models::MediaSource;
    use schema::MediaSources::dsl::*;

    let sources: Vec<MediaSource> = MediaSources
        .select(MediaSource::as_select())
        .get_results(&mut establish_connection())
        .unwrap();

    let cover_uri = &info.path;

    if cover_uri == &PathBuf::from("/music") {
        let cover = include_str!("../../assets/music_default.svg");
        HttpResponse::Ok()
            .insert_header((header::CONTENT_TYPE, "image/svg+xml".to_string()))
            .body(cover.bytes().collect::<Vec<u8>>())
    } else {
        if !cover_uri.exists() {
            return HttpResponse::NotFound().json(ErrorCode::FileDoesNotExist.as_error_message());
        }

        let cover_path = cover_uri
            .canonicalize()
            .expect("Canonicalizing a path failed.");

        let allowed_cover_file_extensions = ["png", "jpg", "jpeg", "webp", "gif", "tiff"];
        if !allowed_cover_file_extensions
            .contains(&cover_path.extension().unwrap().to_str().unwrap())
        {
            return HttpResponse::UnsupportedMediaType()
                .json(ErrorCode::ProtectedExtension.as_error_message());
        }

        if !is_media_path_whitelisted(sources, cover_uri.to_path_buf()) {
            HttpResponse::Forbidden().json(ErrorCode::MissingApiPermissions.as_error_message())
        } else {
            let fh = fs::read(cover_path).unwrap();
            HttpResponse::Ok().body(fh.bytes().map(|x| x.unwrap_or(0_u8)).collect::<Vec<u8>>())
        }
    }
}

fn get_media_enabled_database() -> bool {
    use models::Configurations;
    use schema::Configuration::dsl::*;

    Configuration
        .select(Configurations::as_select())
        .first(&mut establish_connection())
        .unwrap()
        .media_enabled
}

#[derive(Serialize, Deserialize, ToSchema)]
struct MediaEnabledSchema {
    enabled: bool,
}

/// Is media center enabled?
#[utoipa::path(get, path = "/private/media/enabled", responses((status = 200, body = MediaEnabledSchema)), tags = ["media", "private"])]
async fn get_media_enabled_handler() -> HttpResponse {
    HttpResponse::Ok().json(MediaEnabledSchema {
        enabled: get_media_enabled_database(),
    })
}

/// Set media center activation
#[utoipa::path(post, path = "/private/media/enabled", responses((status = 200)), request_body = MediaEnabledSchema, tags = ["media", "private"])]
async fn set_enable_media(e: web::Json<MediaEnabledSchema>) -> HttpResponse {
    use schema::Configuration::dsl::*;

    let connection = &mut establish_connection();

    let database_update_execution = diesel::update(Configuration)
        .set(media_enabled.eq(e.enabled))
        .execute(connection);

    if let Err(database_error) = database_update_execution {
        return HttpResponse::InternalServerError()
            .json(ErrorCode::DatabaseUpdateFailed(database_error.to_string()).as_error_message());
    }

    return HttpResponse::Ok().json(MessageRes::from(
        "The media center status has been updated.",
    ));
}

#[derive(Serialize, ToSchema)]
struct RecommendationsRes {
    recommendations: Vec<RecommendedMediaEntry>,
}

/// Media files history
#[utoipa::path(get, path = "/private/media/history", tags = ["private", "media"], responses((status = 200, body = RecommendationsRes)))]
async fn read_full_media_history() -> HttpResponse {
    use models::RecommendedMediaEntry;
    use schema::RecommendedMedia::dsl::*;

    let connection = &mut establish_connection();

    let queried_entries = RecommendedMedia
        .select(RecommendedMediaEntry::as_select())
        .get_results(connection)
        .unwrap();

    let filtered_entries: Vec<RecommendedMediaEntry> = queried_entries
        .into_iter()
        .filter(|e| PathBuf::from(&e.file_path).exists())
        .collect();

    return HttpResponse::Ok().json(RecommendationsRes {
        recommendations: filtered_entries,
    });
}

#[derive(Deserialize)]
struct MetadataReq {
    name: Option<String>,
    genre: Option<String>,
    cover: Option<String>,
    artist: Option<String>,
}

#[utoipa::path(get, path = "/private/media/metadata/{file}", params(("file" = String, Path)), tags = ["private", "media"], responses((status = 200, body = RecommendationsRes)))]
/// Update media metadata
async fn update_media_metadata(
    path: web::Path<PathBuf>,
    json: web::Json<MetadataReq>,
) -> HttpResponse {
    use models::MediaEntry;
    use schema::Media::dsl::*;

    let new_media_entry = MediaEntry {
        file_path: path.into_inner().to_string_lossy().to_string(),
        genre: json.genre.clone(),
        name: json.name.clone(),
        artist: json.artist.clone(),
        cover: json.cover.clone(),
    };

    let connection = &mut establish_connection();

    let wx = diesel::insert_into(Media)
        .values(&new_media_entry)
        .on_conflict(file_path)
        .do_update()
        .set((
            genre.eq(&new_media_entry.genre),
            name.eq(&new_media_entry.name),
            artist.eq(&new_media_entry.artist),
            cover.eq(&new_media_entry.cover),
        ))
        .execute(connection);

    if let Err(database_error) = wx {
        return HttpResponse::InternalServerError()
            .json(ErrorCode::DatabaseInsertFailed(database_error.to_string()));
    }

    return HttpResponse::Ok().json(MessageRes::from("The media metadata has been updated."));
}

// Networking

#[derive(Serialize)]
struct NetworkInterfacesRes {
    interfaces: Vec<MeasuredInterface>,
}

#[derive(Serialize, ToSchema)]
struct NetworkRoutesRes {
    routes: Vec<Route>,
}

/// List of known network interfaces
#[utoipa::path(get, path = "/private/network/interfaces", tags = ["private", "network"], responses((status = 200, body = RecommendationsRes)))]
async fn network_interfaces(state: Data<AppState>) -> HttpResponse {
    let interfaces = state.network_interfaces.lock().unwrap().clone();

    return HttpResponse::Ok().json(NetworkInterfacesRes { interfaces });
}

#[utoipa::path(get, path = "/private/network/routes", tags = ["private", "network"], responses((status = 200, body = NetworkRoutesRes)))]
/// List of network routes
async fn network_routes() -> HttpResponse {
    let routes = net_data::get_routes();

    return HttpResponse::Ok().json(NetworkRoutesRes {
        routes: routes.unwrap(),
    });
}

#[derive(Deserialize, ToSchema)]
struct AdressRequestSchema {
    adress: String,
    subnet: Option<i32>,
}

#[derive(Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
struct DeleteNetworkRouteReq {
    device: String,
    destination: Option<AdressRequestSchema>,
    gateway: Option<AdressRequestSchema>,
    sudo_password: String,
}

#[utoipa::path(post, path = "/private/network/route/delete", request_body = DeleteNetworkRouteReq, responses((status = 200), (status = 401, description = "The provided sudo password was wrong.")), tags = ["private", "network"])]
/// Delete network route
async fn delete_network_route(json: web::Json<DeleteNetworkRouteReq>) -> HttpResponse {
    let built_deletion_route = DeletionRoute {
        device: json.device.clone(),
        nexthop: None,
        gateway: {
            if let Some(gateway_adress) = &json.gateway {
                Some(IpAddrWithSubnet {
                    address: IpAddr::from_str(&gateway_adress.adress).unwrap(),
                    subnet: gateway_adress.subnet,
                })
            } else {
                None
            }
        },
        destination: {
            if let Some(destination_adress) = &json.destination {
                Destination::Prefix(IpAddrWithSubnet {
                    address: IpAddr::from_str(&destination_adress.adress).unwrap(),
                    subnet: destination_adress.subnet,
                })
            } else {
                Destination::Default
            }
        },
    };

    let deletion_execution =
        net_data::delete_route(built_deletion_route, json.sudo_password.clone());

    match deletion_execution {
        sudo::SudoExecutionOutput::Success(_) => {
            HttpResponse::Ok().json(MessageRes::from("The route has been updated."))
        }
        sudo::SudoExecutionOutput::ExecutionError(err) => {
            HttpResponse::InternalServerError().json(ErrorCode::CommandFailed(err))
        }
        _ => HttpResponse::Unauthorized().json(ErrorCode::BadSudoPassword),
    }
}

#[derive(Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
struct NetworkingInterfaceActivityReq {
    activity: bool,
    interface: String,
    sudo_password: String,
}

/// Set activity of a network interface
#[utoipa::path(post, path = "/private/network/interface/active", responses((status = 200)), request_body = NetworkingInterfaceActivityReq, tags = ["private", "network"])]
async fn network_interface_active(json: web::Json<NetworkingInterfaceActivityReq>) -> HttpResponse {
    if json.activity {
        net_data::enable_interface(json.sudo_password.clone(), json.interface.clone());
    } else {
        net_data::disable_interface(json.sudo_password.clone(), json.interface.clone());
    }
    return HttpResponse::Ok().json(MessageRes::from("The interface has been updated."));
}

// Processes API

#[derive(Serialize, ToSchema)]
struct ListProcessesRes {
    processes: Vec<SerializableProcess>,
}

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
struct SerializableProcess {
    name: Option<String>,
    cpu_usage: f32,
    memory_usage_bytes: u64,
    username: Option<String>,
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

#[utoipa::path(get, path = "/private/processes/list", responses((status = 200, body = ListProcessesRes)), tags = ["processes", "private"])]
/// List of processes
async fn list_processes() -> HttpResponse {
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
        let cpu_usage = x.1.cpu_usage() / 100_f32;

        let executable_path = match x.1.exe() {
            Some(v) => Some(v.to_str().unwrap().to_string()),
            None => None,
        };

        let mut username: Option<String> = None;
        if let Some(uid) = x.1.user_id() {
            username = Some(NativeUser::from_uid(uid.div(1)).unwrap().username);
        }

        let pid = x.1.pid().as_u32();
        let name = Some(x.1.name().to_string_lossy().to_string());
        processes_for_response.push(SerializableProcess {
            name,
            cpu_usage,
            memory_usage_bytes,
            executable_path,
            username,
            pid,
        });
    });

    return HttpResponse::Ok().json(ListProcessesRes {
        processes: processes_for_response,
    });
}

/// Kill process by PID.
#[utoipa::path(post,
    path = "/private/processes/kill/{pid}",
    params(("pid" = u32, Path)),
    responses((status = 200), (status = 404, description = "The pid was not found.")),
    tags = ["processes", "private"]
)]
async fn kill_process(path: Path<u32>) -> HttpResponse {
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
                return HttpResponse::Ok()
                    .json(MessageRes::from("The singal has been sent successfully."));
            } else {
                return HttpResponse::InternalServerError()
                    .json(ErrorCode::SignalError.as_error_message());
            }
        }
        None => {
            return HttpResponse::NotFound().json(ErrorCode::UnknownPid.as_error_message());
        }
    }
}

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
struct ProcessDetailsRes {
    name: String,
    pid: u32,
    user: NativeUser,
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

#[utoipa::path(get,
    path = "/private/processes/details/{pid}",
    params(("pid" = u32, Path)),
    responses((status = 200, body = ProcessDetailsRes), (status = 404, description = "The pid was not found.")),
    tags = ["processes", "private"]
)]
/// Details about process
async fn details_process(path: Path<u32>) -> HttpResponse {
    // NOTE Part of this should be moved into a helper library

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
    let user = NativeUser::from_uid(uid.div(1) as u32).unwrap();
    let memory_usage_bytes = selected_process.memory();
    let cpu_usage = selected_process.cpu_usage() / 100_f32;
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

    return HttpResponse::Ok().json(ProcessDetailsRes {
        name: name.to_str().unwrap().to_string(),
        pid: pid.as_u32(),
        user,
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

// Cronjob API

#[derive(Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
enum CronjobVariant {
    /// A cronjob that runs at a specific time pattern (i.e. every Monday and Tuesday at 5am and)
    Specific,
    /// A cronjob that runs at an time interval (i.e. every day)
    Interval,
}

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
struct ListCronjobsRes {
    specific_jobs: Vec<SpecificCronJob>,
    interval_jobs: Vec<IntervalCronJob>,
    crontab_exists: bool,
}

#[derive(Deserialize)]
struct CronjobListReq {
    specific: Option<String>,
}

/// List users' cronjobs
#[utoipa::path(get, path = "/private/cronjobs/list", params(("specific" = Option<String>, Query)), responses((status = 200, body = ListCronjobsRes), (status = 404, description = "No crontab file was found."), (status = 500, description = "Cronjobs could not be read.")), tags = ["private", "cronjobs"])]
async fn list_cronjobs(query: Query<CronjobListReq>) -> HttpResponse {
    let user: User;

    if let Some(specific_user) = &query.specific {
        user = User::Specific(specific_user.clone());
    } else {
        user = User::Current;
    }

    let crons = cron::list_cronjobs(user);
    let mut interval_cronjobs: Vec<IntervalCronJob> = vec![];
    let mut specific_cronjobs: Vec<SpecificCronJob> = vec![];
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
            cron::CronError::NoCronFile => {
                return HttpResponse::NotFound().json(ErrorCode::NoCronjobs.as_error_message());
            }
            _ => {
                return HttpResponse::InternalServerError()
                    .json(ErrorCode::NoCronjobs.as_error_message());
            }
        },
    }

    return HttpResponse::Ok().json(ListCronjobsRes {
        specific_jobs: specific_cronjobs,
        interval_jobs: interval_cronjobs,
        crontab_exists: true,
    });
}

#[derive(Deserialize, ToSchema)]
struct CronjobCommandReq {
    index: usize,
    variant: CronjobVariant,
    user: Option<String>,
}

/// Run cronjob command
#[utoipa::path(post, path = "/private/cronjobs/runCommand", request_body = CronjobCommandReq, responses((status = 200)), tags = ["private", "cronjobs", "responding_job"])]
async fn run_cronjob_command(
    state: Data<AppState>,
    json: web::Json<CronjobCommandReq>,
) -> HttpResponse {
    // NOTE: The following could be improved.
    // TODO Capture command output and store in Status code

    let uuid = Uuid::new_v4();
    state
        .background_jobs
        .lock()
        .unwrap()
        .insert(uuid, BackgroundTaskState::Pending);

    let user: cron::User;

    if let Some(specified_user) = &json.user {
        user = cron::User::Specific(specified_user.clone())
    } else {
        user = cron::User::Current
    }

    let cronjobs_list_request = cron::list_cronjobs(user);

    let mut command_from_cronjob = None;

    if let Ok(cronjobs) = cronjobs_list_request {
        let relevant_cronjobs: Vec<&cron::CronJob> = match &json.variant {
            CronjobVariant::Specific => cronjobs
                .iter()
                .filter(|e| match e {
                    cron::CronJob::Specific(_c) => true,
                    _ => false,
                })
                .collect(),
            CronjobVariant::Interval => cronjobs
                .iter()
                .filter(|e| match e {
                    cron::CronJob::Interval(_c) => true,
                    _ => false,
                })
                .collect(),
        };

        if let Some(cronjob_at_index) = relevant_cronjobs.get(json.index) {
            command_from_cronjob = Some(match cronjob_at_index {
                cron::CronJob::Specific(c) => c.command.clone(),
                cron::CronJob::Interval(c) => c.command.clone(),
            });
        }
    } else {
        HttpResponse::InternalServerError().json(ErrorCode::NoCronjobs.as_error_message());
    }

    if command_from_cronjob.is_none() {
        return HttpResponse::NotFound().json(ErrorCode::NoSuchVariant.as_error_message());
    }

    let _ = actix_web::web::block(move || {
        let status;

        match Command::new("sh")
            .arg("-c")
            .arg(command_from_cronjob.unwrap())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .stdin(Stdio::null())
            .spawn()
        {
            Ok(mut h) => {
                let waited = h.wait();
                if let Ok(s) = waited {
                    if s.success() {
                        status = BackgroundTaskState::Success;
                    } else {
                        status = BackgroundTaskState::Fail;
                    }
                } else {
                    status = BackgroundTaskState::Fail;
                }
            }
            Err(_) => status = BackgroundTaskState::Fail,
        };

        state.background_jobs.lock().unwrap().insert(uuid, status);
    });

    return HttpResponse::Ok().body(uuid.to_string());
}

/// Delete cronjob
#[utoipa::path(post, path = "/private/cronjobs/delete/{index}/{variant}", params(("index" = u32, Path), ("variant" = CronjobVariant, Path)),responses((status = 200)), tags = ["private", "cronjobs"])]
async fn delete_cronjob(path: web::Path<(u32, CronjobVariant)>) -> HttpResponse {
    let index = path.0;
    let variant = &path.1;

    let _ = match variant {
        &CronjobVariant::Specific => delete_specific_cronjob(index, User::Current),
        &CronjobVariant::Interval => delete_interval_cronjob(index, User::Current),
    };

    HttpResponse::Ok().json(MessageRes::from("The cronjob has been deleted."))
}

/// TODO This all could be an enum containing struct
/// -> Requires re-writing cronjob handling as well, thus staged for later
#[derive(Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
struct CreateCronjobReq {
    variant: CronjobVariant,
    command: String,
    interval: Option<cron::Interval>,
    minute: Option<String>,
    hour: Option<String>,
    day_of_month: Option<String>,
    day_of_week: Option<String>,
    month: Option<String>,
}

/// Create new cronjob
#[utoipa::path(post, path = "/private/cronjobs/new", request_body = CreateCronjobReq, responses((status = 200)), tags = ["private", "cronjobs"])]
async fn create_cronjob(json: web::Json<CreateCronjobReq>) -> HttpResponse {
    let variant = &json.variant;

    match variant {
        CronjobVariant::Specific => {
            let day_of_month =
                match cron::Digit::try_from(json.day_of_month.clone().unwrap().as_str()) {
                    Ok(c) => c,
                    Err(_) => {
                        return HttpResponse::BadRequest()
                            .json(ErrorCode::SanitizationError.as_error_message());
                    }
                };

            let day_of_week = cron::DayOfWeek::from(json.day_of_week.clone().unwrap().as_str());
            let month = cron::Month::from(json.month.clone().unwrap().as_str());
            let minute = match cron::Digit::try_from(json.minute.clone().unwrap().as_str()) {
                Ok(c) => c,
                Err(_) => {
                    return HttpResponse::BadRequest()
                        .json(ErrorCode::SanitizationError.as_error_message());
                }
            };

            let hour = match cron::Digit::try_from(json.hour.clone().unwrap().as_str()) {
                Ok(c) => c,
                Err(_) => {
                    return HttpResponse::BadRequest()
                        .json(ErrorCode::SanitizationError.as_error_message());
                }
            };

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
                Ok(_) => HttpResponse::Ok()
                    .json(MessageRes::from("A new specific cronjob has been created.")),
                Err(_) => {
                    error!("Failed to create specific cronjob");
                    HttpResponse::InternalServerError()
                        .json(ErrorCode::CronjobCreationFailed.as_error_message())
                }
            }
        }
        CronjobVariant::Interval => {
            match cron::create_new_interval_cronjob(
                cron::IntervalCronJob {
                    interval: json.interval.clone().unwrap(),
                    command: json.command.clone(),
                },
                User::Current,
            ) {
                Ok(_) => HttpResponse::Ok()
                    .json(MessageRes::from("A new interval cronjob has been created.")),
                Err(_) => {
                    error!("Failed to create interval cronjob");
                    HttpResponse::InternalServerError()
                        .json(ErrorCode::CronjobCreationFailed.as_error_message())
                }
            }
        }
    }
}

#[derive(Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
struct FileSharingReq {
    #[schema(value_type = String)]
    file_path: PathBuf,
    password: Option<String>,
}

#[utoipa::path(post, path = "/private/sharing/new", request_body = FileSharingReq, responses((status = 200)), tags = ["private", "sharing"])]
/// Create new file sharing
async fn share_file(json: web::Json<FileSharingReq>) -> HttpResponse {
    use models::SharedFile;
    use schema::FileSharing;

    let code: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(64)
        .map(char::from)
        .collect();

    let file_path = &json.file_path.clone();
    let password = &json.password.clone();

    if !file_path.exists() {
        return HttpResponse::NotFound().json(ErrorCode::FileDoesNotExist.as_error_message());
    }

    let password_insert_value = match password {
        Some(v) => {
            let hashed_password = argon2_derive_key(v).unwrap();
            Some(hex::encode(hashed_password))
        }
        None => None,
    };

    let new_shared_file = SharedFile {
        code: code.clone(),
        file_path: file_path.to_str().unwrap().to_string(),
        use_password: password.is_some(),
        password: password_insert_value,
        shared_since: std::time::SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as i64,
    };

    let sharing_creation_database_execution = diesel::insert_into(FileSharing::dsl::FileSharing)
        .values(&new_shared_file)
        .on_conflict(FileSharing::dsl::code)
        .do_update()
        .set(&new_shared_file)
        .execute(&mut establish_connection());

    if let Err(database_error) = sharing_creation_database_execution {
        return HttpResponse::InternalServerError()
            .json(ErrorCode::DatabaseInsertFailed(database_error.to_string()));
    }

    return HttpResponse::Ok().body(code);
}

#[derive(Serialize, ToSchema)]
struct SharedFilesListRes {
    files: Vec<SharedFile>,
}

#[utoipa::path(get, path = "/private/sharing/list", responses((status = 200, body = SharedFilesListRes)), tags = ["private", "sharing"])]
/// List of shared files
async fn get_shared_files_list() -> HttpResponse {
    use models::SharedFile;
    use schema::FileSharing::dsl::*;

    let files: Vec<SharedFile> = FileSharing
        .select(SharedFile::as_select())
        .get_results(&mut establish_connection())
        .unwrap();

    return HttpResponse::Ok().json(SharedFilesListRes { files });
}

#[derive(Deserialize, ToSchema)]
struct SharedFileReq {
    code: String,
    password: Option<String>,
}

#[utoipa::path(post, path = "/public/shared/get", request_body = SharedFileReq, responses((status = 200, content_type = "application/octet-stream")), tags = ["public", "sharing"])]
/// Contents of shared file
async fn get_shared_file(json: web::Json<SharedFileReq>) -> HttpResponse {
    use models::SharedFile;
    use schema::FileSharing;

    let request_password = &json.password;

    let connection = &mut establish_connection();

    let shared_files: Vec<SharedFile> = FileSharing::dsl::FileSharing
        .select(SharedFile::as_select())
        .get_results(connection)
        .unwrap();

    let file_with_code = shared_files.into_iter().find(|e| e.code == json.code);

    // Check if the requested code even exists
    if file_with_code.is_some() {
        let e_unwrap = file_with_code.unwrap();
        let password_checking = e_unwrap.use_password;
        let database_hash = e_unwrap.password.clone();

        if password_checking && request_password.is_none() {
            return HttpResponse::Forbidden()
                .json(ErrorCode::MissingSharedFilePermissions.as_error_message());
        }

        let file_path = e_unwrap.file_path.clone();
        let file_contents = fs::read(file_path).unwrap();

        if password_checking {
            let hashed_request_password =
                argon2_derive_key(&request_password.clone().unwrap()).unwrap();
            if hex::encode(hashed_request_password)
                == database_hash.expect(
                    "A file sharing password hash could not be found, even though it has to exist.",
                )
            {
                return HttpResponse::Ok().body(file_contents);
            } else {
                warn!("User entered wrong file sharing password.");
                return HttpResponse::Forbidden()
                    .json(ErrorCode::MissingSharedFilePermissions.as_error_message());
            }
        }

        return HttpResponse::Ok().body(file_contents);
    }
    return HttpResponse::NotFound().json(ErrorCode::NoSuchSharedFile.as_error_message());
}

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
struct SharedFileMetadataRes {
    file_path: String,
    use_password: bool,
    size: u32,
}

#[utoipa::path(get, path = "/private/shared/getMetadata", request_body = SharedFileReq, responses((status = 200, body = SharedFileMetadataRes)), tags = ["public", "sharing"])]
/// Metadata of shared file
async fn get_shared_file_metadata(json: web::Json<SharedFileReq>) -> HttpResponse {
    use models::SharedFile;
    use schema::FileSharing::dsl::*;

    let f: Vec<SharedFile> = FileSharing
        .select(SharedFile::as_select())
        .filter(code.eq(&json.code))
        .get_results(&mut establish_connection())
        .unwrap();

    if f.is_empty() {
        return HttpResponse::BadRequest().json(ErrorCode::InsufficientData.as_error_message());
    }

    let p = PathBuf::from(f[0].file_path.clone());

    if !p.exists() {
        return HttpResponse::NotFound().json(ErrorCode::NoSuchSharedFile.as_error_message());
    }

    let size = p.metadata().unwrap().len();

    return HttpResponse::Ok().json(SharedFileMetadataRes {
        file_path: f[0].file_path.clone(),
        use_password: f[0].use_password,
        size: size as u32,
    });
}

#[utoipa::path(post, path = "/private/sharing/delete/{code}", params(("code" = String, Path)), responses((status = 200)), tags = ["private", "sharing"])]
/// Delete file sharing
async fn unshare_file(request_code: web::Path<String>) -> HttpResponse {
    use schema::FileSharing::dsl::*;

    let delete_file_sharing_database_execution = diesel::delete(FileSharing)
        .filter(code.eq(request_code.into_inner()))
        .execute(&mut establish_connection());

    if let Err(database_error) = delete_file_sharing_database_execution {
        return HttpResponse::InternalServerError().json(ErrorCode::DatabaseDeletionFailed(
            database_error.to_string(),
        ));
    }

    return HttpResponse::Ok().json(MessageRes::from("The file is no longer being shared."));
}

fn configure_multipart(cfg: &mut web::ServiceConfig) {
    // Configure multipart form settings
    let multipart_config = MultipartFormConfig::default()
        .total_limit(1024 * 1024 * 1024 * 32)
        .error_handler(|err, _req| {
            actix_web::error::InternalError::from_response(err, HttpResponse::Conflict().into())
                .into()
        });

    // Store the configuration in app data
    cfg.app_data(multipart_config);
}

/// This function leverages the `fn is_admin_state()` from admin.rs to verify if a request comes
/// from a user authenticated as administrator. For this, it requires the current application state
/// and request session.
async fn authorization_middleware(
    mut req: ServiceRequest,
    next: Next<BoxBody>,
) -> Result<ServiceResponse<impl MessageBody>, actix_web::Error> {
    if is_admin_state(&req.get_session(), req.extract::<Data<AppState>>().await?) {
        next.call(req).await
    } else {
        warn!("A request to a private route will be denied.");
        Ok(req.into_response(
            HttpResponse::Forbidden().json(ErrorCode::MissingApiPermissions.as_error_message()),
        ))
    }
}

async fn media_authorization_middleware(
    req: ServiceRequest,
    next: Next<BoxBody>,
) -> Result<ServiceResponse<impl MessageBody>, actix_web::Error> {
    if get_media_enabled_database() {
        next.call(req).await
    } else {
        warn!("A request to a media route will be denied.");
        Ok(req.into_response(
            HttpResponse::Forbidden().json(ErrorCode::MediaCenterDisabled.as_error_message()),
        ))
    }
}

// TODO  When Zentrox has been split up into different crates, move this into a module.
fn generate_openapi_contract(store_path: Option<&String>) {
    println!("Generating OpenAPI contracg in pretty printed JSON format.");

    #[derive(OpenApi)]
    #[openapi(
        paths(
            verification,
            logout,
            use_otp,
            otp_activation,
            verify_sudo_password,
            routes::dashboard::device_information,
            package_database,
            package_statistics,
            update_package_database,
            install_package,
            remove_package,
            update_package,
            update_all_packages,
            remove_orphaned_packages,
            orphaned_packages,
            fetch_job_status,
            firewall_has_ufw,
            firewall_information,
            switch_ufw,
            delete_firewall_rule,
            new_firewall_rule,
            download_file,
            files_list,
            delete_file,
            move_path,
            burn_file,
            get_file_metadata,
            upload_file,
            list_drives,
            drive_information,
            is_vault_configured,
            vault_configure,
            vault_tree,
            delete_vault_file,
            vault_new_folder,
            upload_vault,
            vault_file_download,
            rename_vault_file,
            power_off,
            cert_names,
            upload_tls,
            account_details,
            update_account_details,
            profile_picture,
            upload_profile_picture,
            logs_request,
            get_media_list,
            media_request,
            get_media_source_list,
            update_media_source_list,
            get_cover,
            get_media_enabled_handler,
            set_enable_media,
            read_full_media_history,
            update_media_metadata,
            network_interfaces,
            network_routes,
            delete_network_route,
            network_interface_active,
            list_processes,
            kill_process,
            details_process,
            run_cronjob_command,
            delete_cronjob,
            create_cronjob,
            list_cronjobs,
            share_file,
            get_shared_files_list,
            unshare_file,
            get_shared_file,
            get_shared_file_metadata,
        ),
        tags(
            (name = "private", description = "Routes are restricted to be admin-only."),
            (name = "public", description = "Anyone can access these routes."),
            (name = "account", description = "Administrator account settings and data"),
            (name = "logs", description = "Connection to view logs"),
            (name = "authentication", description = "Authenticate users and passwords."),
            (name = "cronjobs", description = "Manage repetitive commands."),
            (name = "responding_job", description = "Doesn't return the results of the execution but rather an UUID to request that current state of a job. This is useful for slow tasks."),
            (name = "dashboard", description = "Information for the dashboard interface."),
            (name = "drives", description = "Block device managment"),
            (name = "files", description = "File system controls"),
            (name = "firewall", description = "UFW control"),
            (name = "jobs", description = "Job UUID status resolver"),
            (name = "media", description = "Media center data"),
            (name = "network", description = "Networking settings and information"),
            (name = "packages", description = "Package manager connections"),
            (name = "power", description = "Power settings"),
            (name = "processes", description = "System process managment"),
            (name = "sharing", description = "File sharing managment"),
            (name = "tls", description = "TLS encryption settings"),
            (name = "vault", description = "Encrypted data storage settings")
        ),
    )]
    struct ApiDoc;

    let mut document = ApiDoc::openapi();

    document.servers = Some(vec![
        ServerBuilder::new()
            .url("https://localhost:8080/api")
            .build(),
    ]);
    let json = document.to_pretty_json().unwrap();

    if let Some(p) = store_path {
        let _ = fs::write(p, json);
    } else {
        println!("{json}");
    }

    exit(0)
}

fn print_help() {
    println!("Zentrox");
    println!("--help:\t\tPrint this help.");
    println!("--docs <Path | None>:\t\tGenerate OpenAPI docs.");

    exit(0)
}

#[actix_web::main]
/// Prepares Zentrox and starts the server.
async fn main() -> std::io::Result<()> {
    use models::Configurations;
    use schema::Configuration::dsl::*;

    let os_args = std::env::args().collect::<Vec<String>>();

    match os_args.get(1) {
        Some(arg) if arg == "--docs" => generate_openapi_contract(os_args.get(2)),
        Some(arg) if arg == "--help" => print_help(),
        _ => {}
    }

    if !env::current_dir().unwrap().join("static").exists() {
        let _ = env::set_current_dir(dirs::home_dir().unwrap().join("zentrox"));
    }

    let mut gov_vars = std::env::vars();

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

    let tls_cert_filename = Configuration
        .select(Configurations::as_select())
        .first(&mut establish_connection())
        .unwrap()
        .tls_cert;

    if tls_cert_filename == "selfsigned.pem" {
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
        File::open(data_path.join("certificates").join(&tls_cert_filename)).unwrap(),
    );
    debug!(
        "Using certificate file from {}",
        data_path
            .join("certificates")
            .join(&tls_cert_filename)
            .to_str()
            .unwrap()
    );

    let mut key_file =
        BufReader::new(File::open(data_path.join("certificates").join(tls_cert_filename)).unwrap());

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
            .requests_per_minute(2)
            .finish()
            .unwrap()
    } else {
        warn!("Using permissive governor configuration");
        GovernorConfigBuilder::default()
            .permissive(true)
            .finish()
            .unwrap()
    };

    let shared_files_governor_conf = if governor_strict {
        GovernorConfigBuilder::default()
            .requests_per_minute(9) // ~3 downloads / minute
            .finish()
            .unwrap()
    } else {
        warn!("Using permissive governor configuration");
        GovernorConfigBuilder::default()
            .permissive(true)
            .finish()
            .unwrap()
    };

    info!("Zentrox is being serverd on port 8080");

    HttpServer::new(move || {
        let mut cors_vars = std::env::vars();
        let cors_permissive: bool = cors_vars.any(|x| {
            x == ("ZENTROX_MODE".to_string(), "NO_CORS".to_string())
                || x == ("ZENTROX_MODE".to_string(), "DEV".to_string())
        });

        if cors_permissive {
            warn!("CORS policy is set to permissive! This poses a high security risk.");
        }

        App::new()
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
                Cors::default()
                    .allow_any_method()
                    .allowed_origin("http://localhost:3000")
                    .block_on_origin_mismatch(true)
            } else {
                Cors::default()
            })
            .wrap(middleware::Compress::default())
            .wrap(Governor::new(&governor_conf))
            .app_data(Data::new(app_state.clone()))
            .service(web::resource("/").route(web::get().to(index)))
            .service(web::resource("/alerts").route(web::get().to(alerts_page)))
            .service(web::resource("/alerts/manifest.json").route(web::get().to(alerts_manifest)))
            .service(web::scope("/dashboard").route("", web::get().to(dashboard)))
            .service(robots_txt)
            // API routes are separated into public and private, where public routes can be
            // accessed from anyone without authorization prior to the request and private routes
            // require you to be logged in as administrator.
            //
            // Public routes can be accessed under /api/public.
            // Private routes can be accessed under /api/private.
            .service(
                web::scope("/api")
                    .service(
                        web::scope("/public")
                            .service(
                                web::scope("/auth")
                                    .wrap(Governor::new(&harsh_governor_conf))
                                    .route("/login", web::post().to(verification))
                                    .route("/useOtp", web::get().to(use_otp)),
                            )
                            .service(
                                web::scope("/shared")
                                    .wrap(Governor::new(&shared_files_governor_conf))
                                    .route("", web::get().to(shared_page))
                                    .route("/get", web::post().to(get_shared_file))
                                    .route(
                                        "/getMetadata",
                                        web::post().to(get_shared_file_metadata),
                                    ),
                            ),
                    )
                    .service(
                        web::scope("/private")
                            // The following guard protects from unauthorized access.
                            .wrap(from_fn(authorization_middleware))
                            .service(
                                web::scope("/auth")
                                    .route("/logout", web::post().to(logout))
                                    .route("/useOtp", web::put().to(otp_activation))
                                    .service(
                                        web::scope("/sudo")
                                            .route("/verify", web::post().to(verify_sudo_password)),
                                    ),
                            )
                            .service(
                                web::scope("/dashboard")
                                    .route("/information", web::get().to(routes::dashboard::device_information)),
                            )
                            .service(
                                web::scope("/packages")
                                    .route("/database", web::get().to(package_database))
                                    .route("/statistics", web::get().to(package_statistics))
                                    .route(
                                        "/updateDatabase",
                                        web::post().to(update_package_database),
                                    )
                                    .route("/install", web::post().to(install_package))
                                    .route("/remove", web::post().to(remove_package))
                                    .route("/update", web::post().to(update_package))
                                    .route("/updateAll", web::post().to(update_all_packages))
                                    .route(
                                        "/removeOrphaned",
                                        web::post().to(remove_orphaned_packages),
                                    )
                                    .route("/orphaned", web::get().to(orphaned_packages)),
                            )
                            .service(
                                web::scope("/jobs")
                                    .route("status/{id}", web::get().to(fetch_job_status)),
                            )
                            .service(
                                web::scope("/firewall")
                                    .route("/ufwPresent", web::get().to(firewall_has_ufw))
                                    .route("/rules", web::post().to(firewall_information))
                                    .route("/enabled", web::post().to(switch_ufw))
                                    .route("/rule/delete", web::post().to(delete_firewall_rule))
                                    .route("/rule/new", web::post().to(new_firewall_rule)),
                            )
                            .service(
                                web::scope("/files")
                                    .route("/download", web::get().to(download_file))
                                    .route("/directoryReading", web::get().to(files_list))
                                    .route("/delete", web::post().to(delete_file))
                                    .route("/move", web::post().to(move_path))
                                    .route("/burn", web::post().to(burn_file))
                                    .route("/metadata", web::get().to(get_file_metadata))
                                    .route("/upload", web::post().to(upload_file)),
                            )
                            .service(
                                web::scope("/drives")
                                    .route("/list", web::get().to(list_drives))
                                    .route("/statistics", web::get().to(drive_information)),
                            )
                            .service(
                                web::scope("/vault")
                                    .route("/active", web::get().to(is_vault_configured))
                                    .route("/configuration", web::post().to(vault_configure))
                                    .route("/tree", web::post().to(vault_tree))
                                    .route("/delete", web::post().to(delete_vault_file))
                                    .route("/directory", web::post().to(vault_new_folder))
                                    .route("/file", web::post().to(upload_vault))
                                    .route("/file", web::get().to(vault_file_download))
                                    .route("/move", web::post().to(rename_vault_file)),
                            )
                            .service(web::scope("/power").route("/off", web::post().to(power_off)))
                            .service(
                                web::scope("/tls")
                                    .route("/name", web::get().to(cert_names))
                                    .route("/upload", web::post().to(upload_tls)),
                            )
                            .service(
                                web::scope("/account")
                                    .route("/details", web::get().to(account_details))
                                    .route("/details", web::post().to(update_account_details))
                                    .route("/profilePicture", web::get().to(profile_picture))
                                    .route(
                                        "/profilePicture",
                                        web::post().to(upload_profile_picture),
                                    ),
                            )
                            .route("/logs", web::post().to(logs_request))
                            .service(
                                web::scope("/media")
                                    .route("/sources", web::get().to(get_media_source_list))
                                    .route("/sources", web::post().to(update_media_source_list))
                                    .route("/enabled", web::get().to(get_media_enabled_handler))
                                    .route("/enabled", web::post().to(set_enable_media))
                                    .wrap(from_fn(media_authorization_middleware))
                                    .route("", web::get().to(media_page))
                                    .route("/files", web::get().to(get_media_list))
                                    .route("/download", web::get().to(media_request))
                                    .route("/cover", web::get().to(get_cover))
                                    .route("/history", web::get().to(read_full_media_history))
                                    .route(
                                        "/metadata/{file}",
                                        web::post().to(update_media_metadata),
                                    ),
                            )
                            .service(
                                web::scope("/network")
                                    .route("/interfaces", web::get().to(network_interfaces))
                                    .route("/routes", web::get().to(network_routes))
                                    .service(
                                        web::scope("/route")
                                            .route("/delete", web::post().to(delete_network_route)),
                                    )
                                    .service(web::scope("/interface").route(
                                        "/active",
                                        web::post().to(network_interface_active),
                                    )),
                            )
                            .service(
                                web::scope("/processes")
                                    .route("/list", web::get().to(list_processes))
                                    .route("/kill/{pid}", web::post().to(kill_process))
                                    .route("/details/{pid}", web::get().to(details_process)),
                            )
                            .service(
                                web::scope("/cronjobs")
                                    .route("/runCommand", web::post().to(run_cronjob_command))
                                    .route("/delete", web::post().to(delete_cronjob))
                                    .route("/new", web::post().to(create_cronjob))
                                    .route("/list", web::get().to(list_cronjobs)),
                            )
                            .service(
                                web::scope("/sharing")
                                    .route("/new", web::post().to(share_file))
                                    .route("/list", web::get().to(get_shared_files_list))
                                    .route("/delete/{code}", web::post().to(unshare_file)),
                            ),
                    ),
            )
            .service(afs::Files::new("/", "static/"))
    })
    .workers(16)
    .keep_alive(std::time::Duration::from_secs(60 * 6))
    .bind_rustls_0_23(("0.0.0.0", 8080), tls_config)? // TODO Allow user to decide port and IP
    .run()
    .await
}
