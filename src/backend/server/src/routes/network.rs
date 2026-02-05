use actix_web::{
    HttpResponse,
    web::{Data, Json},
};
use serde::{Deserialize, Serialize};
use std::{net::IpAddr, str::FromStr};
use utils::{net_data::Interface, status_com::ErrorCode};
use utils::{
    net_data::{self, DeletionRoute, Destination, IpAddrWithSubnet, Route},
    status_com::MessageRes,
};
use utoipa::ToSchema;

use crate::AppState;

#[derive(Serialize, ToSchema)]
struct NetworkInterfacesRes {
    interfaces: Vec<Interface>,
}

#[derive(Serialize, ToSchema)]
struct NetworkRoutesRes {
    routes: Vec<Route>,
}

/// List of known network interfaces
#[utoipa::path(get, path = "/private/network/interfaces", tags = ["private", "network"], responses((status = 200, body = NetworkInterfacesRes)))]
pub async fn interfaces(state: Data<AppState>) -> HttpResponse {
    let interfaces = state.network_interfaces.lock().unwrap().clone();

    HttpResponse::Ok().json(NetworkInterfacesRes { interfaces })
}

#[utoipa::path(get, path = "/private/network/routes", tags = ["private", "network"], responses((status = 200, body = NetworkRoutesRes)))]
/// List of network routes
pub async fn routes() -> HttpResponse {
    let routes = net_data::get_routes();

    HttpResponse::Ok().json(NetworkRoutesRes {
        routes: routes.unwrap(),
    })
}

#[derive(Deserialize, ToSchema)]
struct AddressRequestSchema {
    adress: String,
    subnet: Option<i32>,
}

#[derive(Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct DeleteNetworkRouteReq {
    device: String,
    destination: Option<AddressRequestSchema>,
    gateway: Option<AddressRequestSchema>,
    sudo_password: String,
}

#[utoipa::path(post, path = "/private/network/route/delete", request_body = DeleteNetworkRouteReq, responses((status = 200), (status = 401, description = "The provided sudo password was wrong.")), tags = ["private", "network"])]
/// Delete network route
pub async fn delete_route(json: Json<DeleteNetworkRouteReq>) -> HttpResponse {
    let built_deletion_route = DeletionRoute {
        device: json.device.clone(),
        nexthop: None,
        gateway: json.gateway.as_ref().map(|x| IpAddrWithSubnet {
            address: IpAddr::from_str(&x.adress).unwrap(),
            subnet: x.subnet,
        }),
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
        Ok(_) => HttpResponse::Ok().json(MessageRes::from("The route has been updated.")),
        _ => HttpResponse::Unauthorized().json(ErrorCode::BadSudoPassword),
    }
}

#[derive(Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct NetworkingInterfaceActivityReq {
    activity: bool,
    interface: String,
    sudo_password: String,
}

/// Set activity of a network interface
#[utoipa::path(post, path = "/private/network/interface/active", responses((status = 200)), request_body = NetworkingInterfaceActivityReq, tags = ["private", "network"])]
pub async fn activate_interface(json: Json<NetworkingInterfaceActivityReq>) -> HttpResponse {
    if json.activity {
        let _ = net_data::enable_interface(json.sudo_password.clone(), json.interface.clone());
    } else {
        let _ = net_data::disable_interface(json.sudo_password.clone(), json.interface.clone());
    }
    HttpResponse::Ok().json(MessageRes::from("The interface has been updated."))
}
