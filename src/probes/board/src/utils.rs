//! # File utilities module

use chrono::{SecondsFormat::Millis, Utc};
use log::error;
use serde_json::{json, Value};
use std::{collections::HashMap, error::Error, fs::read_to_string, fs::OpenOptions, io::Write};

pub const HEADER: &'static str = "BOARD";
pub const LOGGER: &'static str = "log/board_data.json";

const BOARD_FILES: [&'static str; 8] = [
    "/sys/class/dmi/id/board_name",
    "/sys/class/dmi/id/board_serial",
    "/sys/class/dmi/id/board_version",
    "/sys/class/dmi/id/board_vendor",
    "/sys/class/dmi/id/bios_date",
    "/sys/class/dmi/id/bios_release",
    "/sys/class/dmi/id/bios_vendor",
    "/sys/class/dmi/id/bios_version",
];

/// Retrieves data of the main motherboard.
/// This function uses the `dmi` directory to gather motherboard information.
///
/// # Returns
///
/// - `data`: Each element found for motherboard info.
pub fn read_dmi_data() -> HashMap<String, String> {
    let mut data = HashMap::new();
    for &path in BOARD_FILES.iter() {
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

/// Writes JSON formatted data in a file
///
/// # Arguments
///
/// * `data` : JSON serialized collected metrics data to write
/// * `path` : File path use to writing data
///
/// # Return
///
/// - Custom error message if an error occurs during JSON data serialization or file handling.
pub fn write_json_to_file<F>(generator: F, path: &'static str) -> Result<(), Box<dyn Error>>
where
    F: FnOnce() -> Result<Value, Box<dyn Error>>,
{
    let mut data: Value = generator()?;

    // Timestamp implementation in JSON object
    let timestamp = Some(Utc::now().to_rfc3339_opts(Millis, true));

    // Format data to JSON object
    if data.is_object() {
        data.as_object_mut()
            .unwrap()
            .insert("timestamp".to_owned(), json!(timestamp));
    } else if data.is_array() {
        for item in data.as_array_mut().unwrap() {
            if item.is_object() {
                item.as_object_mut()
                    .unwrap()
                    .insert("timestamp".to_owned(), json!(timestamp));
            }
        }
    }

    let mut file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(path)?;
    let log = serde_json::to_string_pretty(&data)?;

    file.write_all(log.as_bytes())?;

    Ok(())
}
