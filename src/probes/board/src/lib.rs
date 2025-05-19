//! # Lib file for board data module
//!
//! This module provides functionalities to retrieve motherboard / main board and bios data on Unix-based systems.

use chrono::{SecondsFormat, Utc};
use log::error;
use std::error::Error;

mod utils;
use utils::*;

use core::core::init_db;

const REQUEST: &'static str = "CREATE TABLE IF NOT EXISTS board_data (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        timestamp TEXT NOT NULL,
        bios_date TEXT,
        bios_release TEXT,
        bios_vendor TEXT,
        bios_version TEXT,
        board_name TEXT,
        board_serial TEXT,
        board_vendor TEXT,
        board_version TEXT
    )";

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
            "bios_date" => data.bios_date = Some(value.clone()),
            "bios_release" => data.bios_release = Some(value.clone()),
            "bios_vendor" => data.bios_vendor = Some(value.clone()),
            "bios_version" => data.bios_version = Some(value.clone()),
            "board_name" => data.board_name = Some(value.clone()),
            "board_serial" => data.board_serial = Some(value.clone()),
            "board_vendor" => data.board_vendor = Some(value.clone()),
            "board_version" => data.board_version = Some(value.clone()),
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

/// Public function used to send values in SQLite database,
/// from [`collect_board_data`] function result.
pub fn get_board_info() -> Result<(), Box<dyn Error>> {
    let mut conn = init_db(REQUEST)?;
    let timestamp = Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true);
    let data = collect_board_data()?;
    BoardInfo::insert_db(&mut conn, &timestamp, &data)?;
    Ok(())
}
