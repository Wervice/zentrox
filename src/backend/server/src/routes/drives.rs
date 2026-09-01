use std::{fmt::Display, rc::Rc, sync::mpsc, time::Duration};

use crate::{AppState, background_jobs::Job, drives::drives::Error as DriveError};
use actix_web::{
    HttpResponse,
    web::{Data, Json},
};
use log::error;
use utils::{
    drives::{self, Drive, FileSystem},
    polkit::{self, AuthenticationPortal},
    status_com::{self, ErrorCode, MessageRes},
};

use api::{
    EmptyRes,
    drives::{
        BenchmarkHistory, BenchmarkHistoryEntry, BenchmarkReq, BenchmarkRes, BenchmarkRetrievalReq, BenchmarkUpdate, CheckRes, DriveActionReq, DrivesRes, FileSystemActionReq, HistoricalBenchmarkRes, MountRes, RepairRes
    },
    jobs::JobRes,
    polkit::NeedsPasswordRes,
    units::Bytes,
};

use diesel::prelude::*;

const MOUNT_ACTION: &str = "org.freedesktop.udisks2.filesystem-mount";
const POWEROFF_ACTION: &str = "org.freedesktop.udisks2.power-off-drive";
const EJECT_ACTION: &str = "org.freedesktop.udisks2.eject-media";

// TODO Very ugly error handling/propagation
#[derive(Debug, thiserror::Error)]
pub enum Error {
    DriveNotFound,
    FsNotFound,
    AlreadyMounted,
    NotMounted,
    Changed,
    Interaction(String),
    PolkitFailed(String),
    NotSupported,
    InUse,
    ReadOnly,
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("")
    }
}

impl From<DriveError> for Error {
    fn from(value: DriveError) -> Self {
        match value {
            DriveError::NoSuchDrive => Error::DriveNotFound,
            DriveError::DeviceChanged => Error::Changed,
            _ => Error::Interaction(value.to_string()),
        }
    }
}

impl actix_web::error::ResponseError for Error {
    fn error_response(&self) -> HttpResponse<actix_web::body::BoxBody> {
        match self {
            Self::Changed => {
                HttpResponse::NotFound().json(ErrorCode::DriveChanged.as_error_message())
            }
            Self::DriveNotFound => {
                HttpResponse::NotFound().json(ErrorCode::NoSuchDrive.as_error_message())
            }
            Self::FsNotFound => {
                HttpResponse::NotFound().json(ErrorCode::NoSuchFs.as_error_message())
            }
            Self::AlreadyMounted => {
                HttpResponse::BadRequest().json(ErrorCode::FsAlreadyMounted.as_error_message())
            }
            Self::NotMounted => {
                HttpResponse::NotFound().json(ErrorCode::FsNotMounted.as_error_message())
            }
            Self::NotSupported => HttpResponse::BadRequest()
                .json(ErrorCode::ActionNotSupportedForDrive.as_error_message()),
            Self::InUse => {
                HttpResponse::BadRequest().json(ErrorCode::DriveInUse.as_error_message())
            }
            Self::ReadOnly => {
                HttpResponse::BadRequest().json(ErrorCode::DriveReadOnly.as_error_message())
            }
            Self::PolkitFailed(err) => HttpResponse::InternalServerError()
                .json(ErrorCode::PolkitFailed(err.clone()).as_error_message()),
            Self::Interaction(err) => HttpResponse::InternalServerError()
                .json(ErrorCode::DriveInteractionFailed(err.clone()).as_error_message()),
        }
    }
}

fn utils_fs_to_api_fs(x: FileSystem) -> api::drives::FileSystem {
    api::drives::FileSystem {
        size: x.total_size().map(Bytes),
        available: x.available_size().map(Bytes),
        used: x.used_size().map(Bytes),
        path: x.path(),
        read_only: x.read_only(),
        kind: x.kind(),
        version: x.version(),
        mountpoints: x.mountpoints(),
        name: x.name(),
        children: x
            .children()
            .iter()
            .map(|y| utils_fs_to_api_fs(y.clone()))
            .collect(),
        can_check: x.can_check(),
        can_repair: x.can_repair(),
        uuid: x.uuid(),
    }
}

