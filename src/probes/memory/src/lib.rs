//! # Lib file for memory data module
//!
//! This module provides main functionality to retrieve memories data on Unix-based systems.

use chrono::{SecondsFormat, Utc};
use log::error;
use once_cell::sync::Lazy;
use rusqlite::{Connection, params};
use std::{error::Error, fs::read, sync::Mutex};
use sysinfo::{MemoryRefreshKind, System};

mod utils;
use core::core::{DMIDECODE_BIN, ENTRY_BIN, init_db};
use utils::*;

/// SQLite request for each data to insert in table.
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

/// Use `init_db` to initialize a SQL connection with [`REQUEST`] and safely lock its resource.
static DB_CONN: Lazy<Mutex<Connection>> = Lazy::new(|| {
    let conn = init_db(REQUEST).expect("DB initialization failed");
    Mutex::new(conn)
});

/// # Returns
///
/// - The [`MemDeviceInfo`] structure composed of statics data concerning memory device hardware.
/// - The [`MemInfo`] structure composed of dynamics data concerning memory on system.
/// - Error if we can retrieve these information.
type DataResult = Result<(MemInfo, Option<Vec<MemDeviceInfo>>), Box<dyn Error>>;

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
/// The [`MemDeviceInfo`] is a set of statics information, their are retrieved only one time.
/// The [`MemInfo`] is a set of dynamics information retrieved and refresh at each call.
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

/// Public function used to send values in SQLite database,
/// from [`mem_data_collect`] function result.
fn mem_data_collect<F1, F2>(ram_test: F1, ram_device: F2, sys: &System) -> DataResult
where
    F1: Fn() -> Result<(Option<f64>, Option<f64>), Box<dyn Error>>,
    F2: Fn() -> Result<Option<Vec<MemDeviceInfo>>, Box<dyn Error>>,
{
    Ok(mem_data_build(ram_test()?, ram_device()?, sys))
}

/// Push in SQLite database memory information retrieve by:
/// - [`MemDeviceInfo`]: Information about memory device(s) module(s) detected on OS.
/// - [`MemInfo`]: Global information about memory.
///
/// # Arguments
///
/// - `collect_mem`: Get data stored in [`MemDeviceInfo`] and [`MemInfo`] structures.
/// - `insert_db`: Insert memory information in database with a timestamp.
pub fn mem_data_push<F2, F3>(ram_collect: F2, mut insert_db: F3) -> Result<(), Box<dyn Error>>
where
    F2: Fn() -> DataResult,
    F3: FnMut(
        &mut Connection,
        &str,
        &MemInfo,
        Option<&Vec<MemDeviceInfo>>,
    ) -> Result<(), Box<dyn Error>>,
{
    let mut conn = DB_CONN
        .lock()
        .map_err(|e| Box::<dyn Error>::from(format!("Mutex error: {e:?}")))?;
    let timestamp = Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true);
    let (data, ram_devices) = ram_collect()?;
    insert_db(&mut conn, &timestamp, &data, ram_devices.as_ref())?;
    Ok(())
}

/// Initialize the [`sysinfo`] library to start the collect by [`mem_data_collect`].
///
/// # Returns
///
/// Failure if we can't retrieves information or push it in database.
pub fn get_mem_info() -> Result<(), Box<dyn Error>> {
    let entry_buf = read(ENTRY_BIN)?;
    let dmi_buf = read(DMIDECODE_BIN)?;

    mem_data_push(
        || {
            let mut sys = System::new_all();
            sys.refresh_memory_specifics(MemoryRefreshKind::everything());
            mem_data_collect(get_mem_test, || get_mem_device(&entry_buf, &dmi_buf), &sys)
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
    use sysinfo::System;

    const TIMESTAMP: &'static str = "2025-08-15T14:00:00Z";

    fn mock_data(data: MemInfo, ram_type: Type) -> i64 {
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

        assert_eq!(mock_data(data.clone(), Type::Ddr5), 1);
        assert_eq!(mock_data(data.clone(), Type::Ddr4), 1);
        assert_eq!(mock_data(data.clone(), Type::Ddr3), 1);
        assert_eq!(mock_data(data.clone(), Type::Ddr2), 1);
        assert_eq!(mock_data(data.clone(), Type::Ddr), 1);
        assert_eq!(mock_data(data.clone(), Type::Sdram), 1);
        assert_eq!(mock_data(data.clone(), Type::LpDdr5), 1);
        assert_eq!(mock_data(data.clone(), Type::LpDdr4), 1);
        assert_eq!(mock_data(data.clone(), Type::LpDdr3), 1);
        assert_eq!(mock_data(data.clone(), Type::LpDdr2), 1);
        assert_eq!(mock_data(data.clone(), Type::Unknown), 1);
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

    // Test `collect_mem_data` function with success
    #[test]
    fn test_collect_mem_data_success() {
        let mock_sys = System::new();
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
            &mock_sys,
        );
        assert!(res.is_ok());
    }
}
