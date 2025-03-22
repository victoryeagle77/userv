//! # RAM memory data Module
//!
//! This module provides functionality to retrieve RAM data on Unix-based systems.

use log::error;
use serde::Serialize;
use serde_json::json;
use std::time::{Duration, Instant};
use sysinfo::{System, SystemExt};

use crate::utils::write_json_to_file;

const ARRAY_SIZE: usize = 1_000_000_000;

const HEADER: &str = "RAM";
const LOGGER: &str = "log/ram_data.json";

/// Collection of collected computing memory in kilo bytes.
#[derive(Serialize)]
struct RAMInfo {
    /// Total RAM memory.
    ram_total: Option<f32>,
    /// Available RAM memory.
    ram_available: Option<f32>,
    /// RAM memory usage in percentage.
    ram_usage: Option<f32>,
    /// Free RAM memory.
    ram_free: Option<f32>,
    /// Total swap memory.
    swap_total: Option<f32>,
    /// Free swap memory.
    swap_free: Option<f32>,
    /// Swap memory usage in percentage.
    swap_usage: Option<f32>,
    /// Memory writing bandwidth test in MB/s.
    write_bandwidth: Option<f64>,
    /// Memory reading bandwidth test in MB/s.
    read_bandwidth: Option<f64>,
}

/// Function that calculates the writing and reading speed of RAM,
/// allocating a wide range of test data in memory.
///
/// # Return
///
/// - `write_bandwidth` : Write bandwidth test result in MB/s.
/// - `read_bandwidth` : Read bandwidth test result in MB/s.
fn get_ram_test() -> Option<(Option<f64>, Option<f64>)> {
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
        std::ptr::write_volatile(&mut sum as *mut u64, sum);
        let _ = std::ptr::read_volatile(&sum as *const u64);
    }
    let read_duration: Duration = read_start.elapsed();

    let result: f64 = ARRAY_SIZE as f64;
    let write_bandwidth: f64 = result / write_duration.as_secs_f64();
    let read_bandwidth: f64 = result / read_duration.as_secs_f64();

    Some((Some(write_bandwidth), Some(read_bandwidth)))
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
fn collect_ram_data() -> Result<RAMInfo, String> {
    let mut sys: System = System::new_all();
    sys.refresh_memory();

    let mem_total: u64 = sys.total_memory();
    let mem_available: u64 = sys.available_memory();
    let mem_free: u64 = sys.free_memory();
    let swap_total: u64 = sys.total_swap();
    let swap_free: u64 = sys.free_swap();

    let used_percentage: Option<f32> = if mem_total > 0 {
        Some((mem_total.saturating_sub(mem_available) as f32 / mem_total as f32) * 100.0)
    } else {
        error!("[{HEADER}] Data 'Error of computing used memory percentage'");
        None
    };

    let swap_percentage: Option<f32> = if swap_total > 0 {
        Some((swap_total.saturating_sub(swap_free) as f32 / swap_total as f32) * 100.0)
    } else {
        error!("[{HEADER}] Data 'Error of computing swap memory percentage'");
        None
    };

    let (write_bandwidth, read_bandwidth) = get_ram_test().unwrap_or((None, None));

    let result: RAMInfo = RAMInfo {
        ram_total: Some(mem_total as f32 / 1e6),
        ram_available: Some(mem_available as f32 / 1e6),
        ram_usage: used_percentage,
        ram_free: Some(mem_free as f32 / 1e6),
        swap_total: Some(swap_total as f32 / 1e6),
        swap_free: Some(swap_free as f32 / 1e6),
        swap_usage: swap_percentage,
        write_bandwidth: write_bandwidth.map(|v: f64| v / 1e6),
        read_bandwidth: read_bandwidth.map(|v: f64| v / 1e6),
    };

    Ok(result)
}

/// Public function used to send JSON formatted values,
/// from `collect_ram_data` function result.
pub fn get_ram_info() {
    let data = || {
        let values: RAMInfo = collect_ram_data()?;
        Ok(json!({
            HEADER: values
        }))
    };

    write_json_to_file(data, LOGGER, HEADER);
}
