//! # File utilities module

use log::error;
use rusqlite::{params, Connection};
use serde::Serialize;
use std::{
    error::Error,
    fs::{read_dir, read_to_string},
    path::Path,
    time::Duration,
};
use sysinfo::{Components, Cpu};

use core::core::measure_point;

const HEADER: &'static str = "CPU";
const RAPL: &'static str = "/sys/class/powercap";

/// Collection of collected CPU data
#[derive(Debug, Serialize)]
pub struct CpuInfo {
    /// CPU architecture label
    pub architecture: Option<String>,
    /// CPU model name.
    pub model: Option<String>,
    /// CPU generation.
    pub family: Option<String>,
    /// CPU operating frequency in Mhz.
    pub frequency: Option<String>,
    /// Physical CPU cores.
    pub cores_physic: Option<usize>,
    /// Logical CPU cores.
    pub cores_logic: Option<usize>,
    /// CPU usage cores in percentage.
    pub cores_usage: Option<Vec<(String, f32)>>,
    /// CPU temperatures by zone in °C.
    pub temperature: Option<Vec<(String, f32)>>,
    /// CPU energy consumption by zone in uJ.
    pub power: Option<Vec<(String, f64)>>,
}

impl CpuInfo {
    /// Insert CPU parameters into the database.
    ///
    /// # Arguments
    ///
    /// - `timestamp` : Date trace to history identification.
    /// - `conn` : Allow by a [`Connection`] constructor type the connection with an SQLite database.
    /// - `data` : [`CpuInfo`] information to insert in database.
    ///
    /// # Returns
    ///
    /// - Insert the [`CpuInfo`] filled structure in an SQLite database.
    /// - Logs an error if the SQL insert request failed.
    pub fn insert_db(
        conn: &Connection,
        timestamp: &str,
        data: &Self,
    ) -> Result<(), Box<dyn Error>> {
        // Insert main table
        conn.execute(
            "INSERT INTO cpu_data (
                timestamp,
                architecture,
                model,
                family,
                frequency_MHz,
                cores_physic,
                cores_logic
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                timestamp,
                data.architecture,
                data.model,
                data.family,
                data.frequency,
                data.cores_physic,
                data.cores_logic,
            ],
        )?;

        let id = conn.last_insert_rowid();

        // Insert secondary table "core"
        if let Some(cores) = &data.cores_usage {
            for (core_name, core) in cores {
                conn.execute(
                    "INSERT INTO core (indexation, core_name, usage_percent) VALUES (?1, ?2, ?3)",
                    params![id, core_name, core],
                )?;
            }
        }

        // Insert secondary table "temperature"
        if let Some(temps) = &data.temperature {
            for (zone_name, temp) in temps {
                conn.execute(
                    "INSERT INTO temperature (indexation, zone_name, temperature_C) VALUES (?1, ?2, ?3)",
                    params![id, zone_name, temp],
                )?;
            }
        }

        // Insert secondary table "power"
        if let Some(powers) = &data.power {
            for (zone_name, power) in powers {
                conn.execute(
                    "INSERT INTO power (indexation, zone_name, power_W) VALUES (?1, ?2, ?3)",
                    params![id, zone_name, power],
                )?;
            }
        }

        Ok(())
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

/// Read in RAPL the energy in RAPL domain folder and extract the value in uJ.
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

/// Reading in RAPL directory `/sys/class/powercap/`,
/// to get consumption data in locate each CPU zone to get specific energy consumption.
///
/// # Return
///
/// - `result` : Vector containing CPU zone name and its consumption in J.
/// - An empty vector if no energy consumption file or data are found.
pub fn get_rapl_consumption() -> Option<Vec<(String, f64)>> {
    let entries = read_dir(RAPL).ok()?;

    let result: Vec<(String, f64)> = entries
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();
            let domain = path.file_name()?.to_str()?;
            if !path.is_dir() || !domain.starts_with("intel-rapl:") {
                return None;
            }
            let power = measure_point(
                || read_rapl(&path.join("energy_uj")),
                Duration::from_millis(10),
            )?;
            Some((domain.to_string(), power / 1e6))
        })
        .collect();

    if result.is_empty() {
        error!("[{HEADER}] Data 'Unable to get CPU RAPL energy information'");
        None
    } else {
        Some(result)
    }
}
