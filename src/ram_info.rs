//! # RAM memory data Module
//!
//! This module provides functionality to retrieve RAM data on Unix-based systems.

use log::error;
use serde::Serialize;
use serde_json::json;
use std::collections::HashMap;
use std::time::Instant;

use crate::utils::{parse_file_content, write_json_to_file};

const ARRAY_SIZE: usize = 1000000000;
const MEMINFO: &str = "/proc/meminfo";

const HEADER: &str = "RAM";
const LOGGER: &str = "log/ram_data.json";

/// Collection of collected computing memory in kilo bytes.
#[derive(Serialize)]
struct RAMInfo {
    /// Total RAM memory.
    ram_total: String,
    /// Available RAM memory.
    ram_available: String,
    /// RAM memory usage in percentage.
    ram_usage: f32,
    /// Free RAM memory.
    ram_free: String,
    /// Buffers allocated memory.
    mem_buffers: String,
    /// Kernel stack allocated memory.
    mem_cached: String,
    /// Cached RAM memory.
    kernel_stack: String,
    /// Locked allocated memory.
    mem_locked: String,
    /// Page tables memory.
    page_tables: String,
    /// Total swap memory.
    swap_total: String,
    /// Free swap memory.
    swap_free: String,
    /// Swap memory usage in percentage.
    swap_usage: f32,
    // Write bandwidth memory test result in MB/s.
    write_bandwidth: f64,
    // Read bandwidth memory test result in MB/s.
    read_bandwidth: f64,
}

/// Function that calculates the writing and reading speed of RAM,
/// allocating a wide range of test data in memory.
///
/// # Return
///
/// - `write_bandwidth` : Write bandwidth test result in MB/s.
/// - `read_bandwidth` : Read bandwidth test result in MB/s.
fn get_ram_test() -> Option<(f64, f64)> {
    let mut space_area = vec![0u8; ARRAY_SIZE];

    let write_start = Instant::now();
    for i in 0..ARRAY_SIZE {
        space_area[i] = (i % 256) as u8;
    }
    let write_duration = write_start.elapsed();

    let read_start = Instant::now();
    let mut sum: u64 = 0u64;
    for &value in space_area.iter() {
        sum = sum.wrapping_add(value as u64);
    }
    unsafe {
        std::ptr::write_volatile(&mut sum as *mut u64, sum);
        let _ = std::ptr::read_volatile(&sum as *const u64);
    }
    let read_duration = read_start.elapsed();

    let result: f64 = (ARRAY_SIZE as f64) / 1e6;
    let write_bandwidth: f64 = result / write_duration.as_secs_f64();
    let read_bandwidth: f64 = result / read_duration.as_secs_f64();

    Some((write_bandwidth, read_bandwidth))
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
/// - Buffers allocated memory.
/// - Kernel stack allocated memory.
/// - Cached RAM memory.
/// - Locked allocated memory.
/// - Page tables memory.
/// - Total swap memory.
/// - Free swap memory.
/// - Swap memory usage in percentage.
/// - Write bandwidth memory test result in MB/s.
/// - Read bandwidth memory test result in MB/s.
fn collect_ram_data() -> Result<RAMInfo, String> {
    let data = parse_file_content(MEMINFO, ":");
    let mut values = HashMap::new();

    for (key, value) in data {
        let get_value = value
            .split_whitespace()
            .next()
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(0);
        values.insert(key, get_value);
    }

    let mem_total = *values.get("MemTotal").unwrap_or(&0);
    let mem_available = *values.get("MemAvailable").unwrap_or(&0);
    let swap_total = *values.get("SwapTotal").unwrap_or(&0);
    let swap_free = *values.get("SwapFree").unwrap_or(&0);

    let used_percentage: f32 = if mem_total > 0 {
        (mem_total.saturating_sub(mem_available) as f32 / mem_total as f32) * 100.0
    } else {
        0.0
    };

    let swap_percentage: f32 = if swap_total > 0 {
        (swap_total.saturating_sub(swap_free) as f32 / swap_total as f32) * 100.0
    } else {
        0.0
    };

    let (write_bandwidth, read_bandwidth) = get_ram_test().unwrap_or((0.0, 0.0));

    let result: RAMInfo = RAMInfo {
        ram_total: format!("{:.2} GB", mem_total as f64 / 1024.0 / 1024.0),
        ram_available: format!(
            "{:.2} GB",
            *values.get("MemAvailable").unwrap_or(&0) as f64 / 1024.0 / 1024.0
        ),
        ram_usage: used_percentage,
        ram_free: format!(
            "{:.2} GB",
            *values.get("MemFree").unwrap_or(&0) as f64 / 1024.0 / 1024.0
        ),
        mem_buffers: format!(
            "{:.2} MB",
            *values.get("Buffers").unwrap_or(&0) as f64 / 1024.0
        ),
        mem_cached: format!(
            "{:.2} GB",
            *values.get("Cached").unwrap_or(&0) as f64 / 1024.0 / 1024.0
        ),
        kernel_stack: format!(
            "{:.2} MB",
            *values.get("KernelStack").unwrap_or(&0) as f64 / 1024.0
        ),
        mem_locked: format!("{:.2} KB", *values.get("Mlocked").unwrap_or(&0)),
        page_tables: format!(
            "{:.2} MB",
            *values.get("PageTables").unwrap_or(&0) as f64 / 1024.0
        ),
        swap_total: format!("{:.2} MB", swap_total as f64 / 1024.0),
        swap_free: format!("{:.2} MB", swap_free as f64 / 1024.0),
        swap_usage: swap_percentage,
        write_bandwidth,
        read_bandwidth,
    };

    Ok(result)
}

/// Public function used to send JSON formatted values,
/// from `collect_ram_data` function result.
pub fn get_ram_info() {
    match collect_ram_data() {
        Ok(values) => {
            let data: serde_json::Value = json!({ HEADER: values });

            if let Err(e) = write_json_to_file(data, LOGGER) {
                error!("[{}] {}", HEADER, e);
            }
        }
        Err(e) => {
            error!("[{}] {}", HEADER, e);
        }
    }
}
