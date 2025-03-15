//! # Net data Module
//!
//! This module provides functionality to retrieve internet data consumption.

use log::error;
use serde_json::{json, Map, Value};
use std::error::Error;
use sysinfo::{NetworkExt, System, SystemExt};

use crate::utils::write_json_to_file;

const HEADER: &str = "NET_DATA";
const LOGGER: &str = "log/net_data.json";

/// Collection of collected network data.
#[derive(Debug)]
struct NetworkInterface {
    /// Name of network interface.
    name: String,
    /// Received network packages.
    received: Option<u64>,
    /// Transmitted network packages.
    transmitted: Option<u64>,
}

impl NetworkInterface {
    /// Crée une nouvelle instance de `NetworkInterface`.
    fn new(name: String, received: Option<u64>, transmitted: Option<u64>) -> Self {
        Self {
            name,
            received,
            transmitted,
        }
    }

    /// Calcule le total des données (reçues + transmises).
    fn total(&self) -> Option<u64> {
        self.received
            .and_then(|r| self.transmitted.map(|t: u64| r + t))
    }

    /// Convertit les données en JSON.
    fn to_json(&self) -> Value {
        json!({
            "received_MB": self.received.map(|r| r as f64 / 1e6),
            "transmitted_MB": self.transmitted.map(|t| t as f64 / 1e6),
            "total_MB": self.total().map(|total| total as f64 / 1e6)
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
    let mut system: System = System::new_all();
    system.refresh_networks();

    let mut interfaces: Vec<NetworkInterface> = Vec::new();

    for (interface_name, data) in system.networks() {
        let received: Option<u64> = if data.total_received() > 0 {
            Some(data.total_received())
        } else {
            error!(
                "[{HEADER}] Data 'Failed to retrieve received value for interface {}'",
                interface_name
            );
            None
        };

        let transmitted: Option<u64> = if data.total_transmitted() > 0 {
            Some(data.total_transmitted())
        } else {
            error!(
                "[{HEADER}] Data 'Failed to retrieve transmitted value for interface {}'",
                interface_name
            );
            None
        };

        interfaces.push(NetworkInterface::new(
            interface_name.to_string(),
            received,
            transmitted,
        ));
    }

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
        let interfaces: Vec<NetworkInterface> = collect_interface_data()?;

        let net_data: Map<String, Value> = interfaces
            .into_iter()
            .map(|interface: NetworkInterface| (interface.name.clone(), interface.to_json()))
            .collect();

        Ok(json!({ HEADER: net_data }))
    };

    write_json_to_file(data, LOGGER, HEADER);
}
