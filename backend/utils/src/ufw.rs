//! Provides an interaction layer with the [Uncomplicated Firewall](https://help.ubuntu.com/community/UFW) command interface and backend.
//! The module uses a Python helper program to retrieve data from the frontend.
//! Rules and defaults are changed, created and set using the ufw frontend via commands.
//! Data from the helper program is deserialized and further processed into different structs.
//! To command is exexcuted using dry-run.
//!
//! This module now exposes serveral functions and structs. Those include
//! - `status(password: String)` - [status] retrieves the rules and settings of UFW
//! - `new_rule(password: String, rule: Rule)` - [new_rule] creates a new rule.
//! - `delete_rule(password: String, index: u32)` - [delete_rule] deletes the n-th rule.
//! - `rules_raw()` - [rules_raw] returns the raw user.rule file containing the iptables rules.
//! - `set_defaults(password: String, defaults: Defaults)` - [set_defaults] sets the default actions for incoming, outgoing and forwarded requests.
//! - `set_enabled(password: String, enabled: bool)` - [set_enabled] enables or disables the firewall.

use std::{
    fmt::Display, fs::{self, exists}, net::IpAddr, str::FromStr
};

use log::{error, info, warn};
use serde::{self, Deserialize, Serialize};
use utoipa::ToSchema;

use crate::sudo::{self, SudoCommand, SudoError, SudoOutput};

const UFW_PATH: &str = "/usr/sbin/ufw";
const UFW_HELPER_PATH: &str = "assets/ufw_helper.py";
const UFW_RULES: &str = "/etc/ufw/user.rules";

#[derive(Debug)]
pub enum UfwInteractionError {
    /// The sudo API failed
    SudoFailed(SudoError),
    /// While creating a rule, the creation was skipped because the rule already existed
    RuleSkipped,
    /// A rule could not be found
    NotFound,
    /// UFW responded with an unknown error, capturing stdout, stdder and the arguments
    UnknownState(String, String, Vec<String>),
}

#[derive(Debug)]
/// A helper error is used if the `ufw_helper.py` script fails.
pub enum HelperError {
    /// The `assets/ufw_helper.py` file was not found.
    ProgramNotFound,
    /// The helper script failed.
    BadExitCode(i32),
    /// The helper encountered an issue while running.
    ExecutionError,
}

#[derive(Debug, Serialize, ToSchema, Clone, Copy)]
/// An address can be a specific v4 or v6 IP address or Any.
/// This is used for destinations and source for rules.
pub enum Address {
    Any,
    #[schema(value_type = Option<String>)]
    Specific(IpAddr),
}

#[derive(Deserialize, Debug, Clone, Copy, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
/// A protocol is every protocol supported for UFW rules.
pub enum Protocol {
    Tcp,
    Udp,
    Any,
}

#[derive(Deserialize, Debug, Clone, Copy, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
/// The direction of an action (i.e. DENY IN).
pub enum Direction {
    In,
    Out,
    Forward,
}

#[derive(Debug, Serialize, ToSchema, Copy, Clone)]
pub enum Port {
    Any,
    Specific(u16),
    Range(u16, u16),
}

#[derive(Deserialize, Debug, Copy, Clone, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
/// An action that can be performed by UFW
pub enum Action {
    Deny,
    Allow,
    Reject,
    Limit
}

/// A default action as described by UFW.
/// It extends the default iptables actions `DROP` and `ACCEPT` with `skip`
#[derive(Deserialize, Serialize, Debug, PartialEq, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum DefaultAction {
    Drop,
    Accept,
    Skip,
}

#[derive(Debug)]
/// Describes the global log level of UFW where every value corresponds to a number between 0 to 4.
pub enum LogLevel {
    Off,
    Low,
    Medium,
    High,
    Full,
}

impl Display for Protocol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Tcp => f.write_str("tcp"),
            Self::Udp => f.write_str("udp"),
            Self::Any => f.write_str("any"),
        }
    }
}

impl Display for Direction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::In => f.write_str("in"),
            Self::Out => f.write_str("out"),
            Self::Forward => f.write_str("fwd"),
        }
    }
}

