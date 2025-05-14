//! # Disk Data Module
//!
//! This module provides functionality to retrieve disk data on Unix-based systems.

use libc::{c_void, close, open, read};
use log::error;
use regex::Regex;
use serde::Serialize;
use serde_json::{json, Value};
use std::{error::Error, ffi::CString};
use sysinfo::{Disk, DiskRefreshKind, Disks};

use crate::utils::write_json_to_file;

const HEADER: &str = "STORAGE";
const LOGGER: &str = "log/disk_data.json";

/// Collected more specific and detailed disk data.
#[derive(Debug, Serialize)]
struct SmartInfo {
    /// Reallocated sector count.
    sectors_reallocated: Option<u8>,
    /// Reallocation event count.
    sectors_pending: Option<u8>,
    /// Current pending sector count.
    sectors_pending_current: Option<u8>,
    /// Disk operating temperature.
    temperature: Option<u8>,
    /// Power on Hours.
    uptime_hours: Option<u8>,
}

impl SmartInfo {
    /// Convert the `SmartInfo` into a JSON object.
    fn to_json(&self) -> Value {
        json!({
            "uptime_hours": self.uptime_hours,
            "sectors_reallocated": self.sectors_reallocated,
            "sectors_pending": self.sectors_pending,
            "sectors_pending_current": self.sectors_pending_current,
            "temperature_°C": self.temperature,
        })
    }

    /// Function that retrieves smart disk information.
    /// * 5 : Reallocated Sector Count
    /// * 9 : Power-On Hours
    /// * 194 : Temperature
    /// * 196 : Reallocation Event Count
    /// * 197 : Current Pending Sector Count
    /// * 198 : Offline Uncorrectable
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

/// Collected global disk data.
#[derive(Debug, Serialize)]
struct DiskInfo {
    /// Disk reading data transfer in MB.
    bandwidth_read: Option<u64>,
    /// Disk writing data transfer in MB.
    bandwidth_write: Option<u64>,
    /// Path on the system where the disk device is mounted.
    file_mount: Option<String>,
    /// Disk file system type (ext, NTF, FAT...).
    file_system: Option<String>,
    /// Disk device type (HDD, SDD).
    kind: Option<String>,
    /// Disk path name on the system.
    name: String,
    /// Disk used memory space.
    space_available: Option<u64>,
    /// Disk total memory space.
    space_total: Option<u64>,
    /// Retrieves more detailed information with [`SmartInfo`].
    smart_info: Option<SmartInfo>,
}

impl DiskInfo {
    /// Convert the [`DiskInfo`] into a JSON object.
    fn to_json(&self) -> Value {
        json!({
            "bandwidth_read_MB": self.bandwidth_read,
            "bandwidth_write_MB": self.bandwidth_write,
            "file_mount": self.file_mount,
            "file_system": self.file_system,
            "kind": self.kind,
            "name": self.name,
            "space_available_MB": self.space_available,
            "space_total_MB": self.space_total,
            "smart_info": self.smart_info.as_ref().map(|s| s.to_json()),
        })
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

        Ok(DiskInfo {
            bandwidth_read,
            bandwidth_write,
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
fn collect_disk_data() -> Result<Vec<Value>, Box<dyn Error>> {
    let disks = Disks::new_with_refreshed_list_specifics(DiskRefreshKind::everything());
    let mut result = Vec::new();

    for (index, disk) in disks.list().iter().enumerate() {
        let key = "device_".to_owned() + &index.to_string();
        result.push(json!({
            key: DiskInfo::from_device(disk)?.to_json(),
        }));
    }

    Ok(result)
}

/// Public function used to send JSON formatted values,
/// from [`collect_disk_data`] function result.
pub fn get_disk_info() -> Result<(), Box<dyn Error>> {
    let data = collect_disk_data()?;
    let values = json!({ HEADER: data });
    write_json_to_file(|| Ok(values), LOGGER)?;
    Ok(())
}
