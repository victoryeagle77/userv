//! # System Load data Module
//!
//! This module provides functionality to retrieve system load data on Unix-based systems.

use log::error;
use serde::Serialize;
use serde_json::{json, Value};
use std::{error::Error, thread};
use sysinfo::{Pid, ProcessRefreshKind, ProcessesToUpdate, System};

use crate::utils::write_json_to_file;

const FACTOR: u64 = 1_000_000;

const HEADER: &str = "SYSTEM";
const LOGGER: &str = "log/system_data.json";

/// Collection of process data.
#[derive(Debug, Serialize)]
struct ProcessInfo {
    /// PID of a process.
    pid: usize,
    /// Identification name of a process, given by the system.
    name: Option<String>,
    /// CPU usage by a process in percentage.
    cpu_usage: Option<f32>,
    /// Reading disk usage by a process in MB.
    disk_usage_read: Option<u64>,
    /// Writing disk usage by a process in MB.
    disk_usage_write: Option<u64>,
    /// process group ID of the process.
    id_group: Option<String>,
    /// Session ID of a running process.
    id_session: Option<usize>,
    /// ID of the owner user of this process.
    id_user: Option<String>,
    /// Memory usage by a process in MB.
    memory_usage: Option<u64>,
    /// Virtual memory usage by a process in MB.
    memory_virtual_usage: Option<u64>,
    /// State of a process on the system among `ProcessStatus`.
    status: Option<String>,
    /// Time the process has been running in minutes.
    run_time: Option<u64>,
}

/// Collection of system load data.
#[derive(Debug, Serialize)]
struct SystemInfo {
    /// System hostname based off DNS.
    hostname: Option<String>,
    /// Average system load calculated (1 min, 5 min, 15 min).
    system_load: Option<(f64, f64, f64)>,
    /// Name of the current operating system.
    system_kernel: Option<String>,
    /// Name of the current operating system.
    system_name: Option<String>,
    /// Name of the current operating system.
    system_version: Option<String>,
    /// Default maximum number of open files for a process.
    open_files_limit: Option<usize>,
    /// Total number of processes.
    process_count: Option<u32>,
    /// Process information.
    processes: Option<Vec<ProcessInfo>>,
    /// Time since the last system boot (days, hours, minutes).
    uptime: Option<(u64, u64, u64)>,
}

impl ProcessInfo {
    /// Converts [`ProcessInfo`] into a JSON object.
    fn to_json(&self) -> Value {
        json!({
            "pid": self.pid,
            "name": self.name,
            "cpu_usage_%": self.cpu_usage,
            "disk_usage_reade_MB": self.disk_usage_read,
            "disk_usage_write_MB": self.disk_usage_write,
            "id_group": self.id_group,
            "id_session": self.id_session,
            "id_user": self.id_user,
            "memory_usage_MB": self.memory_usage,
            "memory_virtual_usage_MB": self.memory_virtual_usage,
            "status": self.status,
            "run_time_min": self.run_time,
        })
    }
}

impl SystemInfo {
    /// Converts [`SystemInfo`] into a JSON object.
    fn to_json(&self) -> Value {
        json!({
            "hostname": self.hostname,
            "os_kernel": self.system_kernel,
            "os_name": self.system_name,
            "os_version": self.system_version,
            "os_load": self.system_load.map(|(one, five, fifteen)| json!({
                "load_1_min": one,
                "load_5_min": five,
                "load_15_min": fifteen
            })),
            "open_files_limit": self.open_files_limit,
            "processes": self.processes.as_ref().map(|ps| ps.iter().map(|p| p.to_json()).collect::<Vec<_>>()),
            "total_process": self.process_count,
            "uptime": self.uptime.map(|(days, hours, minutes)| json!({
                "days": days,
                "hours": hours,
                "minutes": minutes
            })),
        })
    }