impl From<&str> for Port {
    fn from(value: &str) -> Self {
        if value == "any" {
            return Port::Any;
        }
        if value.contains(":") {
            let segs = value.split(":").collect::<Vec<&str>>();
            if segs.len() != 2 {
                panic!("Malformed port range");
            }
            return Port::Range(
                u16::from_str(segs[0]).unwrap(),
                u16::from_str(segs[1]).unwrap(),
            );
        }
        Port::Specific(u16::from_str(value).unwrap())
    }
}

impl TryFrom<&str> for Point {
    type Error = &'static str;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if value == "0.0.0.0/0" || value == "::/0" {
            return Ok(Point {
                subnet: None,
                address: Address::Any,
            });
        }
        let segments: Vec<&str> = value.split("/").collect();
        let mut subnet = None;
        if let Ok(address) = IpAddr::from_str(segments[0]) {
            if segments.len() == 2 {
                subnet = Some(u32::from_str(segments[1]).unwrap());
            }
            Ok(Point {
                subnet,
                address: Address::Specific(address),
            })
        } else {
            Err("Malformed IP address.")
        }
    }
}

impl Display for Action {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Action::Deny => f.write_str("deny"),
            Action::Allow => f.write_str("allow"),
            Action::Reject => f.write_str("reject"),
            Action::Limit => f.write_str("limit")
        }
    }
}

#[derive(Debug, Serialize, ToSchema, Clone)]
/// A Rule describes the values of a possible UFW rule. The values can be parsed from the
/// [HelperRule] which contains raw data from the `ufw_helper.py`.
/// A rule can also be built and later created using UFW. When creating a rule, the index may not
/// be out of the bounds of the UFW rule numbers.
/// Optional values can be omitted when creating a rule.
#[serde(rename_all(serialize = "camelCase"))]
pub struct Rule {
    pub v6: bool,
    pub destination: Point,
    pub source: Point,
    pub destination_port: Port,
    pub source_port: Port,
    pub protocol: Protocol,
    pub destination_app: String,
    pub source_app: String,
    pub action: Action,
    pub interface_in: Option<String>,
    pub interface_out: Option<String>,
    pub direction: Direction,
    pub comment: String,
    pub forward: bool,
    pub index: Option<usize>,
}

#[derive(Debug, Serialize, ToSchema, Clone, Copy)]
/// A point is a destination or source of a request.
/// It is composed of a subnet and an ip address.
pub struct Point {
    pub subnet: Option<u32>,
    pub address: Address,
}

#[derive(Debug)]
/// The [`Status`] describes the outputs of `ufw status`, though the data is recieved through the `ufw_helper.py`.
/// The contents of the struct are derived from the contents of a [HelperStatus] struct.
pub struct Status {
    pub enabled: bool,
    pub logging: LogLevel,
    pub defaults: HelperDefaults,
    pub rules: Vec<Rule>,
}

#[derive(Deserialize, Debug)]
/// A [`HelperRule`] is used to deserialize raw data from the `ufw_helper.py` script.
/// The deserialization will perform basic pre-processing and renaming.
pub struct HelperRule {
    v6: bool,
    #[serde(rename = "dst")]
    destination: String,
    #[serde(rename = "src")]
    source: String,
    #[serde(rename = "dport")]
    destination_port: String,
    #[serde(rename = "sport")]
    source_port: String,
    protocol: Protocol,
    #[serde(rename = "dapp")]
    destination_app: String,
    #[serde(rename = "sapp")]
    source_app: String,
    action: Action,
    interface_in: String,
    interface_out: String,
    direction: Direction,
    forward: bool,
    comment: String,
}

#[derive(Deserialize, Serialize, Debug, ToSchema)]
/// The [`HelperDefaults`] are used to deserialize the data of the `ufw_helper.py` to create a [Defaults]
/// struct.
pub struct HelperDefaults {
    #[serde(rename(deserialize = "default_input_policy"))]
    input: DefaultAction,
    #[serde(rename(deserialize = "default_output_policy"))]
    output: DefaultAction,
    #[serde(rename(deserialize = "default_forward_policy"))]
    forward: DefaultAction,
}

#[derive(Deserialize, Debug)]
/// The [`HelperStatus`] is used to deserialize the raw status information from the `ufw_helper.py`
/// to create a [Status] struct.
pub struct HelperStatus {
    enabled: bool,
    logging: u32,
    defaults: HelperDefaults,
    rules: Vec<HelperRule>,
}

