use std::{path::PathBuf, time::Duration};

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::units::Bytes;

#[derive(Deserialize, Serialize, Clone, Debug, ToSchema)]
pub struct DrivesRes {
    pub drives: Vec<Drive>
}

#[derive(Deserialize, Serialize, Clone, Debug, ToSchema, PartialEq, Eq)]
pub struct FileSystem {
    pub size: Option<Bytes>,
    pub available: Option<Bytes>,
    pub used: Option<Bytes>,
    pub read_only: bool,
    pub kind: Option<String>,
    pub version: Option<String>,
    #[schema(value_type = Vec<String>)]
    pub mountpoints: Vec<PathBuf>,
    #[schema(value_type = String)]
    pub path: PathBuf,
    pub children: Vec<FileSystem>,
    pub name: String,
    pub can_check: bool,
    pub can_repair: bool,
    pub uuid: String,
}

#[derive(Deserialize, Serialize, Clone, Debug, ToSchema, PartialEq, Eq)]
pub struct Partition {
    pub size: Bytes,
    pub label: Option<String>,
    pub name: String,
    #[schema(value_type = String)]
    pub path: PathBuf,
    pub filesystem: Option<FileSystem>,
    pub uuid: String
}

#[derive(Deserialize, Serialize, Clone, Debug, ToSchema, PartialEq, Eq)]
pub struct Drive {
    pub rotating: bool,
    pub label: Option<String>,
    pub udisks_id: String,
    pub name: String,
    pub model: Option<String>,
    pub vendor: Option<String>,
    pub revision: Option<String>,
    #[schema(value_type = String)]
    pub path: PathBuf,
    pub size: Bytes,
    pub serial: Option<String>,
    pub partitions: Vec<Partition>,
    pub read_only: bool,
    pub time_detected: i64,
    pub filesystem: Option<FileSystem>,
    pub bus_type: String,
    pub hint_ignore: bool,
    pub hint_system: bool,
    pub media_available: bool,
    pub ejectable: bool,
    pub removable: bool,
    pub can_power_off: bool,
}

#[derive(Deserialize, Serialize, Clone, Debug, ToSchema, PartialEq, Eq)]
pub struct DriveSignature {
    pub time_detected: i64,
    #[schema(value_type = String)]
    pub device_node: PathBuf,
    pub serial: Option<String>,
    pub udisks_id: String
}

#[derive(Deserialize, Serialize, Clone, Debug, ToSchema, PartialEq, Eq)]
pub struct FileSystemActionReq {
    pub drive: DriveSignature,
    #[schema(value_type = String)]
    pub fs: PathBuf,
    pub password: Option<String>
}

#[derive(Deserialize, Serialize, Clone, Debug, ToSchema, PartialEq, Eq)]
pub struct DriveActionReq {
    pub drive: DriveSignature,
    #[schema(value_type = String)]
    pub password: Option<String>
}

#[derive(Deserialize, Serialize, Clone, Debug, ToSchema, PartialEq, Eq)]
pub struct MountRes {
    #[schema(value_type = String)]
    pub mountpoint: PathBuf
}

#[derive(Deserialize, Serialize, Clone, Debug, ToSchema, PartialEq, Eq)]
pub struct CheckRes {
    #[schema(value_type = String)]
    pub consistent: bool
}

#[derive(Deserialize, Serialize, Clone, Debug, ToSchema, PartialEq, Eq)]
pub struct RepairRes {
    #[schema(value_type = String)]
    pub success: bool
}

pub trait WithFilesystem {
    fn label(&self) -> Option<String>;
    fn name(&self) -> String;
    fn fs(&self) -> Option<FileSystem>;
    fn size(&self) -> Bytes;
}

impl WithFilesystem for Drive {
    fn label(&self) -> Option<String> {
        self.label.clone()
    }

    fn name(&self) -> String {
        self.name.clone()
    }

    fn fs(&self) -> Option<FileSystem> {
        self.filesystem.clone()
    }

    fn size(&self) -> Bytes {
        self.size
    }
}

impl WithFilesystem for Partition {
    fn label(&self) -> Option<String> {
        self.label.clone()
    }

    fn name(&self) -> String {
        self.name.clone()
    }

    fn fs(&self) -> Option<FileSystem> {
        self.filesystem.clone()
    }

    fn size(&self) -> Bytes {
        self.size
    }
}

pub trait WithPrettyName {
    fn pretty_name(&self) -> String;
    fn id(&self) -> String;
}

impl WithPrettyName for Drive {
    fn pretty_name(&self) -> String {
        self.label.clone().unwrap_or(self.path.to_string_lossy().to_string())
    }

    fn id(&self) -> String {
        format!("{}-{}-{}", self.pretty_name(), self.path.to_string_lossy(), self.time_detected)
    }
}

impl WithPrettyName for FileSystem {
    fn pretty_name(&self) -> String {
        self.path.to_string_lossy().to_string()
    }

    fn id(&self) -> String {
        format!("{}-{}", self.path.to_string_lossy(), self.uuid)
    }
}

impl WithPrettyName for Partition {
    fn pretty_name(&self) -> String {
        self.label.clone().unwrap_or(self.path.to_string_lossy().to_string())
    }

    fn id(&self) -> String {
        format!("{}-{}-{}", self.pretty_name(), self.path.to_string_lossy(), self.uuid)
    }
}

#[derive(Deserialize, Serialize, Clone, Debug, ToSchema)]
pub struct BenchmarkUpdate {
    pub progress: f64
}

#[derive(Deserialize, Serialize, Clone, Debug, ToSchema)]
pub struct BenchmarkReq {
    pub write: bool,
    pub random: bool,
    pub sample_size: usize,
    pub iterations: usize,
    pub drive: DriveSignature,
    pub password: String
}

#[derive(Deserialize, Serialize, Clone, Debug, ToSchema, PartialEq, Eq)]
pub struct BenchmarkRes {
    #[schema(value_type = f64)]
    pub write: Option<Vec<Duration>>,
    #[schema(value_type = f64)]
    pub read: Vec<Duration>,
    pub drive: DriveSignature,
    pub sample_size: usize,
    pub random: bool
}

#[derive(Deserialize, Serialize, Clone, Debug, ToSchema)]
pub struct HistoricalBenchmarkRes {
    #[schema(value_type = f64)]
    pub write: Option<Vec<Duration>>,
    #[schema(value_type = f64)]
    pub read: Vec<Duration>,
    pub sample_size: usize,
    pub random: bool
}

#[derive(Deserialize, Serialize, Clone, Debug, ToSchema)]
pub struct BenchmarkRetrievalReq {
    pub id: String,
}

#[derive(Deserialize, Serialize, Clone, Debug, ToSchema)]
pub struct BenchmarkHistory {
    pub history: Vec<BenchmarkHistoryEntry>
}

#[derive(Deserialize, Serialize, Clone, Debug, ToSchema, Eq, PartialEq)]
pub struct BenchmarkHistoryEntry {
    pub time: i64,
    pub sample_size: usize,
    pub uuid: String
}
