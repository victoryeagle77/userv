//! # System Load data Module
//!
//! This module provides functionality to retrieve system load data on Unix-based systems.

use log::error;
use serde::Serialize;
use serde_json::{json, Value};
use std::{cmp::Ordering::Equal, error::Error};
use sysinfo::System;

use crate::utils::write_json_to_file;

const HEADER: &str = "LOAD_SYSTEM";
const LOGGER: &str = "log/load_data.json";

/// Collection of collected system load data.
#[derive(Debug, Serialize)]
struct SystemInfo {
    /// Time since the last system boot (days, hours, minutes).
    uptime: Option<(u64, u64, u64)>,
    /// Average system load calculated (1 min, 5 min, 15 min).
    load_avg: Option<(f64, f64, f64)>,
    /// Total of running processes.
    run_proc: Option<u32>,
    /// Total number of processes.
    tot_proc: Option<u32>,
    /// PID of the top resource-consuming process.
    top_process_pid: Option<u32>,
    /// Name of the top resource-consuming process.
    top_process_name: Option<String>,
    /// CPU usage of the top resource-consuming process in percentage.
    top_process_cpu_usage: Option<f32>,
    /// Memory usage of the top resource-consuming process in MB.
    top_process_memory_usage: Option<u64>,
}

impl SystemInfo {
    /// Converts `SystemInfo` into a JSON object.
    fn to_json(&self) -> Value {
        json!({
            "uptime": self.uptime.map(|(days, hours, minutes)| json!({
                "days": days,
                "hours": hours,
                "minutes": minutes
            })),
            "system_load": self.load_avg.map(|(one, five, fifteen)| json!({
                "load_1_min": one,
                "load_5_min": five,
                "load_15_min": fifteen
            })),
            "running_process": self.run_proc,
            "total_process": self.tot_proc,
            "top_process": {
                "pid": self.top_process_pid,
                "name": self.top_process_name,
                "cpu_usage_%": self.top_process_cpu_usage,
                "memory_usage_MB": self.top_process_memory_usage,
            },
        })
    }
}

/// Function that retrieves information about the top resource-consuming process, system load and uptime.
///
/// # Returns
///
/// A tuple containing [`SystemInfo`] structure with all computing memory information
fn collect_load_data() -> Result<SystemInfo, Box<dyn Error>> {
    let mut sys = System::new_all();
    sys.refresh_all();

    let uptime = {
        let secs = System::uptime();
        if secs == 0 {
            error!("[{HEADER}] Data 'Failed to retrieve uptime'");
            None
        } else {
            Some((secs / 86400, (secs % 86400) / 3600, (secs % 3600) / 60))
        }
    };

    let load_avg = {
        let load = System::load_average();
        if load.one == 0.0 && load.five == 0.0 && load.fifteen == 0.0 {
            error!("[{HEADER}] Data 'Failed to retrieve load averages'");
            None
        } else {
            Some((load.one, load.five, load.fifteen))
        }
    };

    let run_proc = Some(sys.processes().len() as u32);
    if run_proc.is_none() {
        error!("[{HEADER}] Data 'Failed to retrieve running processes count'");
    }

    let tot_proc = Some(sys.processes().len() as u32);
    if tot_proc.is_none() {
        error!("[{HEADER}] Data 'Failed to retrieve total processes count'");
    }

    let top_process = sys
        .processes()
        .iter()
        .max_by(|(_, a), (_, b)| a.cpu_usage().partial_cmp(&b.cpu_usage()).unwrap_or(Equal));
    if top_process.is_none() {
        error!("[{HEADER}] Data 'Failed to find the top resource-consuming process'");
    }

    Ok(SystemInfo {
        uptime,
        load_avg,
        run_proc,
        tot_proc,
        top_process_pid: top_process.map(|(pid, _)| pid.as_u32()),
        top_process_name: top_process.map(|(_pid, process)| process.name().to_string()),
        top_process_cpu_usage: top_process.map(|(_pid, process)| process.cpu_usage()),
        top_process_memory_usage: top_process.map(|(_pid, process)| process.memory() / 1_000),
    })
}

/// Public function used to send JSON formatted values,
/// from [`collect_load_data`] function result.
///
/// # Returns
///
/// Returns a Result to propagate errors.
pub fn get_load_info() -> Result<(), Box<dyn Error>> {
    let data = collect_load_data()?;
    let values = json!({ HEADER: data.to_json() });
    write_json_to_file(|| Ok(values), LOGGER, HEADER)?;
    Ok(())
}