#[derive(Deserialize, Debug)]
/// The [`HelperResponse`] struct is used to deserialize the contents of the response of the
/// `ufw_helper`.py script.
pub struct HelperResponse {
    success: bool,
    data: HelperStatus,
}

/// Represents the default values for UFW when setting values.
/// This is not suitable for reading such values.
/// Setting None will skip the field when setting the values using [set_defaults].
///
/// [`CreationDefaults`]
pub struct CreationDefaults {
    input: Option<Action>,
    output: Option<Action>,
    forward: Option<Action>,
}

/// Recieves the data for a [Status] from the `ufw_helper.py` script using sudo and ufw.
/// If successful it will return a [Status] containing all rules.
///
/// [`status`]
pub fn status(password: String) -> Result<Status, HelperError> {
    info!("Requesting ufw_helper.py to get ufw backend information.");
    if !exists(UFW_HELPER_PATH).unwrap() {
        error!("Failed to locate ufw_helper.py.");
        return Err(HelperError::ProgramNotFound);
    }

    let command = SudoCommand::new(password.to_string(), "python3".to_string())
        .arg(UFW_HELPER_PATH)
        .output()
        .unwrap();

    let status_code = command.status.unwrap();
    if status_code != 0 {
        warn!("UFW helper exited with a bad exit code.");
        return Err(HelperError::BadExitCode(status_code));
    }
    let json = command.stdout;
    let response: HelperResponse = serde_json::from_str(json.as_str()).unwrap();
    if !response.success {
        warn!("UFW helper failed.");
        return Err(HelperError::ExecutionError);
    }
    let helper_status = response.data;

    let enabled = helper_status.enabled;
    let logging = match helper_status.logging {
        0 => LogLevel::Off,
        1 => LogLevel::Low,
        2 => LogLevel::Medium,
        3 => LogLevel::High,
        4 => LogLevel::Full,
        _ => LogLevel::Off,
    };

    let helper_defaults = helper_status.defaults;

    let helper_rules = helper_status.rules;
    let rules = helper_rules
        .iter()
        .enumerate()
        .map(|r| Rule {
            v6: r.1.v6,
            destination: Point::try_from(r.1.destination.as_str())
                .expect("Failed to parse destination."),
            source: Point::try_from(r.1.source.as_str()).expect("Failed to parse source."),
            destination_port: Port::from(r.1.destination_port.as_str()),
            source_port: Port::from(r.1.source_port.as_str()),
            protocol: r.1.protocol,
            destination_app: r.1.destination_app.clone(),
            source_app: r.1.source_app.clone(),
            action: r.1.action,
            interface_in: if r.1.interface_in.is_empty() {
                None
            } else {
                Some(r.1.interface_in.clone())
            },
            interface_out: if r.1.interface_out.is_empty() {
                None
            } else {
                Some(r.1.interface_out.clone())
            },
            direction: r.1.direction,
            comment: if !r.1.comment.is_empty() {
                String::from_utf8(hex::decode(r.1.comment.clone()).unwrap()).unwrap()
            } else {
                "".to_string()
            },
            forward: r.1.forward,
            index: Some(r.0),
        })
        .collect();

    Ok(Status {
        enabled,
        defaults: helper_defaults,
        logging,
        rules,
    })
}

