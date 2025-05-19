//! # Lib file for memory data module
//!
//! This module provides main functionality to retrieve memories data on Unix-based systems.

use serde::Serialize;
use serde_json::{json, Value};
use std::error::Error;
use sysinfo::{MemoryRefreshKind, System};

mod utils;
use crate::utils::*;

/// Collection of collected memory based in bytes.
#[derive(Serialize)]
struct MemInfo {
    /// Memory reading bandwidth test in MB/s.
    bandwidth_read: Option<f64>,
    /// Memory writing bandwidth test in MB/s.
    bandwidth_write: Option<f64>,
    /// Available RAM memory in MB.
    ram_available: Option<u64>,
    /// Free RAM memory in MB.
    ram_free: Option<u64>,
    /// RAM power consumption according its type in W.
    ram_power_consumption: Option<f64>,
    /// Total RAM memory in MB.
    ram_total: Option<u64>,
    /// Type of RAM detected.
    ram_types: Option<Vec<String>>,
    /// Used RAM memory in MB.
    ram_used: Option<u64>,
    /// Free swap memory in MB.
    swap_free: Option<u64>,
    /// Total swap memory in MB.
    swap_total: Option<u64>,
    /// Used swap memory in MB.
    swap_used: Option<u64>,
}

impl MemInfo {
    /// Converts [`MemInfo`] into a JSON object.
    fn to_json(&self) -> Value {
        json!({
            "bandwidth_read_MB.s": self.bandwidth_read,
            "bandwidth_write_MB.s": self.bandwidth_write,
            "ram_available_MB": self.ram_available,
            "ram_free_MB": self.ram_free,
            "ram_power_consumption_W": self.ram_power_consumption,
            "ram_types": self.ram_types,
            "ram_total_MB": self.ram_total,
            "ram_usage_MB": self.ram_used,
            "swap_free_MB": self.swap_free,
            "swap_total_MB": self.swap_total,
            "swap_usage_MB": self.swap_used,
        })
    }
}

/// Retrieves detailed computing and SWAP memories data.
///
/// # Returns
///
/// - Completed [`MemInfo`] structure with all memories information.
/// - An error when some important and critical metrics can't be retrieved.
fn collect_mem_data() -> Result<MemInfo, Box<dyn Error>> {
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

    let sticks = get_ram_sticks()?.filter(|data| !data.is_empty());
    let (ram_types, ram_power_consumption) = match sticks {
        Some(ref sticks) if !sticks.is_empty() => {
            let types = sticks.iter().map(|s| s.ram_type.clone()).collect();
            let power = estimated_power_consumption(sticks, ram_used);
            (Some(types), power)
        }
        _ => (None, None),
    };

    Ok(MemInfo {
        ram_available,
        ram_free,
        ram_types,
        ram_power_consumption,
        ram_total: Some(ram_total),
        ram_used: Some(ram_used),
        swap_free,
        swap_total,
        swap_used,
        bandwidth_read,
        bandwidth_write,
    })
}

/// Public function used to send JSON formatted values,
/// from [`collect_mem_data`] function result.
pub fn get_mem_info() -> Result<(), Box<dyn Error>> {
    let data = collect_mem_data()?;
    let values = json!({ HEADER: data.to_json() });
    write_json_to_file(|| Ok(values), LOGGER)?;
    Ok(())
}
