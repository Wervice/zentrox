//! Interact with physical storage devices and partitions using `udisks2` and `lsblk`.
//! This file provides functionality for
//! - listing,
//! - mounting,
//! - unmounting,
//! - getting details about,
//! - ejecting,
//! - powering off,
//! - benchmarking
//! drives or partitions respectively.
//!
//! It does so by calling `lsblk` to get information about physical devices partitions and file
//! systems.
//! To perform actions on physical devices and mount file systems, `udisks2` is used in order to
//! avoid unnecessary permission escalation and use existing software.
//! To communicate with `udisks2`, (the system) D-Bus is used.

// TODO Encrypted devices
// TODO CD/DVD ejection and prevent benchmark because it is a horrible idea
// TODO Documentation comments insufficient or wrong!

use chrono::{DateTime, Utc};
use serde::Deserialize;
use std::{collections::HashMap, path::PathBuf, process::Command, time::Duration};

use std::rc::Rc;
use std::sync::{Arc, Mutex};

use crate::{
    drives::Error::DeviceChanged,
    polkit::{AuthenticationPortal, RequestError},
    storage_benchmark as bm,
};

/// Reexport
pub use crate::storage_benchmark::{BenchmarkError, BenchmarkResults, Throughput, Update};

use dbus::{
    self, Message, Path,
    arg::{RefArg, Variant},
    blocking::{BlockingSender, Connection, stdintf::org_freedesktop_dbus::Properties},
};

const LSBLK_FIELDS: &str = "NAME,LABEL,MODEL,PATH,SIZE,OWNER,MOUNTPOINTS,FSUSED,FSTYPE,FSVER,FSSIZE,FSAVAIL,UUID,ROTA,SERIAL,TYPE,VENDOR,PARTUUID,RO,REV";

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("The lsblk command was not executed correctly.")]
    CommandError,
    #[error("Parsing the lsblk output failed, yielding {0}")]
    ParseError(String),
    #[error("No drive with such a name exists.")]
    NoSuchDrive,
    #[error(
        "The requested drive was temporarily disconnected and may not be the same physical device anymore!"
    )]
    DeviceChanged,
    #[error("Connecting to the system D-Bus failed, yielding: {0}")]
    DbusConnectionFailed(Box<dyn std::error::Error>),
    #[error("Communicating with the system D-Bus failed, yielding: {0}")]
    DbusCommunicationFailed(Box<dyn std::error::Error>),
    #[error("Exclusive device access is required.")]
    NoExclusiveAccess,
    #[error("Authorization via PolicyKit failed, yielding: {0}")]
    PolicyKitError(RequestError),
    #[error("The file descriptor returned by Udisks2 is invalid.")]
    InvalidFd,
    #[error("The information provided by the system is insufficient or invalid.")]
    InvalidData,
}

// Describes the mode in which a file descriptor can be.
pub enum FdMode {
    Read,
    Write,
    ReadWrite,
}

/// Struct used for deserializing a partition from JSON.
#[derive(Deserialize, Clone, Debug)]
struct PartitionRaw {
    size: u64,
    name: String,
    label: Option<String>,
    #[serde(rename(deserialize = "fssize"))]
    fs_size: Option<u64>,
    #[serde(rename(deserialize = "fstype"))]
    fs_kind: Option<String>,
    #[serde(rename(deserialize = "fsver"))]
    fs_version: Option<String>,
    #[serde(rename(deserialize = "fsavail"))]
    fs_available: Option<u64>,
    #[serde(rename(deserialize = "fsused"))]
    fs_used: Option<u64>,
    #[serde(rename(deserialize = "ro"))]
    read_only: bool,
    mountpoints: Vec<Option<PathBuf>>,
    path: PathBuf,
    partuuid: Option<String>,
    uuid: Option<String>,
    children: Option<Vec<PartitionRaw>>,
}

impl PartitionRaw {
    /// Derive a [`FileSystem`] from the `fs_`-fields.
    fn extract_fs(self) -> FileSystem {
        FileSystem {
            size: self.fs_size,
            available: self.fs_available,
            used: self.fs_used,
            kind: self.fs_kind.clone(),
            version: self.fs_version,
            read_only: self.read_only,
            path: self.path,
            mountpoints: self.mountpoints.iter().filter_map(|p| p.clone()).collect(),
            children: {
                if let Some(c) = self.children {
                    let mut v = vec![];
                    for ele in c {
                        v.push(ele.extract_fs())
                    }
                    v
                } else {
                    vec![]
                }
            },
            name: self.name,
            can_check: if let Some(fs) = self.fs_kind.clone() {
                FileSystem::determine_can_check(fs).unwrap_or(false)
            } else {
                false
            },
            can_repair: if let Some(fs) = self.fs_kind {
                FileSystem::determine_can_repair(fs).unwrap_or(false)
            } else {
                false
            },
            uuid: self.uuid.expect("A filesystem should have a UUID."),
        }
    }
}

#[derive(Clone, Debug)]
pub struct FileSystem {
    size: Option<u64>,
    available: Option<u64>,
    used: Option<u64>,
    kind: Option<String>,
    version: Option<String>,
    read_only: bool,
    path: PathBuf,
    children: Vec<FileSystem>,
    mountpoints: Vec<PathBuf>,
    name: String,
    can_check: bool,
    can_repair: bool,
    uuid: String,
}

impl FileSystem {
    pub fn total_size(&self) -> Option<u64> {
        self.size
    }

    pub fn available_size(&self) -> Option<u64> {
        self.available
    }

    pub fn used_size(&self) -> Option<u64> {
        self.used
    }

