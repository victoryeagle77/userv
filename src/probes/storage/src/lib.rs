//! # Lib file for storage data module
//!
//! This module provides functionalities to retrieve storage data on Unix-based systems.

use chrono::{SecondsFormat, Utc};
use libc::{c_void, close, open, read};
use log::error;
use regex::Regex;
use rusqlite::{params, Connection};
use std::{error::Error, ffi::CString};
use sysinfo::{Disk, DiskRefreshKind, Disks};

mod utils;
use utils::*;

const DATABASE: &'static str = "log/data.db";

/// Initialize the SQLite database and create the table if needed.
///
/// # Arguments
///
/// - `path` : Path to database file.
///
/// # Returns
///
/// - A [`Connection`] constructor to initialize database parameters.
/// - An error if the table creation or database initialization failed.
fn init_db(path: &'static str) -> Result<Connection, Box<dyn Error>> {
    let conn = Connection::open(path)?;
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS storage_data (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            timestamp TEXT NOT NULL,
            name TEXT NOT NULL,
            bandwidth_read INTEGER,
            bandwidth_write INTEGER,
            energy_consumed REAL,
            file_mount TEXT,
            file_system TEXT,
            kind TEXT,
            space_available INTEGER,
            space_total INTEGER
        );
        CREATE TABLE IF NOT EXISTS smart_data (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            device_id INTEGER NOT NULL,
            uptime_hours INTEGER,
            sectors_reallocated INTEGER,
            sectors_pending INTEGER,
            sectors_pending_current INTEGER,
            temperature INTEGER,
            FOREIGN KEY(device_id) REFERENCES disk_data(id)
        );
        ",
    )?;
    Ok(conn)
}

impl SmartInfo {
    /// Insert network interface parameters into the database.
    ///
    /// # Arguments
    ///
    /// - `conn` : Allow by a [`Connection`] constructor type the connection with an SQLite database.
    /// - `data` : The data structure to insert in database.
    ///
    /// # Returns
    ///
    /// - Insert the [`SmartInfo`] filled structure in an SQLite database.
    /// - Logs an error if the SQL insert request failed.
    fn insert_db(&self, conn: &Connection, device_id: i64) -> Result<(), Box<dyn Error>> {
        conn.execute(
            "INSERT INTO smart_data (
                device_id,
                uptime_hours,
                sectors_reallocated,
                sectors_pending,
                sectors_pending_current,
                temperature
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                device_id,
                self.uptime_hours,
                self.sectors_reallocated,
                self.sectors_pending,
                self.sectors_pending_current,
                self.temperature,
            ],
        )?;
        Ok(())
    }

    /// Function that retrieves smart disk information.
    /// * 5 : Reallocated Sector Count
    /// * 9 : Power-On Hours
    /// * 194 : Temperature
    /// * 196 : Reallocation Event Count
    /// * 197 : Current Pending Sector Count
    ///
    /// # Arguments
    ///
    /// - `path` : Disk device path in system.
    ///
    /// # Returns
    ///
    /// - [`SmartInfo`] filled structure with disk information.
    /// - Error message if CString can not be created, file descriptor content or final extracted data are null.
    fn collect_smart_data(path: &str) -> Result<SmartInfo, Box<dyn Error>> {
        let device =
            CString::new(path).map_err(|e| format!("INIT 'Failed to create CString' : {e}"))?;
        let fd = unsafe { open(device.as_ptr(), 0) };

        if fd < 0 {
            error!("[{HEADER}] Data 'Failed to open device for smart information'");
            return Err("Data 'Failed to open device for smart information'".into());
        }

        let mut buffer = [0u8; 512];
        let bytes = unsafe { read(fd, buffer.as_mut_ptr() as *mut c_void, buffer.len()) };

        if bytes < 0 {
            unsafe { close(fd) };
            error!("[{HEADER}] Data 'Failed to retrieve disk smart information'");
            return Err("Data 'Failed to retrieve disk smart information'".into());
        }

        let sectors_reallocated = buffer.get(5).copied();
        let sectors_pending = buffer.get(196).copied();
        let sectors_pending_current = buffer.get(197).copied();
        let temperature = buffer.get(194).copied();
        let uptime_hours = buffer.get(9).copied();

        unsafe { close(fd) };

        Ok(SmartInfo {
            uptime_hours,
            sectors_reallocated,
            sectors_pending,
            sectors_pending_current,
            temperature,
        })
    }
}

