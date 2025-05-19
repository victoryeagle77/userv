//! # Lib file for board data module
//!
//! This module provides functionalities to retrieve motherboard / main board and bios data on Unix-based systems.

use chrono::{SecondsFormat, Utc};
use log::error;
use rusqlite::{params, Connection};
use std::{error::Error, path::Path};

mod utils;
use utils::*;

const DATABASE: &'static str = "log/data.db";

/// Initialize the SQLite database and create the table if needed.
///
/// # Arguments
///
/// - `path` : Path to database file.
///
/// # Returns
///
/// - A [`Connection`] constructor to initialize database parameters.
/// - An error if the table creation or database initialization failed.
fn init_db(path: &'static str) -> Result<Connection, Box<dyn Error>> {
    let conn = Connection::open(Path::new(path))?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS board_data (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            timestamp TEXT,
            bios_date TEXT,
            bios_release TEXT,
            bios_version TEXT,
            board_name TEXT,
            board_serial TEXT,
            board_version TEXT,
            board_vendor TEXT
        )",
        [],
    )?;
    Ok(conn)
}

/// Insert network interface parameters into the database.
///
/// # Arguments
///
/// - `conn` : Allow by a [`Connection`] constructor type the connection with an SQLite database.
/// - `data` : The data structure to insert in database.
///
/// # Returns
///
/// - Insert the [`NetworkInterface`] filled structure in an SQLite database.
/// - Logs an error if the SQL insert request failed.
fn insert_db(conn: &Connection, timestamp: &str, data: &BoardInfo) -> Result<(), Box<dyn Error>> {
    conn.execute(
        "INSERT INTO board_data (
            timestamp,
            bios_date,
            bios_release,
            bios_version,
            board_name,
            board_serial,
            board_version,
            board_vendor
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            timestamp,
            data.bios_date,
            data.bios_release,
            data.bios_version,
            data.board_name,
            data.board_serial,
            data.board_version,
            data.board_vendor
        ],
    )?;
    Ok(())
}

/// Retrieves information about the motherboard of an IT equipment.
///
/// # Returns
///
/// - Completed [`BoardInfo`] structure with all board and BIOS information.
/// - An error when no information about BIOS or Motherboard found.
fn collect_board_data() -> Result<BoardInfo, Box<dyn Error>> {
    let dmi = read_dmi_data();
    let mut data = BoardInfo::default();

    for (key, value) in dmi.iter() {
        match key.as_str() {
            "board_name" => data.board_name = Some(value.clone()),
            "board_serial" => data.board_serial = Some(value.clone()),
            "board_version" => data.board_version = Some(value.clone()),
            "board_vendor" => data.board_vendor = Some(value.clone()),
            "bios_date" => data.bios_date = Some(value.clone()),
            "bios_release" => data.bios_release = Some(value.clone()),
            "bios_vendor" => data.bios_vendor = Some(value.clone()),
            "bios_version" => data.bios_version = Some(value.clone()),
            _ => error!("[{HEADER}] Data 'Unknown DMI key' : {key}"),
        }
    }

    if data.board_name.is_none() {
        error!("[{HEADER}] Data 'Failed to retrieve motherboard name'");
    } else if data.board_serial.is_none() {
        error!("[{HEADER}] Data 'Failed to retrieve motherboard serial'");
    } else if data.board_version.is_none() {
        error!("[{HEADER}] Data 'Failed to retrieve motherboard version'");
    } else if data.board_vendor.is_none() {
        error!("[{HEADER}] Data 'Failed to retrieve motherboard vendor'");
    } else if data.bios_date.is_none() {
        error!("[{HEADER}] Data 'Failed to retrieve BIOS date'");
    } else if data.bios_release.is_none() {
        error!("[{HEADER}] Data 'Failed to retrieve BIOS release'");
    } else if data.bios_vendor.is_none() {
        error!("[{HEADER}] Data 'Failed to retrieve BIOS vendor'");
    } else if data.bios_version.is_none() {
        error!("[{HEADER}] Data 'Failed to retrieve BIOS version'");
    }

    if data.is_empty() {
        Err("Data 'No information about BIOS or Motherboard found'".into())
    } else {
        Ok(data)
    }
}

/// Public function used to send JSON formatted values,
/// from [`collect_board_data`] function result.
pub fn get_board_info() -> Result<(), Box<dyn Error>> {
    let mut conn = init_db(DATABASE)?;
    let timestamp = Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true);
    let data = collect_board_data()?;
    insert_db(&mut conn, &timestamp, &data)?;
    Ok(())
}
