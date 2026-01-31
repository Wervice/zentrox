use actix_web::{
    HttpResponse,
    web::{Data, Json},
};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use std::time::UNIX_EPOCH;
use utils::packages;
use utils::status_com::ErrorCode;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{AppState, BackgroundTaskState, SudoPasswordReq};

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
struct PackageDatabaseRes {
    installed: Vec<String>,
    available: Vec<String>,
    package_manager: Option<packages::PackageManager>,
    updates: Option<Vec<String>>,
    last_database_update: Option<i64>, // The last database update expressed as seconds since the
                                       // UNIX epoch
}

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
struct PackageStatisticsRes {
    installed: usize,
    available: usize,
    package_manager: Option<packages::PackageManager>,
    updates: Option<usize>,
    last_database_update: Option<i64>, // The last database update expressed as seconds since the
                                       // UNIX epoch
}

#[utoipa::path(
    get,
    path = "/private/packages/database",
    responses((status = 200, body = PackageDatabaseRes)),
    tags = ["private", "packages"]
)]
/// Package database
///
/// This includes the full names of all packages known to the package manager. Updates can not be listed if the package
/// manager is PacMan.
pub async fn database(state: Data<AppState>) -> HttpResponse {
    use utils::models::PackageAction;
    use utils::schema::PackageActions::dsl::*;
    let connection = &mut state.db_pool.lock().unwrap().get().unwrap();

    let stored_last_database_update = match PackageActions
        .select(PackageAction::as_select())
        .first(connection)
    {
        Ok(v) => v.last_database_update,
        Err(database_error) => {
            return HttpResponse::InternalServerError().json(
                ErrorCode::DatabaseReadFailed(database_error.to_string()).as_error_message(),
            );
        }
    };

    let installed = packages::list_installed_packages();

    let available = match packages::list_available_packages() {
        Ok(packages) => packages,
        Err(_) => {
            return HttpResponse::InternalServerError()
                .json(ErrorCode::PackageManagerFailed.as_error_message());
        }
    };

    let updates = packages::list_updates().ok();

    HttpResponse::Ok().json(PackageDatabaseRes {
        installed,
        available,
        package_manager: packages::get_package_manager().ok(),
        updates,
        last_database_update: stored_last_database_update,
    })
}

#[utoipa::path(
    get,
    path = "/private/packages/statistics",
    responses((status = 200, body = PackageStatisticsRes)),
    tags = ["private", "packages"]
)]
/// Package database counts
pub async fn statistics(state: Data<AppState>) -> HttpResponse {
    use utils::models::PackageAction;
    use utils::schema::PackageActions::dsl::*;
    let connection = &mut state.db_pool.lock().unwrap().get().unwrap();

    let stored_last_database_update = match PackageActions
        .select(PackageAction::as_select())
        .first(connection)
    {
        Ok(v) => v.last_database_update,
        Err(database_error) => {
            return HttpResponse::InternalServerError().json(
                ErrorCode::DatabaseReadFailed(database_error.to_string()).as_error_message(),
            );
        }
    };

    let installed = packages::list_installed_packages().len();

    let available = match packages::list_available_packages() {
        Ok(packages) => packages.len(),
        Err(_) => {
            return HttpResponse::InternalServerError()
                .json(ErrorCode::PackageManagerFailed.as_error_message());
        }
    };

    let updates = match packages::list_updates() {
        Ok(packages) => Some(packages.len()),
        Err(_) => None,
    };

    HttpResponse::Ok().json(PackageStatisticsRes {
        installed,
        available,
        package_manager: packages::get_package_manager().ok(),
        updates,
        last_database_update: stored_last_database_update,
    })
}

#[derive(Serialize, ToSchema)]
struct OrphanedPackagesRes {
    packages: Vec<String>,
}

