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
    let data = BoardInfo::from_map(&dmi);

    for key in DmiInfo::INFO.iter() {
        if !dmi.contains_key(key) {
            error!("[{HEADER}] Data 'Failed to retrieve {}'", key.from_file());
        }
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
