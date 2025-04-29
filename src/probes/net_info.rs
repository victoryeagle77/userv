//! # Net data Module
//!
//! This module provides functionality to retrieve internet data consumption.

use log::error;
use serde::Serialize;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::error::Error;
use std::{thread, time};
use sysinfo::Networks;

use crate::utils::write_json_to_file;

const FACTOR: f64 = 1e6;
const HEADER: &str = "NET_DATA";
const LOGGER: &str = "log/net_data.json";

/// Collection of network data consumption.
#[derive(Debug, Serialize)]
struct NetworkInterface {
    address_mac: Option<String>,
    /// Name of network interface.
    name: String,
    /// Received network packages in bytes.
    received: Option<u64>,
    /// Transmitted network packages in bytes.
    transmitted: Option<u64>,
    /// Total network packages in bytes.
    total: Option<u64>,
}

impl NetworkInterface {
    /// Convert bytes with a [`FACTOR`] size.
    fn convert(opt: Option<u64>) -> Option<f64> {
        opt.map(|v| v as f64 / FACTOR)
    }

    /// Converts [`NetworkInterface`] into a JSON object with MB values.
    fn to_json(&self) -> Value {
        json!({
            "address_mac": self.address_mac,
            "received_MB": Self::convert(self.received),
            "transmitted_MB": Self::convert(self.transmitted),
            "total_MB": Self::convert(self.total),
        })
    }
}

/// Collects detailed network interface data.
///
/// # Returns
///
/// - A vector of [`NetworkInterface`] containing network data consumption.
/// - An error when no valid network interface found.
fn collect_interface_data() -> Result<Vec<NetworkInterface>, Box<dyn Error>> {
    let mut networks = Networks::new_with_refreshed_list();
    thread::sleep(time::Duration::from_millis(10)); // Waiting a bit to get data from network
    networks.refresh(true); // Refreshing again to generate diff

    let mut interfaces = Vec::new();

    for (name, data) in &networks {
        if name.trim().is_empty() {
            return Err("Data 'Network interface with empty name found'"
                .to_string()
                .into());
        }

        let received = data.total_received();
        let transmitted = data.total_transmitted();
        let address_mac = data.mac_address();

        // Ignoring network interface without traffic
        if received == 0 && transmitted == 0 {
            error!("[{HEADER}] Data 'Interface exists but no data consumed' {name}");
            continue;
        }

        interfaces.push(NetworkInterface {
            address_mac: Some(address_mac.to_string()),
            name: name.to_string(),
            received: Some(received),
            transmitted: Some(transmitted),
            total: Some(received + transmitted),
        });
    }

    if interfaces.is_empty() {
        Err("Data 'No valid network interfaces found'"
            .to_string()
            .into())
    } else {
        Ok(interfaces)
    }
}

/// Public function used to send JSON formatted values,
/// from [`collect_interface_data`] function result.
pub fn get_net_info() -> Result<(), Box<dyn Error>> {
    let interfaces = collect_interface_data()?;
    let data: HashMap<_, _> = interfaces
        .into_iter()
        .map(|iface| (iface.name.clone(), iface.to_json()))
        .collect();
    let values = json!({ HEADER: data });
    write_json_to_file(|| Ok(values), LOGGER)?;
    Ok(())
}
