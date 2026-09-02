use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Deserialize, Serialize, Clone, Debug, ToSchema, PartialEq, Eq)]
pub struct NeedsPasswordRes {
    pub needs_password: bool
}
