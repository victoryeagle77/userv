//! # Lib file for storage data module
//!
//! This module provides functionalities to retrieve storage data on Unix-based systems.

use chrono::{SecondsFormat, Utc};
use rusqlite::Connection;
use std::error::Error;
use sysinfo::{DiskRefreshKind, Disks};

mod utils;
use core::core::init_db;
use utils::*;

const REQUEST: &str = "
    CREATE TABLE IF NOT EXISTS storage_data (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        timestamp TEXT NOT NULL,
        name TEXT NOT NULL,
        bandwidth_read_MB INTEGER,
        bandwidth_write_MB INTEGER,
        energy_consumed_J REAL,
        file_mount TEXT,
        file_system TEXT,
        kind TEXT,
        space_available_MB INTEGER,
        space_total_MB INTEGER
    );
    CREATE TABLE IF NOT EXISTS smart_data (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        device_id INTEGER NOT NULL,
        uptime_hours INTEGER,
        sectors_reallocated INTEGER,
        sectors_pending INTEGER,
        sectors_pending_current INTEGER,
        temperature INTEGER,
        FOREIGN KEY(device_id) REFERENCES disk_data(id)
    );";

/// Function that retrieves all detailed disk information.
///
/// # Returns
///
/// The compilation of completed structures concerning all disk information.
/// * [`DiskInfo`] concerning global system info of the device storage.
/// * [`SmartInfo`] concerning smart info for the device storage if it's possible.
fn collect_storage_data(conn: &Connection) -> Result<(), Box<dyn Error>> {
    let timestamp = Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true);
    let disks = Disks::new_with_refreshed_list_specifics(DiskRefreshKind::everything());

    for disk in disks.list() {
        let disk_info = DiskInfo::from_device(disk)?;
        disk_info.insert_db(conn, &timestamp)?;
        let id = conn.last_insert_rowid();
        if let Some(smart) = &disk_info.smart_info {
            smart.insert_db(conn, id)?;
        }
    }
    Ok(())
}

/// Public function used to send JSON formatted values,
/// from [`collect_storage_data`] function result.
pub fn get_storage_info() -> Result<(), Box<dyn Error>> {
    let conn = init_db(REQUEST)?;
    collect_storage_data(&conn)?;
    Ok(())
}
