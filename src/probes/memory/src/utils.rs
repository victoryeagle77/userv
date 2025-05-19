//! # File utilities module
//!
//! This module provides functionalities to get specific data concerning memories on Unix-based systems.

use log::error;
use rusqlite::{params, Connection};
use serde::Serialize;
use std::{
    error::Error,
    process::Command,
    ptr::{read_volatile, write_volatile},
    time::{Duration, Instant},
};

const HEADER: &'static str = "MEMORY";
const DEFAULT_ARRAY_SIZE: usize = 100_000_000;
pub const FACTOR: u64 = 1_000_000;

/// Typical power consumption per GB for each memory type,
/// based on voltage specifications and average module datasheets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
pub enum RamType {
    SDRAM,
    DDR,
    DDR2,
    DDR3,
    DDR4,
    DDR5,
    LPDDR2,
    LPDDR3,
    LPDDR4,
    LPDDR5,
    Unknown,
}

impl RamType {
    fn from_str(s: &str) -> RamType {
        match s {
            "SDRAM" => RamType::SDRAM,
            "DDR" => RamType::DDR,
            "DDR2" => RamType::DDR2,
            "DDR3" => RamType::DDR3,
            "DDR4" => RamType::DDR4,
            "DDR5" => RamType::DDR5,
            "LPDDR2" => RamType::LPDDR2,
            "LPDDR3" => RamType::LPDDR3,
            "LPDDR4" => RamType::LPDDR4,
            "LPDDR5" => RamType::LPDDR5,
            _ => RamType::Unknown,
        }
    }

    /// Attribution of specification according the computing memory technology type,
    /// based on specifications given for memory device module datasheets.
    ///
    /// # Returns
    ///
    /// - Typical power consumption per GB for each memory type.
    /// - Reference voltage for each memory type.
    fn from_specs(&self) -> Option<(f64, f64)> {
        match self {
            RamType::SDRAM => Some((3.3, 0.70)),
            RamType::DDR => Some((2.5, 0.60)),
            RamType::DDR2 => Some((1.8, 0.48)),
            RamType::DDR3 => Some((1.5, 0.45)),
            RamType::DDR4 => Some((1.2, 0.32)),
            RamType::DDR5 => Some((1.1, 0.25)),
            RamType::LPDDR2 => Some((1.2, 0.19)),
            RamType::LPDDR3 => Some((1.2, 0.16)),
            RamType::LPDDR4 => Some((1.1, 0.16)),
            RamType::LPDDR5 => Some((1.05, 0.12)),
            RamType::Unknown => None,
        }
    }

    /// Identifying the name corresponding to a device module.
    ///
    /// # Returns
    ///
    /// Computing memory device type name.
    fn from_type(&self) -> &'static str {
        match self {
            RamType::SDRAM => "SDRAM",
            RamType::DDR => "DDR",
            RamType::DDR2 => "DDR2",
            RamType::DDR3 => "DDR3",
            RamType::DDR4 => "DDR4",
            RamType::DDR5 => "DDR5",
            RamType::LPDDR2 => "LPDDR2",
            RamType::LPDDR3 => "LPDDR3",
            RamType::LPDDR4 => "LPDDR4",
            RamType::LPDDR5 => "LPDDR5",
            RamType::Unknown => "Unknown",
        }
    }
}

