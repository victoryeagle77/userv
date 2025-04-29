//! # CPU data Module
//!
//! This module provides functionality to retrieve processor data on Unix-based systems.

use log::error;
use serde::Serialize;
use serde_json::{json, Value};
use std::{
    error::Error,
    fs::read_dir,
    thread::{self, sleep},
    time::{Duration, Instant},
};
use sysinfo::{Components, CpuRefreshKind, RefreshKind, System};

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
    cores_usage: Option<Vec<(String, f32)>>,
    /// CPU temperatures by zone in °C.
    temperature: Option<Vec<(String, Option<f32>)>>,
    /// CPU energy consumption by zone in uJ.
    power: Option<Vec<(String, f64)>>,
}

impl CpuInfo {
    /// Converts the [`CpuInfo`] structure into a JSON value.
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

/// Retrieves the current CPU usage by cores.
/// This function uses the `sysinfo` crate to gather CPU usage information.
/// It takes two snapshots of CPU usage with a 1-second interval between them,
/// to calculate the current usage percentage for each CPU core.
///
/// # Return
///
/// - `result` : Vector where each element represents cores and its usage in percentage.
/// - An empty vector if no thermal files or data are found.
///
/// # Performance considerations
///
/// This function introduces a [`sysinfo::MINIMUM_CPU_UPDATE_INTERVAL`] delay due to the sleep between CPU usage snapshots.
/// This delay is necessary to calculate an accurate usage percentage.
fn get_cpu_usage() -> Result<Vec<(String, f32)>, Box<dyn Error>> {
    let mut sys =
        System::new_with_specifics(RefreshKind::nothing().with_cpu(CpuRefreshKind::everything()));
    // Wait a bit because CPU usage is based on diff.
    thread::sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL);
    // Refresh CPUs again to get actual value.
    sys.refresh_cpu_usage();

    let cpus = sys.cpus();

    if cpus.is_empty() {
        return Err("Unable to get CPU core".to_string().into());
    }

    let result = cpus
        .iter()
        .enumerate()
        .filter_map(|(core, cpu)| {
            let usage = cpu.cpu_usage();
            let name = cpu.name().to_string();

            if usage.is_nan() || usage.is_infinite() {
                error!("[{HEADER}] Data 'Invalid CPU usage for core {core}'");
                None
            } else {
                Some((name, usage))
            }
        })
        .collect::<Vec<_>>();

    if result.is_empty() {
        return Err("Unable to get CPU usage information".to_string().into());
    }

    Ok(result)
}

/// Retrieves CPU temperature information from the system.
/// This function scans the thermal zones in the system (typically located in `/sys/class/thermal`)
/// and attempts to read and display the temperature for each zone that starts with "thermal_zone".
///
/// # Return
///
/// - `result` : Vector where each element represents cores and its thermal state in Celsius.
/// - An empty vector if no thermal files or data are found.
fn get_cpu_temp() -> Result<Vec<(String, Option<f32>)>, Box<dyn Error>> {
    let components = Components::new_with_refreshed_list();

    let result = components
        .iter()
        .filter_map(|component| {
            let name = component.label().to_string();
            let temperature = component.temperature();

            if let Some(temp) = temperature {
                if !temp.is_nan() {
                    Some((name, Some(temp)))
                } else {
                    error!("[{HEADER}] Data 'Unable to get value for thermal zone ({name})'");
                    None
                }
            } else {
                error!("[{HEADER}] Data 'Invalid temperature value for thermal zone ({name})'");
                None
            }
        })
        .collect::<Vec<_>>();

    if result.is_empty() {
        return Err("Unable to get CPU temperature information"
            .to_string()
            .into());
    }

    Ok(result)
}

/// Function reading in RAPL directory : `/sys/class/powercap/`,
/// to get consumption data in locate each CPU zone to get specific energy consumption.
///
/// # Return
///
/// - `result` : Vector containing CPU zone name and its consumption.
/// - An empty vector if no energy consumption file or data are found.
fn get_cpu_consumption() -> Result<Vec<(String, f64)>, Box<dyn Error>> {
    let mut result = Vec::new();
    let start_time = Instant::now();

    if let Ok(entries) = read_dir(RAPL) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if let Some(domain) = path.file_name().and_then(|name| name.to_str()) {
                    if domain.starts_with("intel-rapl:") {
                        if let Some(start_energy) =
                            read_file_content(path.join("energy_uj").to_str().unwrap())
                                .and_then(|content| content.trim().parse::<f64>().ok())
                        {
                            sleep(Duration::from_secs(1));
                            if let Some(end_energy) =
                                read_file_content(path.join("energy_uj").to_str().unwrap())
                                    .and_then(|content| content.trim().parse::<f64>().ok())
                            {
                                let elapsed = start_time.elapsed().as_secs_f64();
                                let power = (end_energy - start_energy) / (elapsed * 1_000_000.0);
                                result.push((domain.to_string(), power));
                            }
                        }
                    }
                }
            }
        }
    }

    if result.is_empty() {
        return Err("Unable to get CPU power consumption information"
            .to_string()
            .into());
    }

    Ok(result)
}

/// Public function reading and using `/proc/cpuinfo` file values,
/// and retrieves detailed CPU data.
///
/// # Return
///
/// - Completed [`CpuInfo`] structure with all retrieved and computing CPU information.
/// - An error when some important and critical metrics can't be retrieved.
fn collect_cpu_data() -> Result<CpuInfo, Box<dyn Error>> {
    let mut sys: System = System::new_all();
    sys.refresh_cpu_all();

    let cpu = sys.cpus().first();

    Ok(CpuInfo {
        model: cpu.map(|c| c.brand().to_string()),
        family: cpu.map(|c| c.vendor_id().to_string()),
        frequency: cpu.map(|c| c.frequency().to_string()),
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
