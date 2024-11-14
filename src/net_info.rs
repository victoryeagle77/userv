//! # Net data Module
//!
//! This module provides functionality to retrieve internet data consumtion.

use std::fs::File;
use std::io::{BufRead, BufReader};
use std::collections::HashMap;
use colored::Colorize;
use serde_json::json;

use crate::utils::{parse_file_content, format_unit};

const NETDEV: &'static str = "/proc/net/dev";
const FIELD: &'static usize = &16;
const JSONKEY: &'static str = "NET_DATA";

fn calc_net_info() -> HashMap<String, (u64, u64)> {
    let data = parse_file_content(NETDEV, ":");
    let mut values = HashMap::new();

    for (key, value) in data.into_iter() {
        let parts: Vec<&str> = value.split_whitespace().collect();
        if parts.len() >= *FIELD {
            let received = parts[0].parse::<u64>().unwrap_or_else(|_| {
                eprintln!("Erreur de parsing pour 'received' : {}", parts[0]);
                0
            });
            let transmitted = parts[8].parse::<u64>().unwrap_or_else(|_| {
                eprintln!("Erreur de parsing pour 'transmitted' : {}", parts[8]);
                0
            });
            values.insert(key, (received, transmitted));
        } else {
            eprintln!("Pas assez de champs pour l'interface : {}", key);
        }
    }

    return values;
}

// Fonction pour placer les valeurs dans un JSON et les afficher
pub fn get_net_info() {
    let values = calc_net_info();
    let mut data = json!({});

    for (key, (received, transmitted)) in values {
        data[key] = json!({
            "recieved": format_unit(received),
            "transmited": format_unit(transmitted),
            "total": format_unit(received + transmitted)
        });
    }

    let net_json_info = json!({
        JSONKEY: data
    });

    println!("{}", "\n[[ NET DATA ]]\n".magenta().bold());
    println!("{}", serde_json::to_string_pretty(&net_json_info).unwrap());
}