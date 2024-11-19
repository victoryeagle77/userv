//! # RAM memory data Module
//!
//! This module provides functionality to retrieve RAM data on Unix-based systems.

use std::collections::HashMap;
use serde_json::json;
use std::time::Instant;

use crate::utils::parse_file_content;

const ARRAY_SIZE: usize = 1000000000;
const MEMINFO: &'static str = "/proc/meminfo";

fn parse_kb_value(value: &str) -> u64 {
    value.split_whitespace()
        .next()
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(0)
}

/// # Function
/// 
/// Public function `get_ram_test` that calculates the writing and reading speed of RAM, 
/// allocating a wide range of test data in memory.
/// 
/// # Return
/// 
/// - `write_bandwidth`: `f64` = Write bandwidth test result
/// - `read_bandwidth`: `f64` = Read bandwidth test result
/// 
fn get_ram_test() -> Option<(f64, f64)>{
    let mut space_area = vec![0u8; ARRAY_SIZE];

    let write_start = Instant::now();
    for i in 0..ARRAY_SIZE {
        space_area[i] = (i % 256) as u8;
    }
    let write_duration = write_start.elapsed();

    let read_start = Instant::now();
    let mut sum: u64 = 0u64;
    for &value in space_area.iter() {
        sum += value as u64;
    }
    let read_duration = read_start.elapsed();

    // Calculating write and read bandwidth RAM in GB
    let gb: f64 = (ARRAY_SIZE as f64) / 1e9;
    let write_bandwidth: f64 = gb / write_duration.as_secs_f64();
    let read_bandwidth: f64 = gb / read_duration.as_secs_f64();

    Some((write_bandwidth, read_bandwidth))
}

/// # Function
/// 
/// Public function `get_ram_info` retrieves detailed RAM data 
/// from `/proc/meminfo` file and `get_ram_test` function
///
/// # Output
///
/// The function retrieves the following data :
/// - Total RAM memory
/// - Available RAM memory
/// - RAM memory usage
/// - Free RAM memory
/// - Buffers allocated RAM memory
/// - Kernel stack allocated RAM memory
/// - Cached RAM memory
/// - Swap RAM memory
///
pub fn get_ram_info() {
    let data = parse_file_content(MEMINFO, ":");
    let mut values = HashMap::new();

    for (key, value) in data {
        let value = parse_kb_value(&value);
        values.insert(key, value);
    }

    let mem_total = *values.get("MemTotal").unwrap_or(&0);
    let mem_available = *values.get("MemAvailable").unwrap_or(&0);
    let mem_free = *values.get("MemFree").unwrap_or(&0);
    let buffers = *values.get("Buffers").unwrap_or(&0);
    let cached = *values.get("Cached").unwrap_or(&0);
    let swap_total = *values.get("SwapTotal").unwrap_or(&0);
    let swap_free = *values.get("SwapFree").unwrap_or(&0);
    let kernel_stack = *values.get("KernelStack").unwrap_or(&0);
    let mem_locked = *values.get("Mlocked").unwrap_or(&0);
    let page_tables = *values.get("PageTables").unwrap_or(&0);

    let used_percentage: f64 = if mem_total > 0 {
        (mem_total.saturating_sub(mem_available) as f64 / mem_total as f64) * 100.0
    } else {
        0.0
    };

    let swap_percentage: f64 = if swap_total > 0 {
        (swap_total.saturating_sub(swap_free) as f64 / swap_total as f64) * 100.0
    } else {
        0.0
    };

    let ram_test = get_ram_test().unwrap_or((0.0, 0.0));

    println!("\n[[ RAM ]]\n");

    let ram_json_info: serde_json::Value = json!({
        "RAM": {
            "total_ram": format!("{:.2} GB", mem_total as f64 / 1024.0 / 1024.0),
            "available_ram": format!("{:.2} GB", mem_available as f64 / 1024.0 / 1024.0),
            "ram_usage": format!("{:.2} %", used_percentage),
            "free_ram": format!("{:.2} GB", mem_free as f64 / 1024.0 / 1024.0),
            "buffers": format!("{:.2} MB", buffers as f64 / 1024.0),
            "cached_memory": format!("{:.2} GB", cached as f64 / 1024.0 / 1024.0),
            "kernel_stack_memory": format!("{:.2} MB", kernel_stack as f64 / 1024.0),
            "locked_memory": format!("{:.2} KB", mem_locked),
            "page_tables": format!("{:.2} GB", page_tables as f64 / 1024.0 / 1024.0),
            "swap_total": format!("{:.2} MB", swap_total as f64 / 1024.0),
            "swap_free": format!("{:.2} MB", swap_free as f64 / 1024.0),
            "swap_usage": format!("{:.2} %", swap_percentage),
            "write_bandwidth": format!("{:.2} GB/s", ram_test.0),
            "read_bandwidth": format!("{:.2} GB/s", ram_test.1)
        }
    });

    println!("{}", serde_json::to_string_pretty(&ram_json_info).unwrap());
}