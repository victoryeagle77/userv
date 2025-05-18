//! # RAM memory data Module
//!
//! This module provides functionality to retrieve RAM and SWAP data on Unix-based systems.

use log::error;
use serde::Serialize;
use serde_json::{json, Value};
use std::error::Error;
use std::{
    process::Command,
    ptr::{read_volatile, write_volatile},
    time::{Duration, Instant},
};
use sysinfo::{MemoryRefreshKind, System};

use crate::utils::write_json_to_file;

const HEADER: &str = "RAM";
const LOGGER: &str = "log/ram_data.json";

const ARRAY_SIZE: usize = 1_000_000_000;
const FACTOR: u64 = 1_000_000;

const RAM_TYPE_POWER: &[(&str, f64)] = &[
    ("DDR3", 0.45),   // DDR3 : 1.5V, typically 3 to 4W for 8 Go => ~0.38 to 0.50 W/Go
    ("DDR4", 0.32),   // DDR4 : 1.2V, typically 2 to 3W for 8 Go => ~0.25 to 0.38 W/Go
    ("DDR5", 0.25),   // DDR5 : 1.1V, typically 1.5 to 2.5W for 8 Go => ~0.19 to 0.31 W/Go
    ("LPDDR4", 0.16), // LPDDR4 : 1.1V, typically 1 to 1.5W for 8 Go => ~0.13 to 0.19 W/Go
    ("LPDDR5", 0.12), // LPDDR5 : 1.05V, typically 0.8 to 1.2W for 8 Go => ~0.10 to 0.15 W/Go
];

/// Collection of collected memory based in bytes.
#[derive(Serialize)]
struct RAMInfo {
    /// Available RAM memory in MB.
    ram_available: Option<u64>,
    /// Free RAM memory in MB.
    ram_free: Option<u64>,
    /// RAM power consumption according its type in W.
    ram_power_consumption: Option<Vec<f64>>,
    /// Total RAM memory in MB.
    ram_total: Option<u64>,
    /// Used RAM memory in MB.
    ram_used: Option<u64>,
    /// Free swap memory in MB.
    swap_free: Option<u64>,
    /// Total swap memory in MB.
    swap_total: Option<u64>,
    /// Used swap memory in MB.
    swap_used: Option<u64>,
    /// Memory reading bandwidth test in MB/s.
    read_bandwidth: Option<f64>,
    /// Memory writing bandwidth test in MB/s.
    write_bandwidth: Option<f64>,
    ram_types: Option<Vec<String>>,
}

impl RAMInfo {
    /// Converts [`RAMInfo`] into a JSON object.
    fn to_json(&self) -> Value {
        json!({
            "ram_available_MB": self.ram_available,
            "ram_free_MB": self.ram_free,
            "ram_power_consumption_W": self.ram_power_consumption,
            "ram_types": self.ram_types,
            "ram_total_MB": self.ram_total,
            "ram_usage_MB": self.ram_used,
            "swap_free_MB": self.swap_free,
            "swap_total_MB": self.swap_total,
            "swap_usage_MB": self.swap_used,
            "write_bandwidth_MB.s": self.write_bandwidth,
            "read_bandwidth_MB.s": self.read_bandwidth,
        })
    }
}

