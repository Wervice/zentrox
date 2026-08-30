use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, ToSchema)]
#[allow(unused)]
pub struct LoginReq {
    pub username: String,
    pub password: String,
    pub otp: Option<String>,
}