    pub fn kind(&self) -> Option<String> {
        self.kind.clone()
    }

    pub fn version(&self) -> Option<String> {
        self.version.clone()
    }

    pub fn read_only(&self) -> bool {
        self.read_only
    }

    pub fn path(&self) -> PathBuf {
        self.path.clone()
    }

    pub fn children(&self) -> Vec<FileSystem> {
        self.children.clone()
    }

    pub fn mountpoints(&self) -> Vec<PathBuf> {
        self.mountpoints.clone()
    }

    pub fn name(&self) -> String {
        self.name.clone()
    }

    pub fn uuid(&self) -> String {
        self.uuid.clone()
    }

    pub fn mounted(&self) -> bool {
        !self.mountpoints().is_empty()
    }

    /// Attempts to mount a filesystem using `udisks2`, usually to the `/run/media/` or `/media` directory.
    /// The command may return an error if the Partition is not understood as such by `udisks2`, the `udisks2`
    /// daemon not being available or the name of the partition seeming insecure to pass to D-Bus.
    ///
    /// Uses [`mount`] under the hood.
    pub fn mount(&self) -> Result<PathBuf, Box<dyn std::error::Error>> {
        mount(self.name.clone())
    }

    /// Attempts to unmount a filesystem using `udisks2`.
    /// The command may return an error if the Partition is not understood as such by `udisks2`, the `udisks2`
    /// daemon not being available or the name of the partition seeming insecure to pass to D-Bus
    /// or the partition has not been mounted.
    ///
    /// Uses [`unmount`] under the hood.
    pub fn unmount(&self) -> Result<(), Box<dyn std::error::Error>> {
        if !self.mounted() {
            return Err(
                "The drive has not yet been mounted anywhere, no reason to unmount.".into(),
            );
        }
        unmount(self.name.clone())
    }

    /// Uses `udisks2` to check if `udisks2` supports checking this file-system for consistency.
    ///
    /// The function will return an error if
    /// - `udisks2` does not support the file-system
    /// - communicating with `udisks2` failed
    fn determine_can_check(fs: String) -> Result<bool, Box<dyn std::error::Error>> {
        let conn = Connection::new_system()?;
        let m = Message::new_method_call(
            "org.freedesktop.UDisks2",
            "/org/freedesktop/UDisks2/Manager",
            "org.freedesktop.UDisks2.Manager",
            "CanCheck",
        )
        .map_err(|err| Error::DbusCommunicationFailed(err.into()))?
        .append1(fs);

        match conn.send_with_reply_and_block(m, Duration::from_secs(2)) {
            Ok(res) => Ok(res.get1::<(bool, String)>().map(|v| v.0).unwrap_or(false)),
            Err(_) => Ok(false),
        }
    }

    /// Whether `udisks2` is able to check the file-system for consistency.
    pub fn can_check(&self) -> bool {
        self.can_check
    }

    /// Uses `udisks2` to check if `udisks2` supports repairing this file-system.
    ///
    /// The function will return an error if
    /// - `udisks2` does not support the file-system
    /// - communicating with `udisks2` failed
    fn determine_can_repair(fs: String) -> Result<bool, Box<dyn std::error::Error>> {
        let conn = Connection::new_system()?;
        let m = Message::new_method_call(
            "org.freedesktop.UDisks2",
            "/org/freedesktop/UDisks2/Manager",
            "org.freedesktop.UDisks2.Manager",
            "CanRepair",
        )
        .map_err(|err| Error::DbusCommunicationFailed(err.into()))?
        .append1(fs);

        match conn.send_with_reply_and_block(m, Duration::from_secs(2)) {
            Ok(res) => Ok(res.get1::<(bool, String)>().map(|v| v.0).unwrap_or(false)),
            Err(_) => Ok(false),
        }
    }

    /// Whether `udisks2` is able to repair the file-system.
    pub fn can_repair(&self) -> bool {
        self.can_repair
    }

    /// Use `udisks2` to check if the file-system is consistent.
    /// `false` - inconsistent, and thus a damaged file-system
    /// `true` - consistent, and thus a healthy file-system
    ///
    /// The function will return an error if
    /// - `udisks2` does not support the file-system,
    /// - the partition is already mounted, or
    /// - communicating with `udisks2` failed
    ///
    /// Uses [`check`] under the hood.
    ///
    /// The operation avoids modifications and repairs to the partition.
    pub fn check(&self) -> Result<bool, Box<dyn std::error::Error>> {
        if self.mounted() {
            return Err("A mounted drive can not be checked.".into());
        }
        check(self.name.clone())
    }

    /// Use `udisks2` to repair a corrupted/damaged partition.
    /// `true` - Was able to repair the partition.
    /// `false` - Was unable to repair the partition.
    ///
    /// The function will return an error if
    /// - `udisks2` does not support the file-system,
    /// - the partition is already mounted, or
    /// - communicating with `udisks2` failed
    ///
    /// Uses [`repair`] under the hood.
    ///
    /// The operation avoids modifications and repairs to the partition.
    pub fn repair(&self) -> Result<bool, Box<dyn std::error::Error>> {
        if self.mounted() {
            return Err("A mounted drive can not be checked.".into());
        }
        repair(self.name.clone())
    }
}

#[derive(Clone, Debug)]
pub struct Partition {
    size: u64,
    name: String,
    path: PathBuf,
    label: Option<String>,
    uuid: String,
    fs: Option<FileSystem>,
}

impl Partition {
    pub fn name(&self) -> String {
        self.name.clone()
    }

    pub fn path(self) -> PathBuf {
        self.path
    }

    pub fn label(&self) -> Option<String> {
        self.label.clone()
    }

