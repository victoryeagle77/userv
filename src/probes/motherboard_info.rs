//! # Motherboard data Module
//!
//! This module provides functionality to retrieve motherboard and bios data on Unix-based systems.

use log::error;
use serde::Serialize;
use serde_json::{json, Value};
use std::{collections::HashMap, error::Error};

use crate::utils::{read_file_content, write_json_to_file};

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
    /// Converts `MotherboardInfo` into a JSON object.
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
}

/// Retrieves data of the main motherboard.
/// This function uses the `dmi` directory to gather motherboard information.
///
/// # Returns
///
/// - `HashMap<String, String>`: Each element found for motherboard info.
fn read_dmi_data() -> HashMap<String, String> {
    MOTHERBOARD_FILES
        .iter()
        .filter_map(|&path| match read_file_content(path) {
            Some(content) => {
                let key: &str = path.split('/').last().unwrap_or("");
                Some((key.to_string(), content.trim().to_string()))
            }
            None => {
                error!("[{HEADER}] Data 'Failed to parse file' : {path}");
                None
            }
        })
        .collect()
}

/// Function that retrieves detailed motherboard information,
/// By dmi files system reading and data collecting.
///
/// # Returns
///
/// `Result<MotherboardInfo, String>`: Completed `MotherboardInfo` structure with all motherboard information
///
/// - Motherboard name
/// - Motherboard serial number
/// - Motherboard version
/// - Motherboard vendor
/// - Bios update
/// - Bios date release
/// - Bios vendor
/// - Bios version
fn collect_motherboard_data() -> MotherboardInfo {
    let dmi_info: HashMap<String, String> = read_dmi_data();

    let mut data: MotherboardInfo = MotherboardInfo {
        board_name: None,
        board_serial: None,
        board_version: None,
        board_vendor: None,
        bios_date: None,
        bios_release: None,
        bios_vendor: None,
        bios_version: None,
    };

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
            _ => error!("[{HEADER}] Unknown DMI key: {key}"),
        }
    }

    // Log missing information
    if data.board_name.is_none() {
        error!("[{HEADER}] Data 'Failed to retrieve motherboard name'");
    }
    if data.board_serial.is_none() {
        error!("[{HEADER}] Data 'Failed to retrieve motherboard serial'");
    }
    if data.board_version.is_none() {
        error!("[{HEADER}] Data 'Failed to retrieve motherboard version'");
    }
    if data.board_vendor.is_none() {
        error!("[{HEADER}] Data 'Failed to retrieve motherboard vendor'");
    }
    if data.bios_date.is_none() {
        error!("[{HEADER}] Data 'Failed to retrieve BIOS date'");
    }
    if data.bios_release.is_none() {
        error!("[{HEADER}] Data 'Failed to retrieve BIOS release'");
    }
    if data.bios_vendor.is_none() {
        error!("[{HEADER}] Data 'Failed to retrieve BIOS vendor'");
    }
    if data.bios_version.is_none() {
        error!("[{HEADER}] Data 'Failed to retrieve BIOS version'");
    }

    data
}

/// Public function used to send JSON formatted values,
/// from `collect_motherboard_data` function result.
pub fn get_motherboard_info() {
    let data = || -> Result<Value, Box<dyn Error>> {
        let values: MotherboardInfo = collect_motherboard_data();
        Ok(json!({ HEADER: values.to_json() }))
    };

    write_json_to_file(data, LOGGER, HEADER);
}
