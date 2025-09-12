// Rust short-hands for Linux commands to get information about network configuration on the system
// This library requires CAP_NET_ADMIN using `setcap` or admin permissions.
use serde::{Deserialize, Deserializer, Serialize};
use std::collections::HashMap;
use std::fmt::Display;
use std::net::IpAddr;
use std::process::Command;
use std::str::FromStr;
use utoipa::{ToResponse, ToSchema};

use crate::sudo::{SudoExecutionOutput, SudoExecutionResult, SwitchedUserCommand};

/// Determines the current private IP address of the current active network interface.
pub fn private_ip() -> Result<IpAddr, ()> {
    #[derive(Serialize, Deserialize, ToSchema, ToResponse)]
    struct Route {
        dst: String,
        gateway: String,
        dev: String,
        prefsrc: String,
        flags: Vec<String>,
        uid: u32,
        cache: Vec<String>,
    }

    let mut ip_c_program = Command::new("ip");
    let ip_c = ip_c_program.arg("-j").arg("route").arg("get").arg("1");
    let ip_c_x = ip_c.output();
    match ip_c_x {
        Ok(v) => {
            let output = v.stdout;
            let mut r = String::new();
            output.into_iter().for_each(|c| {
                r.push(c as char);
            });
            if !v.status.success() {
                return Err(());
            }
            let routes: Vec<Route> = serde_json::from_str(&r).unwrap();
            if routes.is_empty() {
                Err(())
            } else {
                Ok(routes[0].prefsrc.parse::<IpAddr>().unwrap())
            }
        }
        Err(_) => Err(()),
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TransmissionStatistics {
    pub bytes: f64,
    pub packets: i64,
    pub errors: i64,
    pub dropped: i64,
    pub over_errors: Option<i64>,
    pub multicast: Option<i64>,
    pub carrier_errors: Option<i64>,
    pub collisions: Option<i64>,
}

#[derive(Serialize, Debug, Clone, ToSchema)]
pub enum OperationalState {
    Up,
    Down,
    Dormant,
    NotPresent,
    LowerLayerDown,
    Unknown,
}

impl ToString for OperationalState {
    fn to_string(&self) -> String {
        match *self {
            Self::Up => "UP",
            Self::Down => "DOWN",
            Self::Dormant => "DORMANT",
            Self::NotPresent => "NOTPRESENT",
            Self::LowerLayerDown => "LOWERLAYERDOWN",
            Self::Unknown => "UNKNOWN",
        }
        .to_string()
    }
}

impl<'de> Deserialize<'de> for OperationalState {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(match s.as_str() {
            "UP" => Self::Up,
            "DOWN" => Self::Down,
            "DORMANT" => Self::Dormant,
            "NOTPRESENT" => Self::NotPresent,
            "LOWERLAYERDOWN" => Self::LowerLayerDown,
            "UNKNOWN" => Self::Unknown,
            _ => Self::Unknown,
        })
    }
}

/// Interface is a public struct to collect information about network interfaces.
#[derive(Debug, Deserialize)]
pub struct Interface {
    #[serde(rename(deserialize = "ifindex"))]
    pub index: i64,
    #[serde(rename(deserialize = "ifname"))]
    pub name: String,
    pub flags: Vec<String>,
    #[serde(rename(deserialize = "mtu"))]
    pub max_transmission_unit: u64,
    #[serde(rename(deserialize = "qdisc"))]
    pub queueing_discipline: String,
    #[serde(rename(deserialize = "operstate"))]
    pub operational_state: OperationalState,
    #[serde(rename(deserialize = "linkmode"))]
    pub link_mode: String,
    pub group: String,
    #[serde(rename(deserialize = "txqlen"))]
    pub transmit_queue: Option<i64>,
    pub link_type: String,
    pub address: String,
    pub broadcast: String,
    #[serde(rename(deserialize = "stats64"))]
    pub statistics: HashMap<String, TransmissionStatistics>,
    #[serde(rename(deserialize = "altnames"))]
    pub alternative_names: Option<Vec<String>>,
}

/// Get a vector of all network interfaces currently connected to the system, active or not.
/// The function does not take in any arguments.
/// It spawns the command `ip -j -s link show` to get network information.
/// Check the documentation for Interface to learn more about the return data and how to interpret
/// it.
pub fn get_network_interfaces() -> Result<Vec<Interface>, std::io::ErrorKind> {
    let mut ip_c_program = Command::new("ip");
    let ip_c = ip_c_program.arg("-j").arg("-s").arg("link").arg("show");
    let ip_c_x = ip_c.output();
    match ip_c_x {
        Ok(v) => {
            let output = v.stdout;
            let mut r = String::new();
            output.into_iter().for_each(|c| {
                r.push(c as char);
            });
            let interfaces: Vec<Interface> = serde_json::from_str(&r).unwrap();
            Ok(interfaces)
        }
        Err(_) => Err(std::io::ErrorKind::NotFound), // Return NotFound, it the execution failed
    }
}

/// Enables an interface by its name
pub fn enable_interface(sudo_password: String, interface: String) -> SudoExecutionResult {
    let mut c = SwitchedUserCommand::new(sudo_password, "ip");
    c.args(vec!["link", "set", interface.as_str(), "up"]);
    c.spawn()
}

/// Disables an interface by its name
pub fn disable_interface(sudo_password: String, interface: String) -> SudoExecutionResult {
    let mut c = SwitchedUserCommand::new(sudo_password, "ip");
    c.args(vec!["link", "set", interface.as_str(), "down"]);
    c.spawn()
}

#[derive(Debug)]
#[allow(unused)]
pub enum RouteError {
    ExecutionError,
    BadExitStatus(std::process::ExitStatus),
}

#[derive(Debug, Clone, Serialize)]
#[allow(unused)]
pub struct IpAddrWithSubnet {
    pub address: IpAddr,
    pub subnet: Option<i32>,
}

impl Display for IpAddrWithSubnet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.subnet.is_some() {
            write!(f, "{}/{}", self.address, self.subnet.unwrap())
        } else {
            write!(f, "{}", self.address)
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[allow(unused)]
pub enum Destination {
    Default,
    Prefix(IpAddrWithSubnet),
}

impl Display for Destination {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Destination::Default => write!(f, "default"),
            Destination::Prefix(v) => {
                write!(f, "{}", v.to_string())
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub enum Scope {
    Global,
    Host,
    Local,
    Site,
}

impl Display for Scope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Global => write!(f, "global"),
            Self::Host => write!(f, "host"),
            Self::Local => write!(f, "local"),
            Self::Site => write!(f, "site"),
        }
    }
}

impl From<String> for Scope {
    fn from(value: String) -> Self {
        match value.as_str() {
            "global" => Self::Global,
            "host" => Self::Host,
            "local" => Self::Local,
            "site" => Self::Site,
            _ => Self::Global,
        }
    }
}

#[derive(Debug)]
#[allow(unused)]
pub enum Protocol {
    Static,
    Kernel,
    Boot,
    Dhcp,
    Ra,
    Redirect,
    Bird,
    Babel,
    Bgp,
    Isp,
    Ospf,
    Rip,
}

#[allow(unused)]
impl Protocol {
    fn as_str(&self) -> &'static str {
        match self {
            Protocol::Static => "static",
            Protocol::Kernel => "kernel",
            Protocol::Boot => "boot",
            Protocol::Dhcp => "dhcp",
            Protocol::Ra => "ra",
            Protocol::Redirect => "redirect",
            Protocol::Bird => "bird",
            Protocol::Babel => "babel",
            Protocol::Bgp => "bgp",
            Protocol::Isp => "isp",
            Protocol::Ospf => "ospf",
            Protocol::Rip => "rip",
        }
    }
}

