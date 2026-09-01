use std::fmt::Display;

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, ToSchema, Clone)]
pub enum ContainerState {
    EMPTY,
    CREATED,
    RUNNING,
    PAUSED,
    RESTARTING,
    REMOVING,
    EXITED,
    DEAD,
}

impl Display for ContainerState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ContainerState::EMPTY => write!(f, "empty"),
            ContainerState::CREATED => write!(f, "created"),
            ContainerState::RUNNING => write!(f, "running"),
            ContainerState::PAUSED => write!(f, "paused"),
            ContainerState::RESTARTING => write!(f, "restarting"),
            ContainerState::REMOVING => write!(f, "removing"),
            ContainerState::EXITED => write!(f, "exited"),
            ContainerState::DEAD => write!(f, "dead"),
        }
    }
}

#[derive(Serialize, Deserialize, ToSchema, Clone)]
pub struct Container {
    pub id: String,
    pub names: Option<Vec<String>>,
    pub image: Option<String>,
    pub created: Option<i64>,
    pub state: Option<ContainerState>
}

#[derive(Serialize, Deserialize, ToSchema, Clone)]
pub struct ContainersRes {
    pub containers: Vec<Container>
}
