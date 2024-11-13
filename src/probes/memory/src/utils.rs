//! # File utilities module
//!
//! This module provides functionalities to get specific data concerning memories on Unix-based systems.

use dmidecode::{EntryPoint, Structure, structures::memory_device::Type};
use log::error;
use serde::Serialize;
use std::{
    env::var,
    error::Error,
    ptr::{read_volatile, write_volatile},
    time::{Duration, Instant},
};

pub const HEADER: &str = "MEMORY";

const FACTOR: u64 = 1_000_000;
const DEFAULT_ARRAY_SIZE: usize = 100_000_000;

/// Trait to implement for [`Type`] a reference function which associating voltage and ratio for each memory type.
pub trait Reference {
    fn reference(&self) -> Option<(f64, f64)>;
}

/// Trait to implement for [`Type`] a function to convert in a string each [`Type`] of memory.
pub trait TypeToStr {
    fn as_str(&self) -> String;
}

impl Reference for Type {
    /// Attribution of specification according the computing memory technology [`Type`],
    /// based on specifications given for memory device module datasheets.
    ///
    /// # Returns
    ///
    /// - Typical power consumption per GB for each memory type.
    /// - Reference voltage for each memory type.
    fn reference(&self) -> Option<(f64, f64)> {
        match self {
            Type::Sdram => Some((3.3, 0.70)),
            Type::Ddr => Some((2.5, 0.60)),
            Type::Ddr2 => Some((1.8, 0.48)),
            Type::Ddr3 => Some((1.5, 0.45)),
            Type::Ddr4 => Some((1.2, 0.32)),
            Type::Ddr5 => Some((1.1, 0.25)),
            Type::LpDdr2 => Some((1.2, 0.19)),
            Type::LpDdr3 => Some((1.2, 0.16)),
            Type::LpDdr4 => Some((1.1, 0.16)),
            Type::LpDdr5 => Some((1.05, 0.12)),
            _ => None,
        }
    }
}

impl TypeToStr for Type {
    /// Convert in a string each [`Type`] of memory.
    ///
    /// # Returns
    ///
    /// Formatted string for the memory type concerned.
    fn as_str(&self) -> String {
        format!("{self:?}")
    }
}

/// Information about memory device info.
#[derive(Debug, Clone)]
pub struct MemDeviceInfo {
    /// Type of computing memory.
    pub kind: Type,
    /// Serial number of the memory device.
    pub id: Option<String>,
    /// Voltage in V.
    pub voltage: Option<f64>,
    /// Size in MB.
    pub size: Option<u16>,
    /// Speed data transfer in Mega transfer.
    pub speed: Option<u16>,
}

/// Collection of collected memory based in bytes.
#[derive(Clone, Debug, Serialize)]
pub struct MemInfo {
    /// Memory reading bandwidth test in MB/s.
    pub bandwidth_read: Option<f64>,
    /// Memory writing bandwidth test in MB/s.
    pub bandwidth_write: Option<f64>,
    /// Available RAM memory in MB.
    pub ram_available: Option<u64>,
    /// Free RAM memory in MB.
    pub ram_free: Option<u64>,
    /// RAM power consumption according its type in W.
    pub ram_power_consumption: Option<f64>,
    /// Total RAM memory in MB.
    pub ram_total: Option<u64>,
    /// Used RAM memory in MB.
    pub ram_used: Option<u64>,
    /// Free swap memory in MB.
    pub swap_free: Option<u64>,
    /// Total swap memory in MB.
    pub swap_total: Option<u64>,
    /// Used swap memory in MB.
    pub swap_used: Option<u64>,
}