/// Information about memory device info.
#[derive(Debug, Clone)]
pub struct MemDeviceInfo {
    /// Type of computing memory.
    pub kind: RamType,
    /// Serial number of the memory device.
    pub id: Option<String>,
    /// Voltage in V.
    pub voltage: Option<f64>,
    // Size in MB.
    pub size: Option<u64>,
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
/// Base on the typical power consumption per GB based on the memory type defined in [`RAM_TYPE_POWER`].
///
/// # Returns
///
/// - Returns the estimated RAM power consumption in W.
/// - None if memory type is unknown or total memory is zero.
pub fn estimated_power_consumption(device: &[MemDeviceInfo], used: u64) -> Option<f64> {
    let total_size: u64 = device.iter().map(|s| s.size.unwrap_or(0)).sum();
    if total_size == 0 {
        error!("[{HEADER}] Data 'No RAM devices detected for power estimation'");
        return None;
    }

    let mut total_power = 0.0;
    for i in device {
        let size = i.size.unwrap_or(0) as f64 / 1e3;
        if size == 0.0 {
            continue;
        }
        if let Some((ref_voltage, ref_energy)) = i.kind.from_specs() {
            let voltage = i.voltage.unwrap_or(ref_voltage);
            let energy = ref_energy * (voltage / ref_voltage);
            total_power += energy * size;
        }
    }

    Some(total_power * (used as f64 / total_size as f64))
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

    // Insert RAM modules if provided
    fn calculate_power(module: &MemDeviceInfo) -> Option<f64> {
        module.kind.from_specs().map(|(ref_voltage, ref_energy)| {
            let voltage = module.voltage.unwrap_or(ref_voltage);
            let size = module.size.unwrap_or(0) as f64 / 1e3; // taille en Go
            ref_energy * (voltage / ref_voltage) * size
        })
    }

    if let Some(ram_devices) = ram_devices {
        ram_devices.iter()
            .filter_map(|module| {
                let exists: Result<bool, _> = conn.query_row(
                    "SELECT EXISTS(SELECT 1 FROM memory_modules WHERE device_id = ?1)",
                    params![module.id],
                    |row| row.get(0),
                );
                match exists {
                    Ok(true) => None,
                    Ok(false) => Some(module),
                    Err(e) => {
                        error!("[{HEADER}] Data 'I/O failure for memory_modules database' : {e}");
                        None
                    }
                }
            })
            .try_for_each(|module| {
                let power = calculate_power(module);
                conn.execute(
                    "INSERT INTO memory_modules (device_id, ram_type, size_MB, power_W) VALUES (?1, ?2, ?3, ?4)",
                    params![module.id, module.kind.from_type(), module.size, power],
                ).map(|_| ())
            })?;
    }

    Ok(())
}

/// Function that calculates the writing and reading speed of computing memory,
/// allocating a wide range [`ARRAY_SIZE`] of test data in memory.
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
pub fn get_ram_device() -> Result<Option<Vec<MemDeviceInfo>>, Box<dyn Error>> {
    /// Extract by pattern a value after a prefix,
    /// to get data with the output result of command from [`get_ram_device`] function.
    ///
    /// # Arguments
    ///
    /// - `line` : Pattern corresponding to the data line wanted.
    /// - `prefix` : Character after the line pattern.
    fn extract_value<'a>(line: &'a str, prefix: &str) -> Option<&'a str> {
        line.strip_prefix(prefix).map(|s| s.trim())
    }

    /// Parse size string in MB,
    /// to get data with the output result of command from [`get_ram_device`] function.
    fn parse_size(value: &str) -> Option<u64> {
        let mut parts = value.split_whitespace();
        let size = parts.next()?.parse::<u64>().ok()?;
        match parts.next()? {
            "GB" => Some(size * 1_000),
            "MB" => Some(size),
            _ => None,
        }
    }

    let output = Command::new("dmidecode").args(["-t", "memory"]).output()?;

    if !output.status.success() {
        return Err(format!(
            "Data 'dmidecode command failed with status' : {}",
            output.status
        )
        .into());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut devices = Vec::new();
    let mut current = MemDeviceInfo {
        kind: RamType::Unknown,
        id: None,
        voltage: None,
        size: None,
    };

    for line in stdout.lines() {
        let line = line.trim();
        if let Some(val) = extract_value(line, "Size:") {
            current.size = parse_size(val);
        } else if let Some(val) = extract_value(line, "Type:") {
            let kind = RamType::from_str(val);
            if kind != RamType::Unknown {
                current.kind = kind;
            }
        } else if let Some(val) = extract_value(line, "Configured Voltage:") {
            current.voltage = val.replace(",", ".").parse().ok();
        } else if let Some(val) = extract_value(line, "Serial Number:") {
            if val != "Unknown" {
                current.id = Some(val.to_string());
            }
        }

        // End of memory block
        if line.is_empty() && current.kind != RamType::Unknown && current.size.is_some() {
            devices.push(current.clone());
            current = MemDeviceInfo {
                kind: RamType::Unknown,
                id: None,
                voltage: None,
                size: None,
            };
        }
    }

    // Last memory device
    if current.kind != RamType::Unknown && current.size.is_some() {
        devices.push(current);
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
    use std::error::Error;

    const TIMESTAMP: &'static str = "2025-08-15T14:00:00Z";

    // Test `from_str` function for RamType branches validity
    #[test]
    fn test_from_str_known_values() {
        assert_eq!(RamType::from_str("SDRAM"), RamType::SDRAM);
        assert_eq!(RamType::from_str("DDR"), RamType::DDR);
        assert_eq!(RamType::from_str("DDR2"), RamType::DDR2);
        assert_eq!(RamType::from_str("DDR3"), RamType::DDR3);
        assert_eq!(RamType::from_str("DDR4"), RamType::DDR4);
        assert_eq!(RamType::from_str("DDR5"), RamType::DDR5);
        assert_eq!(RamType::from_str("LPDDR2"), RamType::LPDDR2);
        assert_eq!(RamType::from_str("LPDDR3"), RamType::LPDDR3);
        assert_eq!(RamType::from_str("LPDDR4"), RamType::LPDDR4);
        assert_eq!(RamType::from_str("LPDDR5"), RamType::LPDDR5);
    }

    // Test `from_str` function for unknown RamType
    #[test]
    fn test_from_str_unknown_value() {
        assert_eq!(RamType::from_str("UNKNOWN_TYPE"), RamType::Unknown);
        assert_eq!(RamType::from_str(""), RamType::Unknown);
    }

    fn insert_mock_data(data: MemInfo, ram_type: RamType) -> i64 {
        let ram_devices = vec![MemDeviceInfo {
            kind: ram_type,
            id: Some(String::new()),
            voltage: Some(1.2),
            size: Some(8192),
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

        assert_eq!(insert_mock_data(data.clone(), RamType::DDR5), 1);
        assert_eq!(insert_mock_data(data.clone(), RamType::DDR4), 1);
        assert_eq!(insert_mock_data(data.clone(), RamType::DDR3), 1);
        assert_eq!(insert_mock_data(data.clone(), RamType::DDR2), 1);
        assert_eq!(insert_mock_data(data.clone(), RamType::DDR), 1);
        assert_eq!(insert_mock_data(data.clone(), RamType::SDRAM), 1);
        assert_eq!(insert_mock_data(data.clone(), RamType::LPDDR5), 1);
        assert_eq!(insert_mock_data(data.clone(), RamType::LPDDR4), 1);
        assert_eq!(insert_mock_data(data.clone(), RamType::LPDDR3), 1);
        assert_eq!(insert_mock_data(data.clone(), RamType::LPDDR2), 1);
        assert_eq!(insert_mock_data(data.clone(), RamType::Unknown), 1);
    }

    // Test `insert_db` function while the RAM device module database was already written
    #[test]
    fn test_insert_db_existing_modules() -> Result<(), Box<dyn std::error::Error>> {
        let mut conn = Connection::open_in_memory()?;
        conn.execute_batch(REQUEST).unwrap();
        conn.execute(
            "INSERT INTO memory_modules (device_id, ram_type, size_MB, power_W) VALUES (?1, ?2, ?3, ?4)",
            params!["ABC123", "DDR4", 16000, 5.12f64],
        )?;

        let ram_devices = vec![
            MemDeviceInfo {
                kind: RamType::DDR4,
                id: Some("ABC123".to_string()),
                voltage: Some(1.2),
                size: Some(16000),
            },
            MemDeviceInfo {
                kind: RamType::DDR4,
                id: Some("DEF456".to_string()),
                voltage: Some(1.2),
                size: Some(8000),
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
            kind: RamType::DDR4,
            id: Some("ERROR".to_string()),
            voltage: Some(1.2),
            size: Some(16000),
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

        let result = insert_db(&mut conn, TIMESTAMP, &data, Some(&ram_devices));
        if let Err(e) = result {
            let msg = format!("{e}");
            assert!(msg.contains("no such table"), "Error message: {msg}");
        }

        Ok(())
    }

    // Test `get_mem_test` function with calculation success
    #[test]
    fn test_get_mem_test_success() {
        for &size in &[1_000_000, 5_000_000, 10_000_000] {
            std::env::set_var("MEM_TEST_SIZE", size.to_string());
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
        }
    }

    // Test `get_mem_test` function with invalid bandwidth
    #[test]
    fn test_get_mem_test_error() {
        std::env::set_var("MEM_TEST_SIZE", "0");
        let result = get_mem_test();
        assert!(result.is_err());
        assert!(format!("{}", result.unwrap_err()).contains("Invalid bandwidth calculation"));
    }

    // Test `estimated_power_consumption` function in success case
    #[test]
    fn test_estimated_power_consumption_with_devices() {
        let _ = env_logger::builder().is_test(true).try_init();
        let devices = vec![
            MemDeviceInfo {
                kind: RamType::DDR4,
                id: Some("ABC123".to_string()),
                voltage: Some(1.2),
                size: Some(16000),
            },
            MemDeviceInfo {
                kind: RamType::DDR4,
                id: Some("DEF456".to_string()),
                voltage: None,
                size: Some(8000),
            },
        ];

        let result = estimated_power_consumption(&devices, 12000);
        assert!(result.is_some());
    }

    // Test `estimated_power_consumption` function in calculation error case
    #[test]
    fn test_estimated_power_consumption_no_ram_devices() {
        let _ = env_logger::builder().is_test(true).try_init();
        let ram_devices = vec![
            MemDeviceInfo {
                kind: RamType::DDR4,
                id: Some("ABC123".to_string()),
                voltage: Some(1.2),
                size: None,
            },
            MemDeviceInfo {
                kind: RamType::DDR4,
                id: Some("DEF456".to_string()),
                voltage: Some(1.2),
                size: Some(0),
            },
        ];

        let result = estimated_power_consumption(&ram_devices, 12000);
        assert!(result.is_none());
    }
}
