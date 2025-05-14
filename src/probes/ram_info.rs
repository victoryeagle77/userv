//! # RAM memory data Module
//!
//! This module provides functionality to retrieve RAM and SWAP data on Unix-based systems.

use serde::Serialize;
use serde_json::{json, Value};
use std::error::Error;
use std::{
    ptr::{read_volatile, write_volatile},
    time::{Duration, Instant},
};
use sysinfo::{MemoryRefreshKind, System};

use crate::utils::write_json_to_file;

const ARRAY_SIZE: usize = 1_000_000_000;
const FACTOR: u64 = 1_000_000;

const HEADER: &str = "RAM";
const LOGGER: &str = "log/ram_data.json";

/// Collection of collected memory based in bytes.
#[derive(Serialize)]
struct RAMInfo {
    /// Available RAM memory in MB.
    ram_available: Option<u64>,
    /// Free RAM memory in MB.
    ram_free: Option<u64>,
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
    /// Memory writing bandwidth test in MB/s.
    write_bandwidth: Option<f64>,
    /// Memory reading bandwidth test in MB/s.
    read_bandwidth: Option<f64>,
}

impl RAMInfo {
    /// Converts [`RAMInfo`] into a JSON object.
    fn to_json(&self) -> Value {
        json!({
            "ram_available_MB": self.ram_available,
            "ram_free_MB": self.ram_free,
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
/// allocating a wide range of test data in memory.
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

/// Retrieves detailed computing and SWAP memories data.
///
/// # Returns
///
/// - Completed [`RAMInfo`] structure with all memories information.
/// - An error when some important and critical metrics can't be retrieved.
fn collect_ram_data() -> Result<RAMInfo, Box<dyn Error>> {
    let mut sys = System::new_all();
    sys.refresh_memory_specifics(MemoryRefreshKind::everything());

    let ram_total = Some(sys.total_memory() / FACTOR);
    let ram_available = Some(sys.available_memory() / FACTOR);
    let ram_free = Some(sys.free_memory() / FACTOR);
    let ram_used = Some(sys.used_memory() / FACTOR);
    let swap_total = Some(sys.total_swap() / FACTOR);
    let swap_free = Some(sys.free_swap() / FACTOR);
    let swap_used = Some(sys.used_swap() / FACTOR);

    let (write_bandwidth, read_bandwidth) = get_ram_test()?;

    Ok(RAMInfo {
        ram_available,
        ram_free,
        ram_total,
        ram_used,
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
