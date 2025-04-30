//! # Disk Data Module
//!
//! This module provides functionality to retrieve disk data on Unix-based systems.

use libc::{c_void, close, open, read};
use log::error;
use serde::Serialize;
use serde_json::{json, Value};
use std::{
    error::Error,
    ffi::CString,
    fs::{read_dir, remove_file, File},
    io::{Read, Write},
    thread,
    time::{Duration, Instant},
};

use crate::utils::{read_file_content, write_json_to_file};

const PARTITIONS: &str = "/proc/partitions";
const TEST_FILE_SIZE: usize = 10_000_000;
const ITERATION: usize = 3;

const HEADER: &str = "DISK";
const LOGGER: &str = "log/disk_data.json";

/// Collection of collected disk data.
#[derive(Debug, Serialize)]
struct DiskInfo {
    disk_device: String,
    disk_size: Option<u64>,
    disk_model: Option<String>,
    disk_type: Option<String>,
    disk_partitions: Option<Vec<PartitionInfo>>,
    disk_write_bandwidth: Option<f64>,
    disk_read_bandwidth: Option<f64>,
    disk_smart_info: Option<SmartInfo>,
}

impl DiskInfo {
    /// Convert the `DiskInfo` into a JSON object.
    fn to_json(&self) -> Value {
        json!({
            "disk_device": self.disk_device,
            "disk_size_GB": self.disk_size,
            "disk_model": self.disk_model,
            "disk_type": self.disk_type,
            "disk_partitions": self.disk_partitions.as_ref().map(|parts| {
                parts.iter().map(|part| part.to_json()).collect::<Vec<_>>()
            }),
            "disk_smart_info": self.disk_smart_info.as_ref().map(|smart| smart.to_json()),
            "disk_write_bandwidth_MB.s": self.disk_write_bandwidth,
            "disk_read_bandwidth_MB.s": self.disk_read_bandwidth,
        })
    }
}

/// Collected partitions of a disk.
#[derive(Debug, Serialize)]
struct PartitionInfo {
    partition_name: String,
    partition_size: f64,
}

impl PartitionInfo {
    /// Convert the `PartitionInfo` into a JSON object.
    fn to_json(&self) -> Value {
        json!({
            "part_name": self.partition_name,
            "part_size_MB": self.partition_size
        })
    }
}

/// Collected more specific and detailed disk data.
#[derive(Debug, Serialize)]
struct SmartInfo {
    uptime_hours: Option<u8>,
    health: Option<String>,
    sectors_reallocated: Option<u8>,
    sectors_pending: Option<u8>,
    temperature: Option<u8>,
}

impl SmartInfo {
    /// Convert the `SmartInfo` into a JSON object.
    fn to_json(&self) -> Value {
        json!({
            "uptime_hours": self.uptime_hours,
            "health_status": self.health,
            "sectors_reallocated": self.sectors_reallocated,
            "sectors_pending": self.sectors_pending,
            "temperature_°C": self.temperature,
        })
    }
}

/// Function to test reading and writing disk performances.
///
/// # Arguments
///
/// - `path` : Disk device path in system to test.
///
/// # Returns
///
/// - Writing bandwidth average in MB/s.
/// - Reading bandwidth average in MB/s.
/// - Returning `None` for writing and reading bandwidth if error occurs.
fn get_disk_test(path: &str) -> Option<(f64, f64)> {
    let test_file: String = format!("/tmp/{}_test_file", path.replace('/', "_"));
    let mut buffer: Vec<u8> = Vec::with_capacity(TEST_FILE_SIZE);

    let mut total_write_bandwidth: f64 = 0.0;
    let mut total_read_bandwidth: f64 = 0.0;

    for _ in 0..ITERATION {
        let write_start: Instant = Instant::now();
        if let Ok(mut file) = File::create(&test_file) {
            buffer.resize(TEST_FILE_SIZE, 0u8);
            if file.write_all(&buffer).is_err() {
                error!("[{HEADER}] 'Error during the temporary file writing' : {test_file}");
                return None;
            }
        } else {
            error!("[{HEADER}] 'Error during creation of the temporary file' : {test_file}");
            return None;
        }
        let write_duration: Duration = write_start.elapsed();

        // Flush the cache according the operating system
        #[cfg(unix)]
        unsafe {
            libc::sync();
        }

        // Test de lecture
        let read_start: Instant = Instant::now();
        if let Ok(mut file) = File::open(&test_file) {
            let mut read_buffer: Vec<u8> = vec![0u8; TEST_FILE_SIZE];
            if file.read_exact(&mut read_buffer).is_err() {
                error!("[{HEADER}] 'Error during the temporary file reading' : {test_file}");
                return None;
            }
        } else {
            error!("[{HEADER}] 'Error while the temporary file opening' : {test_file}");
            return None;
        }
        let read_duration: Duration = read_start.elapsed();

        let write_bandwidth: f64 = (TEST_FILE_SIZE as f64 / 1e6) / write_duration.as_secs_f64();
        let read_bandwidth: f64 = (TEST_FILE_SIZE as f64 / 1e6) / read_duration.as_secs_f64();

        total_write_bandwidth += write_bandwidth;
        total_read_bandwidth += read_bandwidth;

        thread::sleep(Duration::from_millis(100));
    }

    if remove_file(&test_file).is_err() {
        error!("[{HEADER}] 'Error during the temporary file removing' : {test_file}");
    }

    let avg_write_bandwidth: f64 = total_write_bandwidth / ITERATION as f64;
    let avg_read_bandwidth: f64 = total_read_bandwidth / ITERATION as f64;

    Some((avg_write_bandwidth, avg_read_bandwidth))
}

