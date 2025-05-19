//! # File utilities module

use log::error;
use rusqlite::{params, Connection};
use serde::Serialize;
use std::error::Error;
use std::{collections::HashMap, fs::read_to_string};

pub const HEADER: &'static str = "BOARD";

/// Available parameters in DMI system file to retrieve motherboard data.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DmiInfo {
    /// BIOS release date.
    BiosDate,
    /// BIOS release version.
    BiosRelease,
    /// BIOS vendor name.
    BiosVendor,
    /// BIOS software version.
    BiosVersion,
    /// Main board (or motherboard) full name.
    BoardName,
    /// Main board (or motherboard) serial number.
    BoardSerial,
    /// Main board (or motherboard) vendor name.
    BoardVendor,
    /// Main board (or motherboard) hardware version.
    BoardVersion,
}

impl DmiInfo {
    /// Linux system file where main board information are located.
    ///
    /// # Returns
    ///
    /// The file path containing the DMI information about the main board.
    pub fn from_file(&self) -> &'static str {
        match self {
            DmiInfo::BiosDate => "/sys/class/dmi/id/bios_date",
            DmiInfo::BiosRelease => "/sys/class/dmi/id/bios_release",
            DmiInfo::BiosVendor => "/sys/class/dmi/id/bios_vendor",
            DmiInfo::BiosVersion => "/sys/class/dmi/id/bios_version",
            DmiInfo::BoardName => "/sys/class/dmi/id/board_name",
            DmiInfo::BoardSerial => "/sys/class/dmi/id/board_serial",
            DmiInfo::BoardVendor => "/sys/class/dmi/id/board_vendor",
            DmiInfo::BoardVersion => "/sys/class/dmi/id/board_version",
        }
    }

    /// Static list of all [`DmiInfo`] parameters.
    pub const INFO: [DmiInfo; 8] = [
        DmiInfo::BiosDate,
        DmiInfo::BiosRelease,
        DmiInfo::BiosVendor,
        DmiInfo::BiosVersion,
        DmiInfo::BoardName,
        DmiInfo::BoardSerial,
        DmiInfo::BoardVendor,
        DmiInfo::BoardVersion,
    ];
}

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
    /// Main board (or motherboard) vendor name.
    pub board_vendor: Option<String>,
    /// Main board (or motherboard) hardware version.
    pub board_version: Option<String>,
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
            && self.board_vendor.is_none()
            && self.board_version.is_none()
    }

    /// Filling all field of [`DmiInfo`] with null value by default.
    pub fn from_map(map: &HashMap<DmiInfo, String>) -> Self {
        BoardInfo {
            bios_date: map.get(&DmiInfo::BiosDate).cloned(),
            bios_release: map.get(&DmiInfo::BiosRelease).cloned(),
            bios_vendor: map.get(&DmiInfo::BiosVendor).cloned(),
            bios_version: map.get(&DmiInfo::BiosVersion).cloned(),
            board_name: map.get(&DmiInfo::BoardName).cloned(),
            board_serial: map.get(&DmiInfo::BoardSerial).cloned(),
            board_version: map.get(&DmiInfo::BoardVersion).cloned(),
            board_vendor: map.get(&DmiInfo::BoardVendor).cloned(),
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
/// Each element found for motherboard info.
pub fn read_dmi_data() -> HashMap<DmiInfo, String> {
    let mut data = HashMap::new();
    for &field in DmiInfo::INFO.iter() {
        let path = field.from_file();
        match read_to_string(path) {
            Ok(content) => {
                data.insert(field, content.trim().to_string());
            }
            Err(e) => {
                error!("[{HEADER}] Data 'Failed to read DMI file' {path} : {e}");
            }
        }
    }
    data
}