#[utoipa::path(get, path = "/private/drives/list", responses((status = 200)), tags = ["private", "drives"])]
pub async fn list() -> Result<HttpResponse, Error> {
    match drives::Drives::current() {
        Ok(drives_raw) => {
            let mut drives_serialized: Vec<api::drives::Drive> = vec![];

            for d in drives_raw.iter() {
                let d_serialized = api::drives::Drive {
                    rotating: d.rotational()?,
                    label: d.label()?,
                    udisks_id: d.udisks_id()?,
                    name: d.name()?,
                    path: d.device_node()?,
                    model: d.model()?,
                    revision: d.revision()?,
                    vendor: d.vendor()?,
                    size: Bytes(d.total_size()?),
                    serial: d.serial()?,
                    partitions: d
                        .partitions()?
                        .iter()
                        .map(|p| api::drives::Partition {
                            size: Bytes(p.size()),
                            label: p.label(),
                            name: p.name(),
                            path: p.clone().path(),
                            filesystem: p.clone().fs().map(utils_fs_to_api_fs),
                            uuid: p.clone().uuid(),
                        })
                        .collect(),
                    read_only: d.read_only()?,
                    filesystem: d.fs()?.map(utils_fs_to_api_fs),
                    bus_type: d.bus_type()?,
                    hint_ignore: d.hint_ignore()?,
                    hint_system: d.hint_system()?,
                    media_available: d.media_available()?,
                    can_power_off: d.can_power_off()?,
                    ejectable: d.ejectable()?,
                    removable: d.removable()?,
                    // NOTE The time of detection (time_detected) must always be a real value.
                    // NOTE Replacement values or None values are strictly forbidden!!
                    time_detected: d.time_detected().map(|x| x.timestamp_millis())?,
                };
                drives_serialized.push(d_serialized);
            }

            Ok(HttpResponse::Ok().json(DrivesRes {
                drives: drives_serialized,
            }))
        }
        Err(err) => {
            error!(
                "Failed to list currently connected physical storage devices due to error: {err}"
            );
            Ok(HttpResponse::InternalServerError().json(
                status_com::ErrorCode::DriveInteractionFailed(err.to_string()).as_error_message(),
            ))
        }
    }
}

fn authorize_for_action(
    action_id: String,
    password: String,
    portal: &mut AuthenticationPortal,
) -> Result<(), Error> {
    portal
        .provide_password(action_id.to_string(), password.to_string())
        .map_err(|err| Error::PolkitFailed(err.to_string()))?;
    Ok(())
}

fn check_is_authorized(action_id: &str) -> Result<bool, Error> {
    polkit::AuthenticationPortal::check_is_authorized(action_id)
        .map_err(|err| Error::PolkitFailed(err.to_string()))
}

fn check_is_authorized_http(action_id: &str) -> Result<HttpResponse, Error> {
    let needs_password = !check_is_authorized(action_id)?;
    Ok(HttpResponse::Ok().json(NeedsPasswordRes { needs_password }))
}

#[utoipa::path(get, path = "/private/drives/can_mount", responses((status = 200)), tags = ["private", "drives"])]
pub async fn can_mount_fs() -> Result<HttpResponse, Error> {
    check_is_authorized_http(MOUNT_ACTION)
}

fn find_real_drive_from_api_drive(
    list: Rc<[Drive]>,
    target: api::drives::DriveSignature,
) -> Result<Option<Drive>, DriveError> {
    list.iter()
        .try_fold::<Option<Drive>, _, Result<Option<Drive>, DriveError>>(None, |acc, d| {
            if acc.is_some() {
                return Ok(acc);
            }
            Ok(
                if d.time_detected()?.timestamp_millis() == target.time_detected
                    && d.device_node()? == target.device_node
                    && d.serial()? == target.serial
                {
                    Some(d.clone())
                } else {
                    None
                },
            )
        })
}

