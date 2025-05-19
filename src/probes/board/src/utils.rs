//! # File utilities module

use log::error;
use rusqlite::{params, Connection};
use serde::Serialize;
use std::error::Error;
use std::{collections::HashMap, fs::read_to_string};

pub const HEADER: &'static str = "BOARD";

const BOARD_FILES: [&'static str; 8] = [
    "/sys/class/dmi/id/bios_date",
    "/sys/class/dmi/id/bios_release",
    "/sys/class/dmi/id/bios_vendor",
    "/sys/class/dmi/id/bios_version",
    "/sys/class/dmi/id/board_name",
    "/sys/class/dmi/id/board_serial",
    "/sys/class/dmi/id/board_version",
    "/sys/class/dmi/id/board_vendor",
];

/// Collection of collected motherboard data.
#[derive(Debug, Serialize)]
pub struct BoardInfo {
    /// BIOS release date.
    pub bios_date: Option<String>,
    /// BIOS release version.
    pub bios_release: Option<String>,
    /// BIOS software version.
    pub bios_version: Option<String>,
    /// BIOS vendor name.
    pub bios_vendor: Option<String>,
    /// Main board (or motherboard) full name.
    pub board_name: Option<String>,
    /// Main board (or motherboard) serial number.
    pub board_serial: Option<String>,
    /// Main board (or motherboard) hardware version.
    pub board_version: Option<String>,
    /// Main board (or motherboard) vendor name.
    pub board_vendor: Option<String>,
}

impl BoardInfo {
    /// Check if we have no information available to store in [`BoardInfo`].
    pub fn is_empty(&self) -> bool {
        self.board_name.is_none()
            && self.bios_date.is_none()
            && self.bios_release.is_none()
            && self.bios_vendor.is_none()
            && self.bios_version.is_none()
            && self.board_serial.is_none()
            && self.board_version.is_none()
            && self.board_vendor.is_none()
    }

    /// Filling all field of [`BoardInfo`] with null value by default.
    pub fn default() -> Self {
        BoardInfo {
            bios_date: None,
            bios_release: None,
            bios_vendor: None,
            bios_version: None,
            board_name: None,
            board_serial: None,
            board_version: None,
            board_vendor: None,
        }
    }

    /// Insert main board parameters in database.
    ///
    /// # Arguments
    ///
    /// - `conn` : Allow by a [`Connection`] constructor type the connection with an SQLite database.
    /// - `timestamp` : Date trace for the history identification.
    /// - `data` : [`BoardInfo`] information to insert in database.
    ///
    /// # Returns
    ///
    /// - Insert the [`BoardInfo`] filled structure in an SQLite database.
    /// - Logs an error if the SQL insert request failed.
    pub fn insert_db(
        conn: &Connection,
        timestamp: &str,
        data: &Self,
    ) -> Result<(), Box<dyn Error>> {
        conn.execute(
            "INSERT INTO board_data (
                timestamp,
                bios_date,
                bios_release,
                bios_vendor,
                bios_version,
                board_name,
                board_serial,
                board_vendor,
                board_version
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                timestamp,
                data.bios_date,
                data.bios_release,
                data.bios_vendor,
                data.bios_version,
                data.board_name,
                data.board_serial,
                data.board_vendor,
                data.board_version
            ],
        )?;
        Ok(())
    }
}

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