    pub fn size(&self) -> u64 {
        self.size
    }

    pub fn fs(self) -> Option<FileSystem> {
        self.fs
    }

    pub fn uuid(self) -> String {
        self.uuid
    }
}

/// A physical block storage device connected to the system.
#[derive(Deserialize, Debug, Clone)]
pub struct Drive {
    model: Option<String>,
    vendor: Option<String>,
    #[serde(rename(deserialize = "rev"))]
    revision: Option<String>,
    path: PathBuf,
    name: String,
    label: Option<String>,
    udisks_id: Option<String>,
    size: u64,
    #[serde(rename(deserialize = "children"))]
    partitions_raw: Option<Vec<PartitionRaw>>,
    #[serde(skip)]
    partitions: Vec<Partition>,
    #[serde(skip)]
    fs: Option<FileSystem>,
    #[serde(rename(deserialize = "fssize"))]
    fs_size: Option<u64>,
    #[serde(rename(deserialize = "fstype"))]
    fs_kind: Option<String>,
    #[serde(rename(deserialize = "fsver"))]
    fs_version: Option<String>,
    #[serde(rename(deserialize = "fsavail"))]
    fs_available: Option<u64>,
    #[serde(rename(deserialize = "fsused"))]
    fs_used: Option<u64>,
    #[serde(rename(deserialize = "ro"))]
    read_only: bool,
    #[serde(rename(deserialize = "rota"))]
    rotational: bool,
    serial: Option<String>,
    mountpoints: Vec<Option<String>>,
    #[serde(skip)]
    time_detected: Option<DateTime<Utc>>,
    #[serde(skip)]
    bus_type: Option<String>,
    #[serde(skip)]
    hint_ignore: Option<bool>,
    #[serde(skip)]
    hint_system: Option<bool>,
    #[serde(skip)]
    can_power_off: Option<bool>,
    #[serde(skip)]
    ejectable: Option<bool>,
    #[serde(skip)]
    media_available: Option<bool>,
    #[serde(skip)]
    removable: Option<bool>,
    uuid: Option<String>,
    r#type: String,
}

impl Drive {
    /// Uses `udisks2` to determine the time the drive represented by this struct was detected.
    /// 
    /// The function will return an error if
    /// - communicating with `udisks2` failed
    fn determine_time_detected(&self) -> Result<DateTime<Utc>, Box<dyn std::error::Error>> {
        let node = self.path.clone();
        let name = node
            .file_name()
            .expect("No device node sdX path ending.")
            .to_string_lossy();

        let conn = Connection::new_system()?;
        let proxy_block = conn.with_proxy(
            "org.freedesktop.UDisks2",
            format!("/org/freedesktop/UDisks2/block_devices/{name}"),
            Duration::from_secs(2),
        );
        let drive_object = proxy_block
            .get::<Path>("org.freedesktop.UDisks2.Block", "Drive")?
            .to_string();

        let proxy_drive = conn.with_proxy(
            "org.freedesktop.UDisks2",
            drive_object,
            Duration::from_secs(2),
        );
        let time = proxy_drive.get::<u64>("org.freedesktop.UDisks2.Drive", "TimeDetected")?;

        Ok(DateTime::from_timestamp_micros(time as i64).unwrap())
    }

    /// Uses `udisks2` to get additional information about the drive represented by this struct.
    /// 
    /// The function will return an error if
    /// - communicating with `udisks2` failed
    fn determine_udisks2_exclusive_blockdevice_data(
        &self,
    ) -> Result<Udisks2Exclusive, Box<dyn std::error::Error>> {
        let node = self.path.clone();
        let name = node
            .file_name()
            .expect("No device node sdX path ending.")
            .to_string_lossy();

        let conn = Connection::new_system()?;
        let proxy_block = conn.with_proxy(
            "org.freedesktop.UDisks2",
            format!("/org/freedesktop/UDisks2/block_devices/{name}"),
            Duration::from_secs(2),
        );

        let drive_object = proxy_block
            .get::<Path>("org.freedesktop.UDisks2.Block", "Drive")?
            .to_string();

        let proxy_drive = conn.with_proxy(
            "org.freedesktop.UDisks2",
            drive_object,
            Duration::from_secs(2),
        );

        let bus_type =
            proxy_drive.get::<String>("org.freedesktop.UDisks2.Drive", "ConnectionBus")?;
        let can_power_off =
            proxy_drive.get::<bool>("org.freedesktop.UDisks2.Drive", "CanPowerOff")?;
        let ejectable = proxy_drive.get::<bool>("org.freedesktop.UDisks2.Drive", "Ejectable")?;
        let removable = proxy_drive.get::<bool>("org.freedesktop.UDisks2.Drive", "Removable")?;
        let udisks_id = proxy_drive.get::<String>("org.freedesktop.UDisks2.Drive", "Id")?;
        let media_available =
            proxy_drive.get::<bool>("org.freedesktop.UDisks2.Drive", "MediaAvailable")?;
        let hint_ignore = proxy_block.get::<bool>("org.freedesktop.UDisks2.Block", "HintIgnore")?;
        let hint_system = proxy_block.get::<bool>("org.freedesktop.UDisks2.Block", "HintSystem")?;

        Ok(Udisks2Exclusive {
            bus_type,
            hint_ignore,
            hint_system,
            can_power_off,
            media_available,
            ejectable,
            removable,
            udisks_id,
        })
    }

