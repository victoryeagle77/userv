//! # Lib file for CPU data module
//!
//! This module provides functionalities to retrieve processor data on Unix-based systems.

use chrono::{SecondsFormat, Utc};
use rusqlite::{Connection, params};
use std::{error::Error, thread::sleep};
use sysinfo::{CpuRefreshKind, RefreshKind, System};

mod dbms;
mod utils;
use crate::{
    dbms::*,
    utils::{CpuInfo, get_cpu_temperature, get_cpu_usage, get_rapl_consumption},
};

use core::core::{db_insert_query, db_table_query_creation, init_db};

/// Insert CPU parameters into the database.
///
/// # Arguments
///
/// - `timestamp` : Date trace to history identification.
/// - `conn` : Allow by a [`Connection`] constructor type the connection with an SQLite database.
/// - `data` : [`CpuInfo`] information to insert in database.
///
/// # Returns
///
/// - Insert the [`CpuInfo`] filled structure in an SQLite database.
/// - Logs an error if the SQL insert request failed.
pub fn insert_db(conn: &Connection, timestamp: &str, data: &CpuInfo) -> Result<(), Box<dyn Error>> {
    // Insert main table
    let query_info = db_insert_query(TABLE_NAME[0], &field_descriptor_info())?;
    conn.execute(
        &query_info,
        params![
            timestamp,
            data.architecture,
            data.model,
            data.family,
            data.frequency,
            data.cores_physic,
            data.cores_logic,
        ],
    )?;

    // Insert secondary table "core"
    if let Some(cores) = &data.cores_usage {
        let query_core = db_insert_query(TABLE_NAME[1], &field_descriptor_core())?;
        for (core_name, core) in cores {
            conn.execute(&query_core, params![timestamp, core_name, core])?;
        }
    }

    // Insert secondary table "temperature"
    if let Some(temps) = &data.temperature {
        let query_temperature = db_insert_query(TABLE_NAME[2], &field_descriptor_temperature())?;
        for (zone_name, temp) in temps {
            conn.execute(&query_temperature, params![timestamp, zone_name, temp])?;
        }
    }

    // Insert secondary table "power"
    if let Some(powers) = &data.power {
        let query_power = db_insert_query(TABLE_NAME[3], &field_descriptor_power())?;
        for (zone_name, power) in powers {
            conn.execute(&query_power, params![timestamp, zone_name, power])?;
        }
    }

    Ok(())
}

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
    let query_info = db_table_query_creation(TABLE_NAME[0], &field_descriptor_info())?;
    let query_core = db_table_query_creation(TABLE_NAME[1], &field_descriptor_core())?;
    let query_temperature =
        db_table_query_creation(TABLE_NAME[2], &field_descriptor_temperature())?;
    let query_power = db_table_query_creation(TABLE_NAME[3], &field_descriptor_power())?;

    let conn = init_db(&query_info)?;
    conn.execute_batch(&query_core)?;
    conn.execute_batch(&query_temperature)?;
    conn.execute_batch(&query_power)?;

    let timestamp = Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true);
    let data = collect_cpu_data()?;
    insert_db(&conn, &timestamp, &data)?;
    Ok(())
}
