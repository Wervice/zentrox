use nix::fcntl;
use std::alloc::{LayoutError, handle_alloc_error};
use std::env::temp_dir;
use std::fs;
use std::io::{Read, Seek, Write};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use std::{
    sync::mpsc::{self, Sender},
    time::Duration,
};
use thiserror::Error;
use uuid::Uuid;

use crate::drives::{Drive, Error as DriveError, FdMode};
use crate::polkit::AuthenticationPortal;

// DEFAULT_SAMPLE_SIZE is aligned for any device that uses a sector size that is a power of two
const DEFAULT_SAMPLE_SIZE: usize = 4 * 1024 * 1024;
const O_DIRECT: i32 = fcntl::OFlag::O_DIRECT.bits();
const O_SYNC: i32 = fcntl::OFlag::O_SYNC.bits();
const O_EXCL: i32 = fcntl::OFlag::O_EXCL.bits();

#[derive(Debug, Error)]
pub enum BenchmarkError {
    #[error("The process failed to get a file descriptor.")]
    CanNotOpen,
    #[error("The benchmark will not be attempted, as the device has mounted partitions.")]
    InUse,
    #[error("The specified device is not accessible.")]
    NoSuchDevice,
    #[error("The device does not provide the required space for testing.")]
    TooSmall,
    #[error("The device is too large.")]
    TooLarge,
    #[error("The program tried to access a region outside of the devices storage bounds.")]
    OutOfBounds,
    #[error("An I/O error occured, yielding: {0}")]
    IOError(std::io::Error),
    #[error("Data write back of offset {offset:?} failed. Data has been backed up at {recovery_path:?}.")]
    WriteBackFailed{ error: Box<BenchmarkError>, recovery_path: PathBuf, offset: usize },
    #[error("At least one read or write test has to be performed done.")]
    ToFewTests,
    #[error("A password is required.")]
    NoPassword,
    #[error("An AuthenticationPortal must be provided.")]
    NoAuthenticationPortal,
    #[error("The block size is not a multiple of 512 or too high or low.")]
    WrongSize,
    #[error("No less than 1 iteration can be performed per test.")]
    TooFewIterations,
    #[error("No more than 1024 iterations can be performed per test.")]
    TooManyIterations,
    #[error("Interacting with the physical drive or an abstraction layer failed.")]
    DriveError(DriveError),
}

impl From<DriveError> for BenchmarkError {
    fn from(value: DriveError) -> Self {
        BenchmarkError::DriveError(value)
    }
}

/// Unsafely creates a zeroed out aligned vector.
fn new_aligned_vector(size: usize, alignment: usize) -> Result<Vec<u8>, LayoutError> {
    let layout = std::alloc::Layout::from_size_align(size, alignment)?;
    let ptr = unsafe {
        let ptr = std::alloc::alloc_zeroed(layout);
        if ptr.is_null() {
            handle_alloc_error(layout)
        }
        ptr
    };
    Ok(unsafe { Vec::from_raw_parts(ptr, size, size) })
}

#[derive(Debug, Clone)]
/// The time measurement for a read or write operations together with the sample size.
pub struct Throughput {
    time_samples: Vec<Duration>,
    sample_size: usize,
}

impl Throughput {
    pub fn samples(&self) -> Vec<Duration> {
        self.time_samples.clone()
    }

    pub fn sample_size(&self) -> usize {
        self.sample_size
    }

    pub fn avg(&self) -> Duration {
        let samples = self.samples();
        let mut sum: Duration = Duration::from_secs(0);
        for s in &samples {
            sum += *s
        }
        sum / samples.len() as u32
    }

    pub fn bytes_over_time(&self) -> f64 {
        self.sample_size() as f64 / self.avg().as_secs_f64()
    }
}

/// Results of a benchmark, optionally containing results from a write benchmark.
pub struct BenchmarkResults {
    read: Throughput,
    write: Option<Throughput>,
}

impl BenchmarkResults {
    pub fn get_read(&self) -> Throughput {
        self.read.clone()
    }

