//! # RAM memory data Module
//!
//! This module provides functionality to retrieve RAM data on Unix-based systems.

use log::error;
use serde::Serialize;
use serde_json::{json, Value};
use std::error::Error;
use std::{
    ptr::{read_volatile, write_volatile},
    time::{Duration, Instant},
};
use sysinfo::{System, SystemExt};

use crate::utils::write_json_to_file;

const ARRAY_SIZE: usize = 1_000_000_000;
const FACTOR: u64 = 1_000_000;

const HEADER: &str = "RAM";
const LOGGER: &str = "log/ram_data.json";

/// Collection of collected computing memory in kilo bytes.
#[derive(Serialize)]
struct RAMInfo {
    /// Total RAM memory in MB.
    ram_total: Option<u64>,
    /// Available RAM memory in MB.
    ram_available: Option<u64>,
    /// RAM memory usage in percentage.
    ram_usage: Option<f32>,
    /// Free RAM memory in MB.
    ram_free: Option<u64>,
    /// Total swap memory in MB.
    swap_total: Option<u64>,
    /// Free swap memory in MB.
    swap_free: Option<u64>,
    /// Swap memory usage in percentage.
    swap_usage: Option<f32>,
    /// Memory writing bandwidth test in MB/s.
    write_bandwidth: Option<f64>,
    /// Memory reading bandwidth test in MB/s.
    read_bandwidth: Option<f64>,
}

impl RAMInfo {
    /// Converts `RAMInfo` into a JSON object.
    fn to_json(&self) -> Value {
        json!({
            "ram_total_MB": self.ram_total,
            "ram_available_MB": self.ram_available,
            "ram_usage_%": self.ram_usage,
            "ram_free_MB": self.ram_free,
            "swap_total_MB": self.swap_total,
            "swap_free_MB": self.swap_free,
            "swap_usage_%": self.swap_usage,
            "write_bandwidth_MB.s": self.write_bandwidth,
            "read_bandwidth_MB.s": self.read_bandwidth,
        })
    }
}

/// Function that calculates the writing and reading speed of RAM,
/// allocating a wide range of test data in memory.
///
/// # Return
///
/// - `write_bandwidth` : Write bandwidth test result in MB/s.
/// - `read_bandwidth` : Read bandwidth test result in MB/s.
fn get_ram_test() -> Result<(Option<f64>, Option<f64>), String> {
    let mut space_area: Vec<u8> = vec![0u8; ARRAY_SIZE];

    let write_start: Instant = Instant::now();
    for (i, item) in space_area.iter_mut().enumerate() {
        *item = (i % 256) as u8;
    }
    let write_duration: Duration = write_start.elapsed();

    let read_start: Instant = Instant::now();
    let mut sum: u64 = 0u64;
    for &value in space_area.iter() {
        sum = sum.wrapping_add(value as u64);
    }
    unsafe {
        write_volatile(&mut sum as *mut u64, sum);
        let _ = read_volatile(&sum as *const u64);
    }
    let read_duration: Duration = read_start.elapsed();

    let result: f64 = ARRAY_SIZE as f64;
    let write_bandwidth: f64 = result / write_duration.as_secs_f64() / 1e6;
    let read_bandwidth: f64 = result / read_duration.as_secs_f64() / 1e6;

    if write_bandwidth.is_nan()
        || read_bandwidth.is_nan()
        || write_bandwidth <= 0.0
        || read_bandwidth <= 0.0
    {
        error!("[{HEADER}] Data 'Invalid bandwidth calculation'");
        return Err("Invalid bandwidth calculation".to_string());
    }

    Ok((Some(write_bandwidth), Some(read_bandwidth)))
}

/// Public function reading and using `/proc/meminfo` file values,
/// and retrieves detailed computing memory data calculated in Kilo Bytes.
///
/// # Returns
///
/// `result` : Completed `RAMInfo` structure with all computing memory information
/// - Total RAM memory.
/// - Available RAM memory.
/// - RAM memory usage in percentage.
/// - Free RAM memory.
/// - Total swap memory.
/// - Free swap memory.
/// - Swap memory usage in percentage.
/// - Write bandwidth memory test result in MB/s.
/// - Read bandwidth memory test result in MB/s.
fn collect_ram_data() -> RAMInfo {
    let mut sys: System = System::new_all();
    sys.refresh_memory();

    let mem_total: u64 = sys.total_memory() / FACTOR;
    let mem_available: u64 = sys.available_memory() / FACTOR;
    let mem_free: u64 = sys.free_memory() / FACTOR;
    let swap_total: u64 = sys.total_swap() / FACTOR;
    let swap_free: u64 = sys.free_swap() / FACTOR;

    let used_percentage: Option<f32> = if mem_total > 0 {
        Some((mem_total - mem_available) as f32 / mem_total as f32 * 100.0)
    } else {
        error!("[{HEADER}] Data 'Error of computing used memory percentage'");
        None
    };

    let swap_percentage: Option<f32> = if swap_total > 0 {
        Some((swap_total - swap_free) as f32 / swap_total as f32 * 100.0)
    } else {
        error!("[{HEADER}] Data 'Error of computing swap memory percentage'");
        None
    };

    let (write_bandwidth, read_bandwidth) = match get_ram_test() {
        Ok(result) => result,
        Err(e) => {
            error!("[{HEADER}] Data 'Error during RAM test' {e}");
            (None, None)
        }
    };

    RAMInfo {
        ram_total: Some(mem_total),
        ram_available: Some(mem_available),
        ram_usage: used_percentage,
        ram_free: Some(mem_free),
        swap_total: Some(swap_total),
        swap_free: Some(swap_free),
        swap_usage: swap_percentage,
        write_bandwidth,
        read_bandwidth,
    }
}

/// Public function used to send JSON formatted values,
/// from `collect_ram_data` function result.
pub fn get_ram_info() {
    let data = || -> Result<Value, Box<dyn Error>> {
        let values: RAMInfo = collect_ram_data();
        Ok(json!({ HEADER: values.to_json() }))
    };

    write_json_to_file(data, LOGGER, HEADER);
}
