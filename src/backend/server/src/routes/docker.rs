use actix_web::HttpResponse;
use api::docker::{Container, ContainerState, ContainersRes};
use bollard::{
    API_DEFAULT_VERSION, Docker, query_parameters::ListContainersOptionsBuilder,
    secret::ContainerSummaryStateEnum,
};
use utils::status_com::ErrorCode;

#[utoipa::path(get, path = "/private/docker/containers", tags = ["private", "docker"], responses((status = 200, body = ContainersRes)))]
pub async fn active_containers() -> HttpResponse {
    let d_conn = Docker::connect_with_local("/var/run/docker.sock", 30, API_DEFAULT_VERSION);

    match d_conn {
        Ok(d) => {
            let containers_req = d
                .list_containers(Some(ListContainersOptionsBuilder::new().all(true).build()))
                .await;
            match containers_req {
                Ok(containers) => HttpResponse::Ok().json(ContainersRes {
                    containers: containers
                        .iter()
                        .map(|c| Container {
                            names: c.names.clone(),
                            id: c.id.clone().unwrap(),
                            image: c.image.clone(),
                            created: c.created,
                            state: c.state.map(|e| match e {
                                // Though ugly, I just copied bollards enumeration over into the
                                // API, so the frontend does not have to use bollard as a crate.
                                ContainerSummaryStateEnum::EMPTY => ContainerState::EMPTY,
                                ContainerSummaryStateEnum::CREATED => ContainerState::CREATED,
                                ContainerSummaryStateEnum::RUNNING => ContainerState::RUNNING,
                                ContainerSummaryStateEnum::PAUSED => ContainerState::PAUSED,
                                ContainerSummaryStateEnum::RESTARTING => ContainerState::RESTARTING,
                                ContainerSummaryStateEnum::REMOVING => ContainerState::REMOVING,
                                ContainerSummaryStateEnum::EXITED => ContainerState::EXITED,
                                ContainerSummaryStateEnum::DEAD => ContainerState::DEAD,
                            }),
                        })
                        .collect(),
                }),
                Err(err) => {
                    log::error!("Failed to list containers with UNIX defaults due to error: {err}");
                    HttpResponse::InternalServerError()
                        .json(ErrorCode::DockerRequestFailed.as_error_message())
                }
            }
        }
        Err(err) => {
            log::error!("Failed to connect with docker using UNIX defaults due to error: {err}");
            HttpResponse::InternalServerError()
                .json(ErrorCode::DockerConnectionFailed.as_error_message())
        }
    }
}
