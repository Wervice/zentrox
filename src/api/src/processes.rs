use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::units::{Bytes, Hertz, Celcius};

#[derive(Deserialize, Serialize, ToSchema, Debug)]
pub struct Cpu {
    pub name: String,
    pub brand: String,
    pub usage: f32,
    pub frequency: Hertz
}

#[derive(Deserialize, Serialize, ToSchema, Debug)]
pub struct Load {
    pub averages: (f32, f32, f32),
    pub latest_process: u32,
}

#[derive(Deserialize, Serialize, ToSchema, Debug)]
pub struct CpuStatsRes {
    pub cpus: Vec<Cpu>,
    pub load_averages: Option<Load>
}

#[derive(Deserialize, Serialize, ToSchema, Debug)]
pub struct MemoryStatsRes {
    pub total: Bytes,
    pub free: Bytes,
    pub swap_total: Bytes,
    pub swap_free: Bytes
}

#[derive(Serialize, Deserialize, ToSchema, Debug, Clone)]
pub struct Thermometer {
    pub label: String,
    pub critical: Option<Celcius>,
    pub reading: Option<Celcius>,
}

#[derive(Serialize, Deserialize, ToSchema, Debug)]
pub struct ThermometersRes {
    pub thermometers: Vec<Thermometer>
}

#[derive(Serialize, Deserialize, ToSchema, Debug)]
pub struct UptimeRes {
    pub uptime: u64
}
