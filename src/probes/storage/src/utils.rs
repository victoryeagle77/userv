//! # File utilities module

use libc::{c_void, close, open, read};
use log::error;
use regex::Regex;
use rusqlite::{Connection, params};
use serde::Serialize;
use std::{error::Error, ffi::CString};
use sysinfo::Disk;

pub const HEADER: &str = "STORAGE";

/// Collected global disk data.
#[derive(Debug, Serialize)]
pub struct DiskInfo {
    /// Disk reading data transfer in MB.
    pub bandwidth_read: Option<u64>,
    /// Disk writing data transfer in MB.
    pub bandwidth_write: Option<u64>,
    /// Estimated consumed energy in W.
    pub energy_consumed: Option<f64>,
    /// Path on the system where the disk device is mounted.
    pub file_mount: Option<String>,
    /// Disk file system type (ext, NTF, FAT...).
    pub file_system: Option<String>,
    /// Disk device type (HDD, SDD).
    pub kind: Option<String>,
    /// Disk path name on the system.
    pub name: String,
    /// Disk used memory space.
    pub space_available: Option<u64>,
    /// Disk total memory space.
    pub space_total: Option<u64>,
    /// Retrieves more detailed information with [`SmartInfo`].
    pub smart_info: Option<SmartInfo>,
}

/// Collected more specific and detailed disk data.
#[derive(Debug, Serialize)]
pub struct SmartInfo {
    /// Reallocated sector count.
    pub sectors_reallocated: Option<u8>,
    /// Reallocation event count.
    pub sectors_pending: Option<u8>,
    /// Current pending sector count.
    pub sectors_pending_current: Option<u8>,
    /// Disk operating temperature.
    pub temperature: Option<u8>,
    /// Power on Hours.
    pub uptime_hours: Option<u8>,
}

impl SmartInfo {
    /// Insert smart information parameters on a storage device into the database.
    ///
    /// # Arguments
    ///
    /// - `conn` : Allow by a [`Connection`] constructor type the connection with an SQLite database.
    ///
    /// # Returns
    ///
    /// - Insert the [`SmartInfo`] filled structure in an SQLite database.
    /// - Logs an error if the SQL insert request failed.
    pub fn insert_db(&self, conn: &Connection, device_id: i64) -> Result<(), Box<dyn Error>> {
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
    /// Insert storage device parameters into the database.
    ///
    /// # Arguments
    ///
    /// - `conn` : Allow by a [`Connection`] constructor type the connection with an SQLite database.
    /// - `timestamp`: Timestamp of the measurement.
    ///
    /// # Returns
    ///
    /// - Insert the [`DiskInfo`] filled structure in an SQLite database.
    /// - Logs an error if the SQL insert request failed.
    pub fn insert_db(&self, conn: &Connection, timestamp: &str) -> Result<(), Box<dyn Error>> {
        conn.execute(
            "INSERT INTO storage_data (
                timestamp,
                name,
                bandwidth_read_MB,
                bandwidth_write_MB,
                energy_consumed_J,
                file_mount,
                file_system,
                kind,
                space_available_MB,
                space_total_MB
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
    pub fn from_device(disk: &Disk) -> Result<DiskInfo, Box<dyn Error>> {
        let bandwidth_read = disk.usage().total_read_bytes / 1_000_000;
        let bandwidth_write = disk.usage().total_written_bytes / 1_000_000;
        let file_system = Some(disk.file_system().to_string_lossy().to_string());
        let file_mount = Some(disk.mount_point().to_string_lossy().to_string());
        let kind = Some(disk.kind().to_string());
        let name = disk.name().to_string_lossy().to_string();
        let space_available = Some(disk.available_space() / 1_000_000_000);
        let space_total = Some(disk.total_space() / 1_000_000_000);

        let smart_info = SmartInfo::collect_smart_data(&Self::device_path(&name)).ok();

        let energy_consumed = Some(estimate_energy(
            kind.as_deref().unwrap_or(""),
            bandwidth_read,
            bandwidth_write,
        ));

        Ok(DiskInfo {
            bandwidth_read: Some(bandwidth_read),
            bandwidth_write: Some(bandwidth_write),
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

pub fn estimate_energy(kind: &str, read_mb: u64, write_mb: u64) -> f64 {
    let (read_energy, write_energy) = match kind {
        k if k.contains("HDD") => (0.006, 0.006),
        k if k.contains("SSD") => (0.0036, 0.0036),
        k if k.contains("NVMe") => (0.005, 0.005),
        _ => (0.005, 0.005),
    };
    (read_mb as f64) * read_energy + (write_mb as f64) * write_energy
}
