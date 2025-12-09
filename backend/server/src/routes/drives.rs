use actix_web::{HttpResponse};
use log::error;
use serde::{Serialize};
use utils::drives::{self};
use utils::status_com::ErrorCode;
use utoipa::ToSchema;

#[derive(Serialize, ToSchema)]
struct DriveListRes {
    drives: Vec<drives::Drive>,
}

#[utoipa::path(
    get,
    path = "/private/drives/list",
    responses((status = 200, body = DriveListRes)),
    tags = ["private", "drives"]
)]
/// List of connected block devices
pub async fn list() -> HttpResponse {
    let drives_out = drives::list();

    let drives = match drives_out {
        Ok(drives_list) => drives_list,
        Err(e) => {
            error!("Failed to get drives: {:?}", e);
            return HttpResponse::InternalServerError()
                .json(ErrorCode::BlockDeviceListingFailed.as_error_message());
        }
    };

    HttpResponse::Ok().json(DriveListRes { drives })
}