    /// Verifies if the physical device which is represented by this function is still accessible to
    /// the system and has not changed position.
    /// This is to prevent miscommunication and invalid states. Call this function in a guard clause
    /// before changing the data or state of the device.
    ///
    /// ```rust
    /// fn foo(&self) -> Result<_, Error> {
    ///     if !is_same(&self) {
    ///         return Err(Error::DeviceChanged);
    ///     }
    ///
    ///     delete_all_data(&self);
    ///     // ^ Examplaray function that should only be executed on the drive
    ///     //   the user selected.
    /// }
    /// ```
    fn is_same(&self) -> bool {
        // In order to prevent infinite recursion, the method must not call any methods that use
        // is_same() them self!

        if let Ok(current_td) = self.determine_time_detected()
            && let Some(old_td) = self.time_detected
        {
            current_td == old_td
        } else {
            false
        }
    }

    /// Derive a [`FileSystem`] from the `fs_`-fields.
    fn extract_fs(self) -> FileSystem {
        FileSystem {
            size: self.fs_size,
            available: self.fs_available,
            used: self.fs_used,
            kind: self.fs_kind.clone(),
            version: self.fs_version,
            read_only: self.read_only,
            path: self.path,
            mountpoints: self
                .mountpoints
                .into_iter()
                .filter(|p| p.is_some())
                .map(|p| {
                    PathBuf::from(p.unwrap())
                        .canonicalize()
                        .expect("LSBLK provided invalid mountpoints.")
                })
                .collect(),
            children: vec![],
            name: self.name,
            can_check: if let Some(fs) = self.fs_kind.clone() {
                FileSystem::determine_can_check(fs).unwrap_or(false)
            } else {
                false
            },
            can_repair: if let Some(fs) = self.fs_kind {
                FileSystem::determine_can_repair(fs).unwrap_or(false)
            } else {
                false
            },
            uuid: self.uuid.expect("A filesystem should have a UUID."),
        }
    }

    /// Gets the filesystem of the device, in case the device has not been partitioned and a filesystem is present.
    pub fn fs(&self) -> Result<Option<FileSystem>, Error> {
        if !self.is_same() {
            return Err(DeviceChanged);
        }

        Ok(self.fs.clone())
    }

    /// Gets all filesystems that can be detected on the device.
    pub fn get_all_fs(&self) -> Result<Vec<FileSystem>, Error> {
        if !self.is_same() {
            return Err(DeviceChanged);
        }

        let mut fs_list = vec![];

        if let Some(fs) = self.fs()? {
            fs_list.push(fs);
        }

        for partition in self.partitions()? {
            if let Some(fs) = partition.fs() {
                fs_list.push(fs)
            }
        }

        Ok(fs_list)
    }

    /// Attempts to get a filesystem on a specific drive by the filesystems path.
    pub fn get_fs_by_path(&self, path: PathBuf) -> Result<Option<FileSystem>, Error> {
        if !self.is_same() {
            return Err(DeviceChanged);
        }

        let all_fs = self.get_all_fs()?;
        Ok(all_fs.iter().find(|fs| fs.path() == path).cloned())
    }

    /// Check whether the device contains any mounted file-systems.
    pub fn has_mounted_fs(&self) -> Result<bool, Error> {
        if !self.is_same() {
            return Err(DeviceChanged);
        }

        Ok(self.get_all_fs()?.iter().any(|fs| fs.mounted()))
    }

    /// Gets the name of the model of the drive.
    pub fn model(&self) -> Result<Option<String>, Error> {
        if !self.is_same() {
            return Err(DeviceChanged);
        }

        Ok(self.model.clone())
    }

    /// Gets the serial identifier of the drive.
    pub fn serial(&self) -> Result<Option<String>, Error> {
        if !self.is_same() {
            return Err(DeviceChanged);
        }

        Ok(self.serial.clone())
    }

    /// Gets vendor name of the drive.
    pub fn vendor(&self) -> Result<Option<String>, Error> {
        if !self.is_same() {
            return Err(DeviceChanged);
        }

        Ok(self.vendor.clone())
    }

    /// Get the device revision.
    pub fn revision(&self) -> Result<Option<String>, Error> {
        if !self.is_same() {
            return Err(DeviceChanged);
        }

        Ok(self.revision.clone())
    }

    /// Get whether the drive is removable or not.
    pub fn removable(&self) -> Result<bool, Error> {
        if !self.is_same() {
            return Err(DeviceChanged);
        }

        self.removable.ok_or(Error::InvalidData)
    }

    /// Get the id assigned to the drive by `udisks`.
    pub fn udisks_id(&self) -> Result<String, Error> {
        if !self.is_same() {
            return Err(DeviceChanged);
        }

        self.udisks_id.clone().ok_or(Error::InvalidData)
    }

    /// Gets the system device-node of the drive.
    pub fn device_node(&self) -> Result<PathBuf, Error> {
        if !self.is_same() {
            return Err(DeviceChanged);
        }

        Ok(self.path.clone())
    }

    /// Gets the label of the drive.
    pub fn label(&self) -> Result<Option<String>, Error> {
        if !self.is_same() {
            return Err(DeviceChanged);
        }

        Ok(self.label.clone())
    }

    /// Gets the systematic name of the drive.
    pub fn name(&self) -> Result<String, Error> {
        if !self.is_same() {
            return Err(DeviceChanged);
        }

        Ok(self.name.clone())
    }

    /// Gets the total size of the drive in bytes.
    /// This is not related to the size of file-systems on the device.
    pub fn total_size(&self) -> Result<u64, Error> {
        if !self.is_same() {
            return Err(DeviceChanged);
        }

        Ok(self.size)
    }

    /// Gets all partitions on the device.
    pub fn partitions(&self) -> Result<Vec<Partition>, Error> {
        if !self.is_same() {
            return Err(DeviceChanged);
        }

        Ok(self.partitions.clone())
    }

