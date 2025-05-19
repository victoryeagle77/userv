//! # Lib file for CPU data module
//!
//! This module provides functionalities to retrieve processor data on Unix-based systems.

use serde::Serialize;
use serde_json::{json, Value};
use std::{error::Error, thread::sleep};
use sysinfo::{CpuRefreshKind, RefreshKind, System};

mod utils;
use crate::utils::*;

/// Collection of collected CPU data
#[derive(Debug, Serialize)]
struct CpuInfo {
    /// CPU architecture label
    architecture: Option<String>,
    /// CPU model name.
    model: Option<String>,
    /// CPU generation.
    family: Option<String>,
    /// CPU operating frequency in Mhz.
    frequency: Option<String>,
    /// Physical CPU cores.
    cores_physic: Option<usize>,
    /// Logical CPU cores.
    cores_logic: Option<usize>,
    /// CPU usage cores in percentage.
    cores_usage: Option<Vec<(String, f32)>>,
    /// CPU temperatures by zone in °C.
    temperature: Option<Vec<(String, f32)>>,
    /// CPU energy consumption by zone in uJ.
    power: Option<Vec<(String, f64)>>,
}

impl CpuInfo {
    /// Converts the [`CpuInfo`] structure into a JSON value.
    fn to_json(&self) -> Value {
        json!({
            "architectrue": self.architecture,
            "cores_physical": self.cores_physic,
            "cores_logical": self.cores_logic,
            "core_usage_%": self.cores_usage,
            "family": self.family,
            "frequency_MHz": self.frequency,
            "model": self.model,
            "power_consumption_W": self.power,
            "temperatures_°C": self.temperature,
        })
    }
}

/// Public function reading and using `/proc/cpuinfo` file values,
/// and retrieves detailed CPU data.
///
/// # Return
///
/// - Completed [`CpuInfo`] structure with all retrieved and computing CPU information.
/// - An error when some important and critical metrics can't be retrieved.
fn collect_cpu_data() -> Result<CpuInfo, Box<dyn Error>> {
    let mut sys =
        System::new_with_specifics(RefreshKind::nothing().with_cpu(CpuRefreshKind::everything()));
    // Wait a bit because CPU usage is based on diff.
    sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL);
    // Refresh CPUs again to get actual value.
    sys.refresh_cpu_all();

    let cpus = sys.cpus();
    if cpus.is_empty() {
        return Err("Failed to get global CPUs information".to_string().into());
    }

    let cores_physic = System::physical_core_count();
    let cores_logic = Some(cpus.len());

    let architecture = Some(System::cpu_arch());
    let model = cpus.first().map(|c| c.brand().to_string());
    let family = cpus.first().map(|c| c.vendor_id().to_string());
    let frequency = cpus.first().map(|c| c.frequency().to_string());

    let cores_usage = Some(get_cpu_usage(cpus)?);
    let temperature = Some(get_cpu_temperature()?);

    let power = get_rapl_consumption();

    Ok(CpuInfo {
        architecture,
        cores_physic,
        cores_logic,
        cores_usage,
        family,
        frequency,
        model,
        power,
        temperature,
    })
}

/// Public function used to send JSON formatted values,
/// from [`collect_cpu_data`] function result.
pub fn get_cpu_info() -> Result<(), Box<dyn Error>> {
    let data = collect_cpu_data()?;
    let values = json!({ HEADER: data.to_json() });
    write_json_to_file(|| Ok(values), LOGGER)?;
    Ok(())
}