/// Function that retrieves smart disk information.
///
/// # Arguments
///
/// - `path` : Disk device path in system.
///
/// # Returns
///
/// - [`SmartInfo`] structure
/// - Error message if CString can not be created, file descriptor content or final extracted data are null.
fn collect_smart_data(path: &str) -> Result<SmartInfo, String> {
    let device: CString = CString::new(path).map_err(|_| "Failed to create CString".to_string())?;
    let fd: i32 = unsafe { open(device.as_ptr(), 0) };

    if fd < 0 {
        error!("[{HEADER}] Data 'Failed to open device for smart information'");
        return Err("Failed to open device".to_string());
    }

    let mut buffer: [u8; 512] = [0u8; 512];
    let bytes: isize = unsafe { read(fd, buffer.as_mut_ptr() as *mut c_void, buffer.len()) };

    if bytes < 0 {
        unsafe { close(fd) };
        error!("[{HEADER}] Data 'Failed to retrieve disk smart information'");
        return Err("Failed to read SMART data".to_string());
    }

    let smart_info: SmartInfo = SmartInfo {
        uptime_hours: buffer.get(9).copied(),
        health: buffer.get(5).copied().map(|status: u8| {
            if status > 0 {
                format!("Worn (reallocated sectors : {status})")
            } else {
                "Unused".to_string()
            }
        }),
        sectors_reallocated: buffer.get(5).copied(),
        sectors_pending: buffer.get(196).copied(),
        temperature: buffer.get(194).copied(),
    };

    unsafe { close(fd) };

    Ok(smart_info)
}

/// Collect information about partitions.
fn collect_partitions(device: &str) -> Result<Vec<PartitionInfo>, String> {
    let mut partitions = Vec::new();

    if let Ok(entries) = read_dir(format!("/sys/block/{device}")) {
        for entry in entries.flatten() {
            if let Some(name) = entry.file_name().to_str() {
                if name.starts_with(device) {
                    if let Some(size_str) =
                        read_file_content(&format!("/sys/block/{device}/{name}/size"))
                    {
                        if let Ok(size_bytes) = size_str.trim().parse::<u64>() {
                            partitions.push(PartitionInfo {
                                partition_name: name.to_string(),
                                partition_size: (size_bytes * 512) as f64 / 1e6,
                            });
                        }
                    }
                }
            }
        }
    }

    Ok(partitions)
}

/// Function that retrieves all detailed disk information.
///
/// # Returns
///
/// The compilation of completed structures concerning all disk information.
/// * [`DiskInfo`] structure
/// * [`PartitionInfo`] structure
/// * [`SmartInfo`] structure
fn collect_disk_data() -> Result<Vec<DiskInfo>, String> {
    let disks = sysinfo::Disks::new_with_refreshed_list();
    for disk in disks.list() {
        println!("------> {}", disk.name().to_string_lossy());
        println!("Read: {}", disk.usage().total_read_bytes / 1_000_000);
        println!("Write: {}", disk.usage().total_written_bytes / 1_000_000);
        println!("Avail: {}", disk.available_space() / 1_000_000);
        println!("Space: {}", disk.total_space() / 1_000_000);
        println!("Kind: {}", disk.kind());
        println!("Mount: {}", disk.mount_point().to_string_lossy());
        println!("Filesystem: {}", disk.file_system().to_string_lossy());
    }

    let content: String =
        read_file_content(PARTITIONS).ok_or_else(|| "Unable to read partition file".to_string())?;

    content
        .lines()
        .skip(2)
        .filter_map(|line: &str| {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() == 4 && !parts[3].chars().any(|c| c.is_ascii_digit()) {
                Some(parts[3])
            } else {
                None
            }
        })
        .map(|device: &str| {
            let device_path: String = format!("/dev/{device}");

            Ok(DiskInfo {
                disk_device: device.to_string(),
                disk_size: read_file_content(&format!("/sys/block/{device}/size"))
                    .and_then(|size: String| size.trim().parse::<u64>().ok())
                    .map(|size: u64| size * 512 / 1_000_000_000),
                disk_model: read_file_content(&format!("/sys/block/{device}/device/model"))
                    .map(|model: String| model.trim().to_string()),
                disk_type: if let Some(rotational) =
                    read_file_content(&format!("/sys/block/{device}/queue/rotational"))
                {
                    match rotational.trim() {
                        "1" => Some("HDD".to_string()),
                        "0" => Some("SSD".to_string()),
                        _ => {
                            error!("[{HEADER}] Data 'Failed to retrieve disk type'");
                            None
                        }
                    }
                } else {
                    error!("[{HEADER}] Data 'Failed to retrieve disk type'");
                    None
                },
                disk_partitions: collect_partitions(device).ok(),
                disk_smart_info: collect_smart_data(&device_path).ok(),
                disk_write_bandwidth: get_disk_test(&device_path).map(|(x, _)| x),
                disk_read_bandwidth: get_disk_test(&device_path).map(|(_, y)| y),
            })
        })
        .collect()
}

/// Public function used to send JSON formatted values,
/// from [`collect_disk_data`] function result.
pub fn get_disk_info() {
    let data = || -> Result<Value, Box<dyn Error>> {
        collect_disk_data()
            .map(|values: Vec<DiskInfo>| {
                json!({
                    HEADER: values.iter().map(|disk| disk.to_json()).collect::<Vec<_>>()
                })
            })
            .map_err(|e| e.into())
    };

    write_json_to_file(data, LOGGER);
}
