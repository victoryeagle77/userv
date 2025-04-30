//! # CPU data Module
//!
//! This module provides functionality to retrieve processor data on Unix-based systems.

use log::error;
use serde::Serialize;
use serde_json::{json, Value};
use std::{
    error::Error,
    fs::{read_dir, read_to_string},
    path::Path,
    thread::sleep,
    time::{Duration, Instant},
};
use sysinfo::{Components, Cpu, CpuRefreshKind, RefreshKind, System};

use crate::utils::write_json_to_file;

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
    temperature: Option<Vec<(String, f32)>>,
    /// CPU energy consumption by zone in uJ.
    power: Option<Vec<(String, f64)>>,
}

impl CpuInfo {
    /// Converts the [`CpuInfo`] structure into a JSON value.
    fn to_json(&self) -> Value {
        json!({
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
fn get_cpu_usage(cpus: &[Cpu]) -> Result<Vec<(String, f32)>, Box<dyn Error>> {
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
        Err("Data 'Unable to get CPU usage information'".into())
    } else {
        Ok(result)
    }
}

/// Retrieves CPU temperature information from the system.
/// This function scans the thermal zones in the system (typically located in `/sys/class/thermal`)
/// and attempts to read and display the temperature for each zone that starts with "thermal_zone".
///
/// # Return
///
/// - `result` : Vector where each element represents cores and its thermal state in Celsius.
/// - An empty vector if no thermal files or data are found.
fn get_cpu_temperature() -> Result<Vec<(String, f32)>, Box<dyn Error>> {
    let components = Components::new_with_refreshed_list();

    let result = components
        .iter()
        .filter_map(|component| {
            let name = component.label().to_string();
            let temperature = component.temperature();

            if let Some(temp) = temperature {
                if !temp.is_nan() {
                    Some((name, temp))
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
        Err("Data 'Unable to get CPU temperature information'".into())
    } else {
        Ok(result)
    }
}

/// Function reading in RAPL directory `/sys/class/powercap/`,
/// to get consumption data in locate each CPU zone to get specific energy consumption.
///
/// # Return
///
/// - `result` : Vector containing CPU zone name and its consumption.
/// - An empty vector if no energy consumption file or data are found.
fn get_cpu_consumption() -> Result<Vec<(String, f64)>, Box<dyn Error>> {
    fn read_energy(path: &Path) -> Result<f64, Box<dyn Error>> {
        let content = read_to_string(path)?;
        let energy = content.trim().parse::<f64>()?;
        Ok(energy)
    }

    let mut result = Vec::new();

    for entry in read_dir(RAPL)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            if let Some(domain) = path.file_name().and_then(|n| n.to_str()) {
                if domain.starts_with("intel-rapl:") {
                    let energy_path = path.join("energy_uj");

                    // Initial energy
                    let start_energy = read_energy(&energy_path)?;

                    // Start measure between two reading values
                    let start_time = Instant::now();
                    sleep(Duration::from_secs(1));
                    let end_energy = read_energy(&energy_path)?;
                    let elapsed = start_time.elapsed().as_secs_f64();

                    // Compute power consumed in Watts with energy in microJoules
                    let power = (end_energy - start_energy) / (elapsed * 1e6);

                    result.push((domain.to_string(), power));
                }
            }
        }
    }

    if result.is_empty() {
        Err("Data 'Unable to get CPU power consumption information'".into())
    } else {
        Ok(result)
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

    let cores_physic = sys.physical_core_count();
    let cores_logic = Some(cpus.len());

    let model = cpus.first().map(|c| c.brand().to_string());
    let family = cpus.first().map(|c| c.vendor_id().to_string());
    let frequency = cpus.first().map(|c| c.frequency().to_string());

    let cores_usage = Some(get_cpu_usage(cpus)?);
    let power = Some(get_cpu_consumption()?);
    let temperature = Some(get_cpu_temperature()?);

    Ok(CpuInfo {
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
