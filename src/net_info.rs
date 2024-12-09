//! # Net data Module
//!
//! This module provides functionality to retrieve internet data consumption.

use serde::Serialize;
use serde_json::json;

use crate::utils::{format_unit, parse_file_content};

const NETDEV: &str = "/proc/net/dev";
const HEADER: &str = "NET_DATA";

/// Collection of collected network data
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

/// Function `collect_interface_data` reading and using `/proc/net/dev` file values,
/// which calculating for each network interfaces data consumption in Bytes
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
        return Err("No valid network interfaces found.".to_string());
    }

    Ok(result)
}

/// Public function used to send JSON formatted values,
/// from `collect_interface_data` function result.
pub fn get_net_info() {
    match collect_interface_data() {
        Ok(values) => {
            let mut data: serde_json::Value = json!({});

            for interface in values {
                data[&interface.name] = json!({
                    "received": format_unit(interface.received.unwrap_or(0)),
                    "transmitted": format_unit(interface.transmitted.unwrap_or(0)),
                    "total": format_unit(interface.total)
                });
            }

            let net_json_info: serde_json::Value = json!({ HEADER: data });

            println!("{}", serde_json::to_string_pretty(&net_json_info).unwrap());
        }
        Err(e) => {
            eprintln!("{}", e);
        }
    }
}
