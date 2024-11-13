//! # Lib file for CPU data module
//!
//! This module provides functionalities to retrieve processor data on Unix-based systems.

use chrono::{SecondsFormat, Utc};
use rusqlite::{Connection, params};
use std::{error::Error, thread::sleep};
use sysinfo::{Components, CpuRefreshKind, MINIMUM_CPU_UPDATE_INTERVAL, RefreshKind, System};

mod dbms;
mod utils;
use crate::{
    dbms::*,
    utils::{
        CpuCoreInfo, CpuGlobalInfo, CpuPowerInfo, CpuTemperatureInfo, collect_cpu_core_data,
        collect_cpu_data, collect_cpu_power_data, collect_cpu_temperature_data,
    },
};

use core::core::{db_insert_query, db_table_query_creation, init_db};

impl CpuGlobalInfo {
    /// Insert global CPU data in database.
    ///
    /// # Arguments
    ///
    /// - `timestamp` : Date trace to history identification.
    /// - `conn` : Allow by a [`Connection`] constructor type the connection with an SQLite database.
    /// - `data` : [`CpuGlobalInfo`] information to insert in database.
    ///
    /// # Returns
    ///
    /// - Insert the [`CpuGlobalInfo`] filled structure in an SQLite database.
    /// - Logs an error if the SQL insert request failed.
    pub fn insert_db(
        conn: &Connection,
        timestamp: &str,
        data: &Self,
    ) -> Result<(), Box<dyn Error>> {
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
        Ok(())
    }
}

impl CpuCoreInfo {
    /// Insert CPU cores usage data in database.
    ///
    /// # Arguments
    ///
    /// - `timestamp` : Date trace to history identification.
    /// - `conn` : Allow by a [`Connection`] constructor type the connection with an SQLite database.
    /// - `data` : [`CpuCoreInfo`] information to insert in database.
    ///
    /// # Returns
    ///
    /// - Insert the [`CpuCoreInfo`] filled structure in an SQLite database.
    /// - Logs an error if the SQL insert request failed.
    fn insert_db(conn: &Connection, timestamp: &str, data: &Self) -> Result<(), Box<dyn Error>> {
        let query = db_insert_query(TABLE_NAME[1], &field_descriptor_core())?;
        for (core_name, core) in &data.cores_usage {
            conn.execute(&query, params![timestamp, core_name, core])?;
        }
        Ok(())
    }
}

impl CpuPowerInfo {
    /// Insert CPU powers data in database.
    ///
    /// # Arguments
    ///
    /// - `timestamp` : Date trace to history identification.
    /// - `conn` : Allow by a [`Connection`] constructor type the connection with an SQLite database.
    /// - `data` : [`CpuPowerInfo`] information to insert in database.
    ///
    /// # Returns
    ///
    /// - Insert the [`CpuPowerInfo`] filled structure in an SQLite database.
    /// - Logs an error if the SQL insert request failed.
    fn insert_db(conn: &Connection, timestamp: &str, data: &Self) -> Result<(), Box<dyn Error>> {
        let query = db_insert_query(TABLE_NAME[2], &field_descriptor_power())?;
        for (zone_name, power) in &data.powers {
            conn.execute(&query, params![timestamp, zone_name, power])?;
        }
        Ok(())
    }
}

impl CpuTemperatureInfo {
    /// Insert CPU temperatures data in database.
    ///
    /// # Arguments
    ///
    /// - `timestamp` : Date trace to history identification.
    /// - `conn` : Allow by a [`Connection`] constructor type the connection with an SQLite database.
    /// - `data` : [`CpuTemperatureInfo`] information to insert in database.
    ///
    /// # Returns
    ///
    /// - Insert the [`CpuTemperatureInfo`] filled structure in an SQLite database.
    /// - Logs an error if the SQL insert request failed.
    fn insert_db(conn: &Connection, timestamp: &str, data: &Self) -> Result<(), Box<dyn Error>> {
        let query = db_insert_query(TABLE_NAME[3], &field_descriptor_temperature())?;
        for (zone_name, temp) in &data.temperatures {
            conn.execute(&query, params![timestamp, zone_name, temp])?;
        }
        Ok(())
    }
}

/// Public function used to send values in SQLite database,
/// from [`collect_cpu_data`] function result.
pub fn get_cpu_info() -> Result<(), Box<dyn Error>> {
    let mut sys =
        System::new_with_specifics(RefreshKind::nothing().with_cpu(CpuRefreshKind::everything()));
    sleep(MINIMUM_CPU_UPDATE_INTERVAL);
    sys.refresh_cpu_all();

    let component = Components::new_with_refreshed_list();

    let cpu = sys.cpus();
    if cpu.is_empty() {
        return Err("Failed to get global CPUs information".to_string().into());
    }

    let data_global = collect_cpu_data(cpu)?;
    let data_cores = collect_cpu_core_data(cpu)?;
    let data_power = collect_cpu_power_data()?;
    let data_temperature = collect_cpu_temperature_data(component)?;

    let query_info = db_table_query_creation(TABLE_NAME[0], &field_descriptor_info())?;
    let query_core = db_table_query_creation(TABLE_NAME[1], &field_descriptor_core())?;
    let query_power = db_table_query_creation(TABLE_NAME[2], &field_descriptor_power())?;
    let query_temperature =
        db_table_query_creation(TABLE_NAME[3], &field_descriptor_temperature())?;

    let conn = init_db(&query_info)?;
    conn.execute_batch(&query_core)?;
    conn.execute_batch(&query_power)?;
    conn.execute_batch(&query_temperature)?;

    let timestamp = Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true);
    CpuGlobalInfo::insert_db(&conn, &timestamp, &data_global)?;
    CpuCoreInfo::insert_db(&conn, &timestamp, &data_cores)?;
    CpuPowerInfo::insert_db(&conn, &timestamp, &data_power)?;
    CpuTemperatureInfo::insert_db(&conn, &timestamp, &data_temperature)?;

    Ok(())
}
