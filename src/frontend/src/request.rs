//! Bindings for error-sensitive HTTP requests.

use dioxus::prelude::*;
use crate::{HTTP_PREFIX, LoadingCtx, LoggedInCtx, LoginTriggerCtx};
use api::account::AccountDetailsRes;
use dioxus::{core::consume_context, signals::Signal};
use gloo_net::http::{Request, Response};
use std::fmt::Display;
use uuid::Uuid;

fn revoke_logged_in_state() {
    let mut logged_in_signal = consume_context::<LoggedInCtx>().0;
    let mut loading_signal = consume_context::<LoadingCtx>().0;
    let mut loggin_trigger_signal = consume_context::<LoginTriggerCtx>().0;
    let mut user_signal = consume_context::<Signal<Option<AccountDetailsRes>>>();

    logged_in_signal.set(None);
    user_signal.set(None);
    loading_signal.set(true);
    loggin_trigger_signal.toggle();
}

async fn format_response<J: serde::de::DeserializeOwned>(res: Response) -> Result<J, String> {
    match res.status() {
        403 => {
            revoke_logged_in_state();
            return Err(
                "The request did not have sufficient permissions to access the requested resource."
                    .into(),
            );
        }
        429 => {
            return Err(
                "Too many requests were sent to the server. This client has been rate-limited."
                    .into(),
            );
        }
        x if x != 200 => {
            let error_json_attempt = res.json::<api::error::MessageRes>();
            match error_json_attempt.await {
                Ok(err_val) => return Err(err_val.message),
                Err(des_err) => {
                    return Err(format!(
                        "The request did not provide a correct backend error message: {des_err}"
                    ));
                }
            }
        }
        _ => {}
    };

    let json_attempt = res.json::<J>();
    match json_attempt.await {
        Ok(json) => Ok(json),
        Err(e) => Err(format!(
            "Failed to deserialize JSON response due to error: {e}"
        )),
    }
}

pub async fn get<J: serde::de::DeserializeOwned>(path: impl Display) -> Result<J, String> {
    let req = Request::get(format!("{HTTP_PREFIX}{path}").as_str())
        .credentials(web_sys::RequestCredentials::Include)
        .send()
        .await;

    match req {
        Ok(res) => format_response::<J>(res).await,
        Err(e) => Err(format!("Failed to send HTTP request due to error: {e}")),
    }
}

pub async fn post<J: serde::de::DeserializeOwned>(
    path: impl Display,
    json: impl serde::Serialize,
) -> Result<J, String> {
    let req = Request::post(format!("{HTTP_PREFIX}{path}").as_str())
        .credentials(web_sys::RequestCredentials::Include)
        .header("Content-Type", "application/json")
        .body(serde_json::to_string(&json).expect("Failed to serialize."))
        .expect("The serialized JSON is invalid.")
        .send()
        .await;

    match req {
        Ok(res) => format_response::<J>(res).await,
        Err(e) => Err(format!("Failed to send HTTP request due to error: {e}")),
    }
}

pub enum Job<O, U> {
    Ongoing,
    Ok(O),
    Update(U),
    Failed(String),
}

pub async fn get_job_status<O: serde::de::DeserializeOwned, U: serde::de::DeserializeOwned>(
    job: Uuid,
) -> Result<Job<O, U>, String> {
    let req = Request::get(format!("{HTTP_PREFIX}/private/jobs/status/{job}").as_str())
        .credentials(web_sys::RequestCredentials::Include)
        .send()
        .await;

    match req {
        Ok(res) => match res.status() {
            200 => match res.json::<O>().await {
                Ok(j) => Ok(Job::Ok(j)),
                Err(e) => Err(format!(
                    "Failed to deserialize JSON response due to error: {e}"
                )),
            },
            201 => match res.json::<U>().await {
                Ok(j) => Ok(Job::Update(j)),
                Err(e) => Err(format!(
                    "Failed to deserialize JSON response due to error: {e}"
                )),
            },
            202 => Ok(Job::Ongoing),
            500 => match res.json::<api::error::MessageRes>().await {
                Ok(j) => Ok(Job::Failed(j.message)),
                Err(e) => Err(format!(
                    "The request did not provide a correct backend error message: {e}"
                )),
            },
            404 => Err("This job does not exist".to_string()),
            s => Err(format!(
                "The backend responded with an unexpected status {s}"
            )),
        },
        Err(e) => Err(format!("Failed to send HTTP request due to error: {e}")),
    }
}
