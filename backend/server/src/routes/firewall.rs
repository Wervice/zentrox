use actix_web::{HttpResponse, web::Json};
use log::error;
use serde::{Deserialize, Serialize};
use utils::{
    status_com::{ErrorCode, MessageRes},
    sudo,
    ufw::{self, FirewallAction, UfwRule},
};
use utoipa::ToSchema;

use crate::SudoPasswordReq;

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
pub async fn firewall_has_ufw() -> HttpResponse {
    let check = utils::packages::list_installed_packages().contains(&String::from("ufw"));
    HttpResponse::Ok().json(HasUfwReq { has: check })
}

#[derive(Serialize, ToSchema)]
struct FirewallInformationRes {
    enabled: bool,
    rules: Vec<UfwRule>,
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
pub async fn firewall_information(json: Json<SudoPasswordReq>) -> HttpResponse {
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
pub struct SwitchUfwReq {
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
/// Set UFW to be enabled or disabled
pub async fn switch_ufw(json: Json<SwitchUfwReq>) -> HttpResponse {
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
pub struct NewFirewallRuleReq {
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
pub async fn new_firewall_rule(json: Json<NewFirewallRuleReq>) -> HttpResponse {
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
pub struct FirewallDeleteRuleReq {
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
pub async fn delete_firewall_rule(json: Json<FirewallDeleteRuleReq>) -> HttpResponse {
    let password = &json.sudo_password;

    match ufw::delete_rule(password.to_string(), json.index) {
        Ok(_) => HttpResponse::Ok().json(MessageRes::from("The rule has been deleted.")),
        Err(e) => HttpResponse::InternalServerError()
            .json(ErrorCode::UfwExecutionFailed(e).as_error_message()),
    }
}
