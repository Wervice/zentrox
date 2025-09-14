// 1. Add new rule
// 2. Remove rule
// 3. Enable/Disable UFW
// 4. List rules
use crate::sudo::{SudoExecutionOutput, SudoExecutionResult, SwitchedUserCommand};
use regex::Regex;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

// TODO More details, more pre-processing
#[derive(Serialize, Debug, ToSchema)]
pub struct UfwRule {
    pub index: u32,
    pub to: String,
    pub from: String,
    pub action: FirewallAction,
}

/// Fetches the current UFW status
///
/// The status is fetched using the command `/usr/sbin/ufw status` which is ran using `sudo`.
/// This invokes the ufw executable and requests the status and a list of rules in form of a table.
/// The first line of the ouput contains the status. i.e.: "Status: active" or "Status: inactive".
/// Bellow that is a table of rules.
///
/// These values are parsed into a tuple containg a bool that shows if the ufw is enable or not.
/// True: Enabled; False: Disabled
/// And a vector containing UfwRules with index, to, from and action.
///
/// * `password` - The password used to authenticate `sudo`
pub fn ufw_status(password: String) -> Result<(bool, Vec<UfwRule>), String> {
    let output =
        match SwitchedUserCommand::new(password, "/usr/sbin/ufw status".to_string()).output() {
            SudoExecutionOutput::Success(v) => v.stdout,
            _ => return Err("Wrong sudo password".to_string()),
        };
    let mut output_lines = output
        .lines()
        .map(|x| x.to_string())
        .filter(|x| !x.is_empty())
        .collect::<Vec<String>>();

    if output_lines.is_empty() {
        return Err("Invalid Sudo Password or failed to read UFW status".to_string());
    }

    let enabled = output_lines[0] == "Status: active";

    let mut rules_vec: Vec<UfwRule> = Vec::new();
    if enabled && output_lines.len() > 3 {
        output_lines.drain(0..3);

        let mut index: i32 = 0;

        let re = Regex::new(r"\s{2,}").unwrap();

        for line in output_lines {
            let l_s: Vec<&str> = re.split(&line).collect();
            let to = l_s[0];
            let action = l_s[1];
            let from = l_s[2];
            rules_vec.push(UfwRule {
                to: String::from(to),
                from: String::from(from),
                action: FirewallAction::from(action),
                index: index as u32,
            });

            index += 1;
        }
    }

    Ok((enabled, rules_vec))
}

/// To component converts values for a new firewall rule into strings used to create a command.
pub trait ToComponent {
    fn to_component(&self) -> String;
}

pub enum SinglePortProtocol {
    Tcp(u64),
    Udp(u64),
}