    /// Retrieves information about a process.
    ///
    /// # Arguments
    ///
    /// - `pid` : Process identification.
    /// - `system` : Generic initializer.
    ///
    /// # Returns
    ///
    /// - Completed [`ProcessInfo`] structure with all information about a process.
    /// - An error occurs when the PID of a process is not found.
    fn collect_process_data(pid: usize, system: &System) -> Result<ProcessInfo, Box<dyn Error>> {
        let process = system
            .process(Pid::from(pid))
            .ok_or_else(|| format!("Data 'Process with PID ({pid}) not found'"))?;

        // Precise value of CPU usage by a process required to divide it by number of CPU cores
        let cpu_count = system.cpus().len() as f32;
        let cpu_usage = if cpu_count > 0.0 {
            Some(process.cpu_usage() / cpu_count)
        } else {
            error!("[{HEADER}] Data 'Failed to calculate the process cpu usage'");
            Some(process.cpu_usage())
        };

        // Disk usage by a process
        let disk_usage_read = Some(process.disk_usage().total_read_bytes / FACTOR);
        let disk_usage_write = Some(process.disk_usage().total_written_bytes / FACTOR);

        // Memories usage by a process
        let memory_usage = Some(process.memory() / FACTOR);
        let memory_virtual_usage = Some(process.virtual_memory() / FACTOR);

        // System info about process
        let name = Some(process.name().to_string_lossy().to_string());
        let status = Some(process.status().to_string());
        let run_time = Some(process.run_time() / 60);

        let id_group = process.group_id().map(|pid| pid.to_string());
        let id_session = process.session_id().map(|pid| pid.into());
        let id_user = process.user_id().map(|pid| pid.to_string());

        Ok(ProcessInfo {
            pid,
            cpu_usage,
            disk_usage_read,
            disk_usage_write,
            id_group,
            id_session,
            id_user,
            memory_usage,
            memory_virtual_usage,
            name,
            status,
            run_time,
        })
    }
}

/// Retrieves information about the top resource-consuming process, system load and uptime.
///
/// # Returns
///
/// - Completed [`SystemInfo`] structure with all processes and system information.
/// - An error when some important and critical metrics can't be retrieved.
fn collect_system_data() -> Result<SystemInfo, Box<dyn Error>> {
    // Uptime
    let uptime = {
        let secs = System::uptime();
        if secs == 0 {
            error!("[{HEADER}] Data 'Failed to retrieve uptime'");
            None
        } else {
            Some((secs / 86400, (secs % 86400) / 3600, (secs % 3600) / 60))
        }
    };

    // System load
    let system_load = {
        let load = System::load_average();
        if load.one == 0.0 && load.five == 0.0 && load.fifteen == 0.0 {
            error!("[{HEADER}] Data 'Failed to retrieve load averages'");
            None
        } else {
            Some((load.one, load.five, load.fifteen))
        }
    };

    // Operating system Kernel version
    let system_kernel = System::kernel_version();
    if system_kernel.is_none() {
        error!("[{HEADER}] Data 'Failed to retrieve operating system kernel version'");
    }

    // Operating system name (Linux distribution name)
    let system_name = System::name();
    if system_name.is_none() {
        error!("[{HEADER}] Data 'Failed to retrieve operating system name'");
    }

    // Operating system version (Linux distribution version)
    let system_version = System::os_version();
    if system_version.is_none() {
        error!("[{HEADER}] Data 'Failed to retrieve operating system version'");
    }

    let mut sys = System::new_all();
    // Wait a bit because CPU usage is based on diff
    thread::sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL);
    // Refresh CPU usage to get actual value
    sys.refresh_processes_specifics(
        ProcessesToUpdate::All,
        true,
        ProcessRefreshKind::nothing().with_cpu(),
    );

    // Counter of total running processes
    let proc_count = sys.processes().len() as u32;
    let process_count = if proc_count > 0 {
        Some(proc_count)
    } else {
        return Err("Data 'No processes found'".into());
    };

    // Information about consuming processes
    let processes: Vec<ProcessInfo> = sys
        .processes()
        .iter()
        .filter_map(|(&pid, _process)| SystemInfo::collect_process_data(pid.into(), &sys).ok())
        .collect();
    let processes = if !processes.is_empty() {
        Some(processes)
    } else {
        return Err("Data 'No processes found'".into());
    };

    let hostname = System::host_name();
    let open_files_limit = System::open_files_limit();

    Ok(SystemInfo {
        hostname,
        system_kernel,
        system_load,
        system_name,
        system_version,
        open_files_limit,
        process_count,
        processes,
        uptime,
    })
}

/// Public function used to send JSON formatted values,
/// from [`collect_system_data`] function result.
pub fn get_system_info() -> Result<(), Box<dyn Error>> {
    let data = collect_system_data()?;
    let values = json!({ HEADER: data.to_json() });
    write_json_to_file(|| Ok(values), LOGGER)?;
    Ok(())
}
