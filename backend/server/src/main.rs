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

use actix_cors::Cors;
use actix_files as afs;
use actix_governor::{self, Governor, GovernorConfigBuilder};
use actix_multipart::form::MultipartFormConfig;
use actix_session::SessionExt;
use actix_session::config::CookieContentSecurity;
use actix_session::{Session, SessionMiddleware, storage::CookieSessionStore};
use actix_web::body::{BoxBody, MessageBody};
use actix_web::cookie::Key;
use actix_web::dev::{ServiceRequest, ServiceResponse};
use actix_web::middleware::{Next, from_fn};
use actix_web::{App, HttpResponse, HttpServer, get, middleware, web, web::Data};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use std::{
    collections::HashMap,
    env,
    fs::File,
    io::BufReader,
    path::PathBuf,
    sync::{Arc, Mutex},
};
use utils::net_data::OperationalState;
use uuid::Uuid;
extern crate inflector;
use diesel::prelude::*;
use log::{debug, info, warn};
use utoipa::ToSchema;

mod generate_contract;
mod help;
mod is_admin;
mod routes;
mod setup;
use routes::*;

use is_admin::is_admin_state;
use utils::status_com::ErrorCode;

use routes::media::get_media_enabled_database;
use utils::database::establish_connection;

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
#[derive(Clone, Serialize, ToSchema)]
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
        let devices_a = utils::net_data::get_network_interfaces().unwrap();
        std::thread::sleep(std::time::Duration::from_millis(1000));
        let devices_b = utils::net_data::get_network_interfaces().unwrap();
        let devices_b_hashmap: HashMap<String, &utils::net_data::Interface> =
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

    fn start_interval_tasks(&self) {
        let network_clone = self.clone();
        std::thread::spawn(move || {
            loop {
                std::thread::sleep(std::time::Duration::from_millis(5 * 1000));
                network_clone.update_network_statistics();
            }
        });
    }
}

