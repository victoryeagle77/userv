//! # File utilities module
//!
//! This module provides functionalities to get specific data concerning memories on Unix-based systems.

use dmidecode::{EntryPoint, Structure, structures::memory_device::Type};
use log::error;
use rusqlite::{Connection, params};
use serde::Serialize;
use std::{
    error::Error,
    fs::read,
    ptr::{read_volatile, write_volatile},
    time::{Duration, Instant},
};

const HEADER: &str = "MEMORY";
const DEFAULT_ARRAY_SIZE: usize = 100_000_000;
const ENTRY_BIN: &str = "/sys/firmware/dmi/tables/smbios_entry_point";
const DMIDECODE_BIN: &str = "/sys/firmware/dmi/tables/DMI";

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
    /// Speed data transfer.
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

/// Insert memory device parameters into the database.
///
/// # Arguments
///
/// - `conn`: Connection to SQLite database.
/// - `timestamp`: Timestamp of the measurement.
/// - `data`: [`MemInfo`] information to insert in database.
/// - `ram_devices`: [`MemDeviceInfo`] list of RAM modules (optional, can be None).
///
/// # Returns
///
/// - Insert the [`MemInfo`] and [`MemDeviceInfo`] filled structures in an SQLite database.
/// - Logs an error if the SQL insert request failed.
///
/// # Operating
///
/// The [`MemDeviceInfo`] is a set of static information, their are retrieved only one time.
/// The [`MemInfo`] is a set of dynamic information retrieved and refresh at each call.
pub fn insert_db(
    conn: &mut Connection,
    timestamp: &str,
    data: &MemInfo,
    ram_devices: Option<&Vec<MemDeviceInfo>>,
) -> Result<(), Box<dyn Error>> {
    // Insert the main memory parameters in table
    conn.execute(
        "INSERT INTO memory_data (
            timestamp,
            bandwidth_read,
            bandwidth_write,
            ram_total_MB,
            ram_used_MB,
            ram_free_MB,
            ram_available_MB,
            ram_power_consumption_W,
            swap_total_MB,
            swap_used_MB,
            swap_free_MB
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
        params![
            timestamp,
            data.bandwidth_read,
            data.bandwidth_write,
            data.ram_total,
            data.ram_used,
            data.ram_free,
            data.ram_available,
            data.ram_power_consumption,
            data.swap_total,
            data.swap_used,
            data.swap_free,
        ],
    )?;

    if let Some(ram_devices) = ram_devices {
        ram_devices
            .iter()
            .filter(|module| {
                let exists: Result<bool, _> = conn.query_row(
                    "SELECT EXISTS(SELECT 1 FROM memory_modules WHERE device_id = ?1)",
                    params![module.id],
                    |row| row.get(0),
                );
                match exists {
                    Ok(true) => false,
                    Ok(false) => true,
                    Err(e) => {
                        error!("[{HEADER}] Data 'I/O failure for memory_modules database' : {e}");
                        false
                    }
                }
            })
            .try_for_each(|module| {
                conn.execute(
                    "INSERT INTO memory_modules (
                    device_id, ram_type, size_MB, speed_Mt, voltage_mV
                ) VALUES (?1, ?2, ?3, ?4, ?5)",
                    params![
                        module.id,
                        module.kind.as_str(),
                        module.size,
                        module.speed,
                        module.voltage
                    ],
                )
                .map(|_| ())
            })?;
    }

    Ok(())
}

