//! # System Load data Module
//!
//! This module provides functionality to retrieve system load data on Unix-based systems.

use log::error;
use serde::Serialize;
use serde_json::json;
use std::time::Duration;
use sysinfo::{PidExt, ProcessExt, System, SystemExt};

use crate::utils::{read_file_content, write_json_to_file};

const UPTIME: &str = "/proc/uptime";
const LOADAVG: &str = "/proc/loadavg";

const HEADER: &str = "LOAD_SYSTEM";
const LOGGER: &str = "log/load_data.json";

/// Collection of collected system load data
#[derive(Debug, Serialize)]
struct SystemLoad {
    /// Time since the last system boot.
    uptime: Option<(u64, u64, u64)>,
    /// System load after 1 minute.
    load_1: Option<f64>,
    /// System load after 5 minutes.
    load_5: Option<f64>,
    /// System load after 15 minutes.
    load_15: Option<f64>,
    /// Total of running processes.
    run_proc: Option<u32>,
    /// Total processes counted.
    tot_proc: Option<u32>,
    /// PID of the top resource-consuming process.
    top_process_pid: Option<u32>,
    /// Name of the top resource-consuming process.
    top_process_name: Option<String>,
    /// CPU usage of the top resource-consuming process in Bytes.
    top_process_cpu_usage: Option<f32>,
    /// Memory usage of the top resource-consuming process in percentage.
    top_process_memory_usage: Option<u64>,
}

/// Function that retrieves the system uptime data from `/proc/uptime`.
///
/// # Returns
///
/// - Option containing a tuple of days, hours, and minutes since last boot
fn get_uptime() -> Option<(u64, u64, u64)> {
    let content = read_file_content(UPTIME)?;

    let uptime_secs = content
        .split_whitespace()
        .next()
        .and_then(|s| s.parse::<f64>().ok())?;

    let uptime = Duration::from_secs_f64(uptime_secs);

    let days = uptime.as_secs() / 86400;
    let hours = (uptime.as_secs() % 86400) / 3600;
    let minutes = (uptime.as_secs() % 3600) / 60;

    Some((days, hours, minutes))
}

/// Function that retrieves information about the top resource-consuming process.
///
/// # Returns
///
/// A tuple containing the following information about the top process :
/// - PID
/// - Process name
/// - CPU usage
/// - Memory usage
fn get_top_process() -> (Option<u32>, Option<String>, Option<f32>, Option<u64>) {
    let mut system: System = System::new_all();
    system.refresh_all();

    let mut top_process = None;
    let mut max_usage: f32 = 0.0;

    for (pid, process) in system.processes() {
        let cpu_usage = process.cpu_usage();
        if cpu_usage > max_usage {
            max_usage = cpu_usage;
            top_process = Some((
                pid.as_u32(),
                process.name().to_string(),
                cpu_usage,
                process.memory() / 1_000,
            ));
        }
    }

    top_process.map_or((None, None, None, None), |(pid, name, cpu, mem)| {
        (Some(pid), Some(name), Some(cpu), Some(mem))
    })
}

/// Function that retrieves detailed system load data.
/// Reads `/proc/loadavg` to get load averages and process data.
///
/// # Returns
///
/// `Result<SystemLoad, String>` : Completed `SystemLoad` structure with all load system information
/// or an error message if data collection fails.
fn collect_load_data() -> Result<SystemLoad, String> {
    let content = read_file_content(LOADAVG)
        .ok_or_else(|| "File 'Unable to read load average file'".to_string())?;

    let line = content.trim();
    let parts: Vec<&str> = line.split_whitespace().collect();

    if parts.len() < 5 {
        error!("[{HEADER}] Data 'Insufficient data in {LOADAVG}'");
        return Err("Insufficient data".to_string());
    }

    let run_proc: Vec<&str> = parts[3].split('/').collect();
    let (running, total) = if run_proc.len() == 2 {
        (
            run_proc[0].parse::<u32>().ok(),
            run_proc[1].parse::<u32>().ok(),
        )
    } else {
        error!("[{HEADER}] Data 'Failed data extraction from {LOADAVG}'",);
        (None, None)
    };

    let (top_process_pid, top_process_name, top_process_cpu_usage, top_process_memory_usage) =
        get_top_process();

    let result = SystemLoad {
        uptime: get_uptime(),
        load_1: parts[0].parse::<f64>().ok(),
        load_5: parts[1].parse::<f64>().ok(),
        load_15: parts[2].parse::<f64>().ok(),
        run_proc: running,
        tot_proc: total,
        top_process_pid,
        top_process_name,
        top_process_cpu_usage,
        top_process_memory_usage,
    };

    Ok(result)
}

/// Public function used to send JSON formatted values,
/// from `collect_load_data` function result.
pub fn get_load_info() {
    let data = || {
        let values: SystemLoad = collect_load_data()?;
        Ok(json!({
            HEADER: {
                "uptime": values.uptime.map(|(days, hours, minutes)| {
                    json!({
                        "days": days,
                        "hours": hours,
                        "minutes": minutes
                    })
                }),
                "system_load": {
                    "1_min": values.load_1,
                    "5_min": values.load_5,
                    "15_min": values.load_15
                },
                "running_process": values.run_proc,
                "total_process": values.tot_proc,
                "top_process": {
                    "pid": values.top_process_pid,
                    "name": values.top_process_name,
                    "cpu_usage": values.top_process_cpu_usage,
                    "memory_usage": values.top_process_memory_usage,
                },
            }
        }))
    };

    write_json_to_file(data, LOGGER, HEADER);
}
