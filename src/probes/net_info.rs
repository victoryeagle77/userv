//! # Net data Module
//!
//! This module provides functionality to retrieve internet data consumption.

use serde::Serialize;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::error::Error;
use sysinfo::{NetworkExt, System, SystemExt};

use crate::utils::write_json_to_file;

const FACTOR: f64 = 1e6;

const HEADER: &str = "NET_DATA";
const LOGGER: &str = "log/net_data.json";

/// Collection of collected network data consumption.
#[derive(Debug, Serialize)]
struct NetworkInterface {
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
/// A vector of [`NetworkInterface`] containing network data consumption.
fn collect_interface_data() -> Result<Vec<NetworkInterface>, Box<dyn Error>> {
    let mut system = System::new();
    system.refresh_networks_list();
    system.refresh_networks();

    let mut interfaces = Vec::new();

    for (name, data) in system.networks() {
        let received = data.total_received();
        let transmitted = data.total_transmitted();

        // Ignoring network interface without traffic
        if received == 0 && transmitted == 0 {
            continue;
        }

        interfaces.push(NetworkInterface {
            name: name.to_string(),
            received: Some(received),
            transmitted: Some(transmitted),
            total: Some(received + transmitted),
        });
    }

    if interfaces.is_empty() {
        Err("Data 'No valid network interfaces found'".into())
    } else {
        Ok(interfaces)
    }
}

/// Public function to gather network info and write it as JSON to file.
///
/// # Returns
///
/// Returns a Result to propagate errors.
pub fn get_net_info() -> Result<(), Box<dyn Error>> {
    let interfaces = collect_interface_data()?;

    let values: HashMap<_, _> = interfaces
        .into_iter()
        .map(|iface| (iface.name.clone(), iface.to_json()))
        .collect();

    let json_value = json!({ HEADER: values });

    write_json_to_file(|| Ok(json_value), LOGGER, HEADER)?;

    Ok(())
}