/// Given a [Rule] this function will invoke ufw using sudo to create the rule in the firewall
/// ruleset.
///
/// # Usage
///
/// [`new_rule`]
pub fn new_rule(password: String, rule: Rule) -> Result<Vec<String>, UfwInteractionError> {
    info!("Creating a ufw rule.");
    let mut args: Vec<String> = vec!["rule".to_string()];

    if let Some(index) = rule.index {
        args.push("insert".to_string());
        args.push(index.to_string());
    }

    args.push(rule.action.to_string());
    args.push(rule.direction.to_string());

    if let Some(interface_in) = rule.interface_in {
        args.push("on".to_string());
        args.push(interface_in)
    }

    args.push("proto".to_string());
    args.push(rule.protocol.to_string());
    args.push("from".to_string());

    match rule.source.address {
        Address::Any => {
            args.push("any".to_string());
        }
        Address::Specific(addr) => args.push(addr.to_string()),
    }

    match rule.source_port {
        Port::Any => {}
        Port::Range(l, r) => {
            args.push("port".to_string());
            args.push(format!("{}:{}", l, r).to_string())
        }
        Port::Specific(p) => {
            args.push("port".to_string());
            args.push(p.to_string())
        }
    }

    args.push("to".to_string());

    match rule.destination.address {
        Address::Any => {
            args.push("any".to_string());
        }
        Address::Specific(addr) => args.push(addr.to_string()),
    }

    match rule.destination_port {
        Port::Any => {}
        Port::Range(l, r) => {
            args.push("port".to_string());
            args.push(format!("{}:{}", l, r).to_string())
        }
        Port::Specific(p) => {
            args.push("port".to_string());
            args.push(p.to_string())
        }
    }

    if !rule.comment.is_empty() {
        args.push("comment".to_string());
        args.push(rule.comment);
    }

    let exec = SudoCommand::new(password, UFW_PATH)
        .args(args.clone())
        .output();
    match exec {
        Ok(output) => {
            if output.stdout.starts_with("Rule added") {
                info!("Created a ufw rule.");
                Ok(args)
            } else if output.stdout.starts_with("Skipping") {
                warn!("Skipping the creation of a ufw rule.");
                Err(UfwInteractionError::RuleSkipped)
            } else {
                error!("UFW failed while creating a rule, resulting in an unknown error.");
                Err(UfwInteractionError::UnknownState(
                    output.stdout,
                    output.stderr,
                    args,
                ))
            }
        }
        Err(e) => {
            error!("Creating a ufw rule failed.");
            Err(UfwInteractionError::SudoFailed(e))
        }
    }
}

/// Given the index of a rule (starting from zero (0)) the function will delete the rule from the
/// ufw ruleset. This function requires the sudo password to run.
///
/// # Usage
/// Deleting the first rule from the ruleset:
///
/// ```rust
/// delete_rule(password, 0);
/// ```
///
/// [`delete_rule`]
pub fn delete_rule(password: String, index: u32) -> Result<Vec<String>, UfwInteractionError> {
    info!("Deleting a ufw rule.");
    let args: Vec<String> = vec![
        "--force".to_string(),
        "delete".to_string(),
        (index + 1).to_string(),
    ];

    let exec = SudoCommand::new(password, UFW_PATH)
        .args(args.clone())
        .output();
    match exec {
        Ok(output) => {
            if output.stdout.starts_with("Rule deleted") {
                info!("Deletd a ufw rule.");
                Ok(args)
            } else if output.stderr.contains("ERROR: Could not find rule") {
                error!("Deleting a ufw rule failed, because it could not be found.");
                Err(UfwInteractionError::NotFound)
            } else {
                error!("UFW failed while deleting a rule, resulting in an unknown error.");
                Err(UfwInteractionError::UnknownState(
                    output.stdout,
                    output.stderr,
                    args,
                ))
            }
        }
        Err(err) => Err(UfwInteractionError::SudoFailed(err)),
    }
}

/// Enables and disables ufw.
///
/// # Usage
/// ```rust
/// if let Err(sudo_error) = set_enabled(password, true) {
///     println!("Failed to disable UFW.");
///     dbg!(sudo_error);
/// }
/// ```
///
/// [`set_enabled`]
pub fn set_enabled(password: String, enabled: bool) -> Result<SudoOutput, SudoError> {
    info!("Setting ufw activation status to: {enabled}");
    SudoCommand::new(password, UFW_PATH)
        .arg("--force")
        .arg(if enabled { "enable" } else { "disable" })
        .output()
}

fn handle_defaults_command_execution(command: SudoCommand) -> Result<(), UfwInteractionError> {
    match command.output() {
        Ok(results) => {
            if results.stdout.contains("changed to") {
                Ok(())
            } else {
                Err(UfwInteractionError::UnknownState(
                    results.stdout,
                    results.stderr,
                    command.get_args(),
                ))
            }
        }
        Err(err) => Err(UfwInteractionError::SudoFailed(err)),
    }
}

