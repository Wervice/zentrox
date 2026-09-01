use std::collections::HashMap;

use actix_web::{HttpResponse, http::header::ContentType};
use api::EmptyRes;
use utils::status_com;
use uuid::Uuid;

pub trait ThreadSafeSerialize: Send + Sync + erased_serde::Serialize + 'static {}
impl<T: erased_serde::Serialize + Send + Sync + 'static> ThreadSafeSerialize for T {}

erased_serde::serialize_trait_object!(ThreadSafeSerialize);

/// A [`Job`] represents the progress of a function that runs on an `actix_web` thread-pool, started
/// using [`actix_web::web::block`]. Jobs should not be created directly.
pub enum Job {
    Ongoing,
    Ok(Box<dyn ThreadSafeSerialize>),
    Update(Box<dyn ThreadSafeSerialize>),
    Err { error: status_com::ErrorCode },
}

#[derive(Clone, Copy, Debug, thiserror::Error)]
pub enum Error {
    #[error("The job is still ongoing.")]
    StillOngoing,
}

#[derive(Default)]
/// Wrapper around [`HashMap`] that allows O(1) look-ups of a [`Job`] by its `uuid`.
pub struct Jobs(HashMap<Uuid, Job>);

impl Jobs {
    /// Create a new [`Job::Ongoing`] and return it's [`uuid::Uuid`] in the [`Jobs`] struct.
    pub fn start(&mut self) -> Uuid {
        let uuid = Uuid::new_v4();
        self.0.insert(uuid, Job::Ongoing);
        uuid
    }

    /// Set a [`Job`] specified using a [`uuid::Uuid`] to [`Job::Err`] with a [`status_com::ErrorCode`] describing the reason of the failure.
    pub fn fail(&mut self, uuid: Uuid, error: status_com::ErrorCode) {
        if let Some(reference) = self.0.get_mut(&uuid) {
            *reference = Job::Err { error }
        }
    }

    /// Set a [`Job`] specified using a [`uuid::Uuid`] to [`Job::Ok`] with a response struct that
    /// implements [`ThreadSafeSerialize`] describing the job results.
    pub fn succeed(&mut self, uuid: Uuid, response: impl ThreadSafeSerialize) {
        if let Some(reference) = self.0.get_mut(&uuid) {
            *reference = Job::Ok(Box::new(response));
        }
    }

    /// Set a [`Job`] specified using a [`uuid::Uuid`] to [`Job::Update`] with a response struct that
    /// implements [`ThreadSafeSerialize`] describing the job progress update.
    pub fn update(&mut self, uuid: Uuid, response: impl ThreadSafeSerialize) -> Result<(), String> {
        if let Some(reference) = self.0.get_mut(&uuid) {
            match reference {
                Job::Ongoing | Job::Update(_) => {
                    *reference = Job::Update(Box::new(response));
                }
                _ => return Err("Already finished".to_string()),
            }
        }
        Ok(())
    }

    /// Gets a [`Job`] by its [`uuid::Uuid`].
    pub fn get(&self, uuid: Uuid) -> Option<&Job> {
        self.0.get(&uuid)
    }

    /// Remove a completed [`Job`] by its [`uuid::Uuid`].
    /// If the job does not exist the function exits.
    /// If the job is still ongoing, the function returns an [`Error::StillOngoing`].
    pub fn remove(&mut self, uuid: Uuid) -> Result<(), Error> {
        match self.0.get(&uuid) {
            Some(Job::Ongoing | Job::Update(_)) => Err(Error::StillOngoing),
            Some(_) => {
                self.0.remove(&uuid);
                Ok(())
            }
            None => Ok(()),
        }
    }
}

impl Into<HttpResponse> for Job {
    fn into(self) -> HttpResponse {
        match self {
            Self::Ongoing => HttpResponse::Accepted().json(EmptyRes {}),
            Self::Ok(res) => {
                let mut bytes: Vec<u8> = vec![];
                let json_serializer = &mut serde_json::Serializer::new(&mut bytes);
                let _ = erased_serde::serialize(&res, json_serializer);
                HttpResponse::Ok()
                    .content_type(ContentType::json())
                    .body(bytes)
            }
            Self::Update(res) => {
                let mut bytes: Vec<u8> = vec![];
                let json_serializer = &mut serde_json::Serializer::new(&mut bytes);
                let _ = erased_serde::serialize(&res, json_serializer);
                HttpResponse::Created()
                    .content_type(ContentType::json())
                    .body(bytes)
            }
            Self::Err { error } => {
                HttpResponse::InternalServerError().json(error.as_error_message())
            }
        }
    }
}

impl Into<HttpResponse> for &Job {
    fn into(self) -> HttpResponse {
        match self {
            Job::Ongoing => HttpResponse::Accepted().json(EmptyRes {}),
            Job::Ok(res) => {
                let mut bytes: Vec<u8> = vec![];
                let json_serializer = &mut serde_json::Serializer::new(&mut bytes);
                let _ = erased_serde::serialize(&res, json_serializer);
                HttpResponse::Ok()
                    .content_type(ContentType::json())
                    .body(bytes)
            }
            Job::Update(res) => {
                let mut bytes: Vec<u8> = vec![];
                let json_serializer = &mut serde_json::Serializer::new(&mut bytes);
                let _ = erased_serde::serialize(&res, json_serializer);
                HttpResponse::Created()
                    .content_type(ContentType::json())
                    .body(bytes)
            }
            Job::Err { error } => {
                HttpResponse::InternalServerError().json(error.clone().as_error_message())
            }
        }
    }
}
