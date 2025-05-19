//! # Lib file for memory data module
//!
//! This module provides main functionality to retrieve memories data on Unix-based systems.

use chrono::{SecondsFormat, Utc};
use rusqlite::Connection;
use std::error::Error;
use sysinfo::{MemoryRefreshKind, System};

mod utils;
use core::core::init_db;
use utils::*;

const FACTOR: u64 = 1_000_000;
const REQUEST: &str = "CREATE TABLE IF NOT EXISTS memory_data (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        timestamp TEXT NOT NULL,
        bandwidth_read REAL,
        bandwidth_write REAL,
        ram_total_MB INTEGER,
        ram_used_MB INTEGER,
        ram_free_MB INTEGER,
        ram_available_MB INTEGER,
        ram_power_consumption_W REAL,
        swap_total_MB INTEGER,
        swap_used_MB INTEGER,
        swap_free_MB INTEGER
    );
    CREATE TABLE IF NOT EXISTS memory_modules (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        device_id TEXT NOT NULL,
        ram_type TEXT NOT NULL,
        size_MB INTEGER,
        speed_Mt INTEGER,
        voltage_mV REAL,
        FOREIGN KEY (device_id) REFERENCES memory_data(id)
    )";

/// Retrieves detailed computing and SWAP memories data.
///
/// # Returns
///
/// - Completed [`MemInfo`] structure with all memories information.
/// - List of RAM modules detected (optional).
fn mem_data_build(
    data_ram_test: (Option<f64>, Option<f64>),
    data_ram_devices: Option<Vec<MemDeviceInfo>>,
    sys: &sysinfo::System,
) -> (MemInfo, Option<Vec<MemDeviceInfo>>) {
    let (bandwidth_write, bandwidth_read) = data_ram_test;
    let ram_total = sys.total_memory() / FACTOR;
    let ram_used = sys.used_memory() / FACTOR;
    let ram_available = Some(sys.available_memory() / FACTOR);
    let ram_free = Some(sys.free_memory() / FACTOR);
    let swap_total = Some(sys.total_swap() / FACTOR);
    let swap_free = Some(sys.free_swap() / FACTOR);
    let swap_used = Some(sys.used_swap() / FACTOR);

    let ram_device = data_ram_devices.filter(|data| !data.is_empty());

    let (ram_power_consumption, ram_devices) = match &ram_device {
        Some(devices) if !devices.is_empty() => {
            let power = estimated_power_consumption(devices, ram_used);
            (power, ram_device.clone())
        }
        _ => (None, None),
    };

    (
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
        },
        ram_devices,
    )
}

type MemDataResult = Result<(MemInfo, Option<Vec<MemDeviceInfo>>), Box<dyn Error>>;

/// Public function used to send values in SQLite database,
/// from [`mem_data_collect`] function result.
fn mem_data_collect<F1, F2>(mem_test: F1, mem_device: F2, sys: &System) -> MemDataResult
where
    F1: Fn() -> Result<(Option<f64>, Option<f64>), Box<dyn Error>>,
    F2: Fn() -> Result<Option<Vec<MemDeviceInfo>>, Box<dyn Error>>,
{
    Ok(mem_data_build(mem_test()?, mem_device()?, sys))
}

/// Push in SQLite database memory information retrieve by:
/// - [`MemDeviceInfo`] : Information about memory device(s) module(s) detected on OS.
/// - [`MemInfo`] : Global information about memory.
///
/// # Arguments
///
/// - `init_db` : Initialize a SQL connection with [`REQUEST`].
/// - `collect_mem` : Get data stored in [`MemDeviceInfo`] and [`MemInfo`] structures.
/// - `insert_db` : Insert memory information in database with a timestamp.
pub fn mem_data_push<'a, F1, F2, F3>(
    init_db: F1,
    collect_mem: F2,
    mut insert_db: F3,
) -> Result<(), Box<dyn Error>>
where
    F1: Fn(&'a str) -> Result<Connection, Box<dyn Error>>,
    F2: Fn() -> MemDataResult,
    F3: FnMut(
        &mut Connection,
        &str,
        &MemInfo,
        Option<&Vec<MemDeviceInfo>>,
    ) -> Result<(), Box<dyn Error>>,
{
    let mut conn = init_db(REQUEST)?;
    let timestamp = Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true);
    let (data, ram_devices) = collect_mem()?;
    insert_db(&mut conn, &timestamp, &data, ram_devices.as_ref())?;
    Ok(())
}

