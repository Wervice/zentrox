//! Zentrox is a Linux server administration application.
//! It support various tasks ranging from sharing and managing files to installing system packages
//! and basic network configuration tasks.
//!
//! Most interactions between Zentrox and the operating system are handled through commands or files.
//!
//! Documentation for the API can be obtained by running the executable with the {`--docs`} flag.
//! This will produce an OpenAPI documentation in JSON format.

use actix_cors::Cors;
use actix_files as afs;
use actix_governor::{self, Governor, GovernorConfigBuilder};
use actix_multipart::form::MultipartFormConfig;
use actix_session::config::PersistentSession;
use actix_session::{
    Session, SessionExt, SessionMiddleware, config::CookieContentSecurity,
    storage::CookieSessionStore,
};
use actix_web::Responder;
use actix_web::body::{BoxBody, MessageBody};
use actix_web::cookie::Key;
use actix_web::dev::{ServiceRequest, ServiceResponse};
use actix_web::middleware::{Next, from_fn};
use actix_web::{
    App, HttpResponse, HttpServer, cookie::time::Duration as ActixDuration, get, middleware, web,
    web::Data,
};
use diesel::prelude::*;
use log::{debug, info, warn};
use permissions::is_admin_state;
use routes::media::get_media_enabled_database;
use serde::{Deserialize, Serialize};
use simplelog::{
    self, ColorChoice, CombinedLogger, Config, LevelFilter, TermLogger, TerminalMode, WriteLogger,
};
use std::{
    collections::HashMap,
    env,
    fs::File,
    io::BufReader,
    path::PathBuf,
    sync::{Arc, Mutex},
    time::{Duration, SystemTime},
};
use utils::database::establish_connection;
use utils::net_data::Interface;
use utils::status_com::ErrorCode;
use utoipa::ToSchema;
use uuid::Uuid;
mod generate_contract;
mod help;
mod permissions;
mod routes;
mod setup;
use routes::*;

const SERVER_PORT: u16 = 8080;

#[derive(Clone)]
#[allow(unused)]
enum BackgroundTaskState {
    Success,
    Fail,
    SuccessOutput(String),
    FailOutput(String),
    Pending,
}

/// Current state of the application
/// This AppState is meant to be accessible for every route in the system
#[derive(Clone)]
struct AppState {
    login_token: Arc<Mutex<Option<String>>>,
    last_login: Arc<Mutex<Option<SystemTime>>>,
    system: Arc<Mutex<sysinfo::System>>,
    username: Arc<Mutex<Option<String>>>,
    network_interfaces: Arc<Mutex<Vec<Interface>>>,
    background_jobs: Arc<Mutex<HashMap<Uuid, BackgroundTaskState>>>,
}

