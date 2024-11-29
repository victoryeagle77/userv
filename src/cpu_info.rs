//! # CPU data Module
//!
//! This module provides functionality to retrieve processor data on Unix-based systems.

use serde_json::json;
use std::fs;
use std::path::Path;
use std::thread::sleep;
use std::time::Duration;
use sysinfo::{CpuExt, System, SystemExt};

use crate::utils::{parse_file_content, read_file_content};

const CPUINFO: &str = "/proc/cpuinfo";
const THERMAL: &str = "/sys/class/thermal";

/// Retrieves the current CPU usage for all cores.
/// This function uses the `sysinfo` crate to gather CPU usage information.
/// It takes two snapshots of CPU usage with a 1-second interval between them to calculate the current usage percentage for each CPU core.
///
/// # Returns
///
/// A `Vec<f32>` where each element represents the usage percentage of a CPU core.
/// The order of the elements corresponds to the order of the CPU cores as reported by the system.
///
/// # Performance considerations
///
/// This function introduces a 1-second delay due to the sleep between CPU usage snapshots.
/// This delay is necessary to calculate an accurate usage percentage.
///
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
///
/// # Behavior
///
/// - Iterates through entries in the thermal directory (defined by `THERMAL` constant).
/// - For each thermal zone, reads the type and temperature.
/// - Displays the temperature in Celsius for each found thermal zone.
/// - If no temperatures are found or if there's an error reading the thermal directory,
///   appropriate error messages are displayed.
///
/// # Returns
///
/// A `Vec<String, f32>` where each element represents cores and its thermal state.
///
/// # Output
///
/// - Successful temperature readings are printed to stdout
/// - If no thermal zones are found or readable, "Unknown" is printed to stderr.
/// - If there's an error accessing the thermal directory, an error message is printed to stderr.
///
/// # Errors
///
/// - Errors during file reading or directory access are handled internally and do not cause
///   the function to panic. Error messages are printed to stderr.
fn get_cpu_temp() -> Vec<(String, f32)> {
    let mut temperatures = Vec::new();
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
                temperatures.push((type_content.trim().to_string(), temperature));
                found = true;
            }
        }
    } else {
        eprintln!("<ERROR_3> File : '{}'", THERMAL);
    }

    if !found {
        eprintln!("Unknown");
    }

    temperatures
}

/// # Function
///
/// Function `get_energy_consumption`
pub fn get_energy_consumption() -> Result<Vec<(f64, f64)>, std::io::Error> {
    let powercap_path = Path::new("/sys/class/powercap");
    let mut results = Vec::new();

    if !powercap_path.exists() {
        println!("Le répertoire /sys/class/powercap n'existe pas.");
        return Ok(results);
    }

    let mut energy_files = Vec::new();

    for entry in fs::read_dir(powercap_path)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            let energy_uj_path = path.join("energy_uj");
            if energy_uj_path.exists() {
                energy_files.push(energy_uj_path);
            }
        }
    }

    let mut first_readings = Vec::new();
    for path in &energy_files {
        if let Some(content) = read_file_content(path.to_str().unwrap()) {
            if let Ok(value) = content.trim().parse::<u64>() {
                first_readings.push(value);
            } else {
                eprintln!("Erreur de parsing pour {}", path.display());
                first_readings.push(0);
            }
        } else {
            first_readings.push(0);
        }
    }

    sleep(Duration::from_secs(1));

    for (i, path) in energy_files.iter().enumerate() {
        if let Some(content) = read_file_content(path.to_str().unwrap()) {
            if let Ok(second_reading) = content.trim().parse::<u64>() {
                let energy_diff_uj = second_reading.saturating_sub(first_readings[i]);
                let energy_j = energy_diff_uj as f64 / 1_000_000.0;
                let power_w = energy_j;
                results.push((energy_j, power_w));
            } else {
                eprintln!("Erreur de parsing pour {}", path.display());
            }
        }
    }

    Ok(results)
}

/// Public function get_cpu_info
/// Retrieves detailed CPU data.
///
/// # Output
///
/// The function retrieves the following data :
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
pub fn get_cpu_info() {
    println!("\n[[ CPU ]]\n");

    let data = parse_file_content(CPUINFO, ":");
    let cpu_usage = get_cpu_usage().into_iter().collect::<Vec<_>>();
    let cpu_temps = get_cpu_temp().into_iter().collect::<Vec<_>>();
    let get_value = |key: &str| {
        data.iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v.as_str())
            .unwrap_or("Unknown")
    };

    let model_name = get_value("model name");
    let vendor_id = get_value("vendor_id");
    let cpu_family = get_value("cpu family");
    let stepping = get_value("stepping");
    let microcode = get_value("microcode");
    let frequency = get_value("cpu MHz");
    let cache_size = get_value("cache size");
    let address_sizes = get_value("address sizes");
    let cpuid_level = get_value("cpuid level");
    let physical_cores = num_cpus::get_physical();
    let logical_cores = num_cpus::get();

    // Build of final Json object
    let cpu_json_info = json!({
        "CPU": {
            "model_name": model_name,
            "vendor_id": vendor_id,
            "cpu_family": cpu_family,
            "revision": stepping,
            "microcode": microcode,
            "frequency": format!("{} MHz", frequency),
            "cache_size": cache_size,
            "address_sizes": address_sizes,
            "cpuid_level": cpuid_level,
            "physical_cores": physical_cores,
            "logical_cores": logical_cores,
            "usage_cores": cpu_usage,
            "temperatures": cpu_temps,
        }
    });

    println!("{}", serde_json::to_string_pretty(&cpu_json_info).unwrap());
}
