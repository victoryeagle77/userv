//! # Disk Data Module
//!
//! This module provides functionality to retrieve disk data on Unix-based systems.

use chrono::{SecondsFormat, Utc};
use libc::{c_void, close, open, read};
use log::error;
use serde::Serialize;
use serde_json::json;
use std::{ffi::CString, fs::read_dir};

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
    disk_size: u64,
    /// Disk model name.
    disk_model: Option<String>,
    /// Disk vendor name.
    disk_vendor: Option<String>,
    /// Disk type (HDD or SSD).
    disk_type: String,
    /// Disk partitions list.
    disk_part: Vec<PartitionInfo>,
    /// More detailed disk information.
    smart_info: SmartInfo,
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
    uptime: u8,
    /// Disk health status.
    health: String,
    /// Reallocated sectors on the disk.
    realloc: u8,
    /// Current pending sectors on the disk.
    pending: u8,
    /// Disk temperature.
    temp: u8,
}

/// Retrieves SMART data for a given disk
fn get_smart_info(device_path: &str) -> Result<SmartInfo, String> {
    let device = CString::new(device_path).map_err(|_| "Erreur lors de la création du CString")?;
    let fd: i32 = unsafe { open(device.as_ptr(), 0) };

    if fd < 0 {
        return Err("Erreur lors de l'ouverture du périphérique".to_string());
    }

    let mut buffer: [u8; 512] = [0; 512];
    let result: isize = unsafe { read(fd, buffer.as_mut_ptr() as *mut c_void, buffer.len()) };

    if result < 0 {
        unsafe { close(fd) };
        return Err("Erreur lors de la lecture des données SMART".to_string());
    }

    let smart_info: SmartInfo = extract_smart_info(&buffer[..result as usize]);
    unsafe { close(fd) };

    Ok(smart_info)
}

/// Extracts SMART information from raw data
fn extract_smart_info(buffer: &[u8]) -> SmartInfo {
    let uptime: u8 = buffer[9];
    let realloc: u8 = buffer[5];
    let pending: u8 = buffer[196];
    let temp: u8 = buffer[194];

    let health = if realloc > 0 {
        format!("Worn (reallocated sectors : {})", realloc)
    } else {
        "Unused".to_string()
    };

    SmartInfo {
        uptime,
        health,
        realloc,
        pending,
        temp,
    }
}

/// Function that retrieves detailed disk information,
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
                disk_size: 0,
                disk_model: None,
                disk_vendor: None,
                disk_type: String::new(),
                disk_part: Vec::new(),
                smart_info: SmartInfo {
                    uptime: 0,
                    health: String::new(),
                    realloc: 0,
                    pending: 0,
                    temp: 0,
                },
            };

            // Size
            if let Some(size) = read_file_content(&format!("/sys/block/{}/size", device)) {
                disk_info.disk_size = size.trim().parse::<u64>().unwrap_or(0) * 512;
            }
            // Model
            if let Some(model) = read_file_content(&format!("/sys/block/{}/device/model", device)) {
                disk_info.disk_model = Some(model.trim().to_string());
            }
            // Vendor
            if let Some(vendor) = read_file_content(&format!("/sys/block/{}/device/vendor", device))
            {
                disk_info.disk_vendor = Some(vendor.trim().to_string());
            }
            // Type
            if let Some(rotational) =
                read_file_content(&format!("/sys/block/{}/queue/rotational", device))
            {
                disk_info.disk_type = if rotational.trim() == "1" {
                    "HDD"
                } else {
                    "SSD"
                }
                .to_string();
            }

            // Partitions
            if let Ok(entries) = read_dir(format!("/sys/block/{}", device)) {
                for entry in entries {
                    if let Ok(entry) = entry {
                        let name = entry.file_name();
                        if name.to_str().map_or(false, |s| s.starts_with(device)) {
                            if let Some(size) = read_file_content(&format!(
                                "/sys/block/{}/{}/size",
                                device,
                                name.to_str().unwrap()
                            )) {
                                let size_bytes = size.trim().parse::<u64>().unwrap_or(0) * 512;
                                disk_info.disk_part.push(PartitionInfo {
                                    part_name: name.to_str().unwrap().to_string(),
                                    part_size: size_bytes as f64 / 1073741824.0,
                                });
                            }
                        }
                    }
                }
            }

            // SMART info
            if let Ok(smart_info) = get_smart_info(&format!("/dev/{}", device)) {
                disk_info.smart_info = smart_info;
            }

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
                .map_or_else(|| None, |ts| Some(ts));
            let data: serde_json::Value = json!({
                HEADER: {
                    "timestamp": timestamp,
                    "disks": disks.iter().map(|disk| {
                        json!({
                            "device": disk.disk_device,
                            "size": disk.disk_size / 1000000000, // Convert to GB
                            "model": disk.disk_model.clone().unwrap_or_else(|| "NULL".to_string()),
                            "vendor": disk.disk_vendor.clone().unwrap_or_else(|| "NULL".to_string()),
                            "type": disk.disk_type,
                            "partitions": disk.disk_part,
                            "smart_info": {
                                "uptime": disk.smart_info.uptime,
                                "health": disk.smart_info.health,
                                "realloc": disk.smart_info.realloc,
                                "pending": disk.smart_info.pending,
                                "temp": disk.smart_info.temp,
                            }
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
