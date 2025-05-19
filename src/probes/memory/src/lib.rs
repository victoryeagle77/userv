//! # Lib file for memory data module
//!
//! This module provides main functionality to retrieve memories data on Unix-based systems.

use chrono::{SecondsFormat, Utc};
use std::error::Error;
use sysinfo::{MemoryRefreshKind, System};

mod utils;
use core::core::init_db;
use utils::*;

const REQUEST: &'static str = "CREATE TABLE IF NOT EXISTS memory_data (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        timestamp TEXT NOT NULL,
        bandwidth_read REAL,
        bandwidth_write REAL,
        ram_total_MB INTEGER,
        ram_used_MB INTEGER,
        ram_free_MB INTEGER,
        ram_available_MB INTEGER,
        ram_power_consumption_W REAL,
        swap_total_MB INTEGER,
        swap_used_MB INTEGER,
        swap_free_MB INTEGER
    );
    CREATE TABLE IF NOT EXISTS memory_modules (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        device_id TEXT NOT NULL,
        ram_type TEXT NOT NULL,
        size_MB INTEGER,
        power_W REAL,
        FOREIGN KEY (device_id) REFERENCES memory_data(id)
    )";

/// Retrieves detailed computing and SWAP memories data.
///
/// # Returns
///
/// - Completed [`MemInfo`] structure with all memories information.
/// - List of RAM modules detected (optional).
fn collect_mem_data() -> Result<(MemInfo, Option<Vec<MemDeviceInfo>>), Box<dyn Error>> {
    let mut sys = System::new_all();
    sys.refresh_memory_specifics(MemoryRefreshKind::everything());

    let (bandwidth_write, bandwidth_read) = get_mem_test()?;

    let ram_total = sys.total_memory() / FACTOR;
    let ram_used = sys.used_memory() / FACTOR;

    let ram_available = Some(sys.available_memory() / FACTOR);
    let ram_free = Some(sys.free_memory() / FACTOR);

    let swap_total = Some(sys.total_swap() / FACTOR);
    let swap_free = Some(sys.free_swap() / FACTOR);
    let swap_used = Some(sys.used_swap() / FACTOR);

    let ram_device = get_ram_device()?.filter(|data| !data.is_empty());
    let (ram_power_consumption, ram_devices) = match ram_device {
        Some(ref ram_devices) if !ram_devices.is_empty() => {
            let power = estimated_power_consumption(ram_devices, ram_used);
            (power, Some(ram_devices.clone()))
        }
        _ => (None, None),
    };

    Ok((
        MemInfo {
            ram_available,
            ram_free,
            ram_power_consumption,
            ram_total: Some(ram_total),
            ram_used: Some(ram_used),
            swap_free,
            swap_total,
            swap_used,
            bandwidth_read,
            bandwidth_write,
        },
        ram_devices,
    ))
}

/// Public function used to send values in SQLite database,
/// from [`collect_mem_data`] function result.
pub fn get_mem_info() -> Result<(), Box<dyn Error>> {
    let mut conn = init_db(REQUEST)?;
    let timestamp = Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true);
    let (data, ram_devices) = collect_mem_data()?;
    insert_db(&mut conn, &timestamp, &data, ram_devices.as_ref())?;
    Ok(())
}
