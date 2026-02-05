use actix_web::{
    HttpResponse,
    web::{Data, Path},
};
use utils::status_com::{ErrorCode, MessageRes};

use crate::{AppState, BackgroundTaskState};

/// Get the status of a job.
///
/// Jobs are used for tasks that would block the server and take a lot of time to finish, making it
/// unreasonable to keep the connection alive for that long. Some browser may even time out.
#[utoipa::path(
    get,
    path = "/private/jobs/status/{id}",
    responses((status = 200, description = "The operation finished and may have provided results."),
    (status = 422, description = "The task failed and may have provided error details."),
    (status = 202, description = "The task is still pending."),
    (status = 404, description = "A job with this ID could not be found.")),
    tags = ["private", "jobs"],
    params(("id" = String, Path))
)]
pub async fn status(state: Data<AppState>, path: Path<String>) -> HttpResponse {
    let requested_id = path.into_inner().to_string();
    let jobs = state.background_jobs.lock().unwrap().clone();
    let background_state = jobs.get(&uuid::Uuid::parse_str(&requested_id).unwrap());

    match background_state {
        Some(bs) => match bs {
            BackgroundTaskState::Success => {
                HttpResponse::Ok().json(MessageRes::from("Operation finished successfully."))
            }
            BackgroundTaskState::Fail => {
                HttpResponse::UnprocessableEntity().json(ErrorCode::TaskFailed.as_error_message())
            }
            BackgroundTaskState::SuccessOutput(s) => {
                HttpResponse::Ok().json(MessageRes::from(s.clone()))
            }
            BackgroundTaskState::FailOutput(f) => HttpResponse::UnprocessableEntity()
                .json(ErrorCode::TaskFailedWithDescription(f.to_string()).as_error_message()),
            BackgroundTaskState::Pending => {
                HttpResponse::Accepted().json(MessageRes::from("The task is still in work."))
            }
        },
        None => HttpResponse::NotFound().json(ErrorCode::NoSuchTask.as_error_message()),
    }
}