#[utoipa::path(post, path = "/private/drives/mount", responses((status = 200)), tags = ["private", "drives"])]
pub async fn mount_fs(
    json: Json<FileSystemActionReq>,
    state: Data<AppState>,
) -> Result<HttpResponse, Error> {
    match drives::Drives::current() {
        Ok(drives_list) => {
            if let Some(drive) = find_real_drive_from_api_drive(drives_list, json.drive.clone())?
                && let Some(fs) = drive.get_fs_by_path(json.fs.clone())?
            {
                if fs.mounted() {
                    return Err(Error::AlreadyMounted);
                }
                if let Some(password) = &json.password {
                    authorize_for_action(
                        MOUNT_ACTION.to_string(),
                        password.clone(),
                        &mut state.polkit_portal.lock().unwrap(),
                    )?;
                } else if !check_is_authorized(MOUNT_ACTION)? {
                    return Err(Error::NotSupported);
                }
                match fs.mount() {
                    Ok(mountpoint) => Ok(HttpResponse::Ok().json(MountRes { mountpoint })),
                    Err(err) => {
                        error!("Failed to mount drive due to error: {err}");
                        Err(Error::Interaction(err.to_string()))
                    }
                }
            } else {
                Err(Error::FsNotFound)
            }
        }
        Err(err) => {
            error!(
                "Failed to list currently connected physical storage devices due to error: {err}"
            );
            Ok(HttpResponse::InternalServerError().json(
                status_com::ErrorCode::DriveInteractionFailed(err.to_string()).as_error_message(),
            ))
        }
    }
}

#[utoipa::path(post, path = "/private/drives/unmount", responses((status = 200)), tags = ["private", "drives"])]
pub async fn unmount_fs(json: Json<FileSystemActionReq>) -> Result<HttpResponse, Error> {
    match drives::Drives::current() {
        Ok(drives_list) => {
            if let Some(drive) = find_real_drive_from_api_drive(drives_list, json.drive.clone())?
                && let Some(fs) = drive.get_fs_by_path(json.fs.clone())?
            {
                if !fs.mounted() {
                    return Err(Error::NotMounted);
                }
                match fs.unmount() {
                    Ok(_) => {
                        Ok(HttpResponse::Ok()
                            .json(MessageRes::from("Successfully unmounted drive.")))
                    }
                    Err(err) => {
                        error!("Failed to unmount drive due to error: {err}");
                        Err(Error::Interaction(err.to_string()))
                    }
                }
            } else {
                Err(Error::FsNotFound)
            }
        }
        Err(err) => {
            error!(
                "Failed to list currently connected physical storage devices due to error: {err}"
            );
            Ok(HttpResponse::InternalServerError().json(
                status_com::ErrorCode::DriveInteractionFailed(err.to_string()).as_error_message(),
            ))
        }
    }
}

#[utoipa::path(get, path = "/private/drives/can_poweroff", responses((status = 200)), tags = ["private", "drives"])]
pub async fn can_power_off_drive() -> Result<HttpResponse, Error> {
    check_is_authorized_http(POWEROFF_ACTION)
}

