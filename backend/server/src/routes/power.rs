use crate::SudoPasswordReq;
use actix_web::HttpResponse;
use actix_web::web::Json;
use utils::{
    status_com::{ErrorCode, MessageRes},
    sudo,
};

#[utoipa::path(
    post,
    path = "/private/power/off",
    request_body = SudoPasswordReq,
    responses((status = 200)),
    tags = ["private", "power"]
)]
pub async fn off(json: Json<SudoPasswordReq>) -> HttpResponse {
    let e =
        sudo::SwitchedUserCommand::new(json.sudo_password.clone(), "poweroff".to_string()).spawn();

    if let sudo::SudoExecutionResult::Success(_) = e {
        HttpResponse::Ok().json(MessageRes::from("The computer is shutting down."))
    } else {
        HttpResponse::InternalServerError().json(ErrorCode::PowerOffFailed.as_error_message())
    }
}
