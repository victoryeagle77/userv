//! # Lib file for network data module
//!
//! This module provides functionality to retrieve internet data consumption.

use serde::Serialize;
use serde_json::{json, Value};
use std::{collections::HashMap, error::Error, thread::sleep, time::Duration};
use sysinfo::{NetworkData, Networks};

mod utils;
use utils::*;

/// Collection of network data consumption.
#[derive(Debug, Serialize)]
struct NetworkInterface {
    /// Interface Mac address.
    address_mac: Option<String>,
    /// Name of network interface.
    name: String,
    /// Received network packages in MB.
    received: Option<f64>,
    /// Transmitted network packages in MB.
    transmitted: Option<f64>,
    /// Network errors received in MB.
    errors_received: Option<f64>,
    /// Network errors transmitted in MB.
    errors_transmitted: Option<f64>,
    /// Number of incoming packets in MB.
    packet_received: Option<f64>,
    /// Number of outcome packets in MB.
    packet_transmitted: Option<f64>,
}

impl NetworkInterface {
    /// For each network interface
    ///
    /// # Arguments
    ///
    /// - `name` : Name of the network interface used
    /// - `network` : Corresponding network data to the network interface.
    fn from_interface(name: &str, network: &NetworkData) -> Self {
        NetworkInterface {
            address_mac: Some(network.mac_address().to_string()),
            name: name.to_string(),
            received: Some(network.total_received() as f64 / FACTOR),
            transmitted: Some(network.total_transmitted() as f64 / FACTOR),
            errors_received: Some(network.total_errors_on_received() as f64 / FACTOR),
            errors_transmitted: Some(network.total_errors_on_transmitted() as f64 / FACTOR),
            packet_received: Some(network.total_packets_received() as f64 / FACTOR),
            packet_transmitted: Some(network.total_packets_transmitted() as f64 / FACTOR),
        }
    }

    /// Converts [`NetworkInterface`] into a JSON object with MB values.
    fn to_json(&self) -> Value {
        json!({
            "address_mac": self.address_mac,
            "received_MB": self.received,
            "transmitted_MB": self.transmitted,
            "errors_received_MB": self.errors_received,
            "errors_transmitted_MB": self.errors_transmitted,
            "packet_received_MB": self.packet_received,
            "packet_transmitted_MB": self.packet_transmitted,
        })
    }
}

/// Collects detailed network interface data.
///
/// # Returns
///
/// - A vector of [`NetworkInterface`] containing network data consumption.
/// - An error when no valid network interface found.
fn collect_net_data() -> Result<HashMap<String, Value>, Box<dyn Error>> {
    let mut networks = Networks::new_with_refreshed_list();
    sleep(Duration::from_millis(10));
    networks.refresh(true);

    let mut data = HashMap::new();

    for (name, network) in &networks {
        if name.trim().is_empty() {
            return Err("Data 'Network interface with empty name found'"
                .to_string()
                .into());
        }
        let interface = NetworkInterface::from_interface(name, network);
        data.insert(name.to_string(), interface.to_json());
    }

    if data.is_empty() {
        Err("Data 'No valid network interfaces found'".into())
    } else {
        Ok(data)
    }
}

/// Public function used to send JSON formatted values,
/// from [`collect_net_data`] function result.
pub fn get_net_info() -> Result<(), Box<dyn Error>> {
    let data = collect_net_data()?;
    let values = json!({ HEADER: data });
    write_json_to_file(|| Ok(values), LOGGER)?;
    Ok(())
}
