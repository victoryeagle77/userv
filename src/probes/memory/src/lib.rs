//! # Lib file for memory data module
//!
//! This module provides main functionality to retrieve memories data on Unix-based systems.

use chrono::{SecondsFormat, Utc};
use rusqlite::{Connection, ToSql, params};
use std::{error::Error, fs::read};
use sysinfo::{MemoryRefreshKind, System};

mod dbms;
mod utils;

use core::core::{
    DMIDECODE_BIN, ENTRY_BIN, db_insert_query, db_insert_unique, db_table_query_creation, init_db,
};
use dbms::*;
use utils::*;

impl MemInfo {
    /// Insert memory global info parameters into the database.
    ///
    /// # Arguments
    ///
    /// - `conn`: Connection to SQLite database.
    /// - `timestamp`: Timestamp of the measurement.
    /// - `data`: [`MemInfo`] information to insert in database.
    ///
    /// # Returns
    ///
    /// - Insert the [`MemInfo`] filled structures in an SQLite database.
    /// - Logs an error if the SQL insert request failed.
    ///
    /// # Operating
    ///
    /// The [`MemInfo`] is a set of dynamics information retrieved and refresh at each call.
    pub fn insert_db(
        conn: &mut Connection,
        timestamp: &str,
        data: &Self,
    ) -> Result<(), Box<dyn Error>> {
        let query_info = db_insert_query(TABLE_NAME[0], &field_descriptor_info())?;
        let mut stmt = conn.prepare(&query_info)?;

        stmt.execute(params![
            timestamp,
            data.bandwidth_read,
            data.bandwidth_write,
            data.ram_total,
            data.ram_used,
            data.ram_free,
            data.ram_available,
            data.ram_power_consumption,
            data.swap_total,
            data.swap_used,
            data.swap_free,
        ])?;

        Ok(())
    }
}

impl MemDeviceInfo {
    /// Insert memory device info parameters into the database.
    ///
    /// # Arguments
    ///
    /// - `conn`: Connection to SQLite database.
    /// - `timestamp`: Timestamp of the measurement.
    /// - `data`: [`MemDeviceInfo`] list of RAM modules (optional, can be None).
    ///
    /// # Returns
    ///
    /// - Insert the [`MemDeviceInfo`] filled structures in an SQLite database.
    /// - Logs an error if the SQL insert request failed.
    ///
    /// # Operating
    ///
    /// The [`MemDeviceInfo`] is a set of statics information, their are retrieved only one time.
    pub fn insert_db(
        conn: &mut Connection,
        timestamp: &str,
        data: Option<&Vec<Self>>,
    ) -> Result<(), Box<dyn Error>> {
        if let Some(data) = data {
            let conflict_param = ["device_id"];
            let update_param = ["id", "timestamp"];

            let (create_index_sql, insert_stmt_sql) = db_insert_unique(
                TABLE_NAME[1],
                &field_descriptor_device(),
                &conflict_param,
                &update_param,
            )?;

            if let Some(index) = create_index_sql {
                conn.execute_batch(&index)?;
            }

            let mut stmt = conn.prepare(&insert_stmt_sql)?;
            for module in data {
                let kind_str = module.kind.as_str();

                let values: Vec<&dyn ToSql> = vec![
                    &timestamp,
                    &module.id,
                    &kind_str,
                    &module.size,
                    &module.speed,
                    &module.voltage,
                ];

                stmt.execute(&*values)?;
            }
        } else {
            return Err("Data 'Memory device table not creatable'".into());
        }
        Ok(())
    }
}

/// Initialize the [`sysinfo`] library to start the collect by [`mem_data_build`].
/// Push in SQLite memory database the data retrieve by:
/// - [`MemDeviceInfo`]: Information about memory device(s) module(s) detected on OS.
/// - [`MemInfo`]: Global information about memory.
///
/// # Returns
///
/// Failure if we can't retrieve information or push it in database.
pub fn get_mem_info() -> Result<(), Box<dyn Error>> {
    let entry_buf = read(ENTRY_BIN)?;
    let dmi_buf = read(DMIDECODE_BIN)?;

    let mut sys = System::new_all();
    sys.refresh_memory_specifics(MemoryRefreshKind::everything());

    let ram_test = get_mem_test()?;
    let ram_device = get_mem_device(&entry_buf, &dmi_buf)?;

    let data_devices = collect_mem_devices(ram_device);
    let data_global = collect_mem_data(ram_test, data_devices.as_ref(), &sys);

    let query_info = db_table_query_creation(TABLE_NAME[0], &field_descriptor_info())?;
    let query_device = db_table_query_creation(TABLE_NAME[1], &field_descriptor_device())?;

    let mut conn = init_db(&query_info)?;
    conn.execute_batch(&query_device)?;

    let timestamp = Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true);
    MemInfo::insert_db(&mut conn, &timestamp, &data_global)?;
    MemDeviceInfo::insert_db(&mut conn, &timestamp, data_devices.as_ref())?;

    Ok(())
}
