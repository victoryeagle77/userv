//! # File utilities module
//!
//! This module provides functionalities to get specific data concerning memories on Unix-based systems.

use chrono::{SecondsFormat::Millis, Utc};
use log::error;
use serde_json::{json, Value};
use std::{
    error::Error,
    fs::OpenOptions,
    io::Write,
    process::Command,
    ptr::{read_volatile, write_volatile},
    time::{Duration, Instant},
};

pub const HEADER: &'static str = "MEMORY";
pub const LOGGER: &'static str = "log/mem_data.json";

pub const ARRAY_SIZE: usize = 1_000_000_000;
pub const FACTOR: u64 = 1_000_000;

/// Typical power consumption per GB for each memory type,
/// based on voltage specifications and average module datasheets.
///
/// | Type     | Voltage   | Typical for 8GB | W/GB |
/// |----------|-----------|-----------------|------|
/// | SDRAM    | 3.3V      | 5.5W            | 0.70 |
/// | DDR      | 2.5V      | 5W              | 0.62 |
/// | DDR2     | 1.8V      | 3.8W            | 0.48 |
/// | DDR3     | 1.5V      | 3–4W            | 0.45 |
/// | DDR4     | 1.2V      | 2–3W            | 0.32 |
/// | DDR5     | 1.1V      | 1.5–2.5W        | 0.25 |
/// | LPDDR2   | 1.2V      | 1.5W            | 0.19 |
/// | LPDDR3   | 1.2V      | 1.3W            | 0.16 |
/// | LPDDR4   | 1.1V      | 1–1.5W          | 0.16 |
/// | LPDDR5   | 1.05V     | 0.8–1.2W        | 0.12 |
pub const RAM_TYPE_POWER: &[(&str, f64, f64)] = &[
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

#[derive(Debug, Clone)]
pub struct MemDeviceInfo {
    pub ram_type: String,
    /// Voltage in V.
    voltage: Option<f64>,
    // Size in MB.
    size: Option<u64>,
}

/// Function that calculates the writing and reading speed of computing memory,
/// allocating a wide range [`ARRAY_SIZE`] of test data in memory.
///
/// # Return
///
/// - `write_bandwidth` : Write bandwidth test result in MB/s.
/// - `read_bandwidth` : Read bandwidth test result in MB/s.
pub fn get_mem_test() -> Result<(Option<f64>, Option<f64>), Box<dyn Error>> {
    let mut space_area = vec![0u8; ARRAY_SIZE];

    let write_start = Instant::now();
    for (i, item) in space_area.iter_mut().enumerate() {
        *item = (i % 256) as u8;
    }
    let write_duration = write_start.elapsed();

    let read_start = Instant::now();
    let mut sum = 0u64;
    for &value in space_area.iter() {
        sum = sum.wrapping_add(value as u64);
    }
    unsafe {
        write_volatile(&mut sum as *mut u64, sum);
        let _ = read_volatile(&sum as *const u64);
    }
    let read_duration: Duration = read_start.elapsed();

    let result = ARRAY_SIZE as f64;
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

/// Parse the `dmidecode` command output to get detected RAM types.
///
/// # Returns
///
/// - A tuple of RAM type values if at least one correct type is found.
/// - An error if no values are available.
///
/// # Operating
///
/// Root privileges are required.
pub fn get_ram_sticks() -> Result<Option<Vec<MemDeviceInfo>>, Box<dyn Error>> {
    let output = Command::new("dmidecode").args(["-t", "memory"]).output()?;

    if !output.status.success() {
        return Err(format!(
            "Data 'dmidecode command failed with status : {}'",
            output.status
        )
        .into());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut device = Vec::new();
    let mut current_type = None;
    let mut current_voltage = None;
    let mut current_size = None;

    for line in stdout.lines() {
        let line = line.trim();
        if line.starts_with("Size:") {
            let value = line.trim_start_matches("Size:").trim();
            if value != "No Module Installed" && value != "Unknown" {
                if let Some(size_str) = value.split_whitespace().next() {
                    if let Ok(mut size) = size_str.parse::<u64>() {
                        if value.contains("GB") {
                            size *= 1_000;
                        }
                        current_size = Some(size);
                    }
                }
            }
        }

        if line.starts_with("Type:") {
            let value = line.trim_start_matches("Type:").trim();
            if value != "Unknown" && value != "Other" && value != "DRAM" {
                current_type = Some(value.to_string());
            }
        }

        if line.starts_with("Configured Voltage:") {
            let value = line.trim_start_matches("Configured Voltage:").trim();
            if let Some(voltage_str) = value.split_whitespace().next() {
                let voltage_str = voltage_str.replace(",", ".");
                if let Ok(voltage) = voltage_str.parse::<f64>() {
                    current_voltage = Some(voltage);
                }
            }
        }

        // End of memory block
        if line.is_empty() && current_type.is_some() && current_size.is_some() {
            device.push(MemDeviceInfo {
                ram_type: current_type.take().unwrap(),
                voltage: current_voltage.take(),
                size: current_size.take(),
            });
        }
    }

    // Last memory device
    if current_type.is_some() && current_size.is_some() {
        device.push(MemDeviceInfo {
            ram_type: current_type.take().unwrap(),
            voltage: current_voltage.take(),
            size: current_size.take(),
        });
    }

    if device.is_empty() {
        Err("Data 'Failed to identify RAM sticks'".into())
    } else {
        Ok(Some(device))
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
    let total: u64 = device.iter().map(|s| s.size.unwrap_or(0)).sum();
    if total == 0 {
        error!("[{HEADER}] Data 'No RAM sticks detected for power estimation'");
        return None;
    }

    let mut total = 0.0;
    for i in device {
        let size = i.size.unwrap_or(0) as f64 / 1e3;
        if size == 0.0 {
            continue;
        }
        if let Some(&(_typ, ref_voltage, ref_w_per_gb)) = RAM_TYPE_POWER
            .iter()
            .find(|&&(typ, _, _)| typ == i.ram_type)
        {
            let voltage = i.voltage.unwrap_or(ref_voltage);
            let w_per_gb = ref_w_per_gb * (voltage / ref_voltage);
            total += w_per_gb * size;
        }
    }

    Some(total * (used as f64 / total as f64))
}

/// Writes JSON formatted data in a file
///
/// # Arguments
///
/// * `data` : JSON serialized collected metrics data to write
/// * `path` : File path use to writing data
///
/// # Return
///
/// - Custom error message if an error occurs during JSON data serialization or file handling.
pub fn write_json_to_file<F>(generator: F, path: &'static str) -> Result<(), Box<dyn Error>>
where
    F: FnOnce() -> Result<Value, Box<dyn Error>>,
{
    let mut data: Value = generator()?;

    // Timestamp implementation in JSON object
    let timestamp = Some(Utc::now().to_rfc3339_opts(Millis, true));

    // Format data to JSON object
    if data.is_object() {
        data.as_object_mut()
            .unwrap()
            .insert("timestamp".to_owned(), json!(timestamp));
    } else if data.is_array() {
        for item in data.as_array_mut().unwrap() {
            if item.is_object() {
                item.as_object_mut()
                    .unwrap()
                    .insert("timestamp".to_owned(), json!(timestamp));
            }
        }
    }

    let mut file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(path)?;
    let log = serde_json::to_string_pretty(&data)?;

    file.write_all(log.as_bytes())?;

    Ok(())
}
