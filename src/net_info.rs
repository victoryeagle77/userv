//! # Net data Module
//!
//! This module provides functionality to retrieve internet data consumption.

use serde_json::json;
use serde::Serialize;

use crate::utils::{format_unit, parse_file_content};

const NETDEV: &str = "/proc/net/dev";

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
/// `NetworkInterface` : A vector containing the data for each network interface.
/// - Name of network interface
/// - Received network packages
/// - Transmitted network packages
/// - Total network packages calculated
fn collect_interface_data() -> Result<Vec<NetworkInterface>, String> {
    let data = parse_file_content(NETDEV, ":");
    let mut interfaces = Vec::new();

    for (key, value) in data.into_iter() {
        let parts: Vec<&str> = value.split_whitespace().collect();
        if parts.len() >= 16 {

            let received = parts[0].parse::<u64>().ok();
            let transmitted = parts[8].parse::<u64>().ok();
            let total = received.unwrap_or(0) + transmitted.unwrap_or(0);

            interfaces.push(NetworkInterface {
                name: key,
                received,
                transmitted,
                total,
            });
        } else {
            return Err(format!("Missing interfaces fields : {}", key));
        }
    }

    if interfaces.is_empty() {
        return Err("No valid network interfaces found.".to_string());
    }

    Ok(interfaces)
}

/// Public function used to send JSON formatted values,
/// from `collect_interface_data` function result.
pub fn get_net_info() {
    match collect_interface_data() {
        Ok(interfaces) => {
            let mut data: serde_json::Value = json!({});

            for interface in interfaces {
                data[&interface.name] = json!({
                    "received": format_unit(interface.received.unwrap_or(0)),
                    "transmitted": format_unit(interface.transmitted.unwrap_or(0)),
                    "total": format_unit(interface.total)
                });
            }

            let net_json_info: serde_json::Value = json!({ "NET_DATA": data });

            println!("\n[[ NET DATA ]]\n");
            println!("{}", serde_json::to_string_pretty(&net_json_info).unwrap());
        },
        Err(e) => {
            eprintln!("{}", e);
        }
    }
}