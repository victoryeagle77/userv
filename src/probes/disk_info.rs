//! # Disk Data Module
//!
//! This module provides functionality to retrieve disk data on Unix-based systems.

use libc::{c_void, close, open, read};
use log::error;
use serde::Serialize;
use serde_json::json;
use std::{
    fs::remove_file,
    io::{Read, Write},
    thread,
    time::{Duration, Instant},
    {
        ffi::CString,
        ffi::NulError,
        fs::{read_dir, File},
    },
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
    /// Path in system attached to device memory.
    disk_device: String,
    /// Space of the disk.
    disk_size: Option<u64>,
    /// Disk model name.
    disk_model: Option<String>,
    /// Disk vendor name.
    disk_vendor: Option<String>,
    /// Disk type (HDD or SSD).
    disk_type: Option<String>,
    /// Disk partitions list.
    disk_part: Option<Vec<PartitionInfo>>,
    /// Disk writing bandwidth test in MB/s.
    read_bandwidth: Option<f64>,
    /// Disk reading bandwidth test in MB/s.
    write_bandwidth: Option<f64>,
    /// More detailed disk information.
    smart_info: Option<SmartInfo>,
}

/// Collected partitions of a disk.
#[derive(Debug, Serialize)]
struct PartitionInfo {
    /// Disk partition path name.
    part_name: String,
    /// Space on the partition disk.
    part_size: f64,
}

/// Collected more specific and detailed disk data.
#[derive(Debug, Serialize)]
struct SmartInfo {
    /// Disk uptime power on hours.
    uptime: Option<u8>,
    /// Disk health status.
    health: Option<String>,
    /// Reallocated sectors on the disk.
    realloc: Option<u8>,
    /// Current pending sectors on the disk.
    pending: Option<u8>,
    /// Disk temperature.
    temp: Option<u8>,
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
    let test_file = format!("/tmp/{}_test_file", path.replace('/', "_"));
    let mut buffer = Vec::with_capacity(TEST_FILE_SIZE);

    let mut total_write_bandwidth: f64 = 0.0;
    let mut total_read_bandwidth: f64 = 0.0;

