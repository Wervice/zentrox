use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, ToSchema, Clone, Default)]
pub struct AccountDetailsRes {
    pub username: String,
}