    /// Determines whether the device is deemed a rotational device according to the kernel.
    /// This may be wrong for devices not fully supporting required protocols (which is common).
    pub fn rotational(&self) -> Result<bool, Error> {
        if !self.is_same() {
            return Err(DeviceChanged);
        }

        Ok(self.rotational)
    }

    /// Determines whether the device is read only or not.
    pub fn read_only(&self) -> Result<bool, Error> {
        if !self.is_same() {
            return Err(DeviceChanged);
        }

        Ok(self.read_only)
    }

    /// Gets the time the represented device was detected by `udisks2`
    pub fn time_detected(&self) -> Result<DateTime<Utc>, Error> {
        if !self.is_same() {
            return Err(DeviceChanged);
        }

        self.time_detected.ok_or(Error::InvalidData)
    }

    /// Gets the bus type used by the represented device. i.e. `usb`
    pub fn bus_type(&self) -> Result<String, Error> {
        if !self.is_same() {
            return Err(DeviceChanged);
        }

        self.bus_type.clone().ok_or(Error::InvalidData)
    }

    /// Gets whether `udisks2` recommends ignoring the device.
    pub fn hint_ignore(&self) -> Result<bool, Error> {
        if !self.is_same() {
            return Err(DeviceChanged);
        }

        self.hint_ignore.ok_or(Error::InvalidData)
    }

    /// Gets whether `udisks2` assumes the device to be a system internal device, for which special
    /// permissions may be required.
    pub fn hint_system(&self) -> Result<bool, Error> {
        if !self.is_same() {
            return Err(DeviceChanged);
        }

        self.hint_system.ok_or(Error::InvalidData)
    }

    /// Gets whether the device can be powered off.
    pub fn can_power_off(&self) -> Result<bool, Error> {
        if !self.is_same() {
            return Err(DeviceChanged);
        }

        self.can_power_off.ok_or(Error::InvalidData)
    }

    /// Gets whether the device can be ejected.
    pub fn ejectable(&self) -> Result<bool, Error> {
        if !self.is_same() {
            return Err(DeviceChanged);
        }

        self.ejectable.ok_or(Error::InvalidData)
    }

    /// Gets whether the media of the drive is available.
    pub fn media_available(&self) -> Result<bool, Error> {
        if !self.is_same() {
            return Err(DeviceChanged);
        }

        self.can_power_off.ok_or(Error::InvalidData)
    }

    /// Attempts to create a new drive struct with updated information.
    pub fn get_updated(&self) -> Result<Drive, Error> {
        if !self.is_same() {
            return Err(DeviceChanged);
        }

        let d = Drives::get(self.device_node()?)?;

        if d.time_detected != self.time_detected {
            return Err(DeviceChanged);
        }

        Ok(d)
    }

    /// Powers of the physical disk, making it safe to remove and loose electrical power.
    ///
    /// <https://storaged.org/doc/udisks2-api/latest/gdbus-org.freedesktop.UDisks2.Drive.html#gdbus-method-org-freedesktop-UDisks2-Drive.PowerOff>
    pub fn power_off(&self) -> Result<(), Error> {
        if !self.is_same() {
            return Err(DeviceChanged);
        }

        power_off(self.clone())
    }

    /// Ejects the physical device. This may result in a DVD being moved outside the drive or
    /// similar.
    ///
    /// <https://storaged.org/doc/udisks2-api/latest/gdbus-org.freedesktop.UDisks2.Drive.html#gdbus-method-org-freedesktop-UDisks2-Drive.Eject>
    pub fn eject(&self) -> Result<(), Error> {
        if !self.is_same() {
            return Err(DeviceChanged);
        }

        eject(self.clone())
    }

    /// Initializes a [bm::Builder] to configure a performance benchmark via a builder pattern.
    /// This method itself does not start any operations. The pattern will need to be finalized
    /// using `execute()` for actions to take place.
    pub fn benchmark(&self) -> Result<bm::Builder, Error> {
        if !self.is_same() {
            return Err(DeviceChanged);
        }

        Ok(bm::Builder::new_for(self.clone()))
    }

    /// Gets a file descriptor to the file representing the block device.
    ///
    /// * `portal` - A live [`AuthenticationPortal`]
    /// * `password` - The password to authenticate at this portal
    /// * `flags` - Flags for the file descriptor to be opened with. (O_RDWR, O_RDONLY, O_WRONLY are forbidden)
    /// * `mode` - Mode with which to open the file (read, write, read-write)
    pub fn get_fd(
        &self,
        portal: Arc<Mutex<AuthenticationPortal>>,
        password: String,
        flags: i32,
        mode: FdMode,
    ) -> Result<std::fs::File, Error> {
        if !self.is_same() {
            return Err(DeviceChanged);
        }

        if !self.mountpoints.is_empty() {
            return Err(Error::NoExclusiveAccess);
        }

        let name = self.name()?;

        match portal
            .lock()
            .unwrap()
            .provide_password("org.freedesktop.udisks2.open-device".to_string(), password)
        {
            Ok(_) | Err(RequestError::AlreadyAuthorised) => {}
            Err(e) => return Err(Error::PolicyKitError(e)),
        }

        std::thread::sleep(Duration::from_millis(100));

        let mut options: HashMap<String, Variant<Box<dyn RefArg>>> = HashMap::new();

        options.insert(
            "auth.no_user_interaction".to_string(),
            Variant(Box::new("true".to_string())),
        );

        options.insert("flags".to_string(), Variant(Box::new(flags)));

        let conn = dbus::blocking::Connection::new_system()
            .map_err(|err| Error::DbusConnectionFailed(err.into()))?;
        let m = Message::new_method_call(
            "org.freedesktop.UDisks2",
            format!("/org/freedesktop/UDisks2/block_devices/{name}"),
            "org.freedesktop.UDisks2.Block",
            "OpenDevice",
        )
        .map_err(|err| Error::DbusCommunicationFailed(err.into()))?
        .append2(
            match mode {
                FdMode::Read => "r",
                FdMode::Write => "w",
                FdMode::ReadWrite => "rw",
            },
            options,
        );
        let r = conn
            .send_with_reply_and_block(m, Duration::from_secs(10))
            .map_err(|err| Error::DbusCommunicationFailed(err.into()))?;

        match r.get1::<std::fs::File>() {
            Some(fd) => Ok(fd),
            None => Err(Error::InvalidFd),
        }
    }
}

