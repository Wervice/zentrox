use crate::{AppState, BackgroundTaskState, routes::cron};
use actix_web::web::Json;
use actix_web::{
    HttpResponse,
    web::{Data, Path},
};
use log::{error, warn};
use serde::{Deserialize, Serialize};
use std::process::{Command, Stdio};
use utils::cron::list_cronjobs;
use utils::status_com::ErrorCode;
use utils::{
    cron::{
        CronError, CronJob, DayOfWeek, Digit, Interval, IntervalCronJob, Month, SpecificCronJob,
        create_new_interval_cronjob, create_new_specific_cronjob, delete_interval_cronjob,
        delete_specific_cronjob,
    },
    status_com::MessageRes,
};
use utoipa::ToSchema;
use uuid::Uuid;
use log::{info, debug};

#[derive(Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub enum CronjobVariant {
    /// A cronjob that runs at a specific time pattern (i.e. every Monday and Tuesday at 5am and)
    Specific,
    /// A cronjob that runs at an time interval (i.e. every day)
    Interval,
}

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
struct ListCronjobsRes {
    specific_jobs: Vec<SpecificCronJob>,
    interval_jobs: Vec<IntervalCronJob>,
    crontab_exists: bool,
}

/// List users' cronjobs
#[utoipa::path(get, path = "/private/cronjobs/list", responses((status = 200, body = ListCronjobsRes), (status = 404, description = "No crontab file was found."), (status = 500, description = "Cronjobs could not be read.")), tags = ["private", "cronjobs"])]
pub async fn list() -> HttpResponse {
    let crons = cron::list_cronjobs();
    let mut interval_cronjobs: Vec<IntervalCronJob> = vec![];
    let mut specific_cronjobs: Vec<SpecificCronJob> = vec![];
    match crons {
        Ok(crons_unwrapped) => {
            for ele in crons_unwrapped {
                match ele {
                    CronJob::Specific(spec) => specific_cronjobs.push(spec),
                    CronJob::Interval(inter) => interval_cronjobs.push(inter),
                }
            }
        }
        Err(e) => match e {
            CronError::NoCronFile => {
                return HttpResponse::NotFound().json(ErrorCode::NoCronjobs.as_error_message());
            }
            _ => {
                return HttpResponse::InternalServerError()
                    .json(ErrorCode::NoCronjobs.as_error_message());
            }
        },
    }

    HttpResponse::Ok().json(ListCronjobsRes {
        specific_jobs: specific_cronjobs,
        interval_jobs: interval_cronjobs,
        crontab_exists: true,
    })
}

#[derive(Deserialize, ToSchema)]
pub struct CronjobCommandReq {
    index: usize,
    variant: CronjobVariant,
}

/// Run cronjob command
#[utoipa::path(post, path = "/private/cronjobs/runCommand", request_body = CronjobCommandReq, responses((status = 200)), tags = ["private", "cronjobs", "responding_job"])]
pub async fn run_command(state: Data<AppState>, json: Json<CronjobCommandReq>) -> HttpResponse {
    info!("Executing aribitrary cronjob command using runCommand.");
    let uuid = Uuid::new_v4();
    state
        .background_jobs
        .lock()
        .unwrap()
        .insert(uuid, BackgroundTaskState::Pending);

    let cronjobs_list_request = cron::list_cronjobs();

    let mut command_from_cronjob = None;

    if let Ok(cronjobs) = cronjobs_list_request {
        let relevant_cronjobs: Vec<&cron::CronJob> = match &json.variant {
            CronjobVariant::Specific => cronjobs
                .iter()
                .filter(|e| matches!(e, cron::CronJob::Specific(_)))
                .collect(),
            CronjobVariant::Interval => cronjobs
                .iter()
                .filter(|e| matches!(e, cron::CronJob::Interval(_)))
                .collect(),
        };

        if let Some(cronjob_at_index) = relevant_cronjobs.get(json.index) {
            command_from_cronjob = Some(match cronjob_at_index {
                cron::CronJob::Specific(c) => c.command.clone(),
                cron::CronJob::Interval(c) => c.command.clone(),
            });
        }
    } else {
        HttpResponse::InternalServerError().json(ErrorCode::NoCronjobs.as_error_message());
    }

    if command_from_cronjob.is_none() {
        return HttpResponse::NotFound().json(ErrorCode::NoSuchVariant.as_error_message());
    }

    drop(actix_web::web::block(move || {
        let status;
        debug!("Cronjob command: {:?}", command_from_cronjob);
        match Command::new("sh")
            .arg("-c")
            .arg(command_from_cronjob.unwrap())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .stdin(Stdio::null())
            .spawn()
        {
            Ok(h) => {
                let waited = h.wait_with_output();
                if let Ok(s) = waited {
                    if s.status.success() {
                        info!("Cronjob command exited sucessfully.");
                        status = BackgroundTaskState::SuccessOutput(
                            String::from_utf8(s.stdout).unwrap(),
                        );
                    } else {
                        warn!("Cronjob command failed with output.");
                        status =
                            BackgroundTaskState::FailOutput(String::from_utf8(s.stderr).unwrap());
                    }
                } else {
                    warn!("Cronjob command execution failed.");
                    status = BackgroundTaskState::Fail;
                }
            }
            Err(_) => status = {
                warn!("Cronjob command initiation failed.");
                BackgroundTaskState::Fail
            },
        };

        debug!("Cronjob command execution was given task id {uuid}");
        state.background_jobs.lock().unwrap().insert(uuid, status);
    }));

    HttpResponse::Ok().body(uuid.to_string())
}

