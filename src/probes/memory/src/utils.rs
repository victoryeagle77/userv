//! # File utilities module
//!
//! This module provides functionalities to get specific data concerning memories on Unix-based systems.

use log::error;
use rusqlite::{params, Connection};
use serde::Serialize;
use std::{
    error::Error,
    process::Command,
    ptr::{read_volatile, write_volatile},
    time::{Duration, Instant},
};

const HEADER: &'static str = "MEMORY";
pub const FACTOR: u64 = 1_000_000;

const DEFAULT_ARRAY_SIZE: usize = 100_000_000;

/// Typical power consumption per GB for each memory type,
/// based on voltage specifications and average module datasheets.
const RAM_TYPE_POWER: &[(&'static str, f64, f64)] = &[
    ("SDRAM", 3.3, 0.70),
    ("DDR", 2.5, 0.60),
    ("DDR2", 1.8, 0.48),
    ("DDR3", 1.5, 0.45),
    ("DDR4", 1.2, 0.32),
    ("DDR5", 1.1, 0.25),
    ("LPDDR2", 1.2, 0.19),
    ("LPDDR3", 1.2, 0.16),
    ("LPDDR4", 1.1, 0.16),
    ("LPDDR5", 1.05, 0.12),
];

/// Information about memory device info.
#[derive(Debug, Clone)]
pub struct MemDeviceInfo {
    /// Type of computing memory.
    pub kind: String,
    /// Serial number of the memory device.
    pub id: Option<String>,
    /// Voltage in V.
    pub voltage: Option<f64>,
    // Size in MB.
    pub size: Option<u64>,
}

/// Collection of collected memory based in bytes.
#[derive(Debug, Serialize)]
pub struct MemInfo {
    /// Memory reading bandwidth test in MB/s.
    pub bandwidth_read: Option<f64>,
    /// Memory writing bandwidth test in MB/s.
    pub bandwidth_write: Option<f64>,
    /// Available RAM memory in MB.
    pub ram_available: Option<u64>,
    /// Free RAM memory in MB.
    pub ram_free: Option<u64>,
    /// RAM power consumption according its type in W.
    pub ram_power_consumption: Option<f64>,
    /// Total RAM memory in MB.
    pub ram_total: Option<u64>,
    /// Used RAM memory in MB.
    pub ram_used: Option<u64>,
    /// Free swap memory in MB.
    pub swap_free: Option<u64>,
    /// Total swap memory in MB.
    pub swap_total: Option<u64>,
    /// Used swap memory in MB.
    pub swap_used: Option<u64>,
}

/// Insert memory device parameters into the database.
///
/// # Arguments
///
/// - `conn`: Connection to SQLite database.
/// - `timestamp`: Timestamp of the measurement.
/// - `data`: [`MemInfo`] information to insert in database.
/// - `ram_devices`: [`MemDeviceInfo`] list of RAM modules (optional, can be None).
///
/// # Returns
///
/// - Insert the [`MemInfo`] and [`MemDeviceInfo`] filled structures in an SQLite database.
/// - Logs an error if the SQL insert request failed.
pub fn insert_db(
    conn: &mut Connection,
    timestamp: &str,
    data: &MemInfo,
    ram_devices: Option<&Vec<MemDeviceInfo>>,
) -> Result<(), Box<dyn Error>> {
    // Insert the main memory parameters in table
    conn.execute(
        "INSERT INTO memory_data (
            timestamp,
            bandwidth_read,
            bandwidth_write,
            ram_total_MB,
            ram_used_MB,
            ram_free_MB,
            ram_available_MB,
            ram_power_consumption_W,
            swap_total_MB,
            swap_used_MB,
            swap_free_MB
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
        params![
            timestamp,
            data.bandwidth_read,
            data.bandwidth_write,
            data.ram_total,
            data.ram_used,
            data.ram_free,
            data.ram_available,
            data.ram_power_consumption,
            data.swap_total,
            data.swap_used,
            data.swap_free,
        ],
    )?;

    // Insert RAM modules if provided
    if let Some(ram_devices) = ram_devices {
        for module in ram_devices {
            let power = estimated_power_consumption(ram_devices, module.size.unwrap_or(0));
            conn.execute(
                "INSERT INTO memory_modules (
                    device_id, ram_type, size_MB, estimated_power_W
                ) VALUES (
                   ?1, ?2, ?3, ?4
                )",
                params![module.id, module.kind, module.size, power],
            )?;
        }
    }

    Ok(())
}

