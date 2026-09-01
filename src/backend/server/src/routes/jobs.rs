use actix_web::{
    HttpResponse,
    web::{Data, Path},
};
use utils::status_com::ErrorCode;
use uuid::Uuid;

use crate::AppState;

/// Get the status of a job.
///
/// Jobs are used for tasks that would block the server and take a lot of time to finish, making it
/// unreasonable to keep the connection alive for that long. Some browser may even time out.
#[utoipa::path(
    get,
    path = "/private/jobs/status/{id}",
    responses((status = 200, description = "The operation finished and may have provided results."),
    (status = 500, description = "The task failed and may have provided error details."),
    (status = 202, description = "The task is still pending."),
    (status = 201, description = "The task is still pending and can provide a status update."),
    (status = 404, description = "A job with this ID could not be found.")),
    tags = ["private", "jobs"],
    params(("id" = String, Path))
)]
pub async fn status(state: Data<AppState>, path: Path<Uuid>) -> HttpResponse {
    let mut jobs = state.background_jobs.lock().unwrap();
    let uuid = path.into_inner();
    let res = if let Some(j) = jobs.get(uuid) {
        j.into()
    } else {
        HttpResponse::NotFound().json(ErrorCode::NoSuchTask.as_error_message())
    };
    let _ = jobs.remove(uuid);
    res
}
