//! # CPU data Module
//!
//! This module provides functionality to retrieve processor data on Unix-based systems.

use serde::Serialize;
use serde_json::json;
use std::{fs, thread::sleep, time::Duration, time::Instant};
use sysinfo::{CpuExt, System, SystemExt};

use crate::utils::{parse_file_content, read_file_content};

const CPUINFO: &str = "/proc/cpuinfo";
const THERMAL: &str = "/sys/class/thermal";
const RAPL: &str = "/sys/class/powercap/";

const HEADER: &str = "CPU";

/// Collection of collected CPU data
#[derive(Debug, Serialize)]
struct CpuInfo {
    /// Model name.
    model: String,
    /// Vendor ID.
    id: String,
    /// CPU generation.
    family: String,
    /// CPU microcode revision.
    rev: String,
    /// CPU microcode version.
    code: String,
    /// CPU frequency.
    freq: String,
    /// CPU memory cache size.
    cache: String,
    /// CPU address sizes available.
    addr: String,
    /// Physical CPU cores.
    phy_cores: usize,
    /// Logical CPU cores.
    lgc_cores: usize,
    /// CPU usage cores.
    use_cores: Vec<f32>,
    /// CPU temperatures by zone.
    temp: Vec<(String, f32)>,
    /// CPU energy consumption by zone.
    pwr: Vec<(String, f64)>,
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
///
/// # Performance considerations
///
/// This function introduces a 1-second delay due to the sleep between CPU usage snapshots.
/// This delay is necessary to calculate an accurate usage percentage.
fn get_cpu_usage() -> Vec<f32> {
    let mut sys = System::new_all();
    sys.refresh_cpu();
    sleep(Duration::from_secs(1));
    sys.refresh_cpu();
    return sys.cpus().iter().map(|cpu| cpu.cpu_usage()).collect();
}

/// Retrieves and displays CPU temperature information from the system.
/// This function scans the thermal zones in the system (typically located in `/sys/class/thermal`)
/// and attempts to read and display the temperature for each zone that starts with "thermal_zone".
/// - Iterates through entries in the thermal directory (defined by `THERMAL` constant).
/// - For each thermal zone, reads the type and temperature.
/// - Displays the temperature in Celsius for each found thermal zone.
/// - If no temperatures are found or if there's an error reading the thermal directory,
///   appropriate error messages are displayed.
///
/// # Return
///
/// `temperatures` : Vector where each element represents cores and its thermal state.
fn get_cpu_temp() -> Vec<(String, f32)> {
    let mut result = Vec::new();
    let mut found = false;

    if let Ok(entries) = std::fs::read_dir(THERMAL) {
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
                let temperature = temp as f32 / 1000.0;
                result.push((type_content.trim().to_string(), temperature));
                found = true;
            }
        }
    } else {
        eprintln!("<ERROR_3> File : '{}'", THERMAL);
    }

    if !found {
        result.push(("NULL".to_string(), 0.0));
    }

    result
}

/// Function reading in RAPL directory to get consumption data.
/// Its locate each CPU zone to get specific energy consumption.
/// 
/// # Return
/// 
/// - `power_readings` : Vector containing CPU zone name and its consumption
fn get_cpu_consumption() -> Vec<(String, f64)> {
    let mut result = Vec::new();
    let start_time = Instant::now();

    if let Ok(entries) = fs::read_dir(RAPL) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let domain = path.file_name().unwrap().to_str().unwrap();
                if domain.starts_with("intel-rapl:") {
                    if let Some(start_energy) = read_file_content(path.join("energy_uj").to_str().unwrap())
                        .and_then(|content| content.trim().parse::<f64>().ok()) 
                    {
                        std::thread::sleep(std::time::Duration::from_secs(1));
                        if let Some(end_energy) = read_file_content(path.join("energy_uj").to_str().unwrap())
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
        result.push(("NULL".to_string(), 0.0));
    }

    result
}

/// Public function that retrieves detailed CPU data.
///
/// # Return
///
/// `result` : Completed `CpuInfo` structure with all cpu information
/// - CPU full model name
/// - CPU maker identification
/// - CPU general generation
/// - CPU family specific model number
/// - CPU revision number that indicates minor changes or corrections to the model
/// - CPU microcode version which is a set of instructions that controls the operation
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
    let cpu_value = |key: &str| {
        parse_file_content(CPUINFO, ":")
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v.to_string())
            .unwrap_or_else(|| "NULL".to_string())
    };

    let result = CpuInfo {
        model: cpu_value("model name"),
        id: cpu_value("vendor_id"),
        family: cpu_value("cpu family"),
        rev: cpu_value("stepping"),
        code: cpu_value("microcode"),
        freq: format!("{} MHz", cpu_value("cpu MHz")),
        cache: cpu_value("cache size"),
        addr: cpu_value("address sizes"),
        phy_cores: num_cpus::get_physical(),
        lgc_cores: num_cpus::get(),
        use_cores: cpu_usage,
        temp: cpu_temps,
        pwr: cpu_pwr,
    };

    Ok(result)
}

/// Public function used to send JSON formatted values,
/// from `collect_cpu_data` function result.
pub fn get_cpu_info() {
    let values = collect_cpu_data();
    let cpu_json_info: serde_json::Value = json!({ HEADER: values });
    println!("{}", serde_json::to_string_pretty(&cpu_json_info).unwrap());
}