impl ToComponent for SinglePortProtocol {
    fn to_component(&self) -> String {
        match *self {
            SinglePortProtocol::Tcp(v) => format!("{v} proto tcp"),
            SinglePortProtocol::Udp(v) => format!("{v} proto udp"),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
// TODO Check if this works with ALLOW OUT etc.
pub enum RuleDirection {
    Out,
    In,
}

impl ToComponent for RuleDirection {
    fn to_component(&self) -> String {
        match *self {
            Self::In => "in".to_string(),
            Self::Out => "out".to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
// TODO Check if this works with ALLOW OUT etc.
pub enum FirewallAction {
    Deny(RuleDirection),
    Allow(RuleDirection),
}

impl From<&str> for FirewallAction {
    fn from(value: &str) -> Self {
        match value {
            "ALLOW" | "ALLOW IN" => FirewallAction::Allow(RuleDirection::In),
            "DENY" | "DENY IN" => FirewallAction::Deny(RuleDirection::In),
            "ALLOW OUT" => FirewallAction::Allow(RuleDirection::Out),
            "DENY OUT" => FirewallAction::Deny(RuleDirection::Out),
            _ => unreachable!("An unknown firewall action was encountered."),
        }
    }
}

impl ToComponent for FirewallAction {
    fn to_component(&self) -> String {
        match *self {
            FirewallAction::Deny(dir) => format!("deny {}", dir.to_component()),
            FirewallAction::Allow(dir) => format!("allow {}", dir.to_component()),
        }
    }
}

pub enum FirewallSender {
    Any,
    Specific(String),
}

impl ToComponent for FirewallSender {
    fn to_component(&self) -> String {
        match self {
            FirewallSender::Any => "any".to_string(),
            FirewallSender::Specific(ip_addr) => ip_addr
                .replace("\"", "\\\"")
                .replace(" ", "")
                .replace("\n", "\\n")
                .to_string(),
        }
    }
}

pub enum PortRangeProtocol {
    Tcp(u64, u64),
    Udp(u64, u64),
}

impl PortRangeProtocol {
    fn to_port_component(&self) -> String {
        match *self {
            PortRangeProtocol::Tcp(l, r) => format!("{l}:{r}").to_string(),
            PortRangeProtocol::Udp(l, r) => format!("{l}:{r}").to_string(),
        }
    }
    fn to_protocol_component(&self) -> String {
        match *self {
            PortRangeProtocol::Tcp(_, _) => "proto tcp".to_string(),
            PortRangeProtocol::Udp(_, _) => "proto udp".to_string(),
        }
    }
}

/// Create new UFW rule by spawning a command containing the from, to and action value.
///
/// The command is spawned with `sudo` to allow for adding rules.
/// * `password`: String - The password used to authenticate sudo
/// * `destination_port`: NetworkProtocol - The destionation port with protocol
/// * `sender`: FirewallSender - The sender where the rule has to apply
/// * `action`: FirewallAction - The action to be taken
pub fn new_rule_port<T: ToString>(
    password: T,
    destination_port: SinglePortProtocol,
    sender: FirewallSender,
    action: FirewallAction,
) -> Result<(), String> {
    let command = format!(
        "ufw {} from {} to any port {}",
        action.to_component(),
        sender.to_component(),
        destination_port.to_component()
    );
    // sudo ufw allow from any to any port 22 proto tcp
    match SwitchedUserCommand::new(password.to_string(), command).spawn() {
        SudoExecutionResult::Success(_) => Ok(()),
        _ => Err("Failed to spawn command".to_string()),
    }
}

/// Create new UFW rule by spawning a command containing the from, to and action value.
///
/// The command is spawned with `sudo` to allow for adding rules.
/// * `password`: String - The password used to authenticate sudo
/// * `destination_port`: NetworkProtocol - The destionation port with protocol
/// * `sender`: FirewallSender - The sender where the rule has to apply
/// * `action`: FirewallAction - The action to be taken
pub fn new_rule_range<T: ToString>(
    password: T,
    destination_range: PortRangeProtocol,
    sender: FirewallSender,
    action: FirewallAction,
) -> Result<(), String> {
    let command = format!(
        "ufw {} {} from {} to any port {}",
        action.to_component(),
        destination_range.to_protocol_component(),
        sender.to_component(),
        destination_range.to_port_component()
    );
    // sudo ufw allow from any to any port 22 proto tcp
    match SwitchedUserCommand::new(password.to_string(), command).spawn() {
        SudoExecutionResult::Success(_) => Ok(()),
        _ => Err("Failed to spawn command".to_string()),
    }
}

/// Deletes UFW rule by spawning a command containing the rules index
///
/// The command is spawned with `sudo` to allow for deletign rules.
/// `password` - The password to authenticate sudo.
/// `index` - The index of the rule to delete.
pub fn delete_rule(password: String, index: u32) -> Result<(), String> {
    // Index is added 1 as ufw starts numbering at [1], but common convention is 0
    let command = format!("/usr/sbin/ufw --force delete {}", index + 1);
    // FIX Direct interpolation of unsanitized user due to index
    // input
    match SwitchedUserCommand::new(password, command).spawn() {
        SudoExecutionResult::Success(_) => Ok(()),
        _ => Err("Failed to spawn command".to_string()),
    }
}

pub fn disable(password: String) -> SudoExecutionResult {
    SwitchedUserCommand::new(
        password.to_string(),
        "/usr/sbin/ufw --force disable".to_string(),
    )
    .spawn()
}

pub fn enable(password: String) -> SudoExecutionResult {
    SwitchedUserCommand::new(
        password.to_string(),
        "/usr/sbin/ufw --force enable".to_string(),
    )
    .spawn()
}