#[utoipa::path(post, path = "/private/drives/poweroff", responses((status = 200)), tags = ["private", "drives"])]
pub async fn power_off(
    json: Json<DriveActionReq>,
    state: Data<AppState>,
) -> Result<HttpResponse, Error> {
    match drives::Drives::current() {
        Ok(drives_list) => {
            if let Some(drive) = find_real_drive_from_api_drive(drives_list, json.drive.clone())? {
                if !drive.can_power_off()? {
                    return Err(Error::NotSupported);
                }
                if drive.has_mounted_fs()? {
                    return Err(Error::InUse);
                }
                if let Some(password) = &json.password {
                    authorize_for_action(
                        POWEROFF_ACTION.to_string(),
                        password.clone(),
                        &mut state.polkit_portal.lock().unwrap(),
                    )?;
                } else if !check_is_authorized(MOUNT_ACTION)? {
                    return Err(Error::NotSupported);
                }
                match drive.power_off() {
                    Ok(_) => Ok(HttpResponse::Ok()
                        .json(MessageRes::from("Successfully powered off drive."))),
                    Err(err) => {
                        error!("Failed to unmount drive due to error: {err}");
                        Err(Error::Interaction(err.to_string()))
                    }
                }
            } else {
                Err(Error::DriveNotFound)
            }
        }
        Err(err) => {
            error!(
                "Failed to list currently connected physical storage devices due to error: {err}"
            );
            Ok(HttpResponse::InternalServerError().json(
                status_com::ErrorCode::DriveInteractionFailed(err.to_string()).as_error_message(),
            ))
        }
    }
}

#[utoipa::path(get, path = "/private/drives/can_eject", responses((status = 200)), tags = ["private", "drives"])]
pub async fn can_eject_drive() -> Result<HttpResponse, Error> {
    check_is_authorized_http(EJECT_ACTION)
}

#[utoipa::path(post, path = "/private/drives/eject", responses((status = 200)), tags = ["private", "drives"])]
pub async fn eject(
    json: Json<DriveActionReq>,
    state: Data<AppState>,
) -> Result<HttpResponse, Error> {
    match drives::Drives::current() {
        Ok(drives_list) => {
            if let Some(drive) = find_real_drive_from_api_drive(drives_list, json.drive.clone())? {
                if !drive.ejectable()? {
                    return Err(Error::NotSupported);
                }
                if drive.has_mounted_fs()? {
                    return Err(Error::InUse);
                }
                if let Some(password) = &json.password {
                    authorize_for_action(
                        EJECT_ACTION.to_string(),
                        password.clone(),
                        &mut state.polkit_portal.lock().unwrap(),
                    )?;
                } else if !check_is_authorized(MOUNT_ACTION)? {
                    return Err(Error::NotSupported);
                }
                match drive.eject() {
                    Ok(_) => Ok(
                        HttpResponse::Ok().json(MessageRes::from("Successfully ejected drive."))
                    ),
                    Err(err) => {
                        error!("Failed to eject drive due to error: {err}");
                        Err(Error::Interaction(err.to_string()))
                    }
                }
            } else {
                Err(Error::DriveNotFound)
            }
        }
        Err(err) => {
            error!(
                "Failed to list currently connected physical storage devices due to error: {err}"
            );
            Ok(HttpResponse::InternalServerError().json(
                status_com::ErrorCode::DriveInteractionFailed(err.to_string()).as_error_message(),
            ))
        }
    }
}

#[utoipa::path(post, path = "/private/drives/check", responses((status = 200)), tags = ["private", "drives"])]
pub async fn check(
    json: Json<FileSystemActionReq>,
    data: Data<AppState>,
) -> Result<HttpResponse, Error> {
    match drives::Drives::current() {
        Ok(drives_list) => {
            if let Some(drive) = find_real_drive_from_api_drive(drives_list, json.drive.clone())?
                && let Some(fs) = drive.get_fs_by_path(json.fs.clone())?
            {
                if fs.mounted() {
                    return Err(Error::InUse);
                }

                let uuid = data.background_jobs.lock().unwrap().start();

                let block = actix_web::web::block(move || match fs.check() {
                    Ok(consistent) => {
                        data.background_jobs
                            .lock()
                            .unwrap()
                            .succeed(uuid, CheckRes { consistent });
                    }
                    Err(err) => {
                        log::error!(
                            "Failed to check {} for consistency.",
                            fs.path().to_string_lossy()
                        );
                        data.background_jobs
                            .lock()
                            .unwrap()
                            .fail(uuid, ErrorCode::DriveInteractionFailed(err.to_string()));
                    }
                });

                drop(block);

                Ok(HttpResponse::Ok().json(JobRes { uuid }))
            } else {
                Err(Error::DriveNotFound)
            }
        }
        Err(err) => {
            error!(
                "Failed to list currently connected physical storage devices due to error: {err}"
            );
            Ok(HttpResponse::InternalServerError().json(
                status_com::ErrorCode::DriveInteractionFailed(err.to_string()).as_error_message(),
            ))
        }
    }
}

