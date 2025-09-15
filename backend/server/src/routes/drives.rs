use actix_web::{HttpResponse, web::Query};
use serde::{Deserialize, Serialize};
use utils::drives::{self, DriveUsageStatistics};
use utils::status_com::ErrorCode;
use utoipa::ToSchema;

#[derive(Serialize, ToSchema)]
struct DriveListRes {
    drives: Vec<drives::BlockDevice>,
}

#[utoipa::path(
    get,
    path = "/private/drives/list",
    responses((status = 200, body = DriveListRes)),
    tags = ["private", "drives"]
)]
/// List of connected block devices
pub async fn list() -> HttpResponse {
    let drives_out = drives::device_list();

    let drives = match drives_out {
        Some(drives_list) => drives_list.blockdevices,
        None => {
            return HttpResponse::InternalServerError()
                .json(ErrorCode::BlockDeviceListingFailed.as_error_message());
        }
    };

    HttpResponse::Ok().json(DriveListRes { drives })
}

#[derive(Serialize, ToSchema)]
struct DriveInformationRes {
    metadata: drives::Drive,
    statistics: DriveUsageStatistics,
}

#[derive(Deserialize)]
pub struct DriveStatisticsQuery {
    drive: String,
}

/// Block device statistics
#[utoipa::path(
    get,
    path = "/private/drives/statistics",
    responses((status = 200, body = DriveInformationRes)),
    params(("drive" = String, Query)),
    tags = ["private", "drives"]
)]
pub async fn statistics(info: Query<DriveStatisticsQuery>) -> HttpResponse {
    let drive = &info.drive;

    let metadata = match drives::drive_information(drive.clone()) {
        Some(m) => m,
        None => {
            return HttpResponse::InternalServerError()
                .json(ErrorCode::DriveMetadataFailed.as_error_message());
        }
    };

    let statistics = match drives::drive_statistics(drive.clone()) {
        Some(s) => s,
        None => {
            return HttpResponse::InternalServerError()
                .json(ErrorCode::DriveStatisticsFailed.as_error_message());
        }
    };

    HttpResponse::Ok().json(DriveInformationRes {
        metadata,
        statistics,
    })
}