    pub fn get_write(&self) -> Option<Throughput> {
        self.write.clone()
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Variant {
    Read,
    Write,
}

#[derive(Clone, Copy, Debug)]
/// A progress update to either a read or write benchmark.
pub struct Update {
    step: usize,
    out_of: usize,
    variant: Variant,
}

impl Update {
    /// Whether the update is for a read or a write measurement.
    pub fn variant(&self) -> Variant {
        self.variant
    }

    pub fn step(&self) -> usize {
        self.step
    }

    pub fn out_of(&self) -> usize {
        self.out_of
    }

    pub fn percentage(&self) -> f64 {
        100_f64 * (self.step() as f64 / self.out_of() as f64)
    }
}

#[derive(Clone, Debug)]
/// Builder pattern for configuring a benchmark.
pub struct Builder {
    drive: Drive,

    write: bool,

    random: bool,
    sample_size: usize,
    iterations: usize,

    update_channel: Option<mpsc::Sender<Update>>,
    portal: Option<Arc<Mutex<AuthenticationPortal>>>,
    password: Option<String>,
}

impl Builder {
    /// Create a new benchmark builder for a device.
    pub fn new_for(drive: Drive) -> Builder {
        Builder {
            drive,

            write: false,

            random: false,
            sample_size: DEFAULT_SAMPLE_SIZE,
            iterations: 64,

            update_channel: None,
            portal: None,
            password: None,
        }
    }

    /// Enable write benchmark. Not available for read-only devices. See [`throughput`] for more.
    pub fn do_write(&mut self, r: bool) -> Self {
        self.write = r;
        self.clone()
    }

    /// Enable random operations. See [`throughput`] for more.
    pub fn do_random(&mut self, r: bool) -> Self {
        self.random = r;
        self.clone()
    }

    /// Set sample / block size.
    /// Must be <= 1GiB.
    /// See [`throughput`] for more.
    pub fn sample_size(&mut self, s: usize) -> Self {
        self.sample_size = s;
        self.clone()
    }

    /// Set amount of iterations to perform.
    /// Must be in the interval [1; 1024].
    /// See [`throughput`] for more.
    pub fn iterations(&mut self, i: usize) -> Self {
        self.iterations = i;
        self.clone()
    }

    /// [`mpsc::Sender`] for progress updates.
    /// See [`throughput`] for more.
    pub fn updater(&mut self, u: Sender<Update>) -> Self {
        self.update_channel = Some(u);
        self.clone()
    }

    /// Portal for authorization.
    pub fn portal(&mut self, p: Arc<Mutex<AuthenticationPortal>>) -> Self {
        self.portal = Some(p);
        self.clone()
    }

    /// Password for authentication at [`AuthenticationPortal`].
    pub fn password(&mut self, s: String) -> Self {
        self.password = Some(s);
        self.clone()
    }

    /// Check if all parameters for a benchmark are appropriate and returns an error if not.
    ///
    /// 1. A password must be provided
    /// 2. A portal must be provided
    /// 3. The sample size must be a multiple of 512
    /// 4. The sample size must be less or equal 1GiB
    /// 5. The sample size must be larger than 0
    /// 6. The iterations must be more than 0
    /// 7. The iterations must be at most 1024
    /// 8. The device must exist
    pub fn verify_configuration(&self) -> Result<(), BenchmarkError> {
        if self.password.is_none() {
            return Err(BenchmarkError::NoPassword);
        }

        if self.portal.is_none() {
            return Err(BenchmarkError::NoAuthenticationPortal);
        }

        if !self.sample_size.is_multiple_of(512)
            || self.sample_size > (1024 * 1024 * 1024)
            || self.sample_size == 0
        {
            return Err(BenchmarkError::WrongSize);
        }

        if self.iterations < 1 {
            return Err(BenchmarkError::TooFewIterations);
        }

        if self.iterations > 1024 {
            return Err(BenchmarkError::TooManyIterations);
        }

        if !self.drive.device_node()?.exists() {
            return Err(BenchmarkError::NoSuchDevice);
        }

        Ok(())
    }

    /// Runs the configured benchmark, returning the results or an error.
    ///
    /// # Errors
    /// The function will error under various conditions which are specified in [`BenchmarkError`].
    pub fn execute(&self) -> Result<BenchmarkResults, BenchmarkError> {
        self.verify_configuration()?;

        if self.drive.partitions()?.iter().any(|x| {
            if let Some(fs) = x.clone().fs() {
                fs.mounted()
            } else {
                false
            }
        }) && self.write
        {
            return Err(BenchmarkError::InUse);
        }

        let mut fd = get_fd(
            self.drive.clone(),
            self.portal.clone().unwrap(),
            self.password.clone().unwrap(),
            self.write,
        )?;

        fd.lock().map_err(BenchmarkError::IOError)?;

        let tp = throughput(
            self.drive.clone(),
            self.iterations,
            self.sample_size,
            self.random,
            self.update_channel.clone(),
            self.write,
            &mut fd,
        )?;

        fd.unlock().map_err(BenchmarkError::IOError)?;

        Ok(BenchmarkResults {
            read: tp.0,
            write: tp.1,
        })
    }
}

/// Given a [`Drive`] this function will attempt to obtain a file-descriptor for the device file, if
/// necessary also in write mode. The function will return an error if the device is read-only and
/// write mode was requested.
fn get_fd(
    drive: Drive,
    portal: Arc<Mutex<AuthenticationPortal>>,
    password: String,
    needs_write: bool,
) -> Result<std::fs::File, BenchmarkError> {
    let use_read = drive.read_only()? || !needs_write;
    let maximum_mode = if use_read {
        FdMode::Read
    } else {
        FdMode::ReadWrite
    };

    let mut flags = O_DIRECT | O_SYNC;

    if !use_read {
        flags |= O_EXCL;
    }

    drive
        .get_fd(portal, password, flags, maximum_mode)
        .map_err(|_| BenchmarkError::CanNotOpen)
}

/// Calculates the last block for which there is sufficient space to perform a benchmark on.
/// A block must be as large as the block size and have at least one block space after it.
///
/// * `total_size` - The total size in bytes of the device file
/// * `block_size` - The size of a block in bytes
///
/// # Errors
/// In case the total size of the device is too low, an error will be returned. In the unlikely case
/// the amount of blocks would overflow the calculation, an error will be returned.
fn get_last_full_block_with_sufficient_space(
    total_size: usize,
    block_size: usize,
) -> Result<usize, BenchmarkError> {
    if block_size == 0 {
        return Err(BenchmarkError::TooSmall);
    }
    if total_size < block_size {
        return Err(BenchmarkError::TooSmall);
    }
    let blocks = total_size
        .checked_div(block_size)
        .ok_or(BenchmarkError::TooLarge)?;
    if blocks == 0 {
        return Err(BenchmarkError::TooSmall);
    }
    Ok(blocks - 1)
}

/// Starts a throughput benchmark on a device file, returning the
/// throughput values, where the write benchmark results will only be available if a write benchmark
/// has be performed. The benchmark supports both read-only and read-write benchmarks.
///
/// The benchmark supports random and sequential operations. The sample size / block size must be
/// specified. It must be bellow the devices total size. The amount of samples must be specified.
/// The benchmark measures the time it takes to read or write a block of data whilst the block size
/// is specified by the `sample_size`. Seeking operations are excluded from the measurement.
///
/// * `drive` - The [`Drive`] representing the device to perform this operation on.
/// * `iterations` - The amount of samples to measure.
/// * `sample_size` - The size in bytes of one sample / block.
/// * `random` - Whether to perform random operations or use sequential operations.
/// * `updates` - An optional [`mpsc::Sender`] for progress updates.
/// * `write` - Whether to perform a write benchmark as well.
/// * `fd` - A file descriptor for the device file of `drive`.
///
/// A write benchmark poses the risk of data loss, a backup of all relevant data on the target
/// device must be done before using this function. A write benchmark involves reading and writing a
/// block of data from the device file and back to the device file.
/// The function will attempt to store a block that is possibly affected by data loss in the
/// temporary directory of the host machine, in order to mitigate damage.
pub fn throughput(
    drive: Drive,
    iterations: usize,
    sample_size: usize,
    random: bool,
    updates: Option<Sender<Update>>,
    write: bool,
    fd: &mut std::fs::File,
) -> Result<(Throughput, Option<Throughput>), BenchmarkError> {
    let size = drive.total_size()? as usize;
    let name = drive.name()?;

    if write && drive.read_only()? {
        log::error!("Benchmark ({name}): A read-only device can not be opened for writing.");
        return Err(BenchmarkError::CanNotOpen);
    }

    let last_full_block_with_space = get_last_full_block_with_sufficient_space(size, sample_size)?;

    if last_full_block_with_space == 0 {
        // The device should have at least two blocks to seek to.
        // Other wise a % by 0 will arise when calculating the positions.
        return Err(BenchmarkError::TooSmall);
    }

    let mut block_positions: Vec<(usize, usize, usize)> = vec![];

    for i in 0..iterations {
        block_positions.push((
            i % last_full_block_with_space as usize,
            rand::random::<usize>() % last_full_block_with_space as usize,
            i,
        ));
    }

    let mut read_time_samples: Vec<Duration> = vec![];
    let mut write_time_samples: Vec<Duration> = vec![];

    log::info!(
        "Starting benchmark for device {name} with settings sample_size={sample_size}, random={random} and write={write}."
    );

    for (seq, rand, index) in block_positions {
        let offset = sample_size * if random { rand } else { seq };

        if offset + sample_size > size {
            return Err(BenchmarkError::OutOfBounds);
        }

        let read_op = read_section(fd, offset, sample_size, name.clone())?;
        read_time_samples.push(read_op.duration);

        if let Some(ref channel) = updates {
            let _ = channel.send(Update {
                variant: Variant::Read,
                step: index + 1,
                out_of: iterations,
            });
        }

        if write {
            let read_data = read_op.data;
            let read_data_len = read_data.len();

            log::debug!(
                "Benchmark (W) ({name}): Performing write test at {offset} wheras {} bytes are currently stored in memory.",
                read_data_len
            );

            log::debug!(
                "Benchmark (W) ({name}): {} bytes are being written back at {offset}.",
                read_data_len
            );

            match write_section(fd, offset, &read_data, name.clone()) {
                Ok(measurement) => {
                    write_time_samples.push(measurement.duration);

                    if let Some(ref channel) = updates {
                        let _ = channel.send(Update {
                            variant: Variant::Write,
                            step: index + 1,
                            out_of: iterations,
                        });
                    }
                }
                Err(err) => {
                    let uuid = Uuid::new_v4().to_string();
                    let rec_p = temp_dir().join(format!(
                        "zentrox-write-bm-failed-{offset}-{name}-{uuid}.bin"
                    ));
                    log::error!("Benchmark (W) ({name}): Failed to write back block at offset {offset}.");
                    fs::write(&rec_p, read_data).map_err(BenchmarkError::IOError)?;
                    log::error!(
                        "Benchmark (W) ({name}): The lost data of offset {} has been temporarily backed up at {}.",
                        offset,
                        rec_p.to_string_lossy()
                    );
                    return Err(BenchmarkError::WriteBackFailed {
                        error: Box::new(err),
                        recovery_path: rec_p,
                        offset,
                    });
                }
            }

            log::debug!(
                "Benchmark (W) ({name}): {} bytes have been written back at {offset}.",
                read_data.len()
            );
        }
    }

    Ok((
        Throughput {
            time_samples: read_time_samples,
            sample_size,
        },
        if write {
            Some(Throughput {
                time_samples: write_time_samples,
                sample_size,
            })
        } else {
            None
        },
    ))
}

/// A wrapper around the outputs of a [`read_section`] operation.
struct SectionReading {
    duration: Duration,
    data: Vec<u8>,
}

/// A wrapper around the outputs of a [`write_section`] operation.
struct SectionWriting {
    duration: Duration,
}

/// Reads a specified number of bytes at an offset, returning the bytes and the duration it took to
/// read them. The function will return an error [`BenchmarkError::IOError`] with
/// [`std::io::ErrorKind::UnexpectedEof`] if the requested bytes exceed the size of the file.
///
/// * `fd` - A file descriptor to a device file
/// * `offset` - The offset in bytes to which to seek to
/// * `size` - The amount of bytes to read
/// * `name` - The name of the device file to write to for logging purposes
fn read_section(
    fd: &mut std::fs::File,
    offset: usize,
    size: usize,
    name: String,
) -> Result<SectionReading, BenchmarkError> {
    fd.seek(std::io::SeekFrom::Start(offset as u64))
        .map_err(BenchmarkError::IOError)?; // Seeks to a random position.

    log::debug!("Benchmark (R) ({name}): Seeking to random position {offset}.");

    let mut buf: Vec<u8> = new_aligned_vector(size, 512).expect("Failed to get aligned vector.");

    let start = Instant::now();
    let read_op = fd.read_exact(&mut buf);
    let time_delta = start.elapsed();

    // Make sure the read operation succeeded and did not read too little.
    read_op.map_err(BenchmarkError::IOError)?;

    fd.rewind().map_err(BenchmarkError::IOError)?;

    Ok(SectionReading {
        duration: time_delta,
        data: buf,
    })
}

/// Writes a slice of bytes (`u8`) into file descriptor at an offset, by seeking and `write_all`,
/// whilst measuring the time it takes to write the data and sync. The operation does not
/// put an end to the file and can extend the file if necessary.
/// For further information see [`Write::write_all`].
///
/// * `fd` - A file descriptor to a device file
/// * `offset` - An offset in bytes at which to write the data
/// * `data` - A slice of bytes to write
/// * `name` - The name of the device file to write to for logging purposes
fn write_section(
    fd: &mut std::fs::File,
    offset: usize,
    data: &[u8],
    name: String,
) -> Result<SectionWriting, BenchmarkError> {
    fd.seek(std::io::SeekFrom::Start(offset as u64))
        .map_err(BenchmarkError::IOError)?;

    log::debug!("Benchmark (W) ({name}): Seeking to random position {offset}.");

    let start = Instant::now();
    let write_op = fd.write_all(data);
    let sync_op = fd.sync_all();
    let time_delta = start.elapsed();
    write_op.map_err(BenchmarkError::IOError)?;
    sync_op.map_err(BenchmarkError::IOError)?;

    fd.rewind().map_err(BenchmarkError::IOError)?;

    Ok(SectionWriting {
        duration: time_delta,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn correct_block_position_calculation() {
        assert!(get_last_full_block_with_sufficient_space(0, 4 * 1024 * 1024).is_err());
        assert_eq!(
            get_last_full_block_with_sufficient_space(4 * 1024 * 1024, 4 * 1024 * 1024).unwrap(),
            0
        );
        assert!(
            get_last_full_block_with_sufficient_space(2 * 1024 * 1024, 4 * 1024 * 1024).is_err()
        );
        assert_eq!(
            get_last_full_block_with_sufficient_space(6 * 1024 * 1024, 4 * 1024 * 1024).unwrap(),
            0
        );
        assert_eq!(
            get_last_full_block_with_sufficient_space(8 * 1024 * 1024, 4 * 1024 * 1024).unwrap(),
            1
        );
        assert_eq!(
            get_last_full_block_with_sufficient_space(20 * 1024 * 1024, 2 * 1024 * 1024).unwrap(),
            9
        );
        assert_eq!(
            get_last_full_block_with_sufficient_space(20 * 1024 * 1024 - 1, 2 * 1024 * 1024)
                .unwrap(),
            8
        );
        assert!(get_last_full_block_with_sufficient_space(20 * 1024 * 1024 - 1, 0).is_err());
        assert!(
            get_last_full_block_with_sufficient_space(20 * 1024 * 1024 - 1, 45 * 1024 * 1024)
                .is_err()
        );
        assert!(get_last_full_block_with_sufficient_space(0, 0).is_err());
        assert!(get_last_full_block_with_sufficient_space(usize::MAX, usize::MIN).is_err());
        assert!(get_last_full_block_with_sufficient_space(usize::MIN, usize::MAX).is_err());
        assert!(get_last_full_block_with_sufficient_space(1, usize::MAX).is_err());

        assert!(get_last_full_block_with_sufficient_space(20, 2).unwrap() * 2 < 20);
        assert!(get_last_full_block_with_sufficient_space(200, 20).unwrap() * 2 < 20);
        assert!(get_last_full_block_with_sufficient_space(4 * 1024 * 1024, 7).unwrap() == 599185);
        assert_eq!(
            get_last_full_block_with_sufficient_space(200, 20).unwrap() * 2,
            18
        );
    }
}
