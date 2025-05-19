//! # File utilities module

use chrono::{SecondsFormat::Millis, Utc};
use log::error;
use serde_json::{json, Value};
use std::{
    error::Error,
    fs::OpenOptions,
    fs::{read_dir, read_to_string},
    io::Write,
    path::Path,
    thread::sleep,
    time::{Duration, Instant},
};
use sysinfo::{Components, Cpu};

pub const HEADER: &'static str = "CPU";
pub const LOGGER: &'static str = "log/cpu_data.json";

const RAPL: &'static str = "/sys/class/powercap";

/// Retrieves the current CPU usage by cores.
/// This function uses the `sysinfo` crate to gather CPU usage information.
/// It takes two snapshots of CPU usage with a 1-second interval between them,
/// to calculate the current usage percentage for each CPU core.
///
/// # Return
///
/// - `result` : Vector where each element represents cores and its usage in percentage.
/// - An error if CPU usage data are not found.
///
/// # Performance considerations
///
/// This function introduces a [`sysinfo::MINIMUM_CPU_UPDATE_INTERVAL`] delay due to the sleep between CPU usage snapshots.
/// This delay is necessary to calculate an accurate usage percentage.
pub fn get_cpu_usage(cpus: &[Cpu]) -> Result<Vec<(String, f32)>, Box<dyn Error>> {
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

/// Retrieves CPU temperature information from the system,
/// and attempts to read and store the temperature for each zone that starts with "thermal_zone".
///
/// # Return
///
/// - `result` : Vector where each element represents cores and its thermal state in Celsius.
/// - An error if CPU thermal data are not found.
pub fn get_cpu_temperature() -> Result<Vec<(String, f32)>, Box<dyn Error>> {
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

/// Reading in RAPL directory `/sys/class/powercap/`,
/// to get consumption data in locate each CPU zone to get specific energy consumption.
///
/// # Return
///
/// - `result` : Vector containing CPU zone name and its consumption.
/// - An empty vector if no energy consumption file or data are found.
pub fn get_rapl_consumption() -> Option<Vec<(String, f64)>> {
    /// Read in RAPL the energy in RAPL domain folder.
    ///
    /// # Arguments
    ///
    /// - `path` : Files in RAPL folder domain.
    ///
    /// # Returns
    ///
    /// - `energy` : The energy information in microJoules in RAPL domain folder.
    /// - An error when we can't to retrieve properly the energy data.
    fn read_rapl(path: &Path) -> Option<f64> {
        let content = read_to_string(path).ok()?;
        content.trim().parse::<f64>().ok()
    }

    fn measure_power(_domain: &str, energy_path: &Path) -> Option<f64> {
        let start_energy = read_rapl(energy_path)?;
        let start_time = Instant::now();
        sleep(Duration::from_secs(1));
        let end_energy = read_rapl(energy_path)?;
        let elapsed = start_time.elapsed().as_secs_f64();
        Some((end_energy - start_energy) / (elapsed * 1e6))
    }

    let entries = read_dir(RAPL).ok()?;

    let result: Vec<(String, f64)> = entries
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();
            let domain = path.file_name()?.to_str()?;
            if !path.is_dir() || !domain.starts_with("intel-rapl:") {
                return None;
            }
            let energy_path = path.join("energy_uj");
            match measure_power(domain, &energy_path) {
                Some(power) => Some((domain.to_string(), power)),
                None => {
                    error!("[{HEADER}] Folder 'Failed to read RAPL domain' : {domain}");
                    None
                }
            }
        })
        .collect();

    if result.is_empty() {
        error!("[{HEADER}] Data 'Unable to get CPU RAPL energy information'");
        None
    } else {
        Some(result)
    }
}

/// Writes JSON formatted data in a file
///
/// # Arguments
///
/// * `data` : JSON serialized collected metrics data to write
/// * `path` : File path use to writing data
///
/// # Return
///
/// - Custom error message if an error occurs during JSON data serialization or file handling.
pub fn write_json_to_file<F>(generator: F, path: &'static str) -> Result<(), Box<dyn Error>>
where
    F: FnOnce() -> Result<Value, Box<dyn Error>>,
{
    let mut data: Value = generator()?;

    // Timestamp implementation in JSON object
    let timestamp = Some(Utc::now().to_rfc3339_opts(Millis, true));

    // Format data to JSON object
    if data.is_object() {
        data.as_object_mut()
            .unwrap()
            .insert("timestamp".to_owned(), json!(timestamp));
    } else if data.is_array() {
        for item in data.as_array_mut().unwrap() {
            if item.is_object() {
                item.as_object_mut()
                    .unwrap()
                    .insert("timestamp".to_owned(), json!(timestamp));
            }
        }
    }

    let mut file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(path)?;
    let log = serde_json::to_string_pretty(&data)?;

    file.write_all(log.as_bytes())?;

    Ok(())
}