#[utoipa::path(post, path = "/private/drives/repair", responses((status = 200)), tags = ["private", "drives"])]
pub async fn repair(
    json: Json<FileSystemActionReq>,
    data: Data<AppState>,
) -> Result<HttpResponse, Error> {
    match drives::Drives::current() {
        Ok(drives_list) => {
            if let Some(drive) = find_real_drive_from_api_drive(drives_list, json.drive.clone())?
                && let Some(fs) = drive.get_fs_by_path(json.fs.clone())?
            {
                if fs.mounted() {
                    return Err(Error::InUse);
                }

                let uuid = data.background_jobs.lock().unwrap().start();

                let block = actix_web::web::block(move || match fs.repair() {
                    Ok(success) => {
                        data.background_jobs
                            .lock()
                            .unwrap()
                            .succeed(uuid, RepairRes { success });
                    }
                    Err(err) => {
                        log::error!("Failed to repair {}.", fs.path().to_string_lossy());
                        data.background_jobs
                            .lock()
                            .unwrap()
                            .fail(uuid, ErrorCode::DriveInteractionFailed(err.to_string()));
                    }
                });

                drop(block);

                Ok(HttpResponse::Ok().json(JobRes { uuid }))
            } else {
                Err(Error::DriveNotFound)
            }
        }
        Err(err) => {
            error!(
                "Failed to list currently connected physical storage devices due to error: {err}"
            );
            Ok(HttpResponse::InternalServerError().json(
                status_com::ErrorCode::DriveInteractionFailed(err.to_string()).as_error_message(),
            ))
        }
    }
}

#[utoipa::path(post, path = "/private/drives/benchmark", responses((status = 200)), tags = ["private", "drives"])]
pub async fn benchmark(
    data: Data<AppState>,
    json: Json<BenchmarkReq>,
) -> Result<HttpResponse, Error> {
    match drives::Drives::current() {
        Ok(drives_list) => {
            if let Some(drive) = find_real_drive_from_api_drive(drives_list, json.drive.clone())? {
                if drive.has_mounted_fs()? && json.write {
                    return Err(Error::InUse);
                }

                if json.write && drive.read_only()? {
                    return Err(Error::ReadOnly);
                }

                let uuid = data.background_jobs.lock().unwrap().start();

                let block = actix_web::web::block(move || match drive.benchmark() {
                    Ok(mut builder) => {
                        let (tx, rx) = mpsc::channel();

                        let portal = data.polkit_portal.clone();
                        let jobs = data.background_jobs.clone();
                        let inner_block = actix_web::web::block(move || {
                            let bm_exec = builder
                                .portal(portal)
                                .password(json.password.clone())
                                .do_write(json.write)
                                .do_random(json.random)
                                .sample_size(json.sample_size)
                                .iterations(json.iterations)
                                .updater(tx)
                                .execute();
                            match bm_exec {
                                Ok(res) => {
                                    jobs.lock().unwrap().succeed(
                                        uuid,
                                        BenchmarkRes {
                                            read: res.get_read().samples(),
                                            write: res.get_write().map(|t| t.samples()),
                                            drive: json.drive.clone(),
                                            sample_size: json.sample_size,
                                            random: json.random
                                        },
                                    );
                                }
                                Err(err) => {
                                    jobs.lock()
                                        .unwrap()
                                        .fail(uuid, ErrorCode::BenchmarkFailed(err.to_string()));
                                }
                            }
                        });
                        drop(inner_block);

                        while let Ok(v) = rx.recv() {
                            log::debug!(
                                "Benchmark for {} is at {}%.",
                                drive.device_node().unwrap().to_string_lossy(),
                                v.percentage()
                            );
                            let mut jobs_mutex = data.background_jobs.lock().unwrap();
                            let job = jobs_mutex.get(uuid);
                            match job {
                                None => {}
                                Some(Job::Ongoing | Job::Update(_)) => {
                                    let _ = jobs_mutex.update(
                                        uuid,
                                        BenchmarkUpdate {
                                            progress: v.percentage(),
                                        },
                                    );
                                }
                                _ => {}
                            }
                            drop(jobs_mutex);
                        }
                    }
                    Err(err) => {
                        data.background_jobs
                            .lock()
                            .unwrap()
                            .fail(uuid, ErrorCode::DriveInteractionFailed(err.to_string()));
                    }
                });

                drop(block);

                Ok(HttpResponse::Ok().json(JobRes { uuid }))
            } else {
                Err(Error::DriveNotFound)
            }
        }
        Err(err) => {
            error!(
                "Failed to list currently connected physical storage devices due to error: {err}"
            );
            Ok(HttpResponse::InternalServerError().json(
                status_com::ErrorCode::DriveInteractionFailed(err.to_string()).as_error_message(),
            ))
        }
    }
}

