//! # Lib file for network data module
//!
//! This module provides main functionality to retrieve network data on Unix-based systems.

use chrono::{SecondsFormat, Utc};
use rusqlite::{params, Connection};
use std::{error::Error, path::Path, thread, time::Duration};
use sysinfo::Networks;

mod utils;
use utils::NetworkInterface;

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
    let conn = Connection::open(Path::new(path))?;
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS network_interfaces (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            timestamp TEXT NOT NULL,
            name TEXT NOT NULL,
            address_mac TEXT,
            network_type TEXT NOT NULL,
            received_MB REAL,
            transmitted_MB REAL,
            errors_received_MB REAL,
            errors_transmitted_MB REAL,
            packet_received_MB REAL,
            packet_transmitted_MB REAL,
            estimated_energy_Wh REAL,
            average_power_W REAL
        )",
    )?;
    Ok(conn)
}

/// Insert network interface parameters into the database.
///
/// # Arguments
///
/// - `conn` : Allow by a [`Connection`] constructor type the connection with an SQLite database.
/// - `data` : The data structure to insert in database.
///
/// # Returns
///
/// - Insert the [`NetworkInterface`] filled structure in an SQLite database.
/// - Logs an error if the SQL insert request failed.
fn insert_db(
    conn: &Connection,
    timestamp: &str,
    data: &NetworkInterface,
) -> Result<(), Box<dyn Error>> {
    conn.execute(
        "INSERT INTO network_interfaces (
            timestamp,
            name,
            address_mac,
            network_type,
            received_MB,
            transmitted_MB,
            errors_received_MB,
            errors_transmitted_MB,
            packet_received_MB,
            packet_transmitted_MB,
            estimated_energy_Wh,
            average_power_W
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
        params![
            timestamp,
            data.name,
            data.address_mac,
            format!("{:?}", data.network_type),
            data.received,
            data.transmitted,
            data.errors_received,
            data.errors_transmitted,
            data.packet_received,
            data.packet_transmitted,
            data.estimated_energy,
            data.average_power,
        ],
    )?;
    Ok(())
}

fn collect_network_data(networks: &Networks) -> Vec<NetworkInterface> {
    networks
        .iter()
        .filter(|(name, _)| !name.trim().is_empty())
        .map(|(name, network)| NetworkInterface::from_interface(name, network))
        .collect()
}

/// Public function used to collecting network data,
/// and stores [`collect_network_data`] function result in an SQLite database.
pub fn get_net_info() -> Result<(), Box<dyn Error>> {
    let conn = init_db(DATABASE)?;
    let mut networks = Networks::new_with_refreshed_list();
    thread::sleep(Duration::from_millis(10));
    networks.refresh(true);

    let timestamp = Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true);
    let interfaces = collect_network_data(&networks);

    for interface in interfaces {
        insert_db(&conn, &timestamp, &interface)?;
    }

    Ok(())
}