#[allow(unused)]
fn is_clean<T: Display>(string: T) -> bool {
    let bad_chars = [' ', '\\', '\n'];
    let a = string.to_string();
    let s = a.chars();
    for c in s {
        if bad_chars.contains(&c) {
            return false;
        }
    }
    true
}

#[derive(Debug, Clone, Serialize, ToSchema, ToResponse)]
#[allow(unused)]
pub struct Route {
    #[schema(value_type = String)]
    pub destination: Destination,
    #[schema(value_type = Option<String>)]
    pub gateway: Option<IpAddrWithSubnet>,
    #[schema(value_type = Option<String>)]
    pub nexthop: Option<Vec<IpAddrWithSubnet>>,
    pub device: Option<String>,
    pub protocol: Option<String>,
    #[schema(value_type = Option<String>)]
    pub preferred_source: Option<IpAddr>,
    pub scope: Scope,
    pub table: Option<String>,
}

#[allow(unused)]
impl Route {
    fn as_deletion_route(self) -> DeletionRoute {
        match self.gateway {
            Some(v) => DeletionRoute {
                destination: self.destination,
                gateway: Some(v),
                nexthop: None,
                device: self.device.unwrap(),
            },
            None => match self.nexthop {
                Some(v) => DeletionRoute {
                    destination: self.destination,
                    gateway: None,
                    nexthop: Some(v),
                    device: self.device.unwrap(),
                },
                None => panic!("Neither nexthop nor gateway were specified"),
            },
        }
    }
}