/// Function that calculates the writing and reading speed of RAM,
/// allocating a wide range [`ARRAY_SIZE`] of test data in memory.
///
/// # Return
///
/// - `write_bandwidth` : Write bandwidth test result in MB/s.
/// - `read_bandwidth` : Read bandwidth test result in MB/s.
fn get_ram_test() -> Result<(Option<f64>, Option<f64>), Box<dyn Error>> {
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
pub fn get_ram_types() -> Result<Option<Vec<String>>, Box<dyn Error>> {
    let output = Command::new("dmidecode").args(["-t", "memory"]).output()?;

    if !output.status.success() {
        return Err(format!(
            "Data 'dmidecode command failed with status : {}'",
            output.status
        )
        .into());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut result = Vec::new();

    for line in stdout.lines() {
        if let Some(rest) = line.trim_start().strip_prefix("Type:") {
            let types = rest.trim();

            if types != "Unknown"
                && types != "Other"
                && types != "DRAM"
                && !result.contains(&types.to_string())
            {
                result.push(types.to_string());
            }
        }
    }

    if result.is_empty() {
        Err("Data 'Failed to identifying the RAM type'".into())
    } else {
        Ok(Some(result))
    }
}

/// Estimation of power consumption by RAM in W.
/// Base on the typical power consumption per GB based on the RAM type with [`RAM_TYPE_POWER`].
/// The values are taken from the voltage specifications and average power consumption of standard modules
/// (see manufacturer documentation and specialized articles).
/// Technical sources: "KingSpec", "FS.com", "Infomax", "Corsair", "Kiatoo", "Crucial".
///
/// # Returns
///
/// - Returns the estimated RAM power consumption in W.
/// - None if RAM type is unknown or total RAM is zero.
fn ram_power_consumption(ram_total: u64, ram_used: u64, ram_type: &str) -> Option<f64> {
    let power_per_gb = RAM_TYPE_POWER
        .iter()
        .find(|&&(t, _)| t == ram_type)
        .map(|&(_, w)| w);

    if power_per_gb.is_none() {
        error!("[{HEADER}] Data 'Failed to determine the RAM power classification'");
    }

    let power_per_gb = power_per_gb?;
    let ram_total_gb = ram_total as f64 / 1e3;
    let ram_used_gb = ram_used as f64 / 1e3;
    if ram_total_gb > 0.0 {
        Some((ram_total_gb * power_per_gb) * (ram_used_gb / ram_total_gb))
    } else {
        error!("[{HEADER}] Data 'Failed to estimate the RAM power consumption'");
        None
    }
}

/// Retrieves detailed computing and SWAP memories data.
///
/// # Returns
///
/// - Completed [`RAMInfo`] structure with all memories information.
/// - An error when some important and critical metrics can't be retrieved.
fn collect_ram_data() -> Result<RAMInfo, Box<dyn Error>> {
    let mut sys = System::new_all();
    sys.refresh_memory_specifics(MemoryRefreshKind::everything());

    let ram_total = sys.total_memory() / FACTOR;
    let ram_used = sys.used_memory() / FACTOR;

    let ram_available = Some(sys.available_memory() / FACTOR);
    let ram_free = Some(sys.free_memory() / FACTOR);

    let swap_total = Some(sys.total_swap() / FACTOR);
    let swap_free = Some(sys.free_swap() / FACTOR);
    let swap_used = Some(sys.used_swap() / FACTOR);

    let (write_bandwidth, read_bandwidth) = get_ram_test()?;

    let types = get_ram_types()?.filter(|data| !data.is_empty());
    let (ram_types, ram_power_consumption) = match types {
        Some(ref data) if !data.is_empty() => {
            let power = data
                .iter()
                .filter_map(|t| ram_power_consumption(ram_total, ram_used, t))
                .collect();
            (Some(data.clone()), Some(power))
        }
        _ => (None, None),
    };

    Ok(RAMInfo {
        ram_available,
        ram_free,
        ram_types,
        ram_power_consumption,
        ram_total: Some(ram_total),
        ram_used: Some(ram_used),
        swap_free,
        swap_total,
        swap_used,
        write_bandwidth,
        read_bandwidth,
    })
}

/// Public function used to send JSON formatted values,
/// from [`collect_ram_data`] function result.
pub fn get_ram_info() -> Result<(), Box<dyn Error>> {
    let data = collect_ram_data()?;
    let values = json!({ HEADER: data.to_json() });
    write_json_to_file(|| Ok(values), LOGGER)?;
    Ok(())
}