/// Function that calculates the writing and reading speed of computing memory,
/// allocating a wide range [`ARRAY_SIZE`] of test data in memory.
///
/// # Return
///
/// - `write_bandwidth` : Write bandwidth test result in MB/s.
/// - `read_bandwidth` : Read bandwidth test result in MB/s.
pub fn get_mem_test() -> Result<(Option<f64>, Option<f64>), Box<dyn Error>> {
    let array_size = std::env::var("MEM_TEST_SIZE")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(DEFAULT_ARRAY_SIZE);

    let mut space_area = vec![0u8; array_size];

    let write_start = Instant::now();
    for (i, item) in space_area.iter_mut().enumerate() {
        *item = (i % 256) as u8;
    }
    let write_duration = write_start.elapsed();

    let read_start = Instant::now();
    let mut sum = 0u64;
    for &value in &space_area {
        sum = sum.wrapping_add(value as u64);
    }
    unsafe {
        write_volatile(&mut sum as *mut u64, sum);
        let _ = read_volatile(&sum as *const u64);
    }
    let read_duration: Duration = read_start.elapsed();

    let result = array_size as f64;
    let write_bandwidth = result / write_duration.as_secs_f64() / 1e6;
    let read_bandwidth = result / read_duration.as_secs_f64() / 1e6;

    if write_bandwidth.is_nan()
        || read_bandwidth.is_nan()
        || write_bandwidth <= 0.0
        || read_bandwidth <= 0.0
    {
        return Err("Data 'Invalid bandwidth calculation'".to_string().into());
    }

    Ok((Some(write_bandwidth), Some(read_bandwidth)))
}

/// Parse the `dmidecode` command output to get data on detected RAM types.
///
/// # Returns
///
/// - A tuple of RAM type values if at least one correct type is found.
/// - An error if no values are available.
///
/// # Operating
///
/// Root privileges are required.
pub fn get_ram_device() -> Result<Option<Vec<MemDeviceInfo>>, Box<dyn Error>> {
    /// Extract by pattern a value after a prefix,
    /// to get data with the output result of command from [`get_ram_device`] function.
    ///
    /// # Arguments
    ///
    /// - `line` : Pattern corresponding to the data line wanted.
    /// - `prefix` : Character after the line pattern.
    fn extract_value<'a>(line: &'a str, prefix: &str) -> Option<&'a str> {
        line.strip_prefix(prefix).map(|s| s.trim())
    }

    /// Parse size string in MB,
    /// to get data with the output result of command from [`get_ram_device`] function.
    fn parse_size(value: &str) -> Option<u64> {
        let mut parts = value.split_whitespace();
        let size = parts.next()?.parse::<u64>().ok()?;
        match parts.next()? {
            "GB" => Some(size * 1_000),
            "MB" => Some(size),
            _ => None,
        }
    }

    let output = Command::new("dmidecode").args(["-t", "memory"]).output()?;

    if !output.status.success() {
        return Err(format!(
            "Data 'dmidecode command failed with status' : {}",
            output.status
        )
        .into());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut devices = Vec::new();
    let mut current = MemDeviceInfo {
        kind: String::new(),
        id: None,
        voltage: None,
        size: None,
    };

    for line in stdout.lines() {
        let line = line.trim();
        if let Some(val) = extract_value(line, "Size:") {
            current.size = parse_size(val);
        } else if let Some(val) = extract_value(line, "Type:") {
            if val != "Unknown" && val != "Other" {
                current.kind = val.to_string();
            }
        } else if let Some(val) = extract_value(line, "Configured Voltage:") {
            current.voltage = val.replace(",", ".").parse().ok();
        } else if let Some(val) = extract_value(line, "Serial Number:") {
            if val != "Unknown" {
                current.id = Some(val.to_string());
            }
        }

        // End of memory block
        if line.is_empty() && !current.kind.is_empty() && current.size.is_some() {
            devices.push(current.clone());
            current = MemDeviceInfo {
                kind: String::new(),
                id: None,
                voltage: None,
                size: None,
            };
        }
    }

    // Last memory device
    if !current.kind.is_empty() && current.size.is_some() {
        devices.push(current);
    }

    if devices.is_empty() {
        Err("Failed to identify RAM device".into())
    } else {
        Ok(Some(devices))
    }
}

/// Estimation of power consumption by memory in W.
/// Base on the typical power consumption per GB based on the memory type defined in [`RAM_TYPE_POWER`].
///
/// # Returns
///
/// - Returns the estimated RAM power consumption in W.
/// - None if memory type is unknown or total memory is zero.
pub fn estimated_power_consumption(device: &[MemDeviceInfo], used: u64) -> Option<f64> {
    let total_size: u64 = device.iter().map(|s| s.size.unwrap_or(0)).sum();
    if total_size == 0 {
        error!("[{HEADER}] Data 'No RAM devices detected for power estimation'");
        return None;
    }

    let mut total_power = 0.0;
    for i in device {
        let size = i.size.unwrap_or(0) as f64 / 1e3;
        if size == 0.0 {
            continue;
        }
        if let Some(&(_, ref_voltage, ref_energy)) =
            RAM_TYPE_POWER.iter().find(|&&(t, _, _)| t == i.kind)
        {
            let voltage = i.voltage.unwrap_or(ref_voltage);
            let energy = ref_energy * (voltage / ref_voltage);
            total_power += energy * size;
        }
    }

    Some(total_power * (used as f64 / total_size as f64))
}