    for _ in 0..ITERATION {
        let write_start = Instant::now();
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
        let write_duration = write_start.elapsed();

        // Flush the cache according the operating system
        #[cfg(unix)]
        unsafe {
            libc::sync();
        }

        // Test de lecture
        let read_start = Instant::now();
        if let Ok(mut file) = File::open(&test_file) {
            let mut read_buffer = vec![0u8; TEST_FILE_SIZE];
            if file.read_exact(&mut read_buffer).is_err() {
                error!("[{HEADER}] 'Error during the temporary file reading' : {test_file}");
                return None;
            }
        } else {
            error!("[{HEADER}] 'Error while the temporary file opening' : {test_file}");
            return None;
        }
        let read_duration = read_start.elapsed();

        let write_bandwidth = (TEST_FILE_SIZE as f64 / 1e6) / write_duration.as_secs_f64();
        let read_bandwidth = (TEST_FILE_SIZE as f64 / 1e6) / read_duration.as_secs_f64();

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
/// * `SmartInfo` structure :
/// > - Disk uptime power on hours
/// > - Disk health status
/// > - Reallocated sectors on the disk
/// > - Current pending sectors on the disk
/// > - Disk temperature
///
/// - Error message if CString can not be created, file descriptor content or final extracted data are null.
fn collect_smart_data(path: &str) -> Result<SmartInfo, String> {
    let device = CString::new(path).map_err(|e: NulError| {
        error!("[{HEADER}_SMART_DATA] Data 'Failed to create CString' : {e}");
        "Failed to create CString".to_string()
    })?;
    let fd: i32 = unsafe { open(device.as_ptr(), 0) };

    if fd < 0 {
        error!("[{HEADER}_SMART_DATA] Data 'Failed to open device'");
        return Err("Failed to open device".to_string());
    }

    let mut buffer: [u8; 512] = [0; 512];
    let data: isize = unsafe { read(fd, buffer.as_mut_ptr() as *mut c_void, buffer.len()) };

    if data < 0 {
        unsafe { close(fd) };
        error!("[{HEADER}_SMART_DATA] Data 'Fail to read SMART data'");
        return Err("Fail to read SMART data".to_string());
    }

    let uptime = buffer.get(9).copied();
    let realloc = buffer.get(5).copied();
    let pending = buffer.get(196).copied();
    let temp = buffer.get(194).copied();
    let health = realloc.map(|status| {
        if status > 0 {
            format!("Worn (reallocated sectors : {status})")
        } else {
            "Unused".to_string()
        }
    });

    let result: SmartInfo = SmartInfo {
        uptime,
        health,
        realloc,
        pending,
        temp,
    };

    unsafe { close(fd) };

    Ok(result)
}

/// Function that retrieves all detailed disk information,
/// especially by reading of "/sys/block/" directory.
///
/// # Returns
///
/// `result` : The compilation of completed structures concerning all disk information.
///
/// * `DiskInfo` structure :
/// > - Path in system attached to device memory
/// > - Space of the disk
/// > - Disk model name
/// > - Disk vendor name
/// > - Disk type (HDD or SSD)
/// > - Disk partitions list
///
/// * `PartitionInfo` structure :
/// > - Disk partition path name
/// > - Space on the partition disk
///
/// * `SmartInfo` structure :
/// > - Disk uptime power on hours
/// > - Disk health status
/// > - Reallocated sectors on the disk
/// > - Current pending sectors on the disk
/// > - Disk temperature
fn collect_disk_data() -> Result<Vec<DiskInfo>, String> {
    let content = read_file_content(PARTITIONS)
        .ok_or_else(|| "File 'Unable to read partition file'".to_string())?;
    let mut disks = Vec::new();

    for line in content.lines().skip(2) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() == 4 && !parts[3].chars().any(|c| c.is_ascii_digit()) {
            let device = parts[3];

            let mut disk_info = DiskInfo {
                disk_device: device.to_string(),
                disk_size: None,
                disk_model: None,
                disk_vendor: None,
                disk_type: None,
                disk_part: None,
                smart_info: None,
                write_bandwidth: None,
                read_bandwidth: None,
            };

            // Bandwidth
            if let Some((write_bw, read_bw)) = get_disk_test(&format!("/dev/{device}")) {
                disk_info.write_bandwidth = Some(write_bw);
                disk_info.read_bandwidth = Some(read_bw);
            }

            // Size
            disk_info.disk_size = read_file_content(&format!("/sys/block/{device}/size"))
                .and_then(|size| size.trim().parse::<u64>().ok())
                .map(|size| size * 512);

            // Model
            disk_info.disk_model = read_file_content(&format!("/sys/block/{device}/device/model"))
                .map(|model| model.trim().to_string());

            // Vendor
            disk_info.disk_vendor =
                read_file_content(&format!("/sys/block/{device}/device/vendor"))
                    .map(|vendor| vendor.trim().to_string());

            // Type
            disk_info.disk_type = if let Some(rotational) =
                read_file_content(&format!("/sys/block/{device}/queue/rotational"))
            {
                match rotational.trim() {
                    "1" => Some("HDD".to_string()),
                    "0" => Some("SSD".to_string()),
                    _ => None,
                }
            } else {
                None
            };

            // Partitions
            let mut partitions = Vec::new();
            if let Ok(entries) = read_dir(format!("/sys/block/{device}")) {
                for entry in entries.flatten() {
                    let name = entry.file_name();
                    if let Some(s) = name.to_str() {
                        if s.starts_with(device) {
                            if let Some(size) = read_file_content(&format!(
                                "/sys/block/{device}/{}/size",
                                name.to_str().unwrap()
                            )) {
                                if let Ok(size_bytes) = size.trim().parse::<u64>() {
                                    partitions.push(PartitionInfo {
                                        part_name: name.to_str().unwrap().to_string(),
                                        part_size: (size_bytes * 512) as f64 / 1e9,
                                    });
                                }
                            }
                        }
                    }
                }
            }

            if !partitions.is_empty() {
                disk_info.disk_part = Some(partitions);
            }

            // SMART info
            disk_info.smart_info = collect_smart_data(&format!("/dev/{device}")).ok();

            disks.push(disk_info);
        }
    }

    Ok(disks)
}

/// Public function used to send JSON formatted values,
/// from `collect_disk_data` function result.
pub fn get_disk_info() {
    let data = || {
        let values: Vec<DiskInfo> = collect_disk_data()?;
        Ok(json!({
            HEADER: {
                "disks": values.iter().map(|disk| {
                    json!({
                        "device": disk.disk_device,
                        "size": disk.disk_size.map(|size| size / 1_000_000_000), // Convert to GB
                        "model": disk.disk_model,
                        "vendor": disk.disk_vendor,
                        "type": disk.disk_type,
                        "partitions": disk.disk_part.as_ref().map(|parts| {
                            json!(parts.iter().map(|part| {
                                json!({
                                    "part_name": part.part_name,
                                    "part_size": part.part_size
                                })
                            }).collect::<Vec<_>>())
                        }),
                        "smart_info": disk.smart_info.as_ref().map(|smart| {
                            json!({
                                "uptime": smart.uptime,
                                "health": smart.health,
                                "realloc": smart.realloc,
                                "pending": smart.pending,
                                "temp": smart.temp,
                            })
                        }),

                        "write_bandwidth": disk.write_bandwidth,
                        "read_bandwidth": disk.read_bandwidth
                    })
                }).collect::<Vec<_>>()
            }
        }))
    };

    write_json_to_file(data, LOGGER, HEADER);
}
