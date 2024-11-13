//! # File utilities module

use log::error;
use rusqlite::{Connection, params};
use serde::Serialize;
use std::error::Error;
use sysinfo::{Pid, System};

pub const FACTOR: u64 = 1_000_000;
pub const HEADER: &str = "SYSTEM";

/// Collection of process data.
#[derive(Debug, Serialize)]
pub struct ProcessInfo {
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
pub struct SystemInfo {
    /// System hostname based off DNS.
    pub hostname: Option<String>,
    /// Average system load calculated (1 min, 5 min, 15 min).
    pub system_load: Option<(f64, f64, f64)>,
    /// Name of the current operating system.
    pub system_kernel: Option<String>,
    /// Name of the current operating system.
    pub system_name: Option<String>,
    /// Name of the current operating system.
    pub system_version: Option<String>,
    /// Default maximum number of open files for a process.
    pub open_files_limit: Option<usize>,
    /// Total number of processes.
    pub process_count: Option<u32>,
    /// Process information.
    pub processes: Option<Vec<ProcessInfo>>,
    /// Time since the last system boot (days, hours, minutes).
    pub uptime: Option<(u64, u64, u64, u64)>,
}

impl ProcessInfo {
    /// Insert system process parameters into the database.
    ///
    /// # Arguments
    ///
    /// - `conn`: Connection to SQLite database.
    /// - `data`: [`ProcessInfo`] information to insert in database.
    /// - `timestamp`: Timestamp of the measurement.
    ///
    /// # Returns
    ///
    /// - Insert the [`ProcessInfo`] filled structure in an SQLite database.
    /// - Logs an error if the SQL insert request failed.
    pub fn insert_db(
        conn: &Connection,
        data: &[Self],
        id: i64,
        timestamp: &str,
    ) -> Result<(), Box<dyn Error>> {
        let mut metrics = conn.prepare(
            "INSERT INTO system_process_data (
                timestamp,
                pid,
                name,
                cpu_usage,
                disk_usage_read_MB,
                disk_usage_write_MB,
                id_group,
                id_session,
                id_user,
                memory_usage_MB,
                memory_virtual_usage_MB,
                status,
                run_time_min,
                system_data_id
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
        )?;

        for p in data {
            metrics.execute(params![
                timestamp,
                p.pid,
                p.name,
                p.cpu_usage,
                p.disk_usage_read,
                p.disk_usage_write,
                p.id_group,
                p.id_session,
                p.id_user,
                p.memory_usage,
                p.memory_virtual_usage,
                p.status,
                p.run_time,
                id
            ])?;
        }
        Ok(())
    }
}

impl SystemInfo {
    /// Insert system parameters into the database.
    ///
    /// # Arguments
    ///
    /// - `conn`: Connection to SQLite database.
    /// - `data`: [`SystemInfo`] information to insert in database.
    ///
    /// # Returns
    ///
    /// - Insert the [`SystemInfo`] filled structure in an SQLite database.
    /// - Logs an error if the SQL insert request failed.
    pub fn insert_db(conn: &Connection, data: &Self) -> Result<i64, Box<dyn Error>> {
        let system_load = data.system_load.map(|(a, b, c)| format!("{a},{b},{c}"));
        let uptime = data.uptime.map(|(d, h, m, s)| format!("{d}:{h}:{m}:{s}"));

        conn.execute(
            "INSERT INTO system_data (
                hostname,
                system_load,
                system_kernel,
                system_name,
                system_version,
                open_files_limit,
                process_count,
                uptime
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                data.hostname,
                system_load,
                data.system_kernel,
                data.system_name,
                data.system_version,
                data.open_files_limit,
                data.process_count,
                uptime,
            ],
        )?;
        Ok(conn.last_insert_rowid())
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
    pub fn collect_process_data(
        pid: usize,
        system: &System,
    ) -> Result<ProcessInfo, Box<dyn Error>> {
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