impl AppState {
    /// Initiate a new AppState
    fn new() -> Self {
        let random_string: Arc<[u8]> = (0..128).map(|_| rand::random::<u8>()).collect();
        AppState {
            login_token: Arc::new(Mutex::new(Some(
                String::from_utf8_lossy(&random_string).to_string(),
            ))),
            last_login: Arc::new(Mutex::new(None)),
            system: Arc::new(Mutex::new(sysinfo::System::new())),
            username: Arc::new(Mutex::new(None)),
            network_interfaces: Arc::new(Mutex::new(Vec::new())),
            background_jobs: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    fn update_network_statistics(&self) {
        if self.username.lock().unwrap().is_none() {
            return;
        }
        let devices_a = utils::net_data::get_network_interfaces().unwrap();
        std::thread::sleep(Duration::from_millis(1000));
        let devices_b = utils::net_data::get_network_interfaces().unwrap();
        let mut result: Vec<Interface> = Vec::new();
        for device in devices_a {
            if let Some(v) = devices_b.iter().find(|straw| straw.index == device.index) {
                let a_up = device.statistics.transmitted.bytes;
                let a_down = device.statistics.recieved.bytes;
                let b_up = v.statistics.transmitted.bytes;
                let b_down = v.statistics.recieved.bytes;
                result.push(Interface {
                    delta_down: Some(b_down - a_down),
                    delta_up: Some(b_up - a_up),
                    ..device
                })
            }
        }
        *self.network_interfaces.lock().unwrap() = result;
    }

    fn start_interval_tasks(&self) {
        let network_clone = self.clone();
        std::thread::spawn(move || {
            loop {
                std::thread::sleep(Duration::from_millis(5 * 1000));
                network_clone.update_network_statistics();
            }
        });
    }
}

#[get("/")]
async fn index(session: Session, state: Data<AppState>) -> HttpResponse {
    if is_admin_state(&session, state) {
        return HttpResponse::Found()
            .append_header(("Location", "/dashboard"))
            .body("You will soon be redirected.");
    }
    HttpResponse::Ok()
        .body(std::fs::read_to_string("static/index.html").expect("Failed to read file"))
}

#[get("/media")]
async fn media_page() -> HttpResponse {
    if !get_media_enabled_database() {
        return HttpResponse::Forbidden().json(ErrorCode::MediaCenterDisabled.as_error_message());
    }
    HttpResponse::Ok()
        .body(std::fs::read_to_string("static/media.html").expect("Failed to read alerts page"))
}

#[get("/shared")]
async fn shared_page() -> impl Responder {
    std::fs::read_to_string("static/shared.html").unwrap()
}

#[get("/robots.txt")]
async fn robots_txt() -> impl Responder {
    include_str!("../../assets/robots.txt")
}

#[get("/dashboard")]
pub async fn dashboard_page(session: Session, state: Data<AppState>) -> HttpResponse {
    if !is_admin_state(&session, state) {
        return HttpResponse::Found()
            .append_header(("Location", "/"))
            .body("You will soon be redirected");
    }
    HttpResponse::Ok()
        .body(std::fs::read_to_string("static/dashboard.html").expect("Failed to read file"))
}

/// Single path schema
#[derive(Deserialize, Serialize)]
struct SinglePath {
    path: PathBuf,
}

/// Only contains a sudo password
#[derive(Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
struct SudoPasswordReq {
    sudo_password: String,
}

fn configure_multipart(cfg: &mut web::ServiceConfig) {
    cfg.app_data(MultipartFormConfig::default().total_limit(1024 * 1024 * 1024 * 32));
}

/// Restricts private routes to the administrator
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
async fn main() -> std::io::Result<()> {
    use utils::models::Configurations;
    use utils::schema::Configuration::dsl::*;

    CombinedLogger::init(vec![
        TermLogger::new(
            LevelFilter::Info,
            Config::default(),
            TerminalMode::Mixed,
            ColorChoice::Auto,
        ),
        WriteLogger::new(
            LevelFilter::Debug,
            Config::default(),
            File::create(format!(
                "zentrox_{}.log",
                utils::time::current_timestamp_iso()
            ))
            .unwrap(),
        ),
    ])
    .unwrap();

    let os_args = std::env::args().collect::<Vec<String>>();

    match os_args.get(1) {
        Some(arg) if arg == "--docs" => generate_contract::generate(os_args.get(2)),
        Some(arg) if arg == "--help" => help::print(),
        _ => {}
    }

    let zentrox_env_dir = dirs::home_dir()
        .unwrap()
        .join(".local")
        .join("share")
        .join("zentrox");

    if !zentrox_env_dir.join("database.db").exists() {
        debug!("No configuration found, running setup.");
        let _ = setup::run_setup();
    } else {
        debug!("Found configurations in {}", zentrox_env_dir.display())
    }

    if !env::current_dir().unwrap().join("static").exists() {
        let _ = env::set_current_dir(&zentrox_env_dir);
    }

    let app_state = Data::new(AppState::new());
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

    let cert_file_path = zentrox_env_dir
        .join("certificates")
        .join(&tls_cert_filename);

    let mut certs_file = BufReader::new(File::open(&cert_file_path).unwrap());

    debug!("Using certificate file from {}", cert_file_path.display());

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

    let tls_config = rustls::ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(tls_certs, rustls::pki_types::PrivateKeyDer::Pkcs8(tls_key))
        .unwrap();

    let governor_conf = GovernorConfigBuilder::default()
        .burst_size(100)
        .period(Duration::from_millis(250))
        .finish()
        .unwrap();

    let harsh_governor_conf = GovernorConfigBuilder::default()
        .requests_per_minute(2)
        .finish()
        .unwrap();

    let shared_files_governor_conf = GovernorConfigBuilder::default()
        .requests_per_minute(9) // ~3 downloads / minute
        .finish()
        .unwrap();

    info!("Zentrox is being serverd on port {}", SERVER_PORT);

    let secret_session_key = Key::try_generate().expect("Failed to generate session key.");

    HttpServer::new(move || {
        let mut cors_vars = std::env::vars();
        let cors_permissive: bool = cors_vars.any(|x| {
            x == ("ZENTROX_MODE".to_string(), "NO_CORS".to_string())
                || x == ("ZENTROX_MODE".to_string(), "DEV".to_string())
        });

        App::new()
            .wrap(middleware::Logger::new("%a %U %s"))
            .wrap(
                SessionMiddleware::builder(
                    CookieSessionStore::default(),
                    secret_session_key.clone(),
                )
                .cookie_content_security(CookieContentSecurity::Private)
                .session_lifecycle(
                    PersistentSession::default().session_ttl(ActixDuration::hours(12)),
                )
                .cookie_secure(true)
                .cookie_name("session".to_string())
                .build(),
            )
            .wrap(if cors_permissive {
                warn!("CORS policy is set to permissive! This poses a high security risk.");
                Cors::default()
                    .allow_any_method()
                    .allowed_origin("http://localhost:3000")
                    .block_on_origin_mismatch(true)
            } else {
                Cors::default()
            })
            .wrap(middleware::Compress::default())
            .wrap(Governor::new(&governor_conf))
            .configure(configure_multipart)
            .app_data(app_state.clone())
            .service(index)
            .service(dashboard_page)
            .service(robots_txt)
            .service(
                web::scope("/api")
                    .service(
                        web::scope("/public")
                            // WARN These routes can be accessed by anyone
                            .service(
                                web::scope("/auth")
                                    .wrap(Governor::new(&harsh_governor_conf))
                                    // WARN Add account lock down on too many failed attempts even from
                                    // multiple IPs
                                    .route("/login", web::post().to(auth::login))
                                    .route("/useOtp", web::get().to(auth::use_otp)), // WARN Should only respond if password was correct
                                                                                     // WARN OTP secret should be encrypted
                            )
                            .service(
                                web::scope("/shared")
                                    .wrap(Governor::new(&shared_files_governor_conf))
                                    .service(shared_page)
                                    .route("/get", web::post().to(sharing::download_file))
                                    .route("/getMetadata", web::post().to(sharing::get_metadata)),
                            ),
                    )
                    .service(
                        web::scope("/private")
                            // These routes are restricted to the administrator by
                            // middleware
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
                                    .service(media_page)
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
    .keep_alive(Duration::from_secs(60 * 6))
    .bind_rustls_0_23(("0.0.0.0", SERVER_PORT), tls_config)? // TODO Allow user to decide port and IP
    .run()
    .await
}
