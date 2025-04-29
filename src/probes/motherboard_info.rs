//! # Motherboard data Module
//!
//! This module provides functionality to retrieve motherboard and bios data on Unix-based systems.

use log::error;
use serde::Serialize;
use serde_json::{json, Value};
use std::{collections::HashMap, error::Error, fs::read_to_string};

use crate::utils::write_json_to_file;

const HEADER: &str = "MOTHERBOARD";
const LOGGER: &str = "log/motherboard_data.json";

const MOTHERBOARD_FILES: [&str; 8] = [
    "/sys/class/dmi/id/board_name",
    "/sys/class/dmi/id/board_serial",
    "/sys/class/dmi/id/board_version",
    "/sys/class/dmi/id/board_vendor",
    "/sys/class/dmi/id/bios_date",
    "/sys/class/dmi/id/bios_release",
    "/sys/class/dmi/id/bios_vendor",
    "/sys/class/dmi/id/bios_version",
];

/// Collection of collected motherboard data.
#[derive(Debug, Serialize)]
struct MotherboardInfo {
    board_name: Option<String>,
    board_serial: Option<String>,
    board_version: Option<String>,
    board_vendor: Option<String>,
    bios_date: Option<String>,
    bios_release: Option<String>,
    bios_version: Option<String>,
    bios_vendor: Option<String>,
}

impl MotherboardInfo {
    /// Converts [`MotherboardInfo`] into a JSON object.
    fn to_json(&self) -> Value {
        json!({
            "motherboard_name": self.board_name,
            "motherboard_serial": self.board_serial,
            "motherboard_version": self.board_version,
            "motherboard_vendor": self.board_vendor,
            "bios_date": self.bios_date,
            "bios_release": self.bios_release,
            "bios_version": self.bios_version,
            "bios_vendor": self.bios_vendor,
        })
    }

    /// Check if we have no information available to store in [`MotherboardInfo`].
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

    /// Filling all field of [`MotherboardInfo`] with null value by default.
    fn default() -> Self {
        MotherboardInfo {
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

/// Retrieves data of the main motherboard.
/// This function uses the `dmi` directory to gather motherboard information.
///
/// # Returns
///
/// - `data`: Each element found for motherboard info.
fn read_dmi_data() -> HashMap<String, String> {
    let mut data = HashMap::new();
    for &path in MOTHERBOARD_FILES.iter() {
        match read_to_string(path) {
            Ok(content) => {
                let key = path.rsplit('/').next().unwrap_or_default();
                data.insert(key.to_string(), content.trim().to_string());
            }
            Err(e) => {
                error!("[{HEADER}] Data 'Failed to read DMI file' {path} : {e}");
            }
        }
    }
    data
}

/// Retrieves information about the motherboard of an IT equipment.
///
/// # Returns
///
/// - Completed [`MotherboardInfo`] structure with all board and BIOS information.
/// - An error when no information about BIOS or Motherboard found.
fn collect_motherboard_data() -> Result<MotherboardInfo, Box<dyn Error>> {
    let dmi_info = read_dmi_data();
    let mut data = MotherboardInfo::default();

    for (key, value) in dmi_info.iter() {
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

    if data.is_empty() {
        return Err("Data 'No information about BIOS or Motherboard found'".into());
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

    Ok(data)
}

/// Public function used to send JSON formatted values,
/// from [`collect_motherboard_data`] function result.
pub fn get_motherboard_info() -> Result<(), Box<dyn Error>> {
    let data = collect_motherboard_data()?;
    let values = json!({ HEADER: data.to_json() });
    write_json_to_file(|| Ok(values), LOGGER)?;
    Ok(())
}
