use std::{fs, process::exit};

use utoipa::{openapi::ServerBuilder, OpenApi};

pub fn generate(store_path: Option<&String>) {
    println!("Generating OpenAPI contracg in pretty printed JSON format.");

    #[derive(OpenApi)]
    #[openapi(
        paths(
            crate::routes::auth::verification,
            crate::routes::auth::logout,
            crate::routes::auth::use_otp,
            crate::routes::auth::otp_activation,
            crate::routes::auth::verify_sudo_password,
            crate::routes::dashboard::device_information,
            crate::routes::packages::package_database,
            crate::routes::packages::package_statistics,
            crate::routes::packages::update_package_database,
            crate::routes::packages::install_package,
            crate::routes::packages::remove_package,
            crate::routes::packages::update_package,
            crate::routes::packages::update_all_packages,
            crate::routes::packages::remove_orphaned_packages,
            crate::routes::packages::orphaned_packages,
            crate::routes::jobs::fetch_job_status,
            crate::routes::firewall::firewall_has_ufw,
            crate::routes::firewall::firewall_information,
            crate::routes::firewall::switch_ufw,
            crate::routes::firewall::delete_firewall_rule,
            crate::routes::firewall::new_firewall_rule,
            crate::routes::files::download_file,
            crate::routes::files::files_list,
            crate::routes::files::delete_file,
            crate::routes::files::move_path,
            crate::routes::files::burn_file,
            crate::routes::files::get_file_metadata,
            crate::routes::files::upload_file,
            crate::routes::drives::list_drives,
            crate::routes::drives::drive_information,
            crate::routes::vault::is_vault_configured,
            crate::routes::vault::vault_configure,
            crate::routes::vault::vault_tree,
            crate::routes::vault::delete_vault_file,
            crate::routes::vault::vault_new_folder,
            crate::routes::vault::upload_vault,
            crate::routes::vault::vault_file_download,
            crate::routes::vault::rename_vault_file,
            crate::routes::power::power_off,
            crate::routes::tls::cert_names,
            crate::routes::tls::upload_tls,
            crate::routes::account::account_details,
            crate::routes::account::update_account_details,
            crate::routes::account::profile_picture,
            crate::routes::account::upload_profile_picture,
            crate::routes::logs::logs_request,
            crate::routes::media::get_media_list,
            crate::routes::media::media_request,
            crate::routes::media::get_media_source_list,
            crate::routes::media::update_media_source_list,
            crate::routes::media::get_cover,
            crate::routes::media::get_media_enabled_handler,
            crate::routes::media::set_enable_media,
            crate::routes::media::read_full_media_history,
            crate::routes::media::update_media_metadata,
            crate::routes::network::network_interfaces,
            crate::routes::network::network_routes,
            crate::routes::network::delete_network_route,
            crate::routes::network::network_interface_active,
            crate::routes::processes::list_processes,
            crate::routes::processes::kill_process,
            crate::routes::processes::details_process,
            crate::routes::cron::run_cronjob_command,
            crate::routes::cron::delete_cronjob,
            crate::routes::cron::create_cronjob,
            crate::routes::cron::list,
            crate::routes::sharing::share_file,
            crate::routes::sharing::get_shared_files_list,
            crate::routes::sharing::unshare_file,
            crate::routes::sharing::get_shared_file,
            crate::routes::sharing::get_shared_file_metadata,
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