#[derive(Clone)]
pub struct Drives;

impl Drives {
    /// Get a `Vec<Drive>` of all currently connected physical disks.
    /// This will return an error if `lsblk` could not be correctly invoked.
    pub fn current() -> Result<Rc<[Drive]>, Error> {
        let mut c = Command::new("lsblk");
        // See `lsblk --list-columns` for description
        c.args(["--bytes", "--json", "-o", LSBLK_FIELDS]);

        if let Ok(output) = c.output() {
            generate_drive_structs(&String::from_utf8_lossy(&output.stdout))
        } else {
            Err(Error::CommandError)
        }
    }

    /// Get a list of all device nodes known to `lsblk`.
    fn nodes_list() -> Result<Vec<PathBuf>, Error> {
        let mut c = Command::new("lsblk");
        c.args(["-o", "PATH", "--noheadings"]);
        match c.output() {
            Ok(output) => {
                let s = String::from_utf8_lossy(&output.stdout);
                Ok(s.lines()
                    .filter(|v| !v.is_empty())
                    .map(PathBuf::from)
                    .collect())
            }
            Err(err) => Err(Error::ParseError(err.to_string())),
        }
    }

    /// Given the device node, retrieve information about one specific disk.
    pub fn get(device_node: PathBuf) -> Result<Drive, Error> {
        // Verify if the device_node even exists
        if !device_node.exists() {
            return Err(Error::NoSuchDrive);
        }

        // Canonicalize and stringify strings
        let path_can = device_node.canonicalize().map_err(|_| Error::NoSuchDrive)?;
        let path_stringified = path_can.to_string_lossy();

        // Make sure only letters, numbers and '/' are included in the path
        if path_stringified
            .chars()
            .any(|c| !c.is_ascii_alphanumeric() && c != '/')
        {
            return Err(Error::NoSuchDrive);
        }

        // Check if the node is in the node list of `lsblk`.
        if !Self::nodes_list()?.contains(&path_can) {
            return Err(Error::NoSuchDrive);
        }

        let mut c = Command::new("lsblk");
        c.args(["--bytes", "--json", "-o", LSBLK_FIELDS, &path_stringified]);

        if let Ok(output) = c.output() {
            if let Some(fst) =
                generate_drive_structs(&String::from_utf8_lossy(&output.stdout))?
                    .first()
            {
                Ok(fst.clone())
            } else {
                Err(Error::NoSuchDrive)
            }
        } else {
            if let Some(code) = c.status().map_err(|_| Error::CommandError)?.code()
                && code == 32
            {
                return Err(Error::NoSuchDrive);
            }

            log::warn!("Failed to spawn lsblk command.");
            Err(Error::CommandError)
        }
    }
}

/// Data that is only available to `udisks2`.
struct Udisks2Exclusive {
    bus_type: String,
    hint_ignore: bool,
    hint_system: bool,
    can_power_off: bool,
    ejectable: bool,
    removable: bool,
    udisks_id: String,
    media_available: bool,
}

/// Used for deserializing the JSON output of `lsblk` when called with [`LSBLK_FIELDS`].
#[derive(Deserialize)]
struct LsblkOutputExhaustive {
    blockdevices: Vec<Drive>,
}

