//! # Lib file for network data module
//!
//! This module provides functionality to retrieve internet data consumption.

use serde::Serialize;
use serde_json::{json, Value};
use std::{collections::HashMap, error::Error, thread::sleep, time::Duration};
use sysinfo::Networks;

mod utils;
use utils::*;

/// Collection of network data consumption.
#[derive(Debug, Serialize)]
struct NetworkInterface {
    address_mac: Option<String>,
    /// Name of network interface.
    name: String,
    /// Received network packages in MB.
    received: Option<u64>,
    /// Transmitted network packages in MB.
    transmitted: Option<u64>,
    /// Network errors received in MB.
    errors_received: Option<u64>,
    /// Network errors transmitted in MB.
    errors_transmitted: Option<u64>,
    /// Number of incoming packets in MB.
    packet_received: Option<u64>,
    /// Number of outcoming packets in MB.
    packet_transmitted: Option<u64>,
}

impl NetworkInterface {
    /// Convert bytes with a [`FACTOR`] size.
    fn to_convert(opt: Option<u64>) -> Option<f64> {
        opt.map(|v| v as f64 / FACTOR)
    }

    /// Converts [`NetworkInterface`] into a JSON object with MB values.
    fn to_json(&self) -> Value {
        json!({
            "address_mac": self.address_mac,
            "received_MB": Self::to_convert(self.received),
            "transmitted_MB": Self::to_convert(self.transmitted),
            "errors_received_MB": Self::to_convert(self.errors_received),
            "errors_transmitted_MB": Self::to_convert(self.errors_transmitted),
            "packet_received_MB": Self::to_convert(self.packet_received),
            "packet_transmitted_MB": Self::to_convert(self.packet_transmitted),
        })
    }
}

/// Collects detailed network interface data.
///
/// # Returns
///
/// - A vector of [`NetworkInterface`] containing network data consumption.
/// - An error when no valid network interface found.
fn collect_net_data() -> Result<Vec<NetworkInterface>, Box<dyn Error>> {
    let mut networks = Networks::new_with_refreshed_list();
    sleep(Duration::from_millis(10)); // Waiting a bit to get data from network
    networks.refresh(true); // Refreshing again to generate diff

    let mut interfaces = Vec::new();

    for (name, network) in &networks {
        if name.trim().is_empty() {
            return Err("Data 'Network interface with empty name found'"
                .to_string()
                .into());
        }

        let address_mac = Some(network.mac_address().to_string());

        let received = Some(network.total_received());
        let transmitted = Some(network.total_transmitted());

        let errors_received = Some(network.total_errors_on_received());
        let errors_transmitted = Some(network.total_errors_on_transmitted());

        let packet_received = Some(network.total_packets_received());
        let packet_transmitted = Some(network.total_packets_transmitted());

        interfaces.push(NetworkInterface {
            address_mac,
            name: name.to_string(),
            received,
            transmitted,
            errors_received,
            errors_transmitted,
            packet_received,
            packet_transmitted,
        });
    }

    if interfaces.is_empty() {
        Err("Data 'No valid network interfaces found'".into())
    } else {
        Ok(interfaces)
    }
}

/// Public function used to send JSON formatted values,
/// from [`collect_net_data`] function result.
pub fn get_net_info() -> Result<(), Box<dyn Error>> {
    let interfaces = collect_net_data()?;
    let data: HashMap<_, _> = interfaces
        .into_iter()
        .map(|iface| (iface.name.clone(), iface.to_json()))
        .collect();
    let values = json!({ HEADER: data });
    write_json_to_file(|| Ok(values), LOGGER)?;
    Ok(())
}
