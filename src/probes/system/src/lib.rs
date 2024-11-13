//! # Lib file for system data module
//!
//! This module provides functionality to retrieve operating system data on Unix-based systems.

use chrono::Utc;
use log::error;
use std::{error::Error, thread};
use sysinfo::{ProcessRefreshKind, ProcessesToUpdate, System};

mod utils;
use core::core::init_db;
use utils::*;

const REQUEST: &str = "CREATE TABLE IF NOT EXISTS system_data (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        hostname TEXT,
        system_load TEXT,
        system_kernel TEXT,
        system_name TEXT,
        system_version TEXT,
        open_files_limit INTEGER,
        process_count INTEGER,
        uptime TIME
    );
    CREATE TABLE IF NOT EXISTS system_process_data (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        timestamp TEXT NOT NULL,
        pid INTEGER NOT NULL,
        name TEXT,
        cpu_usage REAL,
        disk_usage_read_MB INTEGER,
        disk_usage_write_MB INTEGER,
        id_group TEXT,
        id_session INTEGER,
        id_user TEXT,
        memory_usage_MB INTEGER,
        memory_virtual_usage_MB INTEGER,
        status TEXT,
        run_time_min INTEGER,
        system_data_id INTEGER,
        FOREIGN KEY (system_data_id) REFERENCES system_data(id)
    );";

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
            Some((
                secs / 86400,
                (secs % 86400) / 3600,
                (secs % 3600) / 60,
                secs % 60,
            ))
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

/// Public function used to send values in SQLite database,
/// from [`collect_system_data`] function result.
pub fn get_system_info() -> Result<(), Box<dyn Error>> {
    let conn = init_db(REQUEST)?;
    let data = collect_system_data()?;
    let timestamp = Utc::now().to_rfc3339();

    let system_data_id = SystemInfo::insert_db(&conn, &data)?;
    if let Some(ref processes) = data.processes {
        ProcessInfo::insert_db(&conn, processes, system_data_id, &timestamp)?;
    }
    Ok(())
}