/// Estimation of power consumption by memory in W.
/// Base on the typical power consumption per GB based on the memory type defined in [`Type::reference`].
///
/// # Returns
///
/// - Returns the estimated RAM power consumption in W.
/// - None if memory type is unknown or total memory is zero.
pub fn estimated_power_consumption(device: &[MemDeviceInfo], used: u64) -> Option<f64> {
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
/// - `write_bandwidth` : Write bandwidth test result in MB/s.
/// - `read_bandwidth` : Read bandwidth test result in MB/s.
pub fn get_mem_test() -> Result<(Option<f64>, Option<f64>), Box<dyn Error>> {
    let array_size = std::env::var("MEM_TEST_SIZE")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(DEFAULT_ARRAY_SIZE);

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
        let _ = read_volatile(&sum as *const u64);
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
pub fn get_mem_device() -> Result<Option<Vec<MemDeviceInfo>>, Box<dyn Error>> {
    let buf = match read(ENTRY_BIN) {
        Ok(e) => e,
        Err(e) => {
            error!("[{HEADER}] Data 'Reading error smbios_entry_point' : {e:?}");
            return Err(Box::new(e));
        }
    };

    let dmi = match read(DMIDECODE_BIN) {
        Ok(e) => e,
        Err(e) => {
            error!("[{HEADER}] Data 'Reading error DMI' : {e:?}");
            return Err(Box::new(e));
        }
    };

    // Research of an SMBIOS entry point in buffer
    let entry = match EntryPoint::search(&buf) {
        Ok(e) => e,
        Err(e) => {
            error!("[{HEADER}] Data 'EntryPoint research error' : {e:?}");
            return Err(Box::new(e));
        }
    };

    let mut devices = Vec::new();
    let mut mem = MemDeviceInfo {
        kind: Type::Unknown,
        id: None,
        voltage: None,
        size: None,
        speed: None,
    };

    // SMBIOS structures searching
    for table_res in entry.structures(&dmi) {
        let table = match table_res {
            Ok(t) => t,
            Err(e) => {
                error!("[{HEADER}] Data 'SMBIOS structure not properly formatted' : {e:?}");
                return Err(Box::new(e));
            }
        };

        if let Structure::MemoryDevice(device) = table {
            let id = device.serial;
            let kind = device.memory_type;
            let voltage = device.configured_voltage;
            let speed = device.configured_memory_speed;
            let size = device.size;

            if kind != Type::Unknown && !id.is_empty() {
                mem.id = Some(id.to_string());
                mem.kind = kind;
                mem.size = size;
                mem.voltage = voltage.map(|v| v as f64);
                mem.speed = speed;

                devices.push(mem.clone());
            }
        }
    }

    if devices.is_empty() {
        Err("Failed to identify RAM device".into())
    } else {
        Ok(Some(devices))
    }
}

//----------------//
// UNIT CODE TEST //
//----------------//

#[cfg(test)]
mod tests {
    use super::*;
    use crate::REQUEST;
    use rusqlite::Connection;

    const TIMESTAMP: &'static str = "2025-08-15T14:00:00Z";

    fn insert_mock_data(data: MemInfo, ram_type: Type) -> i64 {
        let ram_devices = vec![MemDeviceInfo {
            kind: ram_type,
            id: Some(String::new()),
            voltage: Some(1.2),
            size: Some(8192),
            speed: Some(256),
        }];

        let mut conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(REQUEST).unwrap();

        insert_db(&mut conn, TIMESTAMP, &data, Some(&ram_devices)).unwrap();

        conn.query_row("SELECT COUNT(*) FROM memory_data", [], |row| row.get(0))
            .unwrap()
    }

    // Test `insert_db` function for all RAM type available
    #[test]
    fn test_insert_db_ram_type() {
        let data = MemInfo {
            ram_total: Some(16000),
            ram_used: Some(8000),
            ram_available: Some(7000),
            ram_free: Some(6000),
            ram_power_consumption: Some(5.0),
            swap_total: Some(4000),
            swap_free: Some(2000),
            swap_used: Some(2000),
            bandwidth_read: Some(200.0),
            bandwidth_write: Some(100.0),
        };

        assert_eq!(insert_mock_data(data.clone(), Type::Ddr5), 1);
        assert_eq!(insert_mock_data(data.clone(), Type::Ddr4), 1);
        assert_eq!(insert_mock_data(data.clone(), Type::Ddr3), 1);
        assert_eq!(insert_mock_data(data.clone(), Type::Ddr2), 1);
        assert_eq!(insert_mock_data(data.clone(), Type::Ddr), 1);
        assert_eq!(insert_mock_data(data.clone(), Type::Sdram), 1);
        assert_eq!(insert_mock_data(data.clone(), Type::LpDdr5), 1);
        assert_eq!(insert_mock_data(data.clone(), Type::LpDdr4), 1);
        assert_eq!(insert_mock_data(data.clone(), Type::LpDdr3), 1);
        assert_eq!(insert_mock_data(data.clone(), Type::LpDdr2), 1);
        assert_eq!(insert_mock_data(data.clone(), Type::Unknown), 1);
    }

    // Test `insert_db` function while the RAM device module database was already written
    #[test]
    fn test_insert_db_existing_modules() -> Result<(), Box<dyn std::error::Error>> {
        let mut conn = Connection::open_in_memory()?;
        conn.execute_batch(REQUEST).unwrap();
        conn.execute(
            "INSERT INTO memory_modules (device_id, ram_type, size_MB, speed_Mt) VALUES (?1, ?2, ?3, ?4)",
            params!["ABC123", "DDR4", 16000, 200],
        )?;

        let ram_devices = vec![
            MemDeviceInfo {
                kind: Type::Ddr4,
                id: Some("ABC123".to_string()),
                voltage: Some(1.2),
                size: Some(16000),
                speed: Some(200),
            },
            MemDeviceInfo {
                kind: Type::Ddr4,
                id: Some("DEF456".to_string()),
                voltage: Some(1.2),
                size: Some(8000),
                speed: Some(100),
            },
        ];
        let data = MemInfo {
            bandwidth_read: None,
            bandwidth_write: None,
            ram_available: None,
            ram_free: None,
            ram_power_consumption: None,
            ram_total: None,
            ram_used: None,
            swap_free: None,
            swap_total: None,
            swap_used: None,
        };

        insert_db(&mut conn, TIMESTAMP, &data, Some(&ram_devices))?;

        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM memory_modules WHERE device_id = 'DEF456'",
            [],
            |row| row.get(0),
        )?;

        assert_eq!(count, 1);

        Ok(())
    }

    // Test `insert_db` function
    #[test]
    fn test_insert_db_error() -> Result<(), Box<dyn std::error::Error>> {
        let mut conn = Connection::open_in_memory()?;
        conn.execute_batch(REQUEST).unwrap();
        conn.execute("DROP TABLE memory_modules;", [])?;

        let ram_devices = vec![MemDeviceInfo {
            kind: Type::Ddr4,
            id: Some("ERROR".to_string()),
            voltage: Some(1.2),
            size: Some(16000),
            speed: Some(200),
        }];
        let data = MemInfo {
            bandwidth_read: None,
            bandwidth_write: None,
            ram_available: None,
            ram_free: None,
            ram_power_consumption: None,
            ram_total: None,
            ram_used: None,
            swap_free: None,
            swap_total: None,
            swap_used: None,
        };

        let res = insert_db(&mut conn, TIMESTAMP, &data, Some(&ram_devices));
        if let Err(e) = res {
            let msg = format!("{e}");
            assert!(msg.contains("no such table"), "Error message: {msg}");
        }

        Ok(())
    }

    // Test `get_mem_test` function with calculation success
    #[test]
    fn test_get_mem_test_success() {
        for &size in &[1_000_000, 5_000_000, 10_000_000] {
            unsafe { std::env::set_var("MEM_TEST_SIZE", size.to_string()) };
            match get_mem_test() {
                Ok((write_bw, read_bw)) => {
                    assert!(write_bw.is_some() && read_bw.is_some());
                    let write_bw = write_bw.unwrap();
                    let read_bw = read_bw.unwrap();

                    assert!(write_bw > 0.0, "Write bandwidth should be positive");
                    assert!(read_bw > 0.0, "Read bandwidth should be positive");
                    return;
                }
                Err(e) => {
                    eprintln!("Warning: get_mem_test failed with size {}: {:?}", size, e);
                }
            }
            unsafe { std::env::remove_var("MEM_TEST_SIZE") };
        }
    }

    // Test `get_mem_test` function with invalid bandwidth
    #[test]
    fn test_get_mem_test_error() {
        unsafe { std::env::set_var("MEM_TEST_SIZE", "0") };
        let res = get_mem_test();
        assert!(res.is_err());
        unsafe { std::env::remove_var("MEM_TEST_SIZE") };
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

        let res = estimated_power_consumption(&devices, 1028);
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

        let res = estimated_power_consumption(&ram_devices, 12000);
        assert!(res.is_none());
    }
}
