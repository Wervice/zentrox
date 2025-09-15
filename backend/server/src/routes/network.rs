use actix_web::{
    HttpResponse,
    web::{Data, Json},
};
use serde::{Deserialize, Serialize};
use std::{net::IpAddr, str::FromStr};
use utils::status_com::ErrorCode;
use utils::{
    net_data::{self, DeletionRoute, Destination, IpAddrWithSubnet, Route},
    status_com::MessageRes,
    sudo,
};
use utoipa::ToSchema;

use crate::{AppState, MeasuredInterface};

#[derive(Serialize, ToSchema)]
struct NetworkInterfacesRes {
    interfaces: Vec<MeasuredInterface>,
}

#[derive(Serialize, ToSchema)]
struct NetworkRoutesRes {
    routes: Vec<Route>,
}

/// List of known network interfaces
#[utoipa::path(get, path = "/private/network/interfaces", tags = ["private", "network"], responses((status = 200, body = NetworkInterfacesRes)))]
pub async fn network_interfaces(state: Data<AppState>) -> HttpResponse {
    let interfaces = state.network_interfaces.lock().unwrap().clone();

    return HttpResponse::Ok().json(NetworkInterfacesRes { interfaces });
}

#[utoipa::path(get, path = "/private/network/routes", tags = ["private", "network"], responses((status = 200, body = NetworkRoutesRes)))]
/// List of network routes
pub async fn network_routes() -> HttpResponse {
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
pub struct DeleteNetworkRouteReq {
    device: String,
    destination: Option<AdressRequestSchema>,
    gateway: Option<AdressRequestSchema>,
    sudo_password: String,
}

#[utoipa::path(post, path = "/private/network/route/delete", request_body = DeleteNetworkRouteReq, responses((status = 200), (status = 401, description = "The provided sudo password was wrong.")), tags = ["private", "network"])]
/// Delete network route
pub async fn delete_network_route(json: Json<DeleteNetworkRouteReq>) -> HttpResponse {
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
pub struct NetworkingInterfaceActivityReq {
    activity: bool,
    interface: String,
    sudo_password: String,
}

/// Set activity of a network interface
#[utoipa::path(post, path = "/private/network/interface/active", responses((status = 200)), request_body = NetworkingInterfaceActivityReq, tags = ["private", "network"])]
pub async fn network_interface_active(json: Json<NetworkingInterfaceActivityReq>) -> HttpResponse {
    if json.activity {
        net_data::enable_interface(json.sudo_password.clone(), json.interface.clone());
    } else {
        net_data::disable_interface(json.sudo_password.clone(), json.interface.clone());
    }
    return HttpResponse::Ok().json(MessageRes::from("The interface has been updated."));
}
