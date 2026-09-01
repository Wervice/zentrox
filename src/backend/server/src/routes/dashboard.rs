use actix_web::{HttpResponse, web::Data};
use api::dashboard::DeviceInformationRes;
use diesel::prelude::*;
use diesel::{RunQueryDsl, SelectableHelper};
use std::fs;
use utils::{daemons::Daemon, models::Configurations, schema::Configuration::dsl::*, time};

use crate::AppState;

#[
utoipa::path(
    get,
    path = "/private/dashboard/information",
    tags = ["private", "dashboard"],
    responses((status = 200, body = DeviceInformationRes))
    )
]
pub async fn information(state: Data<AppState>) -> HttpResponse {
    let utc_offset = time::get_utc_offset();

    let mut kernel_version: Option<String> = None;
    if let Ok(contents) = fs::read_to_string("/proc/version") {
        let mut seg = contents.split(' ');
        if seg.clone().count() < 3 {
            log::warn!("Kernel version file /proc/version could not be parsed!");
        } else {
            kernel_version = Some(seg.nth(2).unwrap().to_string())
        }
    }

    // Get operating system name from /etc/os-release
    let os_release = fs::read_to_string("/etc/os-release");
    let mut os_name = None;
    if let Ok(s) = os_release {
        s.lines().for_each(|l| {
            if l.starts_with("PRETTY_NAME") {
                // The operating system is named using this key
                os_name = Some(l.split("=").nth(1).unwrap_or("").replace("\"", ""));
            }
        });
    }

    let mut relevant_services: Vec<(String, bool)> = Vec::new();

    [
        "docker",
        "sshd",
        "telnet",
        "fail2ban",
        "pihole-FTL",
        "apache",
        "ngnix",
        "sendmail",
        "postfix",
        "cron",
        "cronie",
        "NetworkManager",
    ]
    .iter()
    .for_each(|e| {
        if let Ok(data) = Daemon::try_from_id(format!("{e}.service")) {
            relevant_services.push((e.to_string(), data.is_active()));
        }
    });

    let server_name_value = Configuration
        .select(Configurations::as_select())
        .get_result(&mut state.db_pool.lock().unwrap().get().unwrap())
        .unwrap()
        .server_name;

    HttpResponse::Ok().json(DeviceInformationRes {
        kernel_version,
        os_name,
        utc_offset,
        relevant_services,
        server_name: server_name_value,
    })
}
