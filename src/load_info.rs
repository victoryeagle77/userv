//! # System Load data Module
//!
//! This module provides functionality to retrieve system load data on Unix-based systems.

use std::time::Duration;
use serde_json::json;

use crate::utils::read_file_content;

const UPTIME: &'static str = "/proc/uptime";
const LOADAVG: &'static str = "/proc/loadavg";

/// # Function
/// 
/// Function `get_uptime` retrieves the system uptime data from `/proc/uptime`.
///
/// # Returns
///
/// - `days`: `u64` = Number of days since last boot
/// - `hours`: `u64` = Number of hours since last boot
/// - `minutes`: `u64` = Number of minutes since last boot
///
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

/// # Function
/// 
/// Public function `get_load_info` retrieves detailed system load data.
/// Reads `/proc/loadavg` to get load averages and process data.
///
/// # Output
///
/// The function retrieves the following data :
/// - 1, 5, and 15 minute load averages
/// - Number of currently running processes
/// - Number of total processes
/// - Last created process ID
/// - System uptime in days, hours, and minutes
///
pub fn get_load_info() {
    let content = match read_file_content(LOADAVG) {
        Some(content) => content,
        _none => return,
    };

    let line = content.trim();
    let mut parts: Vec<&str> = line.split_whitespace().collect();

    println!("\n[[ SYSTEM LOAD ]]\n");

    if parts.len() < 5 {
        parts = vec!["Unknown", "Unknown", "Unknown", "Unknown", "Unknown"];
    }

    let running_processes: Vec<&str> = parts[3].split('/').collect();
    let (running, total) = if running_processes.len() == 2 {
        (running_processes[0], running_processes[1])
    } else {
        ("Unknown", "Unknown")
    };

    let uptime = get_uptime().unwrap_or((0, 0, 0));

    let load_json_info = json!({
        "LOAD_SYSTEM": {
            "uptime": {
                "days": uptime.0,
                "hours": uptime.1,
                "minutes": uptime.2
            },
            "system_load": { 
                "1_min :": parts[0],
                "5_min": parts[1],
                "15_min": parts[2] 
            },
            "running_process": running,
            "total_process": total,
            "last_pid": parts[4]
        }
    });

    println!("{}", serde_json::to_string_pretty(&load_json_info).unwrap());

}