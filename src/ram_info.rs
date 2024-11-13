//! # RAM memory data Module
//!
//! This module provides functionality to retrieve RAM data on Unix-based systems.

use std::collections::HashMap;
use colored::Colorize;
use serde_json::json;
use std::time::Instant;

const ARRAY_SIZE: usize = 1_000_000_000;
const MEMINFO: &'static str = "/proc/meminfo";

use crate::utils::{parse_file_content};

fn parse_kb_value(value: &str) -> u64 {
    value.split_whitespace()
        .next()
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(0)
}

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

    let used_percentage = if mem_total > 0 {
        (mem_total.saturating_sub(mem_available) as f64 / mem_total as f64) * 100.0
    } else {
        0.0
    };

    let swap_percentage = if swap_total > 0 {
        (swap_total.saturating_sub(swap_free) as f64 / swap_total as f64) * 100.0
    } else {
        0.0
    };

    let mut space_area = vec![0u8; ARRAY_SIZE];
    let write_start = Instant::now();

    for i in 0..ARRAY_SIZE {
        space_area[i] = (i % 256) as u8;
    }

    let write_duration = write_start.elapsed();

    let read_start = Instant::now();
    let mut sum = 0u64;

    for &value in space_area.iter() {
        sum += value as u64;
    }

    let read_duration = read_start.elapsed();

    println!("{}", "\n[[ RAM ]]\n".magenta().bold());

    // Calculating write and read bandwidth RAM in Gb
    let gb = (ARRAY_SIZE as f64) / 1e9;
    let write_bandwidth = gb / write_duration.as_secs_f64();
    let read_bandwidth = gb / read_duration.as_secs_f64();

    let ram_json_info = json!({
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
            "write_bandwidth": format!("{:.2} GB/s",write_bandwidth),
            "read_bandwidth": format!("{:.2} GB/s", read_bandwidth)
        }
    });

    println!("{}", serde_json::to_string_pretty(&ram_json_info).unwrap());
}