/// Estimation of power consumption by memory in W.
/// Base on the typical power consumption per GB based on the memory type defined in [`Type::reference`].
///
/// # Returns
///
/// - Returns the estimated RAM power consumption in W.
/// - None if memory type is unknown or total memory is zero.
pub fn mem_estimated_power_consumption(device: &[MemDeviceInfo], used: u64) -> Option<f64> {
    let total_size: u64 = device.iter().map(|s| s.size.unwrap_or(0) as u64).sum();
    if total_size == 0 {
        error!("[{HEADER}] Data 'No RAM devices detected for power estimation'");
        return None;
    }

    let mut power = 0.0;
    for i in device {
        let size = i.size.unwrap_or(0) as f64;
        if size == 0.0 {
            continue;
        }
        if let Some((ref_voltage, ref_energy)) = i.kind.reference() {
            let voltage = i.voltage.unwrap_or(ref_voltage);
            let energy = ref_energy * (voltage / ref_voltage);
            power += energy * size;
        }
    }

    Some(power * (used as f64 / total_size as f64) / 1e6)
}

/// Function that calculates the writing and reading speed of computing memory,
/// allocating a wide range [`DEFAULT_ARRAY_SIZE`] of test data in memory.
///
/// # Return
///
/// - `write_bandwidth`: Write bandwidth test result in MB/s.
/// - `read_bandwidth`: Read bandwidth test result in MB/s.
pub fn mem_test_bandwidth(array_size: usize) -> Result<(Option<f64>, Option<f64>), Box<dyn Error>> {
    let mut space_area = vec![0u8; array_size];

    let write_start = Instant::now();
    for (i, item) in space_area.iter_mut().enumerate() {
        *item = (i % 256) as u8;
    }
    let write_duration = write_start.elapsed();

    let read_start = Instant::now();
    let mut sum = 0u64;
    for &value in &space_area {
        sum = sum.wrapping_add(value as u64);
    }
    unsafe {
        write_volatile(&mut sum as *mut u64, sum);
        read_volatile(&sum as *const u64);
    }
    let read_duration: Duration = read_start.elapsed();

    let result = array_size as f64;
    let write_bandwidth = result / write_duration.as_secs_f64() / 1e6;
    let read_bandwidth = result / read_duration.as_secs_f64() / 1e6;

    if write_bandwidth.is_nan()
        || read_bandwidth.is_nan()
        || write_bandwidth <= 0.0
        || read_bandwidth <= 0.0
    {
        return Err("Data 'Invalid bandwidth calculation'".to_string().into());
    }

    Ok((Some(write_bandwidth), Some(read_bandwidth)))
}

pub fn get_mem_test() -> Result<(Option<f64>, Option<f64>), Box<dyn Error>> {
    let array_size = var("MEM_TEST_SIZE")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(DEFAULT_ARRAY_SIZE);
    mem_test_bandwidth(array_size)
}

/// Parse the `dmidecode` command output to get data on detected RAM types.
///
/// # Returns
///
/// - A tuple of RAM type values if at least one correct type is found.
/// - An error if no values are available.
///
/// # Operating
///
/// Root privileges are required.
pub fn get_mem_device(
    entry_buf: &[u8],
    dmi_buf: &[u8],
) -> Result<Option<Vec<MemDeviceInfo>>, Box<dyn Error>> {
    let entry = EntryPoint::search(entry_buf).map_err(|e| {
        error!("[{HEADER}] Data 'EntryPoint search error': {e:?}");
        Box::new(e) as Box<dyn Error>
    })?;

    let mut devices = Vec::new();
    let mut data = MemDeviceInfo {
        kind: Type::Unknown,
        id: None,
        voltage: None,
        size: None,
        speed: None,
    };

    for table in entry.structures(dmi_buf).filter_map(Result::ok) {
        if let Structure::MemoryDevice(device) = &table {
            let id = device.serial;
            let kind = device.memory_type;

            if kind != Type::Unknown && !id.is_empty() {
                data.id = Some(id.to_string());
                data.kind = kind;
                data.size = device.size;
                data.voltage = (device.configured_voltage).map(|v| v as f64);
                data.speed = device.configured_memory_speed;

                devices.push(data.clone());
            }
        }
    }

    if devices.is_empty() {
        Err("Failed to identify RAM device".into())
    } else {
        Ok(Some(devices))
    }
}

