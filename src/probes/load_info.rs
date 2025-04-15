//! # System Load data Module
//!
//! This module provides functionality to retrieve system load data on Unix-based systems.

use log::error;
use serde::Serialize;
use serde_json::{json, Value};
use std::{cmp::Ordering::Equal, error::Error};
use sysinfo::{LoadAvg, Pid, PidExt, Process, ProcessExt, System, SystemExt};

use crate::utils::write_json_to_file;

const HEADER: &str = "LOAD_SYSTEM";
const LOGGER: &str = "log/load_data.json";

/// Collection of collected system load data.
#[derive(Debug, Serialize)]
struct SystemLoad {
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

impl SystemLoad {
    /// Converts `SystemLoad` into a JSON object.
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
/// A tuple containing the following information :
/// - Time since the last system boot (days, hours, minutes).
/// - Average system load calculated (1 min, 5 min, 15 min).
/// - Total of running processes.
/// - Total number of processes.
/// - PID of the top resource-consuming process.
/// - Name of the top resource-consuming process.
/// - CPU usage of the top resource-consuming process in percentage.
/// - Memory usage of the top resource-consuming process in MB.
fn collect_load_data() -> SystemLoad {
    let mut system: System = System::new_all();
    system.refresh_all();

    let uptime: Option<(u64, u64, u64)> = {
        let secs: u64 = system.uptime();
        if secs == 0 {
            error!("[{HEADER}] Data 'Failed to retrieve uptime'");
            None
        } else {
            Some((secs / 86400, (secs % 86400) / 3600, (secs % 3600) / 60))
        }
    };

    let load_avg: Option<(f64, f64, f64)> = {
        let load: LoadAvg = system.load_average();
        if load.one == 0.0 && load.five == 0.0 && load.fifteen == 0.0 {
            error!("[{HEADER}] Data 'Failed to retrieve load averages'");
            None
        } else {
            Some((load.one, load.five, load.fifteen))
        }
    };

    let run_proc: Option<u32> = Some(system.processes().len() as u32);
    if run_proc.is_none() {
        error!("[{HEADER}] Data 'Failed to retrieve running processes count'");
    }

    let tot_proc: Option<u32> = Some(system.processes().len() as u32);
    if tot_proc.is_none() {
        error!("[{HEADER}] Data 'Failed to retrieve total processes count'");
    }

    let top_process: Option<(&Pid, &Process)> = system
        .processes()
        .iter()
        .max_by(|(_, a), (_, b)| a.cpu_usage().partial_cmp(&b.cpu_usage()).unwrap_or(Equal));

    if top_process.is_none() {
        error!("[{HEADER}] Data 'Failed to find the top resource-consuming process'");
    }

    SystemLoad {
        uptime,
        load_avg,
        run_proc,
        tot_proc,
        top_process_pid: top_process.map(|(pid, _)| pid.as_u32()),
        top_process_name: top_process.map(|(_, process)| process.name().to_string()),
        top_process_cpu_usage: top_process.map(|(_, process)| process.cpu_usage()),
        top_process_memory_usage: top_process.map(|(_, process)| process.memory() / 1_000),
    }
}

/// Public function used to send JSON formatted values,
/// from `collect_load_data` function result.
pub fn get_load_info() {
    let data = || -> Result<Value, Box<dyn Error>> {
        let values: SystemLoad = collect_load_data();
        Ok(json!({ HEADER: values.to_json() }))
    };

    write_json_to_file(data, LOGGER, HEADER);
}