/// Delete cronjob
#[utoipa::path(post, path = "/private/cronjobs/delete/{index}/{variant}", params(("index" = u32, Path), ("variant" = CronjobVariant, Path)),responses((status = 200)), tags = ["private", "cronjobs"])]
pub async fn delete(path: Path<(u32, CronjobVariant)>) -> HttpResponse {
    let index = path.0;
    let variant = &path.1;

    let _ = match variant {
        CronjobVariant::Specific => delete_specific_cronjob(index),
        CronjobVariant::Interval => delete_interval_cronjob(index),
    };

    HttpResponse::Ok().json(MessageRes::from("The cronjob has been deleted."))
}

#[derive(Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateCronjobReq {
    variant: CronjobVariant,
    command: String,
    interval: Option<Interval>,
    minute: Option<String>,
    hour: Option<String>,
    day_of_month: Option<String>,
    day_of_week: Option<String>,
    month: Option<String>,
}

/// Create new cronjob
#[utoipa::path(post, path = "/private/cronjobs/new", request_body = CreateCronjobReq, responses((status = 200)), tags = ["private", "cronjobs"])]
pub async fn create(json: Json<CreateCronjobReq>) -> HttpResponse {
    let variant = &json.variant;

    match variant {
        CronjobVariant::Specific => {
            let day_of_month =
                match cron::Digit::try_from(json.day_of_month.clone().unwrap().as_str()) {
                    Ok(c) => c,
                    Err(_) => {
                        return HttpResponse::BadRequest()
                            .json(ErrorCode::SanitizationError.as_error_message());
                    }
                };

            let day_of_week = DayOfWeek::from(json.day_of_week.clone().unwrap().as_str());
            let month = Month::from(json.month.clone().unwrap().as_str());
            let minute = match Digit::try_from(json.minute.clone().unwrap().as_str()) {
                Ok(c) => c,
                Err(_) => {
                    return HttpResponse::BadRequest()
                        .json(ErrorCode::SanitizationError.as_error_message());
                }
            };

            let hour = match Digit::try_from(json.hour.clone().unwrap().as_str()) {
                Ok(c) => c,
                Err(_) => {
                    return HttpResponse::BadRequest()
                        .json(ErrorCode::SanitizationError.as_error_message());
                }
            };

            match create_new_specific_cronjob(SpecificCronJob {
                command: json.command.clone(),
                day_of_week,
                day_of_month,
                minute,
                hour,
                month,
            }) {
                Ok(_) => HttpResponse::Ok()
                    .json(MessageRes::from("A new specific cronjob has been created.")),
                Err(_) => {
                    error!("Failed to create specific cronjob");
                    HttpResponse::InternalServerError()
                        .json(ErrorCode::CronjobCreationFailed.as_error_message())
                }
            }
        }
        CronjobVariant::Interval => {
            match create_new_interval_cronjob(cron::IntervalCronJob {
                interval: json.interval.unwrap(),
                command: json.command.clone(),
            }) {
                Ok(_) => HttpResponse::Ok()
                    .json(MessageRes::from("A new interval cronjob has been created.")),
                Err(_) => {
                    error!("Failed to create interval cronjob");
                    HttpResponse::InternalServerError()
                        .json(ErrorCode::CronjobCreationFailed.as_error_message())
                }
            }
        }
    }
}
