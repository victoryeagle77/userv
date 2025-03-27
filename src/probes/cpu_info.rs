//! # CPU data Module
//!
//! This module provides functionality to retrieve processor data on Unix-based systems.

use log::error;
use serde::Serialize;
use serde_json::{json, Value};
use std::{
    error::Error,
    ffi::OsStr,
    fs,
    path::PathBuf,
    thread::sleep,
    time::{Duration, Instant},
};
use sysinfo::{Component, ComponentExt, CpuExt, System, SystemExt};

use crate::utils::{read_file_content, write_json_to_file};

const RAPL: &str = "/sys/class/powercap";

const HEADER: &str = "CPU";
const LOGGER: &str = "log/cpu_data.json";

/// Collection of collected CPU data
#[derive(Debug, Serialize)]
struct CpuInfo {
    /// Model name.
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
    cores_usage: Option<Vec<f32>>,
    /// CPU temperatures by zone in °C.
    temperature: Option<Vec<(String, f32)>>,
    /// CPU energy consumption by zone in uJ.
    power: Option<Vec<(String, f64)>>,
}

impl CpuInfo {
    /// Converts the `CpuInfo` structure into a JSON value.
    fn to_json(&self) -> Value {
        json!({
            "model": self.model,
            "family": self.family,
            "frequency_MHz": self.frequency,
            "physical_cores": self.cores_physic,
            "logical_cores": self.cores_logic,
            "core_usage_%": self.cores_usage,
            "temperatures_°C": self.temperature,
            "power_consumption_W": self.power,
        })
    }
}

/// Retrieves the current CPU usage for all cores.
/// This function uses the `sysinfo` crate to gather CPU usage information.
/// It takes two snapshots of CPU usage with a 1-second interval between them,
/// to calculate the current usage percentage for each CPU core.
///
/// # Return
///
/// A vector where each element represents the usage percentage of a CPU core.
/// The order of the elements corresponds to the order of the CPU cores as reported by the system.
/// If resources are not accessible, return an empty vector and log the error.
///
/// # Performance considerations
///
/// This function introduces a 1-second delay due to the sleep between CPU usage snapshots.
/// This delay is necessary to calculate an accurate usage percentage.
fn get_cpu_usage() -> Result<Vec<f32>, String> {
    let mut sys: System = System::new_all();
    sys.refresh_cpu();
    sleep(Duration::from_secs(1));
    sys.refresh_cpu();

    let cpus: &[sysinfo::Cpu] = sys.cpus();

    if cpus.is_empty() {
        error!("[{HEADER}] Data 'Unable to get CPU core information'");
        return Err("Unable to get CPU core information".to_string());
    }

    Ok(cpus
        .iter()
        .enumerate()
        .filter_map(|(index, cpu)| {
            let usage: f32 = cpu.cpu_usage();
            if usage.is_nan() || usage.is_infinite() {
                error!("[{HEADER}] Data 'Invalid CPU usage for core {index}'");
                None
            } else {
                Some(usage)
            }
        })
        .collect())
}

/// Retrieves and displays CPU temperature information from the system.
/// This function scans the thermal zones in the system (typically located in `/sys/class/thermal`)
/// and attempts to read and display the temperature for each zone that starts with "thermal_zone".
///
/// # Return
///
/// - `result` : Vector where each element represents cores and its thermal state in Celsius.
/// - An empty vector if no thermal files or data are found.
fn get_cpu_temp() -> Result<Vec<(String, f32)>, String> {
    let mut sys: System = System::new_all();
    sys.refresh_components();

    let temps: Vec<(String, f32)> = sys
        .components()
        .iter()
        .filter_map(|component: &Component| {
            let name: String = component.label().to_string();
            let temperature: f32 = component.temperature();
            if name.to_lowercase().contains("cpu") || name.to_lowercase().contains("core") {
                Some((name, temperature))
            } else {
                None
            }
        })
        .collect();

    if temps.is_empty() {
        error!("[{HEADER}] Data 'Unable to get valid CPU temperature information'");
        return Err("Unable to get valid CPU temperature information".to_string());
    }

    Ok(temps)
}