/// List of packages affected by auto-remove.
#[utoipa::path(
    get,
    path = "/private/packages/orphaned",
    responses((status = 200, body = OrphanedPackagesRes)),
    tags = ["private", "packages"]
)]
pub async fn orphaned() -> HttpResponse {
    if let Ok(packages) = packages::list_orphaned_packages() {
        HttpResponse::Ok().json(OrphanedPackagesRes { packages })
    } else {
        HttpResponse::InternalServerError().json(ErrorCode::PackageManagerFailed.as_error_message())
    }
}

#[utoipa::path(
    post,
    path = "/private/packages/updateDatabase",
    request_body = SudoPasswordReq,
    responses((status = 200, description = "The update has been started, a Job ID in UUID format is provided.")),
    tags=["private", "packages", "responding_job"]
)]
/// Update package database
///
/// This action may take several minutes and is useful for discovering outdated packages.
/// The task is ran asynchronous to the rest of the program and a job id is given which can be
/// polled to get the state of the job.
pub async fn update_db(state: Data<AppState>, json: Json<SudoPasswordReq>) -> HttpResponse {
    use utils::models::PackageAction;
    use utils::schema::PackageActions::dsl::*;
    let job_id = Uuid::new_v4();

    state
        .background_jobs
        .lock()
        .unwrap()
        .insert(job_id, BackgroundTaskState::Pending);

    let block = actix_web::web::block(move || {
        let connection = &mut state.db_pool.lock().unwrap().get().unwrap();
        if packages::update_database(json.into_inner().sudo_password).is_ok() {
            let updated_new_database_update = std::time::SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64;

            let new_package_action_row = PackageAction {
                last_database_update: Some(updated_new_database_update),
                key: 0_i32,
            };

            let package_action_time_update_execution = diesel::insert_into(PackageActions)
                .values(new_package_action_row)
                .on_conflict(key)
                .do_update()
                .set(last_database_update.eq(updated_new_database_update))
                .execute(connection);

            if let Err(database_error) = package_action_time_update_execution {
                state.background_jobs.lock().unwrap().insert(
                    job_id,
                    BackgroundTaskState::FailOutput(database_error.to_string()),
                )
            } else {
                state
                    .background_jobs
                    .lock()
                    .unwrap()
                    .insert(job_id, BackgroundTaskState::Success)
            }
        } else {
            state
                .background_jobs
                .lock()
                .unwrap()
                .insert(job_id, BackgroundTaskState::Fail)
        }
    });

    drop(block);

    HttpResponse::Ok().body(job_id.to_string())
}

/// Struct used for all actions performed on a package (install, remove, update...)
#[derive(Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PackageActionReq {
    package_name: String,
    sudo_password: String,
}

#[utoipa::path(
    post,
    path = "/private/packages/install",
    request_body = PackageActionReq,
    responses((status = 200, description = "Job started with ID")),
    tags = ["private", "packages", "responding_job"]
)]
/// Install package
///
/// It requires the package name along side the sudo password in the request body.
/// This only works under apt, dnf and pacman. This request responds only with a job id.
pub async fn install_package(json: Json<PackageActionReq>, state: Data<AppState>) -> HttpResponse {
    let job_id = Uuid::new_v4();

    drop(actix_web::web::block(
        move || match packages::install_package(
            json.package_name.to_string(),
            json.sudo_password.to_string(),
        ) {
            Ok(_) => state
                .background_jobs
                .lock()
                .unwrap()
                .insert(job_id, BackgroundTaskState::Success),
            Err(_) => state
                .background_jobs
                .lock()
                .unwrap()
                .insert(job_id, BackgroundTaskState::Fail),
        },
    ));

    HttpResponse::Ok().body(job_id.to_string())
}

