//! # Lib file for network data module
//!
//! This module provides main functionality to retrieve network data on Unix-based systems.

use chrono::{SecondsFormat, Utc};
use std::{error::Error, thread, time::Duration};
use sysinfo::Networks;

mod utils;
use core::core::init_db;
use utils::NetworkInterface;

const REQUEST: &'static str = "CREATE TABLE IF NOT EXISTS network_data (
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
    )";

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
    let conn = init_db(REQUEST)?;
    let mut networks = Networks::new_with_refreshed_list();
    thread::sleep(Duration::from_millis(10));
    networks.refresh(true);

    let timestamp = Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true);
    let interfaces = collect_network_data(&networks);

    for interface in interfaces {
        NetworkInterface::insert_db(&conn, &timestamp, &interface)?;
    }

    Ok(())
}
