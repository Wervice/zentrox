use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, ToSchema, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DeviceInformationRes {
    pub os_name: Option<String>,
    pub kernel_version: Option<String>,
    pub utc_offset: i32,
    pub relevant_services: Vec<(String, bool)>,
    pub server_name: String
}
