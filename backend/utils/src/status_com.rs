/// Communicating the status of the request to the frontend either through error codes or through a
/// single written message.
use serde::Serialize;
use std::time::UNIX_EPOCH;

fn current_time() -> u128 {
    std::time::SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis()
}

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
            time: current_time(),
            message: value,
        }
    }
}

impl From<&str> for MessageRes {
    fn from(value: &str) -> Self {
        MessageRes {
            time: current_time(),
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
    code: ErrorCode,
}

impl ErrorRes {
    /// Given an error code, construct an ErrorRes with the current time
    fn with_code(code: ErrorCode) -> Self {
        ErrorRes {
            time: current_time(),
            code,
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

#[derive(Serialize, Debug)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
/// An enumeration of error codes configured to serialize all enum variants into
/// SCREAMING_SNAKE_CASE to communicate errors between the backend and frontend.
pub enum ErrorCode {
    /// During login, an OTP code was required but not provided
    /// This is not to be confused with a malformed JSON which lacks the "otp" field.
    MissingOtpCode,
    /// The provided OTP code does not match the one calculated on the backend side at the time of
    /// the request incoming at the server.
    WrongOtpCode,
    /// The password provided during login is wrong.
    WrongPassword,
    /// The provided username is not known.
    UnkownUsername,
    /// While creating a new rule with the UFW, executing the command failed.
    /// The error contains the written error message from UFW.
    UfwExecutionFailed(String),
    /// Like UfwExecutionFailed, but only with a status code and no error message
    UfwExecutionFailedWithStatus(i32),
    /// While sending a signal to a process, an error was encountered. Most likely it was a
    /// permission error.
    SignalError,
    /// The provided process ID (PID) could not be found on the system.
    UnknownPid,
    /// The request was rejected by a middleware as the user is not logged in.
    MissingApiPermissions,
    /// A request for a shared file was rejected because the request lacked a (correct) password
    MissingSharedFilePermissions,
    /// The request could not be finished, because the user does not appear to have sufficient
    /// permissions on the system.
    MissingSystemPermissions,
    /// The user could not obtain permissions for Vault decryption,
    MissingVaultPermissions,
    /// While updating a database table using diesel.rs, the execution failed.
    DatabaseUpdateFailed(String),
    /// While reading a database table using diesel.rs, the execution failed.
    DatabaseReadFailed(String),
    /// While truncating a table using diesel.rs, the execution failed.
    DatabaseTruncateFailed(String),
    /// While inserting into a table using diesel.rs, the execution failed.
    DatabaseInsertFailed(String),
    /// While deleting a row from a table using diesel.rs, the execution failed.
    DatabaseDeletionFailed(String),
    /// While trying to run a command with sudo or verifying the sudo password, the verification
    /// failed.
    BadSudoPassword,
    /// While performing an action related to packages, the package manager encoutnered an error.
    /// It is likely, this was a permission based error or a wrong package name was provided.
    PackageManagerFailed,
    /// While processing a background task, an error without further specification was encountered
    /// and the task failed.
    TaskFailed,
    /// While processing a background task, an error was encountered and the task failed.
    TaskFailedWithDescription(String),
    /// The specified UUID does not correspond to an active task.
    NoSuchTask,
    /// The media center has been disable and access rejected
    MediaCenterDisabled,
    /// A requested file does not existing or could not be found
    FileDoesNotExist,
    /// Interacting with a file failed for another error than the file not existing
    FileError,
    /// A requested direction does not existing or could not be found
    DirectoryDoesNotExist,
    /// Interacting with a directory failed for another error than the file not existing
    DirectoryError,
    /// Insufficient data was provided to the backend
    InsufficientData,
    /// While listing block devices an error was encountered during the command execution
    BlockDeviceListingFailed,
    /// Drive statistics could not be gathered
    DriveStatisticsFailed,
    /// Drive metadata could not be gathered
    DriveMetadataFailed,
    /// Encrypting a file, directory, or string failed
    EncryptionFailed,
    /// A specified path is too long
    VaultPathTooLong,
    /// Vault has not been configured,
    VaultUnconfigured,
    /// While trying to shut down, the command failed
    PowerOffFailed,
    /// The program could not get the requested system logs.
    LogFetchingFailed,
    /// The first aka. left range value for getting a byte range for a media file is out-of-bounds
    LeftRangeTooHigh,
    /// Analogous to LeftRangeTooHigh
    RightRangeTooHigh,
    /// The extension of the requested file is not allowed to be sent back to the frontend
    ProtectedExtension,
    /// No such cronjobs could be found
    NoCronjobs,
    /// Creating a cronjob failed
    CronjobCreationFailed,
    /// The server received a string that describes a variant of an enummeration agreed through the API,
    /// but the variant does not exist.
    NoSuchVariant,
    /// No shared file corresponds to this code
    NoSuchSharedFile,
    /// Running a command failed for a reason other than sudo
    CommandFailed(String),
    /// A value was not properly sanitized
    SanitizationError,
}
