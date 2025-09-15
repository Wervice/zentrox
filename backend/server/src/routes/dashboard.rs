use actix_session::Session;
use actix_web::{HttpResponse, web::Data};
use serde::Serialize;
use std::fs;
use std::net::IpAddr;
use sysinfo::Components;
use utils::net_data::private_ip;
use utoipa::ToSchema;

use crate::{AppState, is_admin::is_admin_state};

/// A single thermometer reading with a name
#[derive(Serialize, ToSchema)]
struct Thermometer {
    label: String,
    critical: Option<f32>,
    reading: Option<f32>,
}

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
struct DeviceInformationRes {
    hostname: Option<String>,
    #[schema(value_type = Option<String>)]
    ip: Option<IpAddr>,
    /// Milliseconds since last boot
    uptime: u128,
    thermometers: Vec<Thermometer>,
    zentrox_pid: u32,
    network_bytes_up: Option<f64>,
    network_bytes_down: Option<f64>,
    most_active_network_interface: Option<String>,
    network_interfaces_count: usize,
    memory_total_bytes: u64,
    memory_free_bytes: u64,
    cpu_usage: f32,
    os_name: Option<String>,
}

#[
utoipa::path(
    get,
    path = "/private/dashboard/information",
    tags = ["private", "dashboard"],
    responses((status = 200, body = DeviceInformationRes))
    )
]
pub async fn information(state: Data<AppState>) -> HttpResponse {
    // Current machines host-name. i.e.: debian_pc or 192.168.1.3
    let hostname = match fs::read_to_string("/etc/hostname") {
        Ok(reading) => Some(reading.replace("\n", "")),
        Err(_) => None,
    };

    let uptime = utils::uptime::get().unwrap().as_millis();

    // A refreshed list of all thermometer components in the system is obtained.
    let thermometers_component_list = Components::new_with_refreshed_list();
    let thermometers: Vec<Thermometer> = thermometers_component_list
        .iter()
        .map(|component| Thermometer {
            label: component.label().to_string(),
            reading: match component.temperature() {
                Some(unwrapped_reading) => {
                    if unwrapped_reading.is_nan() {
                        None
                    } else {
                        Some(unwrapped_reading)
                    }
                }
                None => None,
            },
            critical: match component.critical() {
                Some(unwrapped_reading) => {
                    if unwrapped_reading.is_nan() {
                        None
                    } else {
                        Some(unwrapped_reading)
                    }
                }
                None => None,
            },
        })
        .collect();

    // Refresh current data in the shared system instance.
    let mut locked_system_instance = state.system.lock().unwrap();
    locked_system_instance.refresh_memory();
    locked_system_instance.refresh_cpu_usage();

    // Obtain device statistics
    let cpu_usage = locked_system_instance.global_cpu_usage() / 100_f32;
    let memory_total_bytes = locked_system_instance.total_memory();
    let memory_free_bytes = locked_system_instance.available_memory();

    // Default values are None if no interface could be found to obtain the measurements from.
    let mut network_bytes_down = None;
    let mut network_bytes_up = None;
    let mut most_active_network_interface = None;

    let network_interfaces = state.network_interfaces.lock().unwrap();
    let network_interfaces_count = &network_interfaces.iter().len();

    let mut current_highest_interface_activity: f64 = 0.0;

    for interface in network_interfaces.iter() {
        let sum = interface.up + interface.down;
        if sum > current_highest_interface_activity {
            most_active_network_interface = Some(interface.name.clone());
            network_bytes_up = Some(interface.up);
            network_bytes_down = Some(interface.down);
            current_highest_interface_activity = sum;
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

    HttpResponse::Ok().json(DeviceInformationRes {
        zentrox_pid: std::process::id(),
        hostname,
        uptime,
        thermometers,
        network_bytes_down,
        network_bytes_up,
        most_active_network_interface,
        network_interfaces_count: *network_interfaces_count,
        ip: private_ip().ok(),
        memory_free_bytes,
        memory_total_bytes,
        cpu_usage,
        os_name,
    })
}

/// The dashboard route.
///
/// If the user is logged in, the dashboard is shown, otherwise they get redirected to root.
pub async fn page(session: Session, state: Data<AppState>) -> HttpResponse {
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
