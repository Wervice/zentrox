use std::{fs, process::exit};

use utoipa::{OpenApi, openapi::ServerBuilder};

pub fn generate(store_path: Option<&String>) {
    println!("Generating OpenAPI contract in pretty printed JSON format.");

    #[derive(OpenApi)]
    #[openapi(
        paths(
            crate::routes::auth::login,
            crate::routes::auth::logout,
            crate::routes::auth::use_otp,
            crate::routes::auth::activate_otp,
            crate::routes::auth::verify_sudo_password,
            crate::routes::dashboard::information,
            crate::routes::packages::database,
            crate::routes::packages::statistics,
            crate::routes::packages::update_db,
            crate::routes::packages::install_package,
            crate::routes::packages::remove_package,
            crate::routes::packages::update_package,
            crate::routes::packages::update_all,
            crate::routes::packages::remove_orphaned,
            crate::routes::packages::orphaned,
            crate::routes::jobs::status,
            crate::routes::firewall::has_ufw,
            crate::routes::firewall::status,
            crate::routes::firewall::switch,
            crate::routes::firewall::delete_rule,
            crate::routes::firewall::new_rule,
            crate::routes::files::download,
            crate::routes::files::list,
            crate::routes::files::delete,
            crate::routes::files::move_to,
            crate::routes::files::burn,
            crate::routes::files::metadata,
            crate::routes::files::upload,
            crate::routes::drives::list,
            crate::routes::vault::is_configured,
            crate::routes::vault::configure,
            crate::routes::vault::tree,
            crate::routes::vault::delete_file,
            crate::routes::vault::new_directory,
            crate::routes::vault::upload,
            crate::routes::vault::download_file,
            crate::routes::vault::rename_file,
            crate::routes::power::off,
            crate::routes::tls::name,
            crate::routes::tls::upload,
            crate::routes::account::details,
            crate::routes::account::update_details,
            crate::routes::account::picture,
            crate::routes::account::upload_picture,
            crate::routes::logs::read,
            crate::routes::media::get_sources,
            crate::routes::media::update_sources,
            crate::routes::media::get_media_enabled_handler,
            crate::routes::media::activate_media,
            crate::routes::media::get_contents,
            crate::routes::media::download,
            crate::routes::media::cover,
            crate::routes::media::read_history,
            crate::routes::media::update_metadata,
            crate::routes::network::interfaces,
            crate::routes::network::routes,
            crate::routes::network::delete_route,
            crate::routes::network::activate_interface,
            crate::routes::processes::list,
            crate::routes::processes::kill,
            crate::routes::processes::details,
            crate::routes::cron::run_command,
            crate::routes::cron::delete,
            crate::routes::cron::create,
            crate::routes::cron::list,
            crate::routes::sharing::share,
            crate::routes::sharing::list,
            crate::routes::sharing::unshare,
            crate::routes::sharing::download_file,
            crate::routes::sharing::get_metadata,
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
