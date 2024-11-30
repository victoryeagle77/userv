//! # System Load data Module
//!
//! This module provides functionality to retrieve system load data on Unix-based systems.

use serde::Serialize;
use serde_json::json;
use std::time::Duration;

use crate::utils::read_file_content;

const UPTIME: &str = "/proc/uptime";
const LOADAVG: &str = "/proc/loadavg";

const HEADER: &str = "LOAD_SYSTEM";

/// Collection of collected system load data
#[derive(Debug, Serialize)]
struct SystemLoad {
    /// Time since the last system boot.
    uptime: (u64, u64, u64),
    /// System load after 1 minute.
    load_1: Option<f64>,
    /// System load after 5 minutes.
    load_5: Option<f64>,
    /// System load after 15 minutes.
    load_15: Option<f64>,
    /// Last running process.
    last_pid: Option<u32>,
    /// Total of running processes.
    run_proc: Option<u32>,
    /// Total processes counted.
    tot_proc: Option<u32>,
}

/// Function that retrieves the system uptime data from `/proc/uptime`.
///
/// # Returns
///
/// - `Option<(u64, u64, u64)>` = Number of days, hours, and minutes since last boot
fn get_uptime() -> Option<(u64, u64, u64)> {
    let content = read_file_content(UPTIME)?;

    let uptime_secs = content
        .split_whitespace()
        .next()
        .and_then(|s| s.parse::<f64>().ok())
        .unwrap_or(0.0);

    let uptime = Duration::from_secs_f64(uptime_secs);

    let days = uptime.as_secs() / 86400;
    let hours = (uptime.as_secs() % 86400) / 3600;
    let minutes = (uptime.as_secs() % 3600) / 60;

    Some((days, hours, minutes))
}

/// Public function that retrieves detailed system load data.
/// Reads `/proc/loadavg` to get load averages and process data.
///
/// # Returns
///
/// `result` : Completed `SystemLoad` structure with all load system information
/// - 1, 5, and 15 minute load averages
/// - Number of currently running processes
/// - Number of total processes
/// - Last created process ID
/// - System uptime in days, hours, and minutes
fn collect_load_data() -> Result<SystemLoad, String> {
    let content = match read_file_content(LOADAVG) {
        Some(content) => content,
        _none => return Err("<ERROR_3> Reading file".to_string()),
    };

    let line = content.trim();
    let mut parts: Vec<&str> = line.split_whitespace().collect();

    if parts.len() < 5 {
        parts.resize(5, "NULL");
    }

    let run_proc: Vec<&str> = parts[3].split('/').collect();
    let (running, total) = if run_proc.len() == 2 {
        (
            run_proc[0].parse::<u32>().ok(),
            run_proc[1].parse::<u32>().ok(),
        )
    } else {
        (None, None)
    };

    let uptime = get_uptime().unwrap_or((0, 0, 0));

    let result = SystemLoad {
        uptime,
        load_1: parts[0].parse::<f64>().ok(),
        load_5: parts[1].parse::<f64>().ok(),
        load_15: parts[2].parse::<f64>().ok(),
        last_pid: parts[4].parse::<u32>().ok(),
        run_proc: running,
        tot_proc: total,
    };

    Ok(result)
}

/// Public function used to send JSON formatted values,
/// from `collect_load_data` function result.
pub fn get_load_info() {
    match collect_load_data() {
        Ok(values) => {
            let load_json_info: serde_json::Value = json!({
                HEADER: {
                    "uptime": {
                        "days": values.uptime.0,
                        "hours": values.uptime.1,
                        "minutes": values.uptime.2
                    },
                    "system_load": {
                        "1_min": values.load_1.unwrap_or(0.0),
                        "5_min": values.load_5.unwrap_or(0.0),
                        "15_min": values.load_15.unwrap_or(0.0)
                    },
                    "running_process": values.run_proc.unwrap_or(0),
                    "total_process": values.tot_proc.unwrap_or(0),
                    "last_pid": values.last_pid.unwrap_or(0),
                }
            });

            println!("{}", serde_json::to_string_pretty(&load_json_info).unwrap());
        }
        Err(e) => {
            eprintln!("{}", e);
        }
    }
}