/// Retrieves detailed computing and SWAP memories data.
///
/// # Arguments
///
/// - `data_ram_test`:
/// - `data_ram_devices`: Tuple containing [`MemDeviceInfo`] structure with data concerning.
/// - `sys`: [`sysinfo`]
///
/// # Returns
///
/// - Completed [`MemInfo`] structure with all memories information.
/// - List of RAM modules detected (optional).
pub fn collect_mem_data(
    data_ram_test: (Option<f64>, Option<f64>),
    data_ram_devices: Option<&Vec<MemDeviceInfo>>,
    sys: &sysinfo::System,
) -> MemInfo {
    let ram_total = sys.total_memory() / FACTOR;
    let ram_used = sys.used_memory() / FACTOR;
    let ram_available = Some(sys.available_memory() / FACTOR);
    let ram_free = Some(sys.free_memory() / FACTOR);
    let swap_total = Some(sys.total_swap() / FACTOR);
    let swap_free = Some(sys.free_swap() / FACTOR);
    let swap_used = Some(sys.used_swap() / FACTOR);

    let (bandwidth_write, bandwidth_read) = data_ram_test;

    let ram_power_consumption =
        data_ram_devices.and_then(|devices| mem_estimated_power_consumption(devices, ram_used));

    MemInfo {
        ram_available,
        ram_free,
        ram_power_consumption,
        ram_total: Some(ram_total),
        ram_used: Some(ram_used),
        swap_free,
        swap_total,
        swap_used,
        bandwidth_read,
        bandwidth_write,
    }
}

pub fn collect_mem_devices(
    data_ram_devices: Option<Vec<MemDeviceInfo>>,
) -> Option<Vec<MemDeviceInfo>> {
    data_ram_devices.filter(|d| !d.is_empty())
}
//----------------//
// UNIT CODE TEST //
//----------------//

#[cfg(test)]
mod tests {
    use super::*;
    use std::env::{remove_var, set_var, var};
    use sysinfo::{MemoryRefreshKind, System};

    // Test `get_mem_device` function with invalid data reading
    #[test]
    fn test_board_info_build_error() {
        let invalid_entry_buf: &[u8] = b"invalid data";
        let dmi_buf: &[u8] = &[];
        let res = get_mem_device(invalid_entry_buf, dmi_buf);
        assert!(res.is_err());
    }

    // Test `mem_test_bandwidth` function with invalid bandwidth
    #[test]
    fn test_mem_test_bandwidth_error() {
        assert!(mem_test_bandwidth(0).is_err());
    }

    // Test `mem_test_bandwidth` function with calculation success
    #[test]
    fn test_mem_test_bandwidth_success() {
        let sizes = [1_000_000, 5_000_000, 10_000_000];
        for &size in sizes.iter() {
            let res = mem_test_bandwidth(size);
            let (write_bw, read_bw) = res.unwrap();
            assert!(write_bw.unwrap() > 0.0);
            assert!(read_bw.unwrap() > 0.0);
        }
    }

    // Test `get_mem_test` function with calculation success
    #[test]
    fn test_get_mem_test_reads_env_var() {
        let key = "MEM_TEST_SIZE";
        let env = var(key).ok();

        unsafe { set_var(key, "1000000") };
        let res = get_mem_test();
        assert!(res.is_ok());

        unsafe { remove_var(key) };
        let res = get_mem_test();
        assert!(res.is_ok());

        match env {
            Some(val) => unsafe { set_var(key, val) },
            None => unsafe { remove_var(key) },
        }
    }