#[utoipa::path(post, path = "/private/drives/store_benchmark", responses((status = 200)), tags = ["private", "drives"])]
pub async fn store_benchmark(data: Data<AppState>, json: Json<BenchmarkRes>) -> HttpResponse {
    use utils::models::DriveBenchmark;
    use utils::models::DriveBenchmarkMeasurement;
    use utils::schema::DriveBenchmarkMeasurements::dsl::*;
    use utils::schema::DriveBenchmarks::dsl::*;

    let connection = &mut data.db_pool.lock().unwrap().get().unwrap();

    let uuid = uuid::Uuid::new_v4();

    let instance_insertion = diesel::insert_into(DriveBenchmarks).values(DriveBenchmark {
        drive_id: json.drive.udisks_id.clone(),
        sample_size: json.sample_size as i64,
        id: uuid.to_string(),
        iterations: json.read.len() as i32,
        random: json.random,
        time: utils::time::current_timestamp_unix() as i64,
    });

    let instance_insertion_execution = instance_insertion.execute(connection);

    if let Err(err) = instance_insertion_execution {
        return HttpResponse::InternalServerError()
            .json(status_com::ErrorCode::DatabaseInsertFailed(err.to_string()).as_error_message());
    }

    let datapoints_insertions_read = diesel::insert_into(DriveBenchmarkMeasurements).values(
        json.read
            .iter()
            .enumerate()
            .map(|(i, m)| DriveBenchmarkMeasurement {
                benchmark_id: uuid.to_string(),
                variant: "R".to_string(),
                idx: i as i64,
                nanos: m.as_nanos() as i64,
            })
            .collect::<Vec<DriveBenchmarkMeasurement>>(),
    );

    if let Err(err) = datapoints_insertions_read.execute(connection) {
        return HttpResponse::InternalServerError()
            .json(status_com::ErrorCode::DatabaseInsertFailed(err.to_string()).as_error_message());
    }

    if let Some(writes) = &json.write {
        let datapoints_insertions_write = diesel::insert_into(DriveBenchmarkMeasurements).values(
            writes
                .iter()
                .enumerate()
                .map(|(i, m)| DriveBenchmarkMeasurement {
                    benchmark_id: uuid.to_string(),
                    variant: "W".to_string(),
                    idx: i as i64,
                    nanos: m.as_nanos() as i64,
                })
                .collect::<Vec<DriveBenchmarkMeasurement>>(),
        );

        if let Err(err) = datapoints_insertions_write.execute(connection) {
            return HttpResponse::InternalServerError().json(
                status_com::ErrorCode::DatabaseInsertFailed(err.to_string()).as_error_message(),
            );
        }
    }

    HttpResponse::Ok().json(EmptyRes {})
}

