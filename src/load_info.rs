//! # System Load data Module
//!
//! This module provides functionality to retrieve system load data on Unix-based systems.

use chrono::{SecondsFormat, Utc};
use log::error;
use serde::Serialize;
use serde_json::json;
use std::time::Duration;

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
        error!("[{}] Data 'Insufficient data in /proc/loadavg'", HEADER);
        return Err("Insufficient data in /proc/loadavg".to_string());
    }

    let run_proc: Vec<&str> = parts[3].split('/').collect();
    let (running, total) = if run_proc.len() == 2 {
        (
            run_proc[0].parse::<u32>().ok(),
            run_proc[1].parse::<u32>().ok(),
        )
    } else {
        error!(
            "[{}] Data 'Failed data extraction from /proc/loadavg'",
            HEADER
        );
        (None, None)
    };

    let result = SystemLoad {
        uptime: get_uptime(),
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
            let timestamp = Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true);
            let data: serde_json::Value = json!({
                HEADER: {
                    "timestamp": timestamp,
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
                    "last_pid": values.last_pid,
                }
            });

            write_json_to_file(data, LOGGER, HEADER);
        }
        Err(e) => {
            error!("[{}] {}", HEADER, e);
        }
    }
}