    // Test `estimated_power_consumption` function in success case
    #[test]
    fn test_estimated_power_consumption_with_devices() {
        let _ = env_logger::builder().is_test(true).try_init();
        let devices = vec![
            MemDeviceInfo {
                kind: Type::Ddr,
                id: Some("ABCDEF01".to_string()),
                voltage: Some(2.5),
                size: Some(4096),
                speed: Some(256),
            },
            MemDeviceInfo {
                kind: Type::Ddr2,
                id: Some("ABCDEF23".to_string()),
                voltage: Some(1.8),
                size: Some(4096),
                speed: Some(256),
            },
            MemDeviceInfo {
                kind: Type::Ddr3,
                id: Some("ABCDEF45".to_string()),
                voltage: Some(1.5),
                size: Some(4096),
                speed: Some(256),
            },
            MemDeviceInfo {
                kind: Type::Ddr4,
                id: Some("ABCDEF67".to_string()),
                voltage: Some(1.2),
                size: Some(4096),
                speed: Some(256),
            },
            MemDeviceInfo {
                kind: Type::Ddr5,
                id: Some("ABCDEF89".to_string()),
                voltage: Some(1.1),
                size: Some(4096),
                speed: Some(256),
            },
            MemDeviceInfo {
                kind: Type::LpDdr2,
                id: Some("ABCDEFA0".to_string()),
                voltage: Some(1.2),
                size: Some(4096),
                speed: Some(256),
            },
            MemDeviceInfo {
                kind: Type::LpDdr3,
                id: Some("ABCDEFA1".to_string()),
                voltage: Some(1.2),
                size: Some(4096),
                speed: Some(256),
            },
            MemDeviceInfo {
                kind: Type::LpDdr4,
                id: Some("ABCDEFA2".to_string()),
                voltage: Some(1.1),
                size: Some(4096),
                speed: Some(256),
            },
            MemDeviceInfo {
                kind: Type::LpDdr5,
                id: Some("ABCDEFA3".to_string()),
                voltage: Some(1.05),
                size: Some(4096),
                speed: Some(256),
            },
            MemDeviceInfo {
                kind: Type::Sdram,
                id: Some("ABCDEFA4".to_string()),
                voltage: Some(3.3),
                size: Some(4096),
                speed: Some(256),
            },
            MemDeviceInfo {
                kind: Type::Unknown,
                id: Some("ABCDEFA5".to_string()),
                voltage: None,
                size: None,
                speed: None,
            },
        ];

        let res = mem_estimated_power_consumption(&devices, 1028);
        assert!(res.is_some());
    }

    // Test `reference` function with unknown type
    #[test]
    fn test_reference_with_unknown_type() {
        let unknown_type = Type::Unknown;
        let res = unknown_type.reference();
        assert!(res.is_none());
    }

    // Test `estimated_power_consumption` function in calculation error case
    #[test]
    fn test_estimated_power_consumption_no_ram_devices() {
        let _ = env_logger::builder().is_test(true).try_init();
        let ram_devices = vec![
            MemDeviceInfo {
                kind: Type::Ddr4,
                id: Some("ABC123".to_string()),
                voltage: Some(1.2),
                size: None,
                speed: Some(200),
            },
            MemDeviceInfo {
                kind: Type::Ddr4,
                id: Some("DEF456".to_string()),
                voltage: Some(1.2),
                size: Some(0),
                speed: Some(100),
            },
        ];

        let res = mem_estimated_power_consumption(&ram_devices, 12000);
        assert!(res.is_none());
    }

    // Test `build_mem_info` function with detected memory device
    #[test]
    fn test_build_mem_info_with_devices() {
        let ram_test = (Some(1500.0), Some(3000.0));
        let ram_device = Some(vec![MemDeviceInfo {
            kind: Type::Ddr4,
            id: Some("ABC123".to_string()),
            voltage: Some(1.2),
            size: Some(8000),
            speed: Some(200),
        }]);

        let mut sys = System::new();
        sys.refresh_memory_specifics(MemoryRefreshKind::everything());

        let data_global = collect_mem_data(ram_test, ram_device.as_ref(), &sys);
        let data_devices = collect_mem_devices(ram_device).expect("should have devices");

        assert_eq!(data_devices[0].id.as_ref().unwrap(), "ABC123");
        assert_eq!(data_global.bandwidth_write, Some(1500.0));
        assert_eq!(data_global.bandwidth_read, Some(3000.0));
    }

    // Test `as_str` function from TypeToStr for dmidecode memory device type
    #[test]
    fn test_type_to_str_known_types() {
        let res = Type::Ddr4;
        assert_eq!(res.as_str(), "Ddr4");
    }
}