/// Initialize the [`sysinfo`] library to start the collect by [`mem_data_collect`].
///
/// # Returns
///
/// Failure if we can't retrieves information or push it in database.
pub fn get_mem_info() -> Result<(), Box<dyn Error>> {
    mem_data_push(
        init_db,
        || {
            let mut sys = System::new_all();
            sys.refresh_memory_specifics(MemoryRefreshKind::everything());
            mem_data_collect(get_mem_test, get_mem_device, &sys)
        },
        insert_db,
    )
}

//----------------//
// UNIT CODE TEST //
//----------------//

#[cfg(test)]
mod tests {
    use super::*;
    use dmidecode::structures::memory_device::Type;
    use rusqlite::Connection;
    use sysinfo::{MemoryRefreshKind, System};

    // Test `build_mem_info` function with detected memory device
    #[test]
    fn test_build_mem_info_with_devices() {
        let mem_test = (Some(1500.0), Some(3000.0));
        let mem_device = Some(vec![MemDeviceInfo {
            kind: Type::Ddr4,
            id: Some("ABC123".to_string()),
            voltage: Some(1.2),
            size: Some(8000),
            speed: Some(200),
        }]);

        let mut sys = System::new();
        sys.refresh_memory_specifics(MemoryRefreshKind::everything());

        let (_mem_info, devices) = mem_data_build(mem_test, mem_device, &sys);
        let devices = devices.expect("should have devices");

        assert_eq!(devices[0].id.as_ref().unwrap(), "ABC123");
    }

    // Test `build_mem_info` function if no memory devices are found
    #[test]
    fn test_build_mem_info_without_devices() {
        let mem_test = (Some(1500.0), Some(3000.0));
        let mem_device = Some(vec![]);

        let mut sys = System::new();
        sys.refresh_memory_specifics(MemoryRefreshKind::everything());

        let (_mem_info, devices) = mem_data_build(mem_test, mem_device, &sys);
        assert!(devices.is_none());
    }

    // Test `collect_mem_data` function with success
    #[test]
    fn test_collect_mem_data_success() {
        let fake_sys = System::new();
        let res = mem_data_collect(
            || Ok((Some(1000.0), Some(2000.0))),
            || {
                Ok(Some(vec![MemDeviceInfo {
                    kind: Type::Ddr4,
                    id: Some("ABC123".to_string()),
                    voltage: Some(1.2),
                    size: Some(4096),
                    speed: Some(128),
                }]))
            },
            &fake_sys,
        );
        assert!(res.is_ok());
    }

    // Test `push_mem_data` function
    #[test]
    fn test_push_mem_data() {
        let init_db = |_: &str| {
            let conn = Connection::open_in_memory().unwrap();
            conn.execute_batch(REQUEST).unwrap();
            Ok(conn)
        };

        let collect_mem_data = || {
            Ok((
                MemInfo {
                    ram_available: Some(1024),
                    ram_free: Some(2048),
                    ram_power_consumption: None,
                    ram_total: Some(4096),
                    ram_used: Some(1024),
                    swap_free: Some(256),
                    swap_total: Some(512),
                    swap_used: Some(64),
                    bandwidth_read: None,
                    bandwidth_write: None,
                },
                None,
            ))
        };

        let mut called = false;
        let mut insert_db =
            |_: &mut rusqlite::Connection, _: &str, _: &MemInfo, _: Option<&Vec<MemDeviceInfo>>| {
                called = true;
                Ok(())
            };

        let res = mem_data_push(init_db, collect_mem_data, &mut insert_db);

        assert!(res.is_ok());
        assert!(called);
    }
}