async fn index(session: Session, state: Data<AppState>) -> HttpResponse {
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

#[get("/robots.txt")]
/// Return the robots.txt file to prevent search engines from indexing this server.
async fn robots_txt() -> HttpResponse {
    HttpResponse::Ok().body(include_str!("../../assets/robots.txt"))
}

async fn alerts_manifest() -> HttpResponse {
    HttpResponse::Ok().body(include_str!("../../assets/manifest.json"))
}

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

#[actix_web::main]
/// Prepares Zentrox and starts the server.
async fn main() -> std::io::Result<()> {
    use utils::models::Configurations;
    use utils::schema::Configuration::dsl::*;

    let os_args = std::env::args().collect::<Vec<String>>();

    match os_args.get(1) {
        Some(arg) if arg == "--docs" => generate_contract::generate(os_args.get(2)),
        Some(arg) if arg == "--help" => help::print(),
        _ => {}
    }

    let mut gov_vars = std::env::vars();
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let zentrox_env_dir = dirs::home_dir()
        .unwrap()
        .join(".local")
        .join("share")
        .join("zentrox")
        .join("database.db");

    if !zentrox_env_dir.join("database.db").exists() {
        let _ = setup::run_setup();
    } else {
        debug!("Found configurations in {}", zentrox_env_dir.display())
    }

    if !env::current_dir().unwrap().join("static").exists() {
        let _ = env::set_current_dir(&zentrox_env_dir);
    }

    let secret_session_key = Key::try_generate().expect("Failed to generate session key");
    let app_state = AppState::new();
    app_state.start_interval_tasks();
    debug!("Started interval tasks");

    let tls_cert_filename = Configuration
        .select(Configurations::as_select())
        .first(&mut establish_connection())
        .unwrap()
        .tls_cert;

    if tls_cert_filename == "selfsigned.pem" {
        warn!("You may be using a self-signed certificate.");
    }

    rustls::crypto::aws_lc_rs::default_provider()
        .install_default()
        .unwrap();

    let mut certs_file = BufReader::new(
        File::open(
            zentrox_env_dir
                .join("certificates")
                .join(&tls_cert_filename),
        )
        .unwrap(),
    );
    debug!(
        "Using certificate file from {}",
        zentrox_env_dir
            .join("certificates")
            .join(&tls_cert_filename)
            .to_str()
            .unwrap()
    );

    let mut key_file = BufReader::new(
        File::open(zentrox_env_dir.join("certificates").join(tls_cert_filename)).unwrap(),
    );

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
            .service(web::scope("/dashboard").route("", web::get().to(dashboard::page)))
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
                                    .route("/login", web::post().to(auth::login))
                                    .route("/useOtp", web::get().to(auth::use_otp)), // FIX Only respond if password was correct
                            )
                            .service(
                                web::scope("/shared")
                                    .wrap(Governor::new(&shared_files_governor_conf))
                                    .route("", web::get().to(shared_page))
                                    .route("/get", web::post().to(sharing::download_file))
                                    .route("/getMetadata", web::post().to(sharing::get_metadata)),
                            ),
                    )
                    .service(
                        web::scope("/private")
                            // The following guard protects from unauthorized access.
                            .wrap(from_fn(authorization_middleware))
                            .service(
                                web::scope("/auth")
                                    .route("/logout", web::post().to(auth::logout))
                                    .route("/useOtp", web::put().to(auth::activate_otp))
                                    .service(web::scope("/sudo").route(
                                        "/verify",
                                        web::post().to(auth::verify_sudo_password),
                                    )),
                            )
                            .service(
                                web::scope("/dashboard")
                                    .route("/information", web::get().to(dashboard::information)),
                            )
                            .service(
                                web::scope("/packages")
                                    .route("/database", web::get().to(packages::database))
                                    .route("/statistics", web::get().to(packages::statistics))
                                    .route("/updateDatabase", web::post().to(packages::update_db))
                                    .route("/install", web::post().to(packages::install_package))
                                    .route("/remove", web::post().to(packages::remove_package))
                                    .route("/update", web::post().to(packages::update_package))
                                    .route("/updateAll", web::post().to(packages::update_all))
                                    .route(
                                        "/removeOrphaned",
                                        web::post().to(packages::remove_orphaned),
                                    )
                                    .route("/orphaned", web::get().to(packages::orphaned)),
                            )
                            .service(
                                web::scope("/jobs")
                                    .route("status/{id}", web::get().to(jobs::status)),
                            )
                            .service(
                                web::scope("/firewall")
                                    .route("/ufwPresent", web::get().to(firewall::has_ufw))
                                    .route("/rules", web::post().to(firewall::status))
                                    .route("/enabled", web::post().to(firewall::switch))
                                    .route("/rule/delete", web::post().to(firewall::delete_rule))
                                    .route("/rule/new", web::post().to(firewall::new_rule)),
                            )
                            .service(
                                web::scope("/files")
                                    .route("/download", web::get().to(files::download))
                                    .route("/directoryReading", web::get().to(files::list))
                                    .route("/delete", web::post().to(files::delete))
                                    .route("/move", web::post().to(files::move_to))
                                    .route("/burn", web::post().to(files::burn))
                                    .route("/metadata", web::get().to(files::metadata))
                                    .route("/upload", web::post().to(files::upload)),
                            )
                            .service(
                                web::scope("/drives")
                                    .route("/list", web::get().to(drives::list))
                                    .route("/statistics", web::get().to(drives::statistics)),
                            )
                            .service(
                                web::scope("/vault")
                                    .route("/active", web::get().to(vault::is_configured))
                                    .route("/configuration", web::post().to(vault::configure))
                                    .route("/tree", web::post().to(vault::tree))
                                    .route("/delete", web::post().to(vault::delete_file))
                                    .route("/directory", web::post().to(vault::new_directory))
                                    .route("/file", web::post().to(vault::upload))
                                    .route("/file", web::get().to(vault::download_file))
                                    .route("/move", web::post().to(vault::rename_file)),
                            )
                            .service(web::scope("/power").route("/off", web::post().to(power::off)))
                            .service(
                                web::scope("/tls")
                                    .route("/name", web::get().to(tls::name))
                                    .route("/upload", web::post().to(tls::upload)),
                            )
                            .service(
                                web::scope("/account")
                                    .route("/details", web::get().to(account::details))
                                    .route("/details", web::post().to(account::update_details))
                                    .route("/profilePicture", web::get().to(account::picture))
                                    .route(
                                        "/profilePicture",
                                        web::post().to(account::upload_picture),
                                    ),
                            )
                            .route("/logs", web::post().to(logs::read))
                            .service(
                                web::scope("/media")
                                    .route("/sources", web::get().to(media::get_sources))
                                    .route("/sources", web::post().to(media::update_sources))
                                    .route(
                                        "/enabled",
                                        web::get().to(media::get_media_enabled_handler),
                                    )
                                    .route("/enabled", web::post().to(media::activate_media))
                                    .wrap(from_fn(media_authorization_middleware))
                                    .route("", web::get().to(media_page))
                                    .route("/files", web::get().to(media::get_contents))
                                    .route("/download", web::get().to(media::download))
                                    .route("/cover", web::get().to(media::cover))
                                    .route("/history", web::get().to(media::read_history))
                                    .route(
                                        "/metadata/{file}",
                                        web::post().to(media::update_metadata),
                                    ),
                            )
                            .service(
                                web::scope("/network")
                                    .route("/interfaces", web::get().to(network::interfaces))
                                    .route("/routes", web::get().to(network::routes))
                                    .route("/route/delete", web::post().to(network::delete_route))
                                    .route(
                                        "/interface/active",
                                        web::post().to(network::activate_interface),
                                    ),
                            )
                            .service(
                                web::scope("/processes")
                                    .route("/list", web::get().to(processes::list))
                                    .route("/kill/{pid}", web::post().to(processes::kill))
                                    .route("/details/{pid}", web::get().to(processes::details)),
                            )
                            .service(
                                web::scope("/cronjobs")
                                    .route("/runCommand", web::post().to(cron::run_command))
                                    .route("/delete", web::post().to(cron::delete))
                                    .route("/new", web::post().to(cron::create))
                                    .route("/list", web::get().to(cron::list)),
                            )
                            .service(
                                web::scope("/sharing")
                                    .route("/new", web::post().to(sharing::share))
                                    .route("/list", web::get().to(sharing::list))
                                    .route("/delete/{code}", web::post().to(sharing::unshare)),
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
