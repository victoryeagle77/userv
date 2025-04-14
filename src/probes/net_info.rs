//! # Net data Module
//!
//! This module provides functionality to retrieve internet data consumption.

use log::error;
use serde::Serialize;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::error::Error;
use sysinfo::{NetworkExt, NetworksExt, System, SystemExt};

use crate::utils::write_json_to_file;

const FACTOR: f32 = 1e6;

const HEADER: &str = "NET_DATA";
const LOGGER: &str = "log/net_data.json";

/// Collection of collected network data.
#[derive(Debug, Serialize)]
struct NetworkInterface {
    /// Name of network interface.
    name: String,
    /// Received network packages.
    received: Option<u64>,
    /// Transmitted network packages.
    transmitted: Option<u64>,
    /// Total network packages.
    total: Option<u64>,
}

impl NetworkInterface {
    /// Converts `NetworkInterface` into a JSON object.
    fn to_json(&self) -> Value {
        json!({
            "received_MB": self.received.map(|r| r as f32 / FACTOR),
            "transmitted_MB": self.transmitted.map(|t| t as f32 / FACTOR),
            "total_MB": self.total.map(|total| total as f32 / FACTOR)
        })
    }
}

/// Function that retrieves detailed network interface data.
///
/// # Returns
///
/// `result` : A vector of `NetworkInterface` structures with all network interface information:
/// - Name of network interface.
/// - Received network packages in bytes.
/// - Transmitted network packages in bytes.
/// - Total network packages calculated.
fn collect_interface_data() -> Result<Vec<NetworkInterface>, Box<dyn Error>> {
    let mut system = System::new_all();
    system.refresh_networks();

    let interfaces: Vec<NetworkInterface> = system
        .networks()
        .iter()
        .map(|(name, data)| {
            let received: Option<u64> = (data.total_received() > 0)
                .then_some(data.total_received())
                .or_else(|| {
                    error!("[{HEADER}] Data 'Failed to retrieve received value for interface' {name}");
                    None
                });

            let transmitted: Option<u64> = (data.total_transmitted() > 0)
                .then_some(data.total_transmitted())
                .or_else(|| {
                    error!("[{HEADER}] Data 'Failed to retrieve transmitted value for interface' {name}");
                    None
                });

            NetworkInterface {
                name: name.to_string(),
                received,
                transmitted,
                total: received.and_then(|r: u64| transmitted.map(|t: u64| r + t)),
            }
        })
        .collect();

    if interfaces.is_empty() {
        error!("[{HEADER}] Data 'No valid network interfaces found'");
        Err("No valid network interfaces found".into())
    } else {
        Ok(interfaces)
    }
}

/// Public function used to send JSON formatted values,
/// from `collect_interface_data` function result.
pub fn get_net_info() {
    let data = || -> Result<Value, Box<dyn Error>> {
        let values: HashMap<_, _> = collect_interface_data()?
            .into_iter()
            .map(|interface: NetworkInterface| (interface.name.clone(), interface.to_json()))
            .collect();

        Ok(json!({ HEADER: values }))
    };

    write_json_to_file(data, LOGGER, HEADER);
}
