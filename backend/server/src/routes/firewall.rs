use std::{net::IpAddr, str::FromStr};

use actix_web::{HttpResponse, web::Json};
use log::error;
use serde::{Deserialize, Serialize};
use utils::{
    status_com::{ErrorCode, MessageRes},
    ufw::{self, Address, HelperDefaults, Port, Rule, UfwInteractionError},
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
pub async fn has_ufw() -> HttpResponse {
    let check = utils::packages::list_installed_packages().contains(&String::from("ufw"));
    HttpResponse::Ok().json(HasUfwReq { has: check })
}

#[derive(Serialize, ToSchema)]
struct FirewallInformationRes {
    enabled: bool,
    rules: Vec<Rule>,
    defaults: HelperDefaults,
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
pub async fn status(json: Json<SudoPasswordReq>) -> HttpResponse {
    let password = &json.sudo_password;

    match ufw::status(password.to_string()) {
        Ok(ufw_status) => {
            let enabled = ufw_status.enabled;
            let rules = ufw_status.rules;
            let defaults = ufw_status.defaults;

            HttpResponse::Ok().json(FirewallInformationRes {
                enabled,
                rules,
                defaults,
            })
        }
        Err(_err) => {
            error!("Failed to get UFW status.");
            HttpResponse::InternalServerError()
                .json(ErrorCode::MissingSystemPermissions.as_error_message())
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
pub async fn switch(json: Json<SwitchUfwReq>) -> HttpResponse {
    let password = json.sudo_password.clone();
    let enabled = json.enabled;
    match ufw::set_enabled(password, enabled) {
        Ok(o) => {
            if o.status == Some(0) {
                HttpResponse::Ok().json(MessageRes::from("UFW has been started."))
            } else {
                HttpResponse::InternalServerError()
                    .json(ErrorCode::UfwExecutionFailedWithStatus(o.status).as_error_message())
            }
        }
        Err(_) => HttpResponse::InternalServerError()
            .json(ErrorCode::MissingSystemPermissions.as_error_message()),
    }
}

/// Deserializes values for a [Rule](utils::ufw::Rule) from JSON.
/// Omitting a value that can be omitted using null, defaults to Any.
#[derive(ToSchema, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ReqRule {
    v6: bool,
    direction: ufw::Direction,
    action: ufw::Action,
    source_address: Option<String>,
    destination_address: Option<String>,
    source_subnet: Option<u32>,
    destination_subnet: Option<u32>,
    source_port: (Option<u16>, Option<u16>),
    destination_port: (Option<u16>, Option<u16>),
    protocol: ufw::Protocol,
    comment: String,
    interface_in: Option<String>,
    interface_out: Option<String>,
}

#[derive(ToSchema, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RuleCreationReq {
    rule: ReqRule,
    sudo_password: String,
}

enum RuleReqParsingError {
    BadIp,
}

fn port_from_pair(pair: (Option<u16>, Option<u16>)) -> Port {
    let (l, r) = pair;
    match l {
        Some(lv) => match r {
            Some(rv) => Port::Range(lv, rv),
            None => Port::Specific(lv),
        },
        None => Port::Any,
    }
}

impl ReqRule {
    fn to_backend_rule(&self) -> Result<ufw::Rule, RuleReqParsingError> {
        let req_rule_destination_address: Address;
        if let Some(addr) = &self.destination_address {
            if let Ok(parsed_addr) = IpAddr::from_str(addr.as_str()) {
                req_rule_destination_address = Address::Specific(parsed_addr)
            } else {
                return Err(RuleReqParsingError::BadIp);
            }
        } else {
            req_rule_destination_address = Address::Any
        }

        let req_rule_source_address: Address;
        if let Some(addr) = &self.source_address {
            if let Ok(parsed_addr) = IpAddr::from_str(addr.as_str()) {
                req_rule_source_address = Address::Specific(parsed_addr)
            } else {
                return Err(RuleReqParsingError::BadIp);
            }
        } else {
            req_rule_source_address = Address::Any
        }

        Ok(Rule {
            v6: self.v6,
            destination: ufw::Point {
                subnet: self.destination_subnet,
                address: req_rule_destination_address,
            },
            source: ufw::Point {
                subnet: self.source_subnet,
                address: req_rule_source_address,
            },
            destination_port: port_from_pair(self.destination_port),
            source_port: port_from_pair(self.source_port),
            protocol: self.protocol,
            destination_app: "".to_string(),
            source_app: "".to_string(),
            action: self.action,
            interface_in: self.interface_in.clone(),
            interface_out: self.interface_out.clone(),
            direction: self.direction,
            comment: self.comment.clone(),
            forward: false,
            index: None,
        })
    }
}

#[utoipa::path(
    post,
    path = "/private/firewall/rule/new",
    responses((status = 200), (status = 500, description = "UFW failed."), (status = 400, description = "The rule already existed."), (status = 401, description = "The sudo password was wrong.")),
    tags = ["private", "firewall"],
    request_body = RuleCreationReq
)]
/// Create firewall rule.
pub async fn new_rule(json: Json<RuleCreationReq>) -> HttpResponse {
    if let Ok(be_rule) = json.rule.clone().to_backend_rule() {
        let exec = ufw::new_rule(json.sudo_password.clone(), be_rule);
        match exec {
            Ok(_) => HttpResponse::Ok().json(MessageRes::from("The rule has been created.")),
            Err(err) => match err {
                UfwInteractionError::RuleSkipped => {
                    HttpResponse::BadRequest().json(ErrorCode::RuleSkipped.as_error_message())
                }
                UfwInteractionError::SudoFailed(_) => {
                    HttpResponse::Unauthorized().json(ErrorCode::BadSudoPassword.as_error_message())
                }
                UfwInteractionError::UnknownState(out, err, segs) => {
                    HttpResponse::InternalServerError()
                        .json(ErrorCode::UfwError(out, err, segs).as_error_message())
                }
                UfwInteractionError::NotFound => {
                    unreachable!("Creating a rule resulted in it not getting found.")
                }
            },
        }
    } else {
        HttpResponse::BadRequest().json(ErrorCode::BadRule.as_error_message())
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
    responses((status = 200), (status = 404, description = "The rule doesn't exist."), (status = 401, description = "The sudo password was wrong."), (status = 500, description = "UFW failed.")),
    request_body = FirewallDeleteRuleReq
)]
/// Delete firewall rule
pub async fn delete_rule(json: Json<FirewallDeleteRuleReq>) -> HttpResponse {
    let exec = ufw::delete_rule(json.sudo_password.clone(), json.index);

    match exec {
        Ok(_) => HttpResponse::Ok().json(MessageRes::from("The rule has been deleted.")),
        Err(err) => match err {
            UfwInteractionError::RuleSkipped => {
                unreachable!("Deleting a rule resulted in a rule being created.")
            }
            UfwInteractionError::SudoFailed(_) => {
                HttpResponse::Unauthorized().json(ErrorCode::BadSudoPassword.as_error_message())
            }
            UfwInteractionError::UnknownState(out, err, segs) => {
                HttpResponse::InternalServerError()
                    .json(ErrorCode::UfwError(out, err, segs).as_error_message())
            }
            UfwInteractionError::NotFound => {
                HttpResponse::NotFound().json(ErrorCode::NoSuchRule.as_error_message())
            }
        },
    }
}
