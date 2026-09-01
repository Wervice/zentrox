use std::net::IpAddr;

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::units::Bytes;

#[derive(Serialize, ToSchema)]
pub struct IpRes {
    #[schema(value_type = Option<String>)]
    pub ip: Option<IpAddr>
}

#[derive(Serialize, Deserialize, ToSchema, Debug, Clone)]
pub struct HostnameRes {
    pub hostname: String
}

#[derive(Deserialize, Serialize, ToSchema, Clone, PartialEq, PartialOrd)]
pub enum OperationalState {
    Up,
    Down,
    Dormant,
    NotPresent,
    LowerLayerDown,
    Unknown,
}

/// Interface is a public struct to collect information about network interfaces.
#[derive(Deserialize, Serialize, ToSchema, Clone, PartialEq, PartialOrd)]
pub struct Interface {
    pub index: u64,
    pub name: String,
    pub flags: Vec<String>,
    pub max_transmission_unit: u64,
    pub queueing_discipline: String,
    pub operational_state: OperationalState,
    pub group: String,
    pub transmit_queue: Option<i64>,
    pub link_type: String,
    pub address: Option<String>,
    pub broadcast: Option<String>,
    pub delta_up_per_five_s: Bytes,
    pub delta_down_per_five_s: Bytes,
    #[schema(value_type = Vec<String>)]
    pub ips: Vec<IpAddr>
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct InterfacesRes {
    pub interfaces: Vec<Interface>,
}
