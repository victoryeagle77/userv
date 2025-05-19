//! # File utilities module

use serde::Serialize;

pub const HEADER: &'static str = "STORAGE";

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

pub fn estimate_energy(kind: &str, read_mb: u64, write_mb: u64) -> f64 {
    let (read_energy, write_energy) = match kind {
        k if k.contains("HDD") => (0.006, 0.006),
        k if k.contains("SSD") => (0.0036, 0.0036),
        k if k.contains("NVMe") => (0.005, 0.005),
        _ => (0.005, 0.005),
    };
    (read_mb as f64) * read_energy + (write_mb as f64) * write_energy
}
