use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Deserialize, Serialize, Clone, Debug, ToSchema)]
pub struct JobRes {
    #[schema(value_type = String)]
    pub uuid: Uuid
}
