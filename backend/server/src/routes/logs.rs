use actix_web::HttpResponse;
use actix_web::web::Json;
use log::error;
use serde::{Deserialize, Serialize};
use utils::status_com::ErrorCode;
use utils::{
    logs::{self, QuickJournalEntry},
    users::NativeUser,
};
use utoipa::ToSchema;

#[derive(Serialize, ToSchema)]
struct MessagesLogRes {
    users: Vec<NativeUser>,
    logs: Vec<QuickJournalEntry>,
}

#[derive(Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct LogReq {
    sudo_password: String,
    since: u64,
    until: u64,
}

/// Journalctl log in certain time frame
#[utoipa::path(
    post,
    path = "/private/logs",
    responses((status = 200, body = MessagesLogRes)),
    request_body = LogReq,
    tags = ["private", "logs"]
)]
pub async fn read(json: Json<LogReq>) -> HttpResponse {
    let since = &json.since;
    let until = &json.until;

    match logs::log_messages(json.sudo_password.clone(), since / 1000, until / 1000) {
        Ok(messages) => {
            let mut users = vec![];
            let messages_minified: Vec<QuickJournalEntry> = messages
                .iter()
                .map(|m| {
                    let user = &m.user;

                    if let Some(valued_user) = user {
                        if !users.contains(valued_user) {
                            users.push(valued_user.clone())
                        }
                    }

                    m.clone().as_quick_journal_entry()
                })
                .collect();

            return HttpResponse::Ok().json(MessagesLogRes {
                users,
                logs: messages_minified,
            });
        }
        Err(_) => {
            error!("Getting logs failed.");
            HttpResponse::InternalServerError()
                .json(ErrorCode::LogFetchingFailed.as_error_message())
        }
    }
}
