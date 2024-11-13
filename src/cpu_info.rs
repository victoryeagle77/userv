//! # CPU data Module
//!
//! This module provides functionality to retrieve processor data on Unix-based systems.

use std::time::Duration;
use std::thread::sleep;
use colored::Colorize;
use sysinfo::{System, SystemExt, CpuExt};
use raw_cpuid::CpuId;
use serde_json::json;

use crate::utils::{read_file_content, parse_file_content};

const CPUINFO: &str = "/proc/cpuinfo";
const THERMAL: &str = "/sys/class/thermal";

/// Function get_cpu_usage
/// Retrieves the current CPU usage for all cores.
///
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
/// # Dependencies
///
/// This function relies on the `sysinfo` crate for system information retrieval
/// and the `std::thread::sleep` function for introducing the delay.
///
fn get_cpu_usage() -> Vec<f32> {
    let mut sys = System::new_all();
    sys.refresh_cpu();
    sleep(Duration::from_secs(1));
    sys.refresh_cpu();
    return sys.cpus().iter().map(|cpu| cpu.cpu_usage()).collect();
}

/// Function get_cpu_temp
/// Retrieves and displays CPU temperature information from the system.
///
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
///
/// # Dependencies
///
/// - Relies on the `read_file_content` function to read file contents.
/// - Uses the `THERMAL` constant which should be defined elsewhere in the code.
///
fn get_cpu_temp() -> Vec<(String, f32)> {
    let mut temperatures = Vec::new();
    let mut found = false;
    
    if let Ok(entries) = std::fs::read_dir(THERMAL) {
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_dir() || !path.file_name().unwrap().to_str().unwrap().starts_with("thermal_zone") {
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
        eprintln!("{} File : '{}'", "<ERROR_3>".red().bold(), THERMAL);
    }

    if !found {
        eprintln!("Unknown");
    }

    return temperatures;
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
///
pub fn get_cpu_info() {
    println!("{}", "\n[[ CPU ]]\n".magenta().bold());

    let data = parse_file_content(CPUINFO, ":");
    let cpu_usage = get_cpu_usage().into_iter().collect::<Vec<_>>();
    let cpu_temps = get_cpu_temp().into_iter().collect::<Vec<_>>();
    let get_value = |key: &str| data.iter().find(|(k, _)| k == key).map(|(_, v)| v.as_str()).unwrap_or("Unknown");

    let model_name = get_value("model name");
    let vendor_id = get_value("vendor_id");
    let cpu_family = get_value("cpu family");
    let model = get_value("model");
    let stepping = get_value("stepping");
    let microcode = get_value("microcode");
    let frequency = get_value("cpu MHz");
    let cache_size = get_value("cache size");
    let address_sizes = get_value("address sizes");
    let cpuid_level = get_value("cpuid level");
    let physical_cores = num_cpus::get_physical();
    let logical_cores = num_cpus::get();

    let cpuid = CpuId::new();
    let mut topology = Vec::new();
    
    if let Some(topo) = cpuid.get_extended_topology_info() {
        for level in topo {
            topology.push(json!({
                "level": level.level_number(),
                "type": format!("{:?}", level.level_type()),
                "core": level.processors(),
            }));
        }
    }

    // Build of final Json object
    let cpu_json_info = json!({
        "CPU": {
            "model_name": model_name,
            "vendor_id": vendor_id,
            "cpu_family": cpu_family,
            "model": model,
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
            "topology": topology,
        }
    });

    println!("{}", serde_json::to_string_pretty(&cpu_json_info).unwrap());
}
