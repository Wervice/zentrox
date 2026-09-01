use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone)]
pub struct MessageRes {
    pub time: u128,
    pub message: String
}
