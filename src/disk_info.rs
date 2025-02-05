//! # Disk Data Module
//!
//! This module provides functionality to retrieve disk data on Unix-based systems.

use chrono::{SecondsFormat, Utc};
use libc::{c_void, close, open, read};
use log::error;
use serde::Serialize;
use serde_json::json;
use std::{ffi::CString, ffi::NulError, fs::read_dir};

use crate::utils::{read_file_content, write_json_to_file};

const PARTITIONS: &str = "/proc/partitions";
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
    disk_part: Vec<PartitionInfo>,
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
        error!("[{}] Failed to create CString: {}", HEADER, e);
        "Failed to create CString".to_string()
    })?;
    let fd: i32 = unsafe { open(device.as_ptr(), 0) };

    if fd < 0 {
        error!("[{}] Fail to open device", HEADER);
        return Err("Fail to open device".to_string());
    }

    let mut buffer: [u8; 512] = [0; 512];
    let data: isize = unsafe { read(fd, buffer.as_mut_ptr() as *mut c_void, buffer.len()) };

    if data < 0 {
        unsafe { close(fd) };
        error!("[{}] Fail to read SMART data", HEADER);
        return Err("Fail to read SMART data".to_string());
    }

    let uptime = buffer.get(9).copied();
    let realloc = buffer.get(5).copied();
    let pending = buffer.get(196).copied();
    let temp = buffer.get(194).copied();
    let health = realloc.map(|r| {
        if r > 0 {
            format!("Worn (reallocated sectors : {})", r)
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
        .ok_or_else(|| "Impossible de lire le fichier des partitions".to_string())?;
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
                disk_part: Vec::new(),
                smart_info: None,
            };

            // Size
            disk_info.disk_size = read_file_content(&format!("/sys/block/{}/size", device))
                .and_then(|size| size.trim().parse::<u64>().ok())
                .map(|size| size * 512);

            // Model
            disk_info.disk_model =
                read_file_content(&format!("/sys/block/{}/device/model", device))
                    .map(|model| model.trim().to_string());

            // Vendor
            disk_info.disk_vendor =
                read_file_content(&format!("/sys/block/{}/device/vendor", device))
                    .map(|vendor| vendor.trim().to_string());

            // Type
            disk_info.disk_type = if let Some(rotational) =
                read_file_content(&format!("/sys/block/{}/queue/rotational", device))
            {
                match rotational.trim() {
                    "1" => Some("HDD".to_string()),
                    "0" => {
                        if read_file_content(&format!("/sys/block/{}/device/transport", device))
                            .map_or(false, |t| t.trim() == "nvme")
                        {
                            Some("NVMe".to_string())
                        } else {
                            Some("SSD".to_string())
                        }
                    }
                    _ => None,
                }
            } else {
                None
            };

            if disk_info.disk_type.is_none() {
                if read_file_content(&format!("/sys/block/{}/device/type", device))
                    .map_or(false, |t| t.trim() == "MMC")
                {
                    disk_info.disk_type = Some("eMMC".to_string());
                } else if read_file_content(&format!("/sys/block/{}/removable", device))
                    .map_or(false, |r| r.trim() == "1")
                {
                    disk_info.disk_type = Some("SD Card".to_string());
                }
            }

            // Partitions
            if let Ok(entries) = read_dir(format!("/sys/block/{}", device)) {
                for entry in entries.flatten() {
                    let name = entry.file_name();
                    if name.to_str().map_or(false, |s| s.starts_with(device)) {
                        if let Some(size) = read_file_content(&format!(
                            "/sys/block/{}/{}/size",
                            device,
                            name.to_str().unwrap()
                        )) {
                            if let Ok(size_bytes) = size.trim().parse::<u64>() {
                                disk_info.disk_part.push(PartitionInfo {
                                    part_name: name.to_str().unwrap().to_string(),
                                    part_size: (size_bytes * 512) as f64 / 1_073_741_824.0,
                                });
                            }
                        }
                    }
                }
            }

            // SMART info
            disk_info.smart_info = collect_smart_data(&format!("/dev/{}", device)).ok();

            disks.push(disk_info);
        }
    }

    Ok(disks)
}

/// Public function used to send JSON formatted values,
/// from `collect_disk_data` function result.
pub fn get_disk_info() {
    match collect_disk_data() {
        Ok(disks) => {
            let timestamp = Some(Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true))
                .map_or_else(|| None, Some);
            let data: serde_json::Value = json!({
                HEADER: {
                    "timestamp": timestamp,
                    "disks": disks.iter().map(|disk| {
                        json!({
                            "device": disk.disk_device,
                            "size": disk.disk_size.map(|size| size / 1_000_000_000), // Convert to GB
                            "model": disk.disk_model,
                            "vendor": disk.disk_vendor,
                            "type": disk.disk_type,
                            "partitions": if disk.disk_part.is_empty() {
                                serde_json::Value::Null
                            } else {
                                json!(disk.disk_part.iter().map(|part| {
                                    json!({
                                        "part_name": part.part_name,
                                        "part_size": part.part_size
                                    })
                                }).collect::<Vec<_>>())
                            },
                            "smart_info": disk.smart_info.as_ref().map(|smart| {
                                json!({
                                    "uptime": smart.uptime,
                                    "health": smart.health,
                                    "realloc": smart.realloc,
                                    "pending": smart.pending,
                                    "temp": smart.temp,
                                })
                            })
                        })
                    }).collect::<Vec<_>>()
                }
            });

            if let Err(e) = write_json_to_file(data, LOGGER) {
                error!("[{}] {}", HEADER, e);
            }
        }
        Err(e) => {
            error!("[{}] {}", HEADER, e);
        }
    }
}
