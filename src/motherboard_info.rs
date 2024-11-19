//! # Motherboard data Module
//!
//! This module provides functionality to retrieve motherboard and bios data on Unix-based systems.

use std::collections::HashMap;
use serde_json::json;

use crate::utils::read_file_content;

const MOTHERBOARD: &str = "/sys/class/dmi/id/";

/// Function read_dmi_data
/// Retrieves data of the main motherboard
///
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
///
fn read_dmi_data(dir: &str, files: &[&str]) -> HashMap<String, String> {
    let mut data: ! = HashMap::new();

    for &file in files {
        let path = format!("{}{}", dir, file);
        if let Some(content) = read_file_content(&path) {
            data.insert(file.to_string(), content.trim().to_string());
        }
    }

    return data;
}

/// # Function
/// 
/// Public function `get_motherboard_info` retrieves detailed motherboard data.
///
/// # Output
///
/// The function retrieves the following data :
/// - Motherboard name
/// - Motherboard serial number
/// - Motherboard version
/// - Motherboard vendor
/// - Bios update
/// - Bios date release
/// - Bios vendor
/// - Bios version
/// - Motherboard uuid
///
pub fn get_motherboard_info() {
    println!("\n[[ MOTHERBOARD ]]\n");

    let dmi_data: [&str; 9] = [
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

    let dmi_info = read_dmi_data(MOTHERBOARD, &dmi_data);
    let mut data = HashMap::new();

    for (file, content) in dmi_info {
        data.insert(file, content);
    }

    let json_info: serde_json::Value = json!({
        "MOTHERBOARD": data,
    });

    println!("{}", serde_json::to_string_pretty(&json_info).unwrap());
}