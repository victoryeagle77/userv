//! # Motherboard data Module
//!
//! This module provides functionality to retrieve motherboard and bios data on Unix-based systems.

use serde::Serialize;
use serde_json::json;
use std::collections::HashMap;

use crate::utils::read_file_content;

const MOTHERBOARD: &str = "/sys/class/dmi/id/";

const HEADER: &str = "MOTHERBOARD";

/// Collection of collected motherboard data
#[derive(Debug, Serialize)]
struct MotherboardInfo {
    /// Motherboard name.
    board_name: Option<String>,
    /// Motherboard serial number.
    board_serial: Option<String>,
    /// Motherboard version.
    board_version: Option<String>,
    /// Motherboard vendor.
    board_vendor: Option<String>,
    /// BIOS release date.
    bios_date: Option<String>,
    /// BIOS release.
    bios_release: Option<String>,
    /// BIOS version.
    bios_version: Option<String>,
    /// BIOS vendor.
    bios_vendor: Option<String>,
    /// Product UUID.
    product_uuid: Option<String>,
}

/// Retrieves data of the main motherboard
/// This function uses the `dmi` directory to gather motherboard information.
///
/// # Arguments
///
/// * `dir` - A string slice that holds the path to the directory to locate the information files.
/// * `files` - A string slice that holds the path to the file to be read in main directory.
///
/// # Returns
///
/// - `data` : `HashMap` = Each element found by files in dmi directory.
fn read_dmi_data(dir: &str, files: &[&str]) -> HashMap<String, String> {
    let mut data = HashMap::new();

    for &file in files {
        let path = format!("{}{}", dir, file);
        if let Some(content) = read_file_content(&path) {
            data.insert(file.to_string(), content.trim().to_string());
        }
    }

    data
}

/// Function that retrieves detailed motherboard information,
/// By dmi files system reading and data collecting.
///
/// # Returns
///
/// `result` : Completed `MotherboardInfo` structure with all motherboard information
/// - Motherboard name
/// - Motherboard serial number
/// - Motherboard version
/// - Motherboard vendor
/// - Bios update
/// - Bios date release
/// - Bios vendor
/// - Bios version
/// - Motherboard uuid
fn collect_motherboard_data() -> Result<MotherboardInfo, String> {
    const DATA: [&str; 9] = [
        "board_name",
        "board_serial",
        "board_version",
        "board_vendor",
        "bios_date",
        "bios_release",
        "bios_vendor",
        "bios_version",
        "product_uuid",
    ];

    let dmi_info = read_dmi_data(MOTHERBOARD, &DATA);

    let result = MotherboardInfo {
        board_name: dmi_info.get(DATA[0]).cloned().map(Some).unwrap_or(None),
        board_serial: dmi_info.get(DATA[1]).cloned().map(Some).unwrap_or(None),
        board_version: dmi_info.get(DATA[2]).cloned().map(Some).unwrap_or(None),
        board_vendor: dmi_info.get(DATA[3]).cloned().map(Some).unwrap_or(None),
        bios_date: dmi_info.get(DATA[4]).cloned().map(Some).unwrap_or(None),
        bios_release: dmi_info.get(DATA[5]).cloned().map(Some).unwrap_or(None),
        bios_vendor: dmi_info.get(DATA[6]).cloned().map(Some).unwrap_or(None),
        bios_version: dmi_info.get(DATA[7]).cloned().map(Some).unwrap_or(None),
        product_uuid: dmi_info.get(DATA[8]).cloned().map(Some).unwrap_or(None),
    };

    Ok(result)
}

/// Public function used to send JSON formatted values,
/// from `collect_motherboard_data` function result.
pub fn get_motherboard_info() {
    match collect_motherboard_data() {
        Ok(values) => {
            let motherboard_json_info: serde_json::Value = json!({
                HEADER: {
                    "board_name": values.board_name.unwrap_or_else(|| "NULL".to_string()),
                    "board_serial": values.board_serial.unwrap_or_else(|| "NULL".to_string()),
                    "board_version": values.board_version.unwrap_or_else(|| "NULL".to_string()),
                    "board_vendor": values.board_vendor.unwrap_or_else(|| "NULL".to_string()),
                    "bios_date": values.bios_date.unwrap_or_else(|| "NULL".to_string()),
                    "bios_release": values.bios_release.unwrap_or_else(|| "NULL".to_string()),
                    "bios_vendor": values.bios_vendor.unwrap_or_else(|| "NULL".to_string()),
                    "bios_version": values.bios_version.unwrap_or_else(|| "NULL".to_string()),
                    "product_uuid": values.product_uuid.unwrap_or_else(|| "NULL".to_string()),
                }
            });

            println!(
                "{}",
                serde_json::to_string_pretty(&motherboard_json_info).unwrap()
            );
        }
        Err(e) => {
            eprintln!("{}", e);
        }
    }
}