impl DiskInfo {
    /// Insert network interface parameters into the database.
    ///
    /// # Arguments
    ///
    /// - `conn` : Allow by a [`Connection`] constructor type the connection with an SQLite database.
    /// - `data` : The data structure to insert in database.
    ///
    /// # Returns
    ///
    /// - Insert the [`DiskInfo`] filled structure in an SQLite database.
    /// - Logs an error if the SQL insert request failed.
    fn insert_db(&self, conn: &Connection, timestamp: &str) -> Result<(), Box<dyn Error>> {
        conn.execute(
            "INSERT INTO storage_data (
                timestamp,
                name,
                bandwidth_read,
                bandwidth_write,
                energy_consumed,
                file_mount,
                file_system,
                kind,
                space_available,
                space_total
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                timestamp,
                self.name,
                self.bandwidth_read,
                self.bandwidth_write,
                self.energy_consumed,
                self.file_mount,
                self.file_system,
                self.kind,
                self.space_available,
                self.space_total,
            ],
        )?;
        Ok(())
    }

    /// List principal patterns of recognized storage devices types to parse it.
    /// It's required to check Smart Info of a device.
    ///
    /// # Arguments
    ///
    /// - `name` : Path to the device storage.
    ///
    /// # Returns
    ///
    /// The default name of `name` input.
    fn device_path(name: &str) -> String {
        // Pattern for /dev/nvme0n1p1
        let re_nvme = Regex::new(r"^(/dev/nvme\d+n\d+)p\d+$").unwrap();
        // Pattern for /dev/mmcblk0p1
        let re_mmcblk = Regex::new(r"^(/dev/mmcblk\d+)p\d+$").unwrap();
        // Pattern for /dev/sda1
        let re_sd = Regex::new(r"^(/dev/sd[a-zA-Z]+)").unwrap();
        // Pattern for /dev/loop0
        let re_loop = Regex::new(r"^(/dev/loop\d+)$").unwrap();
        // Pattern for /dev/mapper/cryptroot
        let re_mapper = Regex::new(r"^(/dev/mapper/[\w\-]+)$").unwrap();

        if let Some(caps) = re_nvme.captures(name) {
            return caps[1].to_string();
        } else if let Some(caps) = re_mmcblk.captures(name) {
            return caps[1].to_string();
        } else if let Some(caps) = re_sd.captures(name) {
            return caps[1].to_string();
        } else if let Some(caps) = re_loop.captures(name) {
            return caps[1].to_string();
        } else if let Some(caps) = re_mapper.captures(name) {
            return caps[1].to_string();
        }

        name.to_string()
    }

    /// Detect a specific device storage on the system, and retrieves its associated information.
    ///
    /// # Arguments
    ///
    /// - `disk` : Device on which we want retrieves data.
    ///
    /// # Returns
    ///
    /// Completed [`DiskInfo`] structure concerning data about the chosen device.
    fn from_device(disk: &Disk) -> Result<DiskInfo, Box<dyn Error>> {
        let bandwidth_read = Some(disk.usage().total_read_bytes / 1_000_000);
        let bandwidth_write = Some(disk.usage().total_written_bytes / 1_000_000);
        let file_system = Some(disk.file_system().to_string_lossy().to_string());
        let file_mount = Some(disk.mount_point().to_string_lossy().to_string());
        let kind = Some(disk.kind().to_string());
        let name = disk.name().to_string_lossy().to_string();
        let space_available = Some(disk.available_space() / 1_000_000_000);
        let space_total = Some(disk.total_space() / 1_000_000_000);

        let smart_info = SmartInfo::collect_smart_data(&Self::device_path(&name)).ok();

        let energy_consumed = Some(estimate_energy(
            kind.as_deref().unwrap_or(""),
            bandwidth_read.unwrap_or(0),
            bandwidth_write.unwrap_or(0),
        ));

        Ok(DiskInfo {
            bandwidth_read,
            bandwidth_write,
            energy_consumed,
            file_mount,
            file_system,
            kind,
            name,
            space_available,
            space_total,
            smart_info,
        })
    }
}

/// Function that retrieves all detailed disk information.
///
/// # Returns
///
/// The compilation of completed structures concerning all disk information.
/// * [`DiskInfo`] concerning global system info of the device storage.
/// * [`SmartInfo`] concerning smart info for the device storage if it's possible.
fn collect_storage_data(conn: &Connection) -> Result<(), Box<dyn Error>> {
    let timestamp = Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true);
    let disks = Disks::new_with_refreshed_list_specifics(DiskRefreshKind::everything());

    for disk in disks.list() {
        let disk_info = DiskInfo::from_device(disk)?;
        disk_info.insert_db(conn, &timestamp)?;
        let id = conn.last_insert_rowid();
        if let Some(smart) = &disk_info.smart_info {
            smart.insert_db(conn, id)?;
        }
    }
    Ok(())
}

/// Public function used to send JSON formatted values,
/// from [`collect_storage_data`] function result.
pub fn get_storage_info() -> Result<(), Box<dyn Error>> {
    let conn = init_db(DATABASE)?;
    collect_storage_data(&conn)?;
    Ok(())
}
