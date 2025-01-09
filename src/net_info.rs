//! # Net data Module
//!
//! This module provides functionality to retrieve internet data consumption.

use chrono::Utc;
use log::error;
use serde::Serialize;
use serde_json::json;

use crate::utils::{format_unit, parse_file_content, write_json_to_file};

const NETDEV: &str = "/proc/net/dev";

const HEADER: &str = "NET_DATA";
const LOGGER: &str = "log/net_data.json";

/// Collection of collected network data in bytes
#[derive(Debug, Serialize)]
struct NetworkInterface {
    /// Name of network interface.
    name: String,
    /// Received network packages.
    received: Option<u64>,
    /// Transmitted network packages.
    transmitted: Option<u64>,
    /// Total network packages calculated.
    total: u64,
}

/// Public function reading and using `/proc/net/dev` file values,
/// and retrieves detailed each network interfaces data consumption calculated in Bytes.
///
/// # Returns
///
/// `result` : Completed `NetworkInterface` structure with all network interfaces information
/// - Name of network interface
/// - Received network packages
/// - Transmitted network packages
/// - Total network packages calculated
fn collect_interface_data() -> Result<Vec<NetworkInterface>, String> {
    let data = parse_file_content(NETDEV, ":");
    let mut result = Vec::new();

    for (key, value) in data.into_iter() {
        let parts: Vec<&str> = value.split_whitespace().collect();
        if parts.len() >= 16 {
            let received = parts[0].parse::<u64>().ok();
            let transmitted = parts[8].parse::<u64>().ok();
            let total = received.unwrap_or(0) + transmitted.unwrap_or(0);

            result.push(NetworkInterface {
                name: key,
                received,
                transmitted,
                total,
            });
        } else {
            return Err(format!("Missing interfaces fields : {}", key));
        }
    }

    if result.is_empty() {
        return Err("No valid network interfaces found".to_string());
    }

    Ok(result)
}

/// Public function used to send JSON formatted values,
/// from `collect_interface_data` function result.
pub fn get_net_info() {
    match collect_interface_data() {
        Ok(values) => {
            let mut item: serde_json::Value = json!({
                "timestamp": Utc::now().to_rfc3339()
            });

            for interface in values {
                item[&interface.name] = json!({
                    "received": format_unit(interface.received.unwrap_or(0)),
                    "transmitted": format_unit(interface.transmitted.unwrap_or(0)),
                    "total": format_unit(interface.total)
                });
            }

            let data: serde_json::Value = json!({ HEADER: item });

            if let Err(e) = write_json_to_file(data, LOGGER) {
                error!("[{}] {}", HEADER, e);
            }
        }
        Err(e) => {
            error!("[{}] {}", HEADER, e);
        }
    }
}