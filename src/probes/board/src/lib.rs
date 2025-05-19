//! # Lib file for board data module
//!
//! This module provides functionalities to retrieve motherboard / main board and bios data on Unix-based systems.

use log::error;
use serde::Serialize;
use serde_json::{json, Value};
use std::error::Error;

mod utils;
use utils::*;

/// Collection of collected motherboard data.
#[derive(Debug, Serialize)]
struct BoardInfo {
    /// BIOS release date.
    bios_date: Option<String>,
    /// BIOS release version.
    bios_release: Option<String>,
    /// BIOS software version.
    bios_version: Option<String>,
    /// BIOS vendor name.
    bios_vendor: Option<String>,
    /// Main board (or motherboard) full name.
    board_name: Option<String>,
    /// Main board (or motherboard) serial number.
    board_serial: Option<String>,
    /// Main board (or motherboard) hardware version.
    board_version: Option<String>,
    /// Main board (or motherboard) vendor name.
    board_vendor: Option<String>,
}

impl BoardInfo {
    /// Converts [`BoardInfo`] into a JSON object.
    fn to_json(&self) -> Value {
        json!({
            "bios_date": self.bios_date,
            "bios_release": self.bios_release,
            "bios_version": self.bios_version,
            "bios_vendor": self.bios_vendor,
            "board_name": self.board_name,
            "board_serial": self.board_serial,
            "board_version": self.board_version,
            "board_vendor": self.board_vendor,
        })
    }

    /// Check if we have no information available to store in [`BoardInfo`].
    fn is_empty(&self) -> bool {
        self.board_name.is_none()
            && self.board_serial.is_none()
            && self.board_version.is_none()
            && self.board_vendor.is_none()
            && self.bios_date.is_none()
            && self.bios_release.is_none()
            && self.bios_vendor.is_none()
            && self.bios_version.is_none()
    }

    /// Filling all field of [`BoardInfo`] with null value by default.
    fn default() -> Self {
        BoardInfo {
            board_name: None,
            board_serial: None,
            board_version: None,
            board_vendor: None,
            bios_date: None,
            bios_release: None,
            bios_vendor: None,
            bios_version: None,
        }
    }
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
    let data = collect_board_data()?;
    let values = json!({ HEADER: data.to_json() });
    write_json_to_file(|| Ok(values), LOGGER)?;
    Ok(())
}
