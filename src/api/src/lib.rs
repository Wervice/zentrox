use serde::{Deserialize, Serialize};
pub mod jobs;
pub mod dashboard;
pub mod login;
pub mod account;
pub mod processes;
pub mod network;
pub mod docker;
pub mod error;
pub mod packages;
pub mod polkit;
pub mod units;
pub mod drives;

#[derive(Serialize, Deserialize, Copy, Clone, Debug, Default)]
pub struct EmptyRes {}

pub struct NoteRes {
    pub title: String,
    pub message: String,
    pub time_millis: i64
}