/// Function reading in RAPL directory : `/sys/class/powercap/`,
/// to get consumption data in locate each CPU zone to get specific energy consumption.
///
/// # Return
///
/// - `result` : Vector containing CPU zone name and its consumption.
/// - An empty vector if no energy consumption file or data are found.
fn get_cpu_consumption() -> Result<Vec<(String, f64)>, String> {
    let mut result: Vec<(String, f64)> = Vec::new();
    let start_time: Instant = Instant::now();

    if let Ok(entries) = fs::read_dir(RAPL) {
        for entry in entries.flatten() {
            let path: PathBuf = entry.path();
            if path.is_dir() {
                if let Some(domain) = path.file_name().and_then(|name: &OsStr| name.to_str()) {
                    if domain.starts_with("intel-rapl:") {
                        if let Some(start_energy) =
                            read_file_content(path.join("energy_uj").to_str().unwrap())
                                .and_then(|content: String| content.trim().parse::<f64>().ok())
                        {
                            sleep(Duration::from_secs(1));
                            if let Some(end_energy) =
                                read_file_content(path.join("energy_uj").to_str().unwrap())
                                    .and_then(|content: String| content.trim().parse::<f64>().ok())
                            {
                                let elapsed: f64 = start_time.elapsed().as_secs_f64();
                                let power: f64 =
                                    (end_energy - start_energy) / (elapsed * 1_000_000.0);
                                result.push((domain.to_string(), power));
                            }
                        }
                    }
                }
            }
        }
    }

    if result.is_empty() {
        error!("[{HEADER}] Data 'Unable to get valid CPU power consumption information'");
        return Err("Unable to get valid CPU power consumption information".to_string());
    }

    Ok(result)
}

/// Public function reading and using `/proc/cpuinfo` file values,
/// and retrieves detailed CPU data.
///
/// # Return
///
/// `result` : Completed `CpuInfo` structure with all cpu information
/// - CPU full model name
/// - CPU general generation
/// - CPU family specific model number
/// - CPU operating frequency
/// - CPU physical cores that are the actual processing units on the chip
/// - CPU logical cores that includes physical and virtual cores
/// - CPU detailed core usage
fn collect_cpu_data() -> CpuInfo {
    let mut sys: System = System::new_all();
    sys.refresh_cpu();

    let cpu: Option<&sysinfo::Cpu> = sys.cpus().first();

    CpuInfo {
        model: cpu.map(|c: &sysinfo::Cpu| c.brand().to_string()),
        family: cpu.map(|c: &sysinfo::Cpu| c.vendor_id().to_string()),
        frequency: cpu.map(|c: &sysinfo::Cpu| c.frequency().to_string()),
        cores_physic: sys.physical_core_count(),
        cores_logic: Some(sys.cpus().len()),
        cores_usage: match get_cpu_usage() {
            Ok(data) => Some(data),
            Err(e) => {
                error!("[{HEADER}] Data '{e}'");
                None
            }
        },
        temperature: match get_cpu_temp() {
            Ok(data) => Some(data),
            Err(e) => {
                error!("[{HEADER}] Data '{e}'");
                None
            }
        },
        power: match get_cpu_consumption() {
            Ok(data) => Some(data),
            Err(e) => {
                error!("[{HEADER}] Data '{e}'");
                None
            }
        },
    }
}

/// Public function used to send JSON formatted values,
/// from `collect_cpu_data` function result.
pub fn get_cpu_info() {
    let data = || -> Result<Value, Box<dyn Error>> {
        let values: CpuInfo = collect_cpu_data();
        Ok(json!({ HEADER: values.to_json() }))
    };

    write_json_to_file(data, LOGGER, HEADER);
}
