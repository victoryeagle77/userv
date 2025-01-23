//! # Disk Data Module
//!
//! This module provides functionality to retrieve disk data on Unix-based systems.

use chrono::{SecondsFormat, Utc};
use libc::{c_void, close, open, read};
use log::error;
use serde::Serialize;
use serde_json::json;
use std::{
    ffi::CString,
    fs::{read_dir, File},
    io::{BufRead, BufReader},
};

use crate::utils::{read_file_content, write_json_to_file};

const PARTITIONS: &str = "/proc/partitions";
const HEADER: &str = "DISK";
const LOGGER: &str = "log/disk_data.json";

/// Collection of collected disk data.
#[derive(Debug, Serialize)]
struct DiskInfo {
    /// Path in system attached to device memory.
    device: String,
    size: u64,
    model: Option<String>,
    vendor: Option<String>,
    disk_type: String,
    partitions: Vec<PartitionInfo>,
    smart_info: SmartInfo,
}

/// Collected partitions of a disk.
#[derive(Debug, Serialize)]
struct PartitionInfo {
    name: String,
    size: f64,
}

#[derive(Debug, Serialize)]
struct SmartInfo {
    power_on_hours: u8,
    health_status: String,
    reallocated_sectors: u8,
    current_pending_sectors: u8,
    temperature: u8,
}

/// Reads file content
fn read_file(path: &str) -> Result<String, std::io::Error> {
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);
    let mut contents = String::new();
    reader.read_line(&mut contents)?;
    Ok(contents)
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
    let power_on_hours: u8 = buffer[9];
    let reallocated_sectors: u8 = buffer[5];
    let current_pending_sectors: u8 = buffer[196];
    let temperature: u8 = buffer[194];

    let health_status = if reallocated_sectors > 0 {
        format!("Worn (reallocated sectors : {})", reallocated_sectors)
    } else {
        "Unused".to_string()
    };

    SmartInfo {
        power_on_hours,
        health_status,
        reallocated_sectors,
        current_pending_sectors,
        temperature,
    }
}

/// Function that retrieves detailed disk information,
///
/// # Returns
///
/// `result` : Completed `MotherboardInfo` structure with all motherboard information
/// - Motherboard name
/// - Motherboard serial number
/// - Motherboard version
/// - Motherboard vendor
/// - Bios update
/// - Bios date release
/// - Bios vendor
/// - Bios version
/// - Motherboard uuid
fn collect_disk_data() -> Result<Vec<DiskInfo>, String> {
    let file = File::open(PARTITIONS).map_err(|e| e.to_string())?;
    let reader = BufReader::new(file);
    let mut disks = Vec::new();

    for line in reader.lines().skip(2) {
        let line = line.map_err(|e| e.to_string())?;
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() == 4 && !parts[3].chars().any(|c| c.is_ascii_digit()) {
            let device = parts[3];

            let mut disk_info = DiskInfo {
                device: device.to_string(),
                size: 0,
                model: None,
                vendor: None,
                disk_type: String::new(),
                partitions: Vec::new(),
                smart_info: SmartInfo {
                    power_on_hours: 0,
                    health_status: String::new(),
                    reallocated_sectors: 0,
                    current_pending_sectors: 0,
                    temperature: 0,
                },
            };

            // Size
            if let Ok(size) = read_file(&format!("/sys/block/{}/size", device)) {
                disk_info.size = size.trim().parse::<u64>().unwrap_or(0) * 512;
            }
            // Model
            if let Ok(model) = read_file(&format!("/sys/block/{}/device/model", device)) {
                disk_info.model = Some(model.trim().to_string());
            }
            // Vendor
            if let Ok(vendor) = read_file(&format!("/sys/block/{}/device/vendor", device)) {
                disk_info.vendor = Some(vendor.trim().to_string());
            }
            // Type
            if let Ok(rotational) = read_file(&format!("/sys/block/{}/queue/rotational", device)) {
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
                            if let Ok(size) = read_file(&format!(
                                "/sys/block/{}/{}/size",
                                device,
                                name.to_str().unwrap()
                            )) {
                                let size_bytes = size.trim().parse::<u64>().unwrap_or(0) * 512;
                                disk_info.partitions.push(PartitionInfo {
                                    name: name.to_str().unwrap().to_string(),
                                    size: size_bytes as f64 / 1073741824.0,
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
                            "device": disk.device,
                            "size": disk.size / 1_073_741_824, // Convert to GB
                            "model": disk.model.clone().unwrap_or_else(|| "NULL".to_string()),
                            "vendor": disk.vendor.clone().unwrap_or_else(|| "NULL".to_string()),
                            "type": disk.disk_type,
                            "partitions": disk.partitions,
                            "smart_info": {
                                "power_on_hours": disk.smart_info.power_on_hours,
                                "health_status": disk.smart_info.health_status,
                                "reallocated_sectors": disk.smart_info.reallocated_sectors,
                                "current_pending_sectors": disk.smart_info.current_pending_sectors,
                                "temperature": disk.smart_info.temperature,
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
