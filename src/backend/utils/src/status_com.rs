use crate::time::{self, current_timestamp_unix};
/// Communicating the status of the request to the frontend either through error codes or through a
/// single written message.
use serde::Serialize;

/// Struct notifying the receiver of having successfully finished a request and providing a small
/// description for developing purposes.
/// The struct is not intended to carry request results.
///
/// * {`time: u128`} - The time of the error as perceived by the server at that point
/// * {`message: u128`} - A written text message for debugging/developing purposes
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MessageRes {
    time: u128,
    message: String,
}

impl From<String> for MessageRes {
    fn from(value: String) -> Self {
        MessageRes {
            time: time::current_timestamp_unix(),
            message: value,
        }
    }
}

impl From<&str> for MessageRes {
    fn from(value: &str) -> Self {
        MessageRes {
            time: time::current_timestamp_unix(),
            message: value.to_string(),
        }
    }
}

/// Struct describing an error response that is serialized into JSON using serde_json.
///
/// * {`time: u128`} - The time of the error as perceived by the server at that point
/// * {`code: ErrorCode`} - The error code of the error
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ErrorRes {
    time: u128,
    message: String,
}

impl ErrorRes {
    /// Given an error code, construct an ErrorRes with the current time
    fn with_code(code: ErrorCode) -> Self {
        ErrorRes {
            time: current_timestamp_unix(),
            message: code.to_string(),
        }
    }
}

impl ErrorCode {
    pub fn as_error_message(self) -> ErrorRes {
        ErrorRes::with_code(self)
    }
}

impl From<ErrorCode> for ErrorRes {
    fn from(value: ErrorCode) -> Self {
        ErrorRes::with_code(value)
    }
}

#[derive(Serialize, Debug, thiserror::Error, Clone)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
/// An enumeration of error codes configured to serialize all enum variants into
/// SCREAMING_SNAKE_CASE to communicate errors between the backend and frontend.
pub enum ErrorCode {
    #[error("The login OTP code was not provided.")]
    MissingOtpCode,
    #[error("The provided login OTP code is wrong.")]
    WrongOtpCode,
    #[error("The provided login password is wrong.")]
    WrongPassword,
    #[error("The provided login username is unknown.")]
    UnknownUsername,
    #[error("Executing a command with UFW failed, yielding: {0}")]
    UfwExecutionFailed(String),
    #[error("Executing a command with UFW failed, yielding error code: {0:?}")]
    UfwExecutionFailedWithStatus(Option<i32>),
    #[error("Sending a signal to a process failed.")]
    SignalError,
    #[error("The provided PID could not be found on the system.")]
    UnknownPid,
    #[error("You do not have adequate permissions.")]
    MissingApiPermissions,
    #[error("You do not have adequate permissions to access this shared file.")]
    MissingSharedFilePermissions,
    #[error("You do not have adequate system permissions.")]
    MissingSystemPermissions,
    #[error("A database update failed, yielding: {0}")]
    DatabaseUpdateFailed(String),
    #[error("A database read failed, yielding: {0}")]
    DatabaseReadFailed(String),
    #[error("A database truncate failed, yielding: {0}")]
    DatabaseTruncateFailed(String),
    #[error("A database insertion failed, yielding: {0}")]
    DatabaseInsertFailed(String),
    #[error("A database deletion failed, yielding: {0}")]
    DatabaseDeletionFailed(String),
    #[error("The provided sudo password is wrong.")]
    BadSudoPassword,
    #[error("The package manager failed.")]
    PackageManagerFailed,
    #[error("The background task failed.")]
    TaskFailed,
    #[error("The background task failed, yielding: {0}")]
    TaskFailedWithDescription(String),
    #[error("The provided background task ID does not exist.")]
    NoSuchTask,
    #[error("The media center has been disabled.")]
    MediaCenterDisabled,
    #[error("The file does not exist.")]
    FileDoesNotExist,
    #[error("The file could not be interacted with, even though it exists.")]
    FileError,
    #[error("The directory does not exist.")]
    DirectoryDoesNotExist,
    #[error("The directory could not be interacted with, even though it exists.")]
    DirectoryError,
    #[error("Insufficient data")]
    InsufficientData,
    #[error("Encrypting a file or directory failed.")]
    EncryptionFailed,
    #[error("Shutdown failed.")]
    PowerOffFailed,
    #[error("The requested log entries could not be acquired.")]
    LogFetchingFailed,
    #[error("The left range for the media file is too high.")]
    LeftRangeTooHigh,
    #[error("The right range for the media file is too high.")]
    RightRangeTooHigh,
    #[error("This extension can not be provided to the frontend.")]
    ProtectedExtension,
    #[error("No cron job exist for this user.")]
    NoCronjobs,
    #[error("Creating a cron job failed.")]
    CronjobCreationFailed,
    #[error("This enum variant does not exist.")]
    NoSuchVariant,
    #[error("No such shared file exists.")]
    NoSuchSharedFile,
    #[error("The data contains invalid characters.")]
    SanitizationError,
    #[error("The rule was malformed.")]
    BadRule,
    #[error("The rule exists already.")]
    RuleSkipped,
    #[error("The rule could not be created.")]
    RuleCreationFailed,
    #[error("The rule could not be deleted.")]
    RuleDeletionFailed,
    #[error("UFW failed, yielding: {0} and {1}")]
    UfwError(String, String, Vec<String>),
    #[error("No such rule exists.")]
    NoSuchRule,
    #[error("Could not get uptime.")]
    UptimeError,
    #[error("Could not connect to docker.")]
    DockerConnectionFailed,
    #[error("Docker was unable to fullfil the request.")]
    DockerRequestFailed,
    #[error("A lock is currently in place.")]
    SystemLocked,
    #[error("Interacting with polkit failed, yielding: {0}")]
    PolkitFailed(String),
    #[error(
        "Interacting with a physical drive or utility to interact with the drive failed, yielding: {0:?}"
    )]
    DriveInteractionFailed(String),
    #[error("A device was temporarily disconnected and may no longer be the same physical device.")]
    DriveChanged,
    #[error("The device does not exist.")]
    NoSuchDrive,
    #[error("The filesystem does not exist.")]
    NoSuchFs,
    #[error("The partition does not exist.")]
    NoSuchPartition,
    #[error("The filesystem has already been mounted.")]
    FsAlreadyMounted,
    #[error("The filesystem has no mountpoints.")]
    FsNotMounted,
    #[error("This action is not supported for this drive.")]
    ActionNotSupportedForDrive,
    #[error("The drive is currently in use. Unmount any active partitions!")]
    DriveInUse,
    #[error("The drive is read-only and can not be opened in write mode.")]
    DriveReadOnly,
    #[error("The benchmark failed, yielding: {0}")]
    BenchmarkFailed(String),
    #[error("The requested stored benchmark could not be retrieved from histoy.")]
    NoSuchBenchmark,
}
