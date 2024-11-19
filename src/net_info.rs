//! # Net data Module
//!
//! This module provides functionality to retrieve internet data consumption.

use std::collections::HashMap;
use serde_json::json;

use crate::utils::{parse_file_content, format_unit};

const NETDEV: &'static str = "/proc/net/dev";
const FIELD: &'static usize = &16;
const JSONKEY: &'static str = "NET_DATA";

/// # Function
/// 
/// Function `net_interface_consumption` reading and using `/proc/net/dev` file values,
/// which calculating for each network interfaces data consumption in Bytes
/// 
/// # Return
/// 
/// The content of `values` return 3 parameters :
/// * `String` : Interface name
/// * `u64` : Received network data in Bytes
/// * `u64` : Transmitted network data in Bytes
/// 
/// # Dependencies
/// 
/// Using `parse_file_content` function to extract net consumption values,
/// and `HashMap` to store the result as a formatted content
///
fn net_interface_consumption() -> HashMap<String, (u64, u64)> {
    let data = parse_file_content(NETDEV, ":");
    let mut values = HashMap::new();

    for (key, value) in data.into_iter() {
        let parts: Vec<&str> = value.split_whitespace().collect();
        if parts.len() >= *FIELD {
            let received = parts[0].parse::<u64>().unwrap_or_else(|_| {
                eprintln!("<ERROR_4> Parsing for 'received' : {}", parts[0]);
                0
            });
            let transmitted = parts[8].parse::<u64>().unwrap_or_else(|_| {
                eprintln!("<ERROR_4> Parsing for 'transmitted' : {}", parts[8]);
                0
            });
            values.insert(key, (received, transmitted));
        } else {
            eprintln!("Missing interfaces fields : {}", key);
        }
    }

    return values;
}

/// # Function
/// 
/// Public function `get_net_info` formatting `net_interface_consumption` return value
/// 
/// # Dependencies
/// 
/// Using `net_interface_consumption` function to get consumption values,
/// and `serde_json` to formate the result as a JSON object.
/// 
pub fn get_net_info() {
    let values = net_interface_consumption();
    let mut data: serde_json::Value = json!({});

    for (key, (received, transmitted)) in values {
        data[key] = json!({
            "received": format_unit(received),
            "transmitted": format_unit(transmitted),
            "total": format_unit(received + transmitted)
        });
    }

    let net_json_info: serde_json::Value = json!({
        JSONKEY: data
    });

    println!("\n[[ NET DATA ]]\n");
    println!("{}", serde_json::to_string_pretty(&net_json_info).unwrap());
}