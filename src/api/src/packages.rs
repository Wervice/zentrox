use std::fmt::Display;

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, ToSchema)]
pub enum PackageManager {
    Apt,
    Dnf,
    Pacman
}

impl Display for PackageManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Apt => write!(f, "apt"),
            Self::Dnf => write!(f, "dnf"),
            Self::Pacman => write!(f, "pacman")
        }
    }
}

#[derive(Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct DatabaseRes {
    pub installed: Vec<String>,
    pub available: Vec<String>,
    pub package_manager: Option<PackageManager>,
    pub updates: Option<Vec<String>>,
    pub last_database_update: Option<i64>, // The last database update expressed as seconds since the
                                       // UNIX epoch
}

#[derive(Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct StatisticsRes {
    pub installed: usize,
    pub available: usize,
    pub package_manager: Option<PackageManager>,
    pub updates: Option<usize>,
    pub last_database_update: Option<i64>, // The last database update expressed as seconds since the
                                       // UNIX epoch
}
