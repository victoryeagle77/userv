//! # CPU data Module
//!
//! This module provides functionality to retrieve processor data on Unix-based systems.

use log::error;
use serde::Serialize;
use serde_json::json;
use std::{fs, thread::sleep, time::Duration, time::Instant};
use sysinfo::{CpuExt, System, SystemExt};

use crate::utils::{parse_file_content, read_file_content, write_json_to_file};

const CPUINFO: &str = "/proc/cpuinfo";
const THERMAL: &str = "/sys/class/thermal";
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
    /// CPU microcode revision.
    rev: Option<String>,
    /// CPU operating frequency in Mhz.
    freq: Option<String>,
    /// CPU memory cache size.
    cache: Option<String>,
    /// CPU address sizes available.
    addr: Option<String>,
    /// Physical CPU cores.
    phy_cores: Option<usize>,
    /// Logical CPU cores.
    lgc_cores: Option<usize>,
    /// CPU usage cores in percentage.
    use_cores: Option<Vec<f32>>,
    /// CPU temperatures by zone in °C.
    temp: Option<Vec<(String, f32)>>,
    /// CPU energy consumption by zone in uJ.
    pwr: Option<Vec<(String, f64)>>,
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
fn get_cpu_usage() -> Vec<f32> {
    let mut sys: System = System::new_all();

    sys.refresh_cpu();
    sleep(Duration::from_secs(1));
    sys.refresh_cpu();

    let cpus: &[sysinfo::Cpu] = sys.cpus();

    if cpus.is_empty() {
        error!("[CPU_CORE_USAGE] Data 'Unable to get CPU core information'");
        return Vec::new();
    }

    cpus.iter()
        .enumerate()
        .filter_map(|(index, cpu)| {
            let usage: f32 = cpu.cpu_usage();
            if usage.is_nan() || usage.is_infinite() {
                error!("[CPU_CORE_USAGE] Data 'Unable to get valid CPU usage for core {index}'");
                None
            } else {
                Some(usage)
            }
        })
        .collect()
}

/// Retrieves and displays CPU temperature information from the system.
/// This function scans the thermal zones in the system (typically located in `/sys/class/thermal`)
/// and attempts to read and display the temperature for each zone that starts with "thermal_zone".
///
/// # Return
///
/// - `result` : Vector where each element represents cores and its thermal state in Celsius.
/// - An empty vector if no thermal files or data are found.
fn get_cpu_temp() -> Vec<(String, f32)> {
    let mut result = Vec::new();
    let mut found: bool = false;

    if let Ok(entries) = fs::read_dir(THERMAL) {
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_dir()
                || !path
                    .file_name()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .starts_with("thermal_zone")
            {
                continue;
            }

            let type_content = match read_file_content(path.join("type").to_str().unwrap()) {
                Some(content) => content,
                None => continue,
            };

            let temp_content = match read_file_content(path.join("temp").to_str().unwrap()) {
                Some(content) => content,
                None => continue,
            };

            if let Ok(temp) = temp_content.trim().parse::<i32>() {
                let temperature = temp as f32 / 1e3;
                result.push((type_content.trim().to_string(), temperature));
                found = true;
            }
        }
    } else {
        error!("[CPU_TEMPERATURE] File '{THERMAL}'");
    }

    if !found || result.is_empty() {
        error!("[CPU_TEMPERATURE] Data 'Unable to get valid CPU temperature information'");
    }

    result
}

/// Function reading in RAPL directory : `/sys/class/powercap/`,
/// to get consumption data in locate each CPU zone to get specific energy consumption.
///
/// # Return
///
/// - `result` : Vector containing CPU zone name and its consumption.
/// - An empty vector if no energy consumption file or data are found.
fn get_cpu_consumption() -> Vec<(String, f64)> {
    let mut result = Vec::new();
    let start_time = Instant::now();

    if let Ok(entries) = fs::read_dir(RAPL) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let domain = path.file_name().unwrap().to_str().unwrap();
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

    if result.is_empty() {
        error!(
            "[CPU_POWER_CONSUMPTION] Data 'Unable to get valid CPU power consumption information'"
        );
    }

    result
}

/// Public function reading and using `/proc/cpuinfo` file values,
/// and retrieves detailed CPU data.
///
/// # Return
///
/// `result` : Completed `CpuInfo` structure with all cpu information
/// - CPU full model name
/// - CPU maker identification
/// - CPU general generation
/// - CPU family specific model number
/// - CPU revision number that indicates minor changes or corrections to the model
/// - CPU operating frequency
/// - CPU cache size which is fast memory built into the processor to speed up operations
/// - CPU address sizes
/// - CPU physical cores that are the actual processing units on the chip
/// - CPU ID level which is the maximum option set that you can safely use to interrogate the processor for information
/// - CPU logical cores that includes physical and virtual cores
/// - CPU detailed core usage
fn collect_cpu_data() -> Result<CpuInfo, String> {
    let cpu_usage = get_cpu_usage();
    let cpu_temps = get_cpu_temp();
    let cpu_pwr = get_cpu_consumption();

    let cpu_value = |key: &str| -> Option<String> {
        parse_file_content(CPUINFO, ":")
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v.trim().to_string())
            .filter(|v| !v.is_empty() && v != "NULL")
    };

    let result: CpuInfo = CpuInfo {
        model: cpu_value("model name"),
        family: cpu_value("cpu family"),
        rev: cpu_value("stepping"),
        freq: cpu_value("cpu MHz"),
        cache: cpu_value("cache size"),
        addr: cpu_value("address sizes"),
        phy_cores: Some(num_cpus::get_physical()).filter(|&cores| cores > 0),
        lgc_cores: Some(num_cpus::get()).filter(|&cores| cores > 0),
        use_cores: if cpu_usage.is_empty() {
            None
        } else {
            Some(cpu_usage)
        },
        temp: if cpu_temps.is_empty() {
            None
        } else {
            Some(cpu_temps)
        },
        pwr: if cpu_pwr.is_empty() {
            None
        } else {
            Some(cpu_pwr)
        },
    };

    Ok(result)
}

/// Public function used to send JSON formatted values,
/// from `collect_cpu_data` function result.
pub fn get_cpu_info() {
    let data = || {
        let values: CpuInfo = collect_cpu_data()?;
        Ok(json!({ HEADER: values }))
    };

    write_json_to_file(data, LOGGER, HEADER);
}