#[utoipa::path(
    post,
    path = "/private/packages/remove",
    request_body = PackageActionReq,
    responses((status = 200, description = "Job started with ID")),
    tags = ["private", "packages", "responding_job"]
)]
/// Remove package
///
/// It requires the package name along side the sudo password in the request body.
/// This only works under apt, dnf and pacman. This request responds only with a job id.
pub async fn remove_package(json: Json<PackageActionReq>, state: Data<AppState>) -> HttpResponse {
    let job_id = Uuid::new_v4();

    drop(actix_web::web::block(
        move || match packages::remove_package(
            json.package_name.to_string(),
            json.sudo_password.to_string(),
        ) {
            Ok(_) => state
                .background_jobs
                .lock()
                .unwrap()
                .insert(job_id, BackgroundTaskState::Success),
            Err(_) => state
                .background_jobs
                .lock()
                .unwrap()
                .insert(job_id, BackgroundTaskState::Fail),
        },
    ));

    HttpResponse::Ok().body(job_id.to_string())
}

#[utoipa::path(
    post,
    path = "/private/packages/update",
    request_body = PackageActionReq,
    responses((status = 200, description = "Job started with ID")),
    tags = ["private", "packages", "responding_job"]
)]
/// Update packages
///
/// It requires the package name along side the sudo password in the request body.
/// This only works under apt, dnf and pacman. This request responds only with a job id.
pub async fn update_package(state: Data<AppState>, json: Json<PackageActionReq>) -> HttpResponse {
    let job_id = Uuid::new_v4();

    drop(actix_web::web::block(
        move || match packages::update_package(
            json.package_name.to_string(),
            json.sudo_password.to_string(),
        ) {
            Ok(_) => state
                .background_jobs
                .lock()
                .unwrap()
                .insert(job_id, BackgroundTaskState::Success),
            Err(_) => state
                .background_jobs
                .lock()
                .unwrap()
                .insert(job_id, BackgroundTaskState::Fail),
        },
    ));

    HttpResponse::Ok().body(job_id.to_string())
}

#[utoipa::path(
    post,
    path = "/private/packages/updateAll",
    request_body = SudoPasswordReq,
    responses((status = 200, description = "Job started with ID")),
    tags = ["private", "packages", "responding_job"]
)]
/// Update all packages
///
/// It requires the package name along side the sudo password in the request body.
/// This only works under apt, dnf and pacman.
pub async fn update_all(state: Data<AppState>, json: Json<SudoPasswordReq>) -> HttpResponse {
    let sudo_password = json.sudo_password.clone();
    let job_id = Uuid::new_v4();

    state
        .background_jobs
        .lock()
        .unwrap()
        .insert(job_id, BackgroundTaskState::Pending);

    drop(actix_web::web::block(
        move || match packages::update_all_packages(sudo_password.to_string()) {
            Ok(_) => state
                .background_jobs
                .lock()
                .unwrap()
                .insert(job_id, BackgroundTaskState::Success),
            Err(_) => state
                .background_jobs
                .lock()
                .unwrap()
                .insert(job_id, BackgroundTaskState::Fail),
        },
    ));

    HttpResponse::Ok().body(job_id.to_string())
}

#[utoipa::path(
    post,
    path = "/private/packages/removeOrphaned",
    request_body = SudoPasswordReq,
    responses((status = 200, description = "Job started with ID")),
    tags = ["private", "packages", "responding_job"]
)]
/// Auto-remove packages
pub async fn remove_orphaned(json: Json<SudoPasswordReq>, state: Data<AppState>) -> HttpResponse {
    let sudo_password = json.sudo_password.clone();
    let job_id = Uuid::new_v4();

    state
        .background_jobs
        .lock()
        .unwrap()
        .insert(job_id, BackgroundTaskState::Pending);

    drop(actix_web::web::block(move || {
        let status = match packages::remove_orphaned_packages(sudo_password.to_string()) {
            Ok(_) => BackgroundTaskState::Success,
            Err(_) => BackgroundTaskState::Fail,
        };
        state.background_jobs.lock().unwrap().insert(job_id, status)
    }));

    HttpResponse::Ok().body(job_id.to_string())
}
