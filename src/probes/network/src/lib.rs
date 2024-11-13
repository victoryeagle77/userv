//! # Lib file for network data module
//!
//! This module provides main functionality to retrieve network data on Unix-based systems.

use chrono::{SecondsFormat, Utc};
use rusqlite::{Connection, params};
use std::error::Error;
use sysinfo::Networks;

mod dbms;
mod utils;

use core::core::{db_insert_query, db_table_query_creation, init_db};
use dbms::*;
use utils::{NetworkInterface, collect_network_data};

/// Insert network interface parameters in the database.
///
/// # Arguments
///
/// - `conn` : Allow by a [`Connection`] constructor type the connection with an SQLite database.
/// - `data` : [`NetworkInterface`] information to insert in database.
/// - `timestamp`: Timestamp of the measurement.
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
    let query = db_insert_query(TABLE_NAME, &field_descriptor())?;
    let mut stmt = conn.prepare(&query)?;

    stmt.execute(params![
        timestamp,
        data.name,
        data.address_mac,
        data.network_type.get_name(),
        data.received,
        data.transmitted,
        data.errors_received,
        data.errors_transmitted,
        data.packet_received,
        data.packet_transmitted,
        data.energy_consumed,
    ])?;
    Ok(())
}

/// Public function used to collecting network data,
/// and stores [`collect_network_data`] function result in an SQLite database.
pub fn get_net_info() -> Result<(), Box<dyn Error>> {
    let query = db_table_query_creation(TABLE_NAME, &field_descriptor())?;
    let mut conn = init_db(&query)?;

    let mut networks = Networks::new_with_refreshed_list();
    networks.refresh(true);

    let timestamp = Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true);
    let interfaces = collect_network_data(&networks);
    let tx = conn.transaction()?;

    for interface in interfaces {
        insert_db(&tx, &timestamp, &interface)?;
    }
    tx.commit()?;

    Ok(())
}
