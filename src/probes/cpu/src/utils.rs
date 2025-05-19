//! # File utilities module

use log::error;
use std::{
    error::Error,
    fs::{read_dir, read_to_string},
    path::Path,
    time::Duration,
};
use sysinfo::{Components, Cpu, System};

use core::core::measure_point;

const HEADER: &str = "CPU";

/// RAPL directory providing power consumption for x86-64 CPU architectures (plus DRAM according the CPU version).
const RAPL: &str = "/sys/class/powercap";

/// Collection of collected CPU data.
#[derive(Debug)]
pub struct CpuGlobalInfo {
    /// CPU architecture label.
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
}

/// Collection of collected CPU cores usage data.
#[derive(Debug)]
pub struct CpuCoreInfo {
    /// CPU usage cores in percentage.
    pub cores_usage: Vec<(String, f32)>,
}

/// Collection of collected CPU power consumption data.
#[derive(Debug)]
pub struct CpuPowerInfo {
    /// CPU energy consumption by zone in uJ.
    pub powers: Vec<(String, f64)>,
}

/// Collection of collected CPU temperature data.
#[derive(Debug)]
pub struct CpuTemperatureInfo {
    /// CPU temperatures of various thermal zone in celsius.
    pub temperatures: Vec<(String, f32)>,
}

/// Retrieves the current CPU usage by cores.
/// This function uses the [`sysinfo`] crate to gather CPU usage information.
/// It takes two snapshots of CPU usage with a 1-second interval between them,
/// to calculate the current usage percentage for each CPU core.
///
/// # Return
///
/// - Cores zone name.
/// - Cores usage in percentage.
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
/// - Cores or thermal zone name.
/// - Thermal zone value in Celsius.
/// - An error if CPU thermal data are not found.
pub fn get_cpu_temperature(component: Components) -> Result<Vec<(String, f32)>, Box<dyn Error>> {
    let result = component
        .iter()
        .filter_map(|c| {
            let name = c.label().to_string();
            let temperature = c.temperature();

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

/// Reading in [`RAPL`] directory to get consumption data in locate each CPU zone to get specific energy consumption.
///
/// # Return
///
/// - `result` : Vector containing CPU zone name and its consumption in J.
/// - An empty vector if no energy consumption file or data are found.
pub fn get_rapl_consumption() -> Result<Vec<(String, f64)>, Box<dyn Error>> {
    /// Read the energy in [`RAPL`] domain folder and extract the value in µJ.
    ///
    /// # Arguments
    ///
    /// - `path` : Files in [`RAPL`] folder domain.
    ///
    /// # Returns
    ///
    /// - `res` : The energy information in microJoules in [`RAPL`] domain folder.
    /// - An error when we can't to retrieve properly the energy data.
    fn read_rapl(path: &Path) -> Result<Option<f64>, Box<dyn Error>> {
        let content = read_to_string(path)?;
        let res = content.trim().parse::<f64>()?;
        Ok(Some(res))
    }

    let entries = read_dir(RAPL)?;

    let result: Vec<(String, f64)> = entries
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();
            let domain = path.file_name()?.to_str()?;
            if !path.is_dir() || !domain.starts_with("intel-rapl:") {
                return None;
            }
            let power = measure_point(
                || read_rapl(&path.join("energy_uj")).ok()?,
                Duration::from_millis(10),
            )?;
            Some((domain.to_string(), power / 1e6))
        })
        .collect();

    if result.is_empty() {
        Err("Data 'Unable to get CPU RAPL energy information'".into())
    } else {
        Ok(result)
    }
}

/// Reading and using `/proc/cpuinfo` file values, to retrieve detailed CPU information.
///
/// # Arguments
///
/// - cpu: Struct [`Cpu`] parameters.
///
/// # Return
///
/// - Completed [`CpuGlobalInfo`] structure with all retrieved and computing CPU information.
/// - An error when some metrics can't be retrieved.
pub fn collect_cpu_data(cpu: &[Cpu]) -> Result<CpuGlobalInfo, Box<dyn Error>> {
    let cores_physic = System::physical_core_count();
    let cores_logic = Some(cpu.len());

    let architecture = Some(System::cpu_arch());
    let model = cpu.first().map(|c| c.brand().to_string());
    let family = cpu.first().map(|c| c.vendor_id().to_string());
    let frequency = cpu.first().map(|c| c.frequency().to_string());

    Ok(CpuGlobalInfo {
        architecture,
        cores_physic,
        cores_logic,
        family,
        frequency,
        model,
    })
}

/// Collect detailed CPU core usage information.
///
/// # Arguments
///
/// - cpu: Struct [`Cpu`] parameters.
///
/// # Return
///
/// - Completed [`CpuCoreInfo`] structure with all retrieved information.
/// - An error when some metrics can't be retrieved.
pub fn collect_cpu_core_data(cpu: &[Cpu]) -> Result<CpuCoreInfo, Box<dyn Error>> {
    let cores_usage = get_cpu_usage(cpu)?;
    Ok(CpuCoreInfo { cores_usage })
}

/// Collect detailed CPU temperature information for each hardware thermal zone.
///
/// # Arguments
///
/// - component: Struct [`Components`] parameters.
///
/// # Return
///
/// - Completed [`CpuTemperatureInfo`] structure with all retrieved information.
/// - An error when some metrics can't be retrieved.
pub fn collect_cpu_temperature_data(
    component: Components,
) -> Result<CpuTemperatureInfo, Box<dyn Error>> {
    let temperatures = get_cpu_temperature(component)?;
    Ok(CpuTemperatureInfo { temperatures })
}

/// Collect detailed CPU power consumption information.
///
/// # Return
///
/// - Completed [`CpuPowerInfo`] structure with all retrieved information.
/// - An error when some metrics can't be retrieved.
pub fn collect_cpu_power_data() -> Result<CpuPowerInfo, Box<dyn Error>> {
    let powers = get_rapl_consumption()?;
    Ok(CpuPowerInfo { powers })
}
