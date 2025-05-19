//! # Lib file for memory data module
//!
//! This module provides main functionality to retrieve memories data on Unix-based systems.

use chrono::{SecondsFormat, Utc};
use rusqlite::{params, Connection};
use std::{error::Error, path::Path};
use sysinfo::{MemoryRefreshKind, System};

mod utils;
use utils::*;

pub const DATABASE: &str = "log/data.db";

/// Initialize the SQLite database and create the table if needed
///
/// # Arguments
///
/// - `path` : Path to database file.
///
/// # Returns
///
/// - A [`Connection`] constructor to initialize database parameters.
/// - An error if the table creation or database initialization failed.
fn init_db(path: &str) -> Result<Connection, Box<dyn Error>> {
    let conn = Connection::open(Path::new(path))?;
    // Main table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS memory_info (
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
        )",
        [],
    )?;
    // Secondary table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS memory_modules (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            device_id INTEGER NOT NULL,
            ram_type TEXT NOT NULL,
            size_MB INTEGER,
            estimated_power_W REAL,
            FOREIGN KEY (device_id) REFERENCES memory_info(id)
        )",
        [],
    )?;
    Ok(conn)
}

/// Insert memory device parameters into the database.
///
/// # Arguments
///
/// - `conn`: Connection to SQLite database.
/// - `timestamp`: Timestamp of the measurement.
/// - `data`: Memory info to insert.
/// - `ram_devices`: List of RAM modules (optional, can be None).
///
/// # Returns
///
/// - Inserts data into memory_info and memory_modules.
fn insert_db(
    conn: &mut Connection,
    timestamp: &str,
    data: &MemInfo,
    ram_devices: Option<&Vec<MemDeviceInfo>>,
) -> Result<(), Box<dyn Error>> {
    let tx = conn.transaction()?;

    // Insert the main memory parameters in table
    tx.execute(
        "INSERT INTO memory_info (
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
    if let Some(ram_devices) = ram_devices {
        for module in ram_devices {
            let power = estimated_power_consumption(ram_devices, module.size.unwrap_or(0));

            tx.execute(
                "INSERT INTO memory_modules (
                    device_id,
                    ram_type,
                    size_MB,
                    estimated_power_W
                ) VALUES (?1, ?2, ?3, ?4)",
                params![module.id, module.kind, module.size, power],
            )?;
        }
    }
    tx.commit()?;
    Ok(())
}

/// Retrieves detailed computing and SWAP memories data.
///
/// # Returns
///
/// - Completed [`MemInfo`] structure with all memories information.
/// - List of RAM modules detected (optional).
fn collect_mem_data() -> Result<(MemInfo, Option<Vec<MemDeviceInfo>>), Box<dyn Error>> {
    let mut sys = System::new_all();
    sys.refresh_memory_specifics(MemoryRefreshKind::everything());

    let (bandwidth_write, bandwidth_read) = get_mem_test()?;

    let ram_total = sys.total_memory() / FACTOR;
    let ram_used = sys.used_memory() / FACTOR;

    let ram_available = Some(sys.available_memory() / FACTOR);
    let ram_free = Some(sys.free_memory() / FACTOR);

    let swap_total = Some(sys.total_swap() / FACTOR);
    let swap_free = Some(sys.free_swap() / FACTOR);
    let swap_used = Some(sys.used_swap() / FACTOR);

    let ram_device = get_ram_device()?.filter(|data| !data.is_empty());
    let (ram_power_consumption, ram_devices) = match ram_device {
        Some(ref ram_devices) if !ram_devices.is_empty() => {
            let power = estimated_power_consumption(ram_devices, ram_used);
            (power, Some(ram_devices.clone()))
        }
        _ => (None, None),
    };

    Ok((
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
    ))
}

/// Public function used to send JSON formatted values,
/// from [`collect_mem_data`] function result.
pub fn get_mem_info() -> Result<(), Box<dyn Error>> {
    let mut conn = init_db(DATABASE)?;
    let timestamp = Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true);
    let (data, ram_devices) = collect_mem_data()?;
    insert_db(&mut conn, &timestamp, &data, ram_devices.as_ref())?;
    Ok(())
}