/// A CreationRoute is used to create a new route.
/// This struct has the following fields:
/// - `destination` - `Destination`: The destination IP prefix
/// - `gateway` - - `Option<IpAddrWithSubnet>` - The gateway for the route with IPv4 address and subnet
/// - `device` - `String` - The network interface / device
/// - `protocol` - `Protocol` - The route protocol
/// - `scope` - `Scope` - The scope of the route
/// - `table` - `String` - The table the route belongs to
#[derive(Debug)]
#[allow(unused)]
pub struct CreationRoute {
    pub destination: Destination,
    pub gateway: Option<IpAddrWithSubnet>,
    pub device: String,
    pub protocol: Protocol,
    pub scope: Scope,
    pub table: String,
}

/// Errors while convering a route into Arguments
#[derive(Debug)]
#[allow(unused)]
pub enum ArgumentsError {
    Unsanizized,
}

impl AsArguments for CreationRoute {
    fn as_arguments(self) -> Result<Vec<String>, ArgumentsError> {
        if !is_clean(&self.device) || !is_clean(&self.scope.to_string()) || !is_clean(&self.table) {
            return Err(ArgumentsError::Unsanizized);
        }

        let mut result: Vec<String> = Vec::new();
        result.push(self.destination.to_string());

        if let Some(v) = self.gateway {
            result.push("via".to_string());
            result.push(v.to_string());
        }

        result.push("dev".to_string());
        result.push(self.device);
        result.push("protocol".to_string());
        result.push(self.protocol.as_str().to_string());
        result.push("scope".to_string());
        result.push(self.scope.to_string());
        result.push("table".to_string());
        result.push(self.table);
        Ok(result)
    }
}

/// A route that is used to delete a route from the table
/// One network interface can always only have one route for one destination with a certain gateway
/// This struct has the following fields, which all are mandatory:
/// - `destination` - `Destination`: The destination of the route
/// - `gateway` - `IpAddrWithSubnet`: The gateway of the route
/// - `device` - `String`: The network interface / device of the route
#[derive(Debug)]
#[allow(unused)]
pub struct DeletionRoute {
    pub destination: Destination,
    pub gateway: Option<IpAddrWithSubnet>,
    pub nexthop: Option<Vec<IpAddrWithSubnet>>,
    pub device: String,
}

impl AsArguments for DeletionRoute {
    fn as_arguments(self) -> Result<Vec<String>, ArgumentsError> {
        if !is_clean(&self.device) {
            return Err(ArgumentsError::Unsanizized);
        }
        if self.gateway.is_some() {
            Ok(vec![
                self.destination.to_string(),
                "via".to_string(),
                self.gateway.unwrap().to_string(),
                "dev".to_string(),
                self.device,
            ])
        } else if self.nexthop.is_some() {
            let mut v = Vec::new();
            v.push(self.destination.to_string());
            for hop in self.nexthop.unwrap() {
                v.push("via".to_string());
                v.push(hop.to_string());
            }
            v.push("dev".to_string());
            return Ok(v);
        } else {
            Ok(vec![
                self.destination.to_string(),
                "dev".to_string(),
                self.device,
            ])
        }
    }
}

#[derive(Deserialize)]
#[allow(unused)]
struct RawRoute {
    #[serde(rename = "dst")]
    destination: String,
    #[serde(rename = "gateway")]
    gateway: Option<String>,
    #[serde(rename = "dev")]
    device: Option<String>,
    #[serde(rename = "protocol")]
    protocol: Option<String>,
    #[serde(rename = "prefsrc")]
    preferred_source: Option<String>,
    #[serde(rename = "scope")]
    scope: Option<String>,
    #[serde(rename = "type")]
    table: Option<String>,
    #[serde(rename = "nexthop")]
    nexthop: Option<Vec<String>>,
}

#[allow(unused)]
trait AsArguments {
    fn as_arguments(self) -> Result<Vec<String>, ArgumentsError>;
}