/// Sets the default actions to take on incoming, outgoing or forwarded requests.
/// This function requires the sudo password.
///
/// # Usage
///
/// Set the default actions for incoming rules to deny and for outgoing to allow:
///
/// ```rust
/// let defaults = CreationDefaults {
///     input: Some(Action::Deny),
///     output: Some(Action::Allow),
///     forward: None(Action::Reject)
/// }
///
/// set_defaults(password, defaults);
/// ```
///
/// [`set_defaults`]
pub fn set_defaults(
    password: String,
    defaults: CreationDefaults,
) -> Result<(), UfwInteractionError> {
    info!("Setting ufw defaults.");
    if let Some(input) = defaults.input {
        let in_act = input.to_string();
        let in_args = vec!["default", in_act.as_str(), "incoming"];
        let mut comm = sudo::SudoCommand::new(&password, UFW_PATH);
        comm.args(in_args);
        handle_defaults_command_execution(comm)?;
        info!("Set ufw input default to: {input}");
    }
    if let Some(output) = defaults.output {
        let out_act = output.to_string();
        let out_args = vec!["default", out_act.as_str(), "outgoing"];
        let mut comm = sudo::SudoCommand::new(&password, UFW_PATH);
        comm.args(out_args);
        handle_defaults_command_execution(comm)?;
        info!("Set ufw output default to: {output}");
    }
    if let Some(forward) = defaults.forward {
        let fwd_act = forward.to_string();
        let fwd_args = vec!["default", fwd_act.as_str(), "forward"];
        let mut comm = sudo::SudoCommand::new(&password, UFW_PATH);
        comm.args(fwd_args);
        handle_defaults_command_execution(comm)?;
        info!("Set ufw forward default to: {forward}");
    }
    Ok(())
}

/// [`rules_raw`]
pub fn rules_raw() -> Result<String, std::io::Error> {
    info!("Getting raw ufw rules.");
    fs::read_to_string(UFW_RULES)
}

#[cfg(test)]
/// These tests should be run in a virtual machine, as they will change system settings and may
/// break configurations and system security.
mod tests {
    use super::*;
    use std::env;

    fn pw() -> String {
        std::env::var("TEST_PASSWORD").expect("Requires TEST_PASSWORD environment variable")
    }

    #[test]
    fn create_single_deny() {
        let rule = Rule {
            v6: false,
            destination: Point {
                subnet: Some(24),
                address: Address::Any,
            },
            source: Point {
                subnet: None,
                address: Address::Any,
            },
            destination_port: Port::Specific(33),
            source_port: Port::Any,
            protocol: Protocol::Udp,
            destination_app: "".to_string(),
            source_app: "".to_string(),
            action: Action::Deny,
            interface_in: None,
            interface_out: None,
            direction: Direction::In,
            comment: "One".to_string(),
            forward: false,
            index: None,
        };
        new_rule(pw(), rule).unwrap();
    }

    #[test]
    fn create_complex_reject() {
        let rule = Rule {
            v6: true,
            destination: Point {
                subnet: None,
                address: Address::Specific(IpAddr::from_str("fe80::1ff:fe23:4567:890a").unwrap()),
            },
            source: Point {
                subnet: None,
                address: Address::Specific(IpAddr::from_str("fe80::1ff:fe23:4567:890a").unwrap()),
            },
            destination_port: Port::Any,
            source_port: Port::Specific(89),
            protocol: Protocol::Any,
            destination_app: "".to_string(),
            source_app: "".to_string(),
            action: Action::Reject,
            interface_in: None,
            interface_out: None,
            direction: Direction::Out,
            comment: "Two".to_string(),
            forward: false,
            index: None,
        };
        new_rule(pw(), rule).unwrap();
    }

    #[test]
    fn list_rules() {
        // The actual executable would have the file in the right place, as the working directory
        // is set in the main() function
        env::set_current_dir("../");
        let s = status(pw()).unwrap();
        if s.rules.is_empty() {
            panic!("No rules");
        }
    }

    #[test]
    fn delete_first() {
        delete_rule(pw(), 0).unwrap();
    }

    #[test]
    fn switch_enabled() {
        set_enabled(pw(), false);
        set_enabled(pw(), true);
    }
}
