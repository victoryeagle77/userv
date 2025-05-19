//! # Lib file for CPU data module
//!
//! This module provides functionalities to retrieve processor data on Unix-based systems.

use chrono::{SecondsFormat, Utc};
use std::{error::Error, thread::sleep};
use sysinfo::{CpuRefreshKind, RefreshKind, System};

mod utils;
use crate::utils::*;

use core::core::init_db;

const REQUEST: &'static str = "
    CREATE TABLE IF NOT EXISTS cpu_data (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        timestamp TEXT NOT NULL,
        architecture TEXT,
        model TEXT,
        family TEXT,
        frequency_MHz TEXT,
        cores_physic INTEGER,
        cores_logic INTEGER
    );
    CREATE TABLE IF NOT EXISTS core (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        indexation INTEGER,
        core_name TEXT,
        usage_percent REAL,
        FOREIGN KEY(indexation) REFERENCES cpu_data(id)
    );
    CREATE TABLE IF NOT EXISTS temperature (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        indexation INTEGER,
        zone_name TEXT,
        temperature_C REAL,
        FOREIGN KEY(indexation) REFERENCES cpu_data(id)
    );
    CREATE TABLE IF NOT EXISTS power (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        indexation INTEGER,
        zone_name TEXT,
        power_W REAL,
        FOREIGN KEY(indexation) REFERENCES cpu_data(id)
    );
    ";

/// Public function reading and using `/proc/cpuinfo` file values,
/// and retrieves detailed CPU data.
///
/// # Return
///
/// - Completed [`CpuInfo`] structure with all retrieved and computing CPU information.
/// - An error when some important and critical metrics can't be retrieved.
fn collect_cpu_data() -> Result<CpuInfo, Box<dyn Error>> {
    let mut sys =
        System::new_with_specifics(RefreshKind::nothing().with_cpu(CpuRefreshKind::everything()));
    // Wait a bit because CPU usage is based on diff.
    sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL);
    // Refresh CPUs again to get actual value.
    sys.refresh_cpu_all();

    let cpus = sys.cpus();
    if cpus.is_empty() {
        return Err("Failed to get global CPUs information".to_string().into());
    }

    let cores_physic = System::physical_core_count();
    let cores_logic = Some(cpus.len());

    let architecture = Some(System::cpu_arch());
    let model = cpus.first().map(|c| c.brand().to_string());
    let family = cpus.first().map(|c| c.vendor_id().to_string());
    let frequency = cpus.first().map(|c| c.frequency().to_string());

    let cores_usage = Some(get_cpu_usage(cpus)?);
    let temperature = Some(get_cpu_temperature()?);

    let power = get_rapl_consumption();

    Ok(CpuInfo {
        architecture,
        cores_physic,
        cores_logic,
        cores_usage,
        family,
        frequency,
        model,
        power,
        temperature,
    })
}

/// Public function used to send values in SQLite database,
/// from [`collect_cpu_data`] function result.
pub fn get_cpu_info() -> Result<(), Box<dyn Error>> {
    let mut conn = init_db(REQUEST)?;
    let timestamp = Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true);
    let data = collect_cpu_data()?;
    CpuInfo::insert_db(&mut conn, &timestamp, &data)?;
    Ok(())
}