/// Gets all routes from all routing tables.
/// A route is defined by the following metrics, which do **not** exhaust all paramters of a UNIX route:
/// - Destination prefix
/// - Interface
/// - All gateways
/// - Protocol
/// - Flags
/// - Metric value
/// - Table name
#[allow(unused)]
pub fn get_routes() -> Result<Vec<Route>, RouteError> {
    let mut c = Command::new("ip");
    c.args(["-j", "route", "show", "table", "all"]);
    let x = c.output();
    match x {
        Ok(v) => {
            if v.status.success() {
                let ip_x_o = str::from_utf8(&v.stdout).expect("Failed to parse output");
                let raw_routes: Vec<RawRoute> = serde_json::from_str(ip_x_o).unwrap();
                let structured_routes = raw_routes
                    .iter()
                    .map(|e| {
                        let dest_split: Vec<&str> = e.destination.split("/").collect();
                        let destination;
                        if dest_split.len() == 2 {
                            if dest_split[0] == "default" {
                                destination = Destination::Default;
                            } else {
                                destination = Destination::Prefix(IpAddrWithSubnet {
                                    address: IpAddr::from_str(dest_split[0])
                                        .expect("Failed to parse IP address"),
                                    subnet: Some(i32::from_str(dest_split[1]).unwrap_or(0_i32)),
                                })
                            }
                        } else if dest_split[0] == "default" {
                            destination = Destination::Default;
                        } else {
                            destination = Destination::Prefix(IpAddrWithSubnet {
                                address: IpAddr::from_str(dest_split[0])
                                    .expect("Failed to parse IP address"),
                                subnet: None,
                            })
                        }

                        let gateway;
                        if e.gateway.is_some() {
                            let gateway_unwrap = e.gateway.clone().unwrap();
                            let gateway_split: Vec<&str> = gateway_unwrap.split("/").collect();
                            if gateway_split.len() == 2 {
                                gateway = Some(IpAddrWithSubnet {
                                    address: IpAddr::from_str(gateway_split[0])
                                        .expect("Failed to parse IP address"),
                                    subnet: Some(i32::from_str(gateway_split[1]).unwrap_or(0_i32)),
                                })
                            } else {
                                gateway = Some(IpAddrWithSubnet {
                                    address: IpAddr::from_str(gateway_split[0])
                                        .expect("Failed to parse IP address"),
                                    subnet: None,
                                })
                            }
                        } else {
                            gateway = None
                        }
                        let nexthop_res: Option<Vec<IpAddrWithSubnet>>;
                        let mut nexthop: Vec<IpAddrWithSubnet> = Vec::new();
                        if e.nexthop.is_some() {
                            let hops_unwrap = e.nexthop.clone().unwrap();
                            for hop in hops_unwrap {
                                let hop_split: Vec<&str> = hop.split("/").collect();
                                if hop_split.len() == 2 {
                                    let n = IpAddrWithSubnet {
                                        address: IpAddr::from_str(hop_split[0])
                                            .expect("Failed to parse IP address"),
                                        subnet: Some(i32::from_str(hop_split[1]).unwrap_or(0_i32)),
                                    };
                                    nexthop.push(n)
                                } else {
                                    let n = IpAddrWithSubnet {
                                        address: IpAddr::from_str(hop_split[0])
                                            .expect("Failed to parse IP address"),
                                        subnet: None,
                                    };
                                    nexthop.push(n);
                                }
                            }
                            nexthop_res = Some(nexthop);
                        } else {
                            nexthop_res = None;
                        }

                        Route {
                            destination,
                            gateway,
                            nexthop: nexthop_res,
                            device: e.device.clone(),
                            protocol: e.protocol.clone(),
                            preferred_source: e
                                .preferred_source
                                .as_ref()
                                .map(|v| IpAddr::from_str(v).expect("Failed to parse IP address")),
                            scope: Scope::from(e.scope.clone().unwrap_or("global".to_string())),
                            table: e.table.clone(),
                        }
                    })
                    .collect();
                Ok(structured_routes)
            } else {
                Err(RouteError::BadExitStatus(v.status))
            }
        }
        Err(_) => Err(RouteError::ExecutionError),
    }
}

/// Creates a new route using a CreationRoute
#[allow(unused)]
pub fn create_route(route: CreationRoute, sudo_password: String) -> SudoExecutionOutput {
    let mut c = SwitchedUserCommand::new(sudo_password, "ip");
    c.args(vec!["route", "add"]);
    let args = route.as_arguments();
    c.args(args.expect("Failed to translate route to arguments"));
    c.output()
}

/// Deletes a route using a `DeletionRoute`
#[allow(unused)]
pub fn delete_route(route: DeletionRoute, sudo_password: String) -> SudoExecutionOutput {
    let mut c = SwitchedUserCommand::new(sudo_password, "ip");
    c.args(vec!["route", "delete"]);
    let args = route.as_arguments();
    c.args(args.expect("Failed to translate route to arguments"));
    c.output()
}