/// Parses the JSON outputs of `lsblk` and already removes any virtual devices, as they are unimportant.
fn generate_drive_structs(json: &str) -> Result<Rc<[Drive]>, Error> {
    match serde_json::from_str::<LsblkOutputExhaustive>(json) {
        Ok(o) => Ok(o
            .blockdevices
            .iter()
            // Keep only physical devices (no [SWAP] and loop devices))
            .filter(|v| !v.mountpoints.contains(&Some("[SWAP]".to_string())) && v.r#type != "loop")
            .map(|bd| {
                let mut bd_c = bd.clone();
                bd_c.time_detected = bd.determine_time_detected().ok();
                let udisks2_exclusive_attempt = bd.determine_udisks2_exclusive_blockdevice_data();
                match udisks2_exclusive_attempt {
                    Ok(data) => {
                        bd_c.bus_type = Some(data.bus_type.clone());
                        bd_c.hint_ignore = Some(data.hint_ignore);
                        bd_c.hint_system = Some(data.hint_system);
                        bd_c.can_power_off = Some(data.can_power_off);
                        bd_c.ejectable = Some(data.ejectable);
                        bd_c.removable = Some(data.removable);
                        bd_c.media_available = Some(data.media_available);
                        bd_c.udisks_id = Some(data.udisks_id);
                    }
                    Err(err) => {
                        log::error!("Failed to get udisks2 exclusive data with error: {err}");
                    }
                }
                let mut partitions_ser: Vec<Partition> = vec![];
                if let Some(parts) = &bd.partitions_raw {
                    for p in parts {
                        if let Some(ref partuuid) = p.partuuid {
                            partitions_ser.push(Partition {
                                size: p.size,
                                name: p.name.clone(),
                                path: p.path.clone(),
                                label: p.label.clone(),
                                uuid: partuuid.to_string(),
                                fs: if p.fs_kind.is_some() {
                                    Some(p.clone().extract_fs())
                                } else {
                                    None
                                },
                            });
                        } else {
                            log::warn!("Discarded {} from partitions as it has not been assigned a partuuid.", p.path.to_string_lossy());
                        }
                    }
                }
                bd_c.partitions = partitions_ser;
                if bd_c.fs_kind.is_some() {
                    bd_c.fs = Some(bd_c.clone().extract_fs());
                } else {
                    bd_c.fs = None
                };
                bd_c
            })
            .collect()),
        Err(err) => Err(Error::ParseError(err.to_string())),
    }
}

/// Creates a hashmap with the only option `auth.no_user_interaction` set to `true`.
fn default_options() -> HashMap<&'static str, Variant<&'static str>> {
    let v = Variant("true");
    let mut hm = HashMap::new();
    hm.insert("auth.no_user_interaction", v);
    hm
}

/// `udisks2` has a very particular way of naming physical drives under the `/org/freedesktop/UDisks2/drives/` path.
/// The function recreates that functionality in Rust.
///
/// The expected output is <VENDOR>_<MODEL>_<SERIAL> where no spaces or non-ASCII-alphanumeric
/// characters are included, but escaped into hex. The only exception to this is '_' which is used
/// to substitute ' ' and escape into hex.
/// Trailing spaces around any part of the name are removed.
///
/// If one of the fields mentioned above (vendor, model, serial) is empty, it is skipped in the
/// naming. If none of the fields contain a name, the name is replaced by "drive".
fn determine_udisk2_internal_drive_name(drive: Drive) -> Result<String, Error> {
    let vendor = drive.vendor()?;
    let model = drive.model()?;
    let serial = drive.serial()?;

    let mut name = String::new();

    if let Some(vendor_value) = vendor
        && !vendor_value.is_empty()
    {
        name.push_str(&vendor_value.trim().replace(" ", "_").replace("-", "_"));
        name.push('_')
    }

    if let Some(model_value) = model
        && !model_value.is_empty()
    {
        name.push_str(&model_value.trim().replace(" ", "_").replace("-", "_"));
        name.push('_')
    }

    if let Some(serial_value) = serial
        && !serial_value.is_empty()
    {
        name.push_str(&serial_value.trim().replace(" ", "_").replace("-", "_"));
        // No `name.push('_')` because there is no trailing _
    }

    if name.is_empty() {
        name.push_str("drive");
    }

    let mut clean_name = String::new();

    name.chars().for_each(|c| {
        if !c.is_ascii_alphanumeric() && c != '_' {
            let escape_seq = format!("_{:02x}", c as i32);
            clean_name.push_str(&escape_seq);
        } else {
            clean_name.push(c)
        }
    });

    Ok(clean_name)
}

/// Mount a partition with a filesystem to the /run/media/ directory using `udisks2`.
/// This does not require root permissions.
///
/// If the specified partition name does not exists, the operation will fail.
/// If communicating with `udisks2` over D-Bus takes longer than five seconds, the operation will
/// fail.
///
/// <https://storaged.org/doc/udisks2-api/latest/gdbus-org.freedesktop.UDisks2.Filesystem.html#gdbus-method-org-freedesktop-UDisks2-Filesystem.Mount>
fn mount(name: String) -> Result<PathBuf, Box<dyn std::error::Error>> {
    if name.chars().any(|e| !e.is_ascii_alphanumeric()) {
        return Err("The provided name may not be passed into D-Bus for security concerns.".into());
    }

    let conn = dbus::blocking::Connection::new_system()?; // `udisks2` requires a system connection

    let m = Message::new_method_call(
        "org.freedesktop.UDisks2",
        format!("/org/freedesktop/UDisks2/block_devices/{name}"), // https://storaged.org/doc/udisks2-api/latest/ref-dbus-block-devices.html
        "org.freedesktop.UDisks2.Filesystem",
        "Mount",
    )?
    .append1(default_options());
    let r = conn.send_with_reply_and_block(m, Duration::from_secs(15))?;

    Ok(PathBuf::from(
        r.get1::<String>().ok_or("No first return argument.")?,
    ))
}

/// Unmount a partition with a filesystem, previously mounted to the /run/media/ directory using `udisks2`.
/// This does not require root permissions.
///
/// If the specified partition name does not exists, the operation will fail.
/// If communicating with `udisks2` over D-Bus takes longer than five seconds, the operation will
/// fail.
///
/// <https://storaged.org/doc/udisks2-api/latest/gdbus-org.freedesktop.UDisks2.Filesystem.html#gdbus-method-org-freedesktop-UDisks2-Filesystem.Unmount>
fn unmount(name: String) -> Result<(), Box<dyn std::error::Error>> {
    if name.chars().any(|e| !e.is_ascii_alphanumeric()) {
        return Err("The provided name may not be passed into D-Bus for security concerns.".into());
    }

    let conn = dbus::blocking::Connection::new_system()?;

    let m = Message::new_method_call(
        "org.freedesktop.UDisks2",
        format!("/org/freedesktop/UDisks2/block_devices/{name}"),
        "org.freedesktop.UDisks2.Filesystem",
        "Unmount",
    )?
    .append1(default_options());
    conn.send_with_reply_and_block(m, Duration::from_secs(15))?;

    Ok(())
}

/// Check if the filesystem of the specified partition is consistent and thus not corrupted.
/// The check will only provide a boolean, no further drive information.
/// Checking avoids any modifications.
///
/// If the filesystem is already mounted or not supported by `udisks2`, the operation will fail.
///
/// <https://storaged.org/doc/udisks2-api/latest/gdbus-org.freedesktop.UDisks2.Filesystem.html#gdbus-method-org-freedesktop-UDisks2-Filesystem.Check>
fn check(name: String) -> Result<bool, Box<dyn std::error::Error>> {
    if name.chars().any(|e| !e.is_ascii_alphanumeric()) {
        return Err("The provided name may not be passed into D-Bus for security concerns.".into());
    }

    let conn = dbus::blocking::Connection::new_system()?;

    let m = Message::new_method_call(
        "org.freedesktop.UDisks2",
        format!("/org/freedesktop/UDisks2/block_devices/{name}"),
        "org.freedesktop.UDisks2.Filesystem",
        "Check",
    )?
    .append1(default_options());
    let r = conn.send_with_reply_and_block(m, Duration::from_mins(15))?;

    Ok(r.get1::<bool>().ok_or("No first return argument.")?)
}

/// Attempt to repair a filesystem of a specified partition.
/// If the repair succeeds, the function will return true, false if the repair failed.
/// This will modify the partition data and can cause damage.
///
/// If the filesystem is already mounted or not supported by `udisks2`, the operation will fail.
///
/// <https://storaged.org/doc/udisks2-api/latest/gdbus-org.freedesktop.UDisks2.Filesystem.html#gdbus-method-org-freedesktop-UDisks2-Filesystem.Repair>
fn repair(name: String) -> Result<bool, Box<dyn std::error::Error>> {
    if name.chars().any(|e| !e.is_ascii_alphanumeric()) {
        return Err("The provided name may not be passed into D-Bus for security concerns.".into());
    }

    let conn = dbus::blocking::Connection::new_system()?;

    let m = Message::new_method_call(
        "org.freedesktop.UDisks2",
        format!("/org/freedesktop/UDisks2/block_devices/{name}"),
        "org.freedesktop.UDisks2.Filesystem",
        "Repair",
    )?
    .append1(default_options());
    let r = conn.send_with_reply_and_block(m, Duration::from_mins(15))?;

    Ok(r.get1::<bool>().ok_or("No first return argument.")?)
}

/// Arranges for the device to be physically powered of and safe to remove.
///
/// <https://storaged.org/doc/udisks2-api/latest/gdbus-org.freedesktop.UDisks2.Drive.html#gdbus-method-org-freedesktop-UDisks2-Drive.PowerOff>
fn power_off(drive: Drive) -> Result<(), Error> {
    let name = determine_udisk2_internal_drive_name(drive)?;
    let conn = dbus::blocking::Connection::new_system()
        .map_err(|err| Error::DbusConnectionFailed(Box::new(err)))?;

    let m = Message::new_method_call(
        "org.freedesktop.UDisks2",
        format!("/org/freedesktop/UDisks2/drives/{name}"),
        "org.freedesktop.UDisks2.Drive",
        "PowerOff",
    )
    .map_err(|err| Error::DbusCommunicationFailed(err.into()))?
    .append1(default_options());
    conn.send_with_reply_and_block(m, Duration::from_secs(15))
        .map_err(|err| Error::DbusCommunicationFailed(err.into()))?;

    Ok(())
}

/// Arranges for the device to be physically ejected.
///
/// <https://storaged.org/doc/udisks2-api/latest/gdbus-org.freedesktop.UDisks2.Drive.html#gdbus-method-org-freedesktop-UDisks2-Drive.Eject>
fn eject(drive: Drive) -> Result<(), Error> {
    let name = determine_udisk2_internal_drive_name(drive)?;
    let conn = dbus::blocking::Connection::new_system()
        .map_err(|err| Error::DbusConnectionFailed(Box::new(err)))?;

    let m = Message::new_method_call(
        "org.freedesktop.UDisks2",
        format!("/org/freedesktop/UDisks2/drives/{name}"),
        "org.freedesktop.UDisks2.Drive",
        "Eject",
    )
    .map_err(|err| Error::DbusCommunicationFailed(err.into()))?
    .append1(default_options());
    conn.send_with_reply_and_block(m, Duration::from_secs(15))
        .map_err(|err| Error::DbusConnectionFailed(Box::new(err)))?;

    Ok(())
}

#[cfg(test)]
/// Many tests in this module require the presence of a storage block device under `/dev/sda`.
/// It is reasonable to use a modern SSD or flash storage device with at least one partition of a
/// popular filesystem.
/// Make sure the device has no actively mounted partitions before the tests.
mod tests {
    use super::*;

    #[test]
    /// Attempts to get the device at `/dev/sda` and read all the values.
    /// Checking if the values are `Option::None` or some is not reasonable, as some devices may
    /// actually not have such attributes.
    fn get_sda_and_read_attributes() {
        let device =
            Drives::get(PathBuf::from("/dev/sda")).expect("Failed to get device /dev/sda.");
        assert_eq!(device.device_node().unwrap(), PathBuf::from("/dev/sda"));
        assert!(device.total_size().unwrap() > 0);
        device.model().expect("Failed to get model");
        device.rotational().expect("Failed to get rotational");
        device.partitions().expect("Failed to get partitions");
        device.vendor().expect("Failed to get vendor");
    }

    #[test]
    fn get_time_detected() {
        Drives::get(PathBuf::from("/dev/sda"))
            .unwrap()
            .time_detected()
            .expect("No expected time could be gathered.");
    }
}