#[utoipa::path(post, path = "/private/drives/benchmark_history", responses((status = 200)), tags = ["private", "drives"])]
pub async fn benchmark_history(data: Data<AppState>, json: Json<DriveActionReq>) -> HttpResponse {
    use utils::models::DriveBenchmark;
    use utils::schema::DriveBenchmarks::dsl::*;

    let connection = &mut data.db_pool.lock().unwrap().get().unwrap();

    let requested_id = json.drive.udisks_id.clone();

    let select_exec = DriveBenchmarks
        .select(DriveBenchmark::as_select())
        .filter(drive_id.eq(requested_id))
        .order_by(time)
        .get_results(connection);

    match select_exec {
        Ok(benchmarks) => {
            let history = benchmarks
                .iter()
                .map(|x| BenchmarkHistoryEntry {
                    uuid: x.id.clone(),
                    sample_size: x.sample_size as usize,
                    time: x.time,
                })
                .collect::<Vec<BenchmarkHistoryEntry>>();
            HttpResponse::Ok().json(BenchmarkHistory { history })
        }
        Err(err) => {
            HttpResponse::InternalServerError().json(ErrorCode::DatabaseReadFailed(err.to_string()))
        }
    }
}

#[utoipa::path(post, path = "/private/drives/past_benchmark", responses((status = 200)), tags = ["private", "drives"])]
pub async fn past_benchmark(
    data: Data<AppState>,
    json: Json<BenchmarkRetrievalReq>,
) -> HttpResponse {
    use utils::models::DriveBenchmark;
    use utils::models::DriveBenchmarkMeasurement;
    use utils::schema::DriveBenchmarkMeasurements::dsl::*;
    use utils::schema::DriveBenchmarks::dsl::*;

    let connection = &mut data.db_pool.lock().unwrap().get().unwrap();

    let requested_id = json.id.clone();

    let select_exec = DriveBenchmarks
        .select(DriveBenchmark::as_select())
        .filter(id.eq(requested_id.clone()))
        .get_result(connection);

    match select_exec {
        Ok(instance) => {
            let mut reads = vec![];
            let mut writes = vec![];

            let res_read = DriveBenchmarkMeasurements
                .select(DriveBenchmarkMeasurement::as_select())
                .filter(
                    benchmark_id
                        .eq(requested_id.clone())
                        .and(variant.eq("R".to_string())),
                )
                .order_by(idx)
                .get_results(connection);

            let res_writes = DriveBenchmarkMeasurements
                .select(DriveBenchmarkMeasurement::as_select())
                .filter(
                    benchmark_id
                        .eq(requested_id)
                        .and(variant.eq("W".to_string())),
                )
                .order_by(idx)
                .get_results(connection);

            match res_read {
                Ok(measurements_vec) => {
                    for m in measurements_vec {
                        reads.push(Duration::from_nanos(m.nanos as u64));
                    }
                }
                Err(err) => {
                    return HttpResponse::InternalServerError()
                        .json(ErrorCode::DatabaseReadFailed(err.to_string()));
                }
            }

            match res_writes {
                Ok(measurements_vec) => {
                    for m in measurements_vec {
                        writes.push(Duration::from_nanos(m.nanos as u64));
                    }
                }
                Err(err) => {
                    return HttpResponse::InternalServerError()
                        .json(ErrorCode::DatabaseReadFailed(err.to_string()));
                }
            }

            HttpResponse::Ok().json(HistoricalBenchmarkRes {
                sample_size: instance.sample_size as usize,
                random: instance.random,
                write: if writes.is_empty() {
                    None
                } else {
                    Some(writes)
                },
                read: reads,
            })
        }
        Err(err) => {
            error!("Did not retrieve benchmark from history with error: {err}");
            HttpResponse::NotFound().json(ErrorCode::NoSuchBenchmark.as_error_message())
        }
    }
}
