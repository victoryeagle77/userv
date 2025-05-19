//! # File utilities module

use rusqlite::{params, Connection};
use serde::Serialize;
use std::{error::Error, time::Duration};

const FACTOR: f64 = 1e6;

/// Existing network interface available.
#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum NetworkType {
    Ethernet,
    Infiniband,
    Wifi,
    Cellular2G,
    Cellular3G,
    Cellular4G,
    Cellular5G,
    Loopback,
    Virtual,
    Unknown,
}

impl NetworkType {
    /// Get the name of the type of an network interface.
    ///
    /// # Returns
    ///
    /// The network interface type name.
    fn get_name(&self) -> &'static str {
        match self {
            NetworkType::Ethernet => "ETHERNET",
            NetworkType::Infiniband => "INFINIBAND",
            NetworkType::Wifi => "WIFI",
            NetworkType::Cellular2G => "2G",
            NetworkType::Cellular3G => "3G",
            NetworkType::Cellular4G => "4G",
            NetworkType::Cellular5G => "5G",
            NetworkType::Loopback => "LOOPBACK",
            NetworkType::Virtual => "VIRTUAL",
            NetworkType::Unknown => "UNKNOWN",
        }
    }

    /// Reference energy ration in Wh/GB,
    /// according ARCEP, CNRS, ADEME and HPC documentations.
    ///
    /// # Returns
    ///
    /// The energy ration per interface.
    fn energy_ratio(&self) -> f64 {
        match self {
            NetworkType::Ethernet => 0.2,
            NetworkType::Infiniband => 0.1,
            NetworkType::Wifi => 0.4,
            NetworkType::Cellular2G => 1.5,
            NetworkType::Cellular3G => 1.2,
            NetworkType::Cellular4G => 1.0,
            NetworkType::Cellular5G => 0.8,
            NetworkType::Loopback => 0.0,
            NetworkType::Virtual => 0.0,
            NetworkType::Unknown => 0.0,
        }
    }

    /// Reference idle power consumption in W,
    /// according ARCEP, CNRS, ADEME and HPC documentations.
    ///
    /// # Returns
    ///
    /// The idle power per interface.
    fn idle_power(&self) -> f64 {
        match self {
            NetworkType::Ethernet => 2.0,
            NetworkType::Infiniband => 1.5,
            NetworkType::Wifi => 3.0,
            NetworkType::Cellular2G => 7.0,
            NetworkType::Cellular3G => 6.0,
            NetworkType::Cellular4G => 5.0,
            NetworkType::Cellular5G => 6.0,
            NetworkType::Loopback => 0.0,
            NetworkType::Virtual => 0.0,
            NetworkType::Unknown => 0.0,
        }
    }
}

/// Network traffic type for energy distinction.
#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum TrafficType {
    /// Big transfers (Files, backups) determined as referenced (optimum efficiency).
    Bulk,
    /// Transfer protocols (SSH, Telnet, ...).
    Interactive,
    /// Optimized and little packets.
    VoIP,
    /// Streaming data (music, video, ...).
    Video,
    /// Not recognized transfer type.
    Unknown,
}

impl TrafficType {
    /// Energy ratio according the traffic type (Wh/Go).
    fn from_traffic(&self) -> f64 {
        match self {
            TrafficType::Bulk => 1.0,        // Optimum efficiency
            TrafficType::Interactive => 1.2, // (Until 20% of supplementary consumption per GB)
            TrafficType::VoIP => 1.5,        // (Until 50% of supplementary consumption per GB)
            TrafficType::Video => 1.1,       // (Light supplementary consumption)
            TrafficType::Unknown => 0.0,
        }
    }
}

/// Check the network type of an interface, and its associated communication technology.
///
/// # Returns
///
/// The string identifying the network interface of [`NetworkType`].
fn get_network_type(interface_name: &str) -> NetworkType {
    let name = interface_name.to_lowercase();
    if name.starts_with("lo") {
        NetworkType::Loopback
    } else if name.starts_with("virbr")
        || name.starts_with("docker")
        || name.starts_with("br-")
        || name.starts_with("veth")
        || name.starts_with("tun")
        || name.starts_with("tap")
        || name.starts_with("vmnet")
        || name.starts_with("bridge")
    {
        NetworkType::Virtual
    } else if name.starts_with("eth") || name.starts_with("enp") || name.starts_with("eno") {
        NetworkType::Ethernet
    } else if name.starts_with("ib") || name.starts_with("infiniband") {
        NetworkType::Infiniband
    } else if name.starts_with("wlan") || name.starts_with("wlp") || name.starts_with("wlx") {
        NetworkType::Wifi
    } else if name.starts_with("wwan") || name.starts_with("ppp") || name.starts_with("rmnet") {
        NetworkType::Cellular4G
    } else {
        NetworkType::Unknown
    }
}

/// Evaluation the traffic type based on data flows.
///
/// # Arguments
///
/// - `received` : Received network data in MB.
/// - `transmitted` : Transmitted network data in MB.
/// - `packet_received` : Received network data packets in MB.
/// - `packet_transmitted` : Transmitted network data packets in MB.
///
/// # Returns
///
/// total energy consumed in W.
fn get_traffic_type(
    received: f64,
    transmitted: f64,
    packet_received: f64,
    packet_transmitted: f64,
) -> TrafficType {
    let total = received + transmitted;
    let total_packets = packet_received + packet_transmitted;

    // Many packets for little data (VoIP)
    if total < 1.0 && total_packets > 0.5 {
        TrafficType::VoIP
    }
    // Many data for little packets (File transfers)
    else if total > 10.0 && total_packets < 0.1 * total {
        TrafficType::Bulk
    }
    // Intermediate flow rate, intermediate number of packets (Video)
    else if total > 5.0 && total_packets > 0.2 * total {
        TrafficType::Video
    }
    // Weak traffic
    else if total < 2.0 {
        TrafficType::Interactive
    }
    // Unknown traffic
    else {
        TrafficType::Unknown
    }
}

/// Calculates an estimation of consumed energy (Wh) and average power (W).
/// according a transferred data volume (in MB) for a network interface.
///
/// # Arguments
///
/// - `received` : Received network data in MB.
/// - `transmitted` : Transmitted network data in MB.
/// - `ratio` : Ratio of the network interface (Wh/GB).
/// - `traffic_ratio` : Ratio according to the traffic type.
/// - `idle_power` : Idle power of the interface (W).
///
/// # Returns
///
/// The total energy consumed in W.
fn estimate_network_energy(
    received: f64,
    transmitted: f64,
    ratio: f64,
    traffic_ratio: f64,
    idle_power: f64,
    duration: f64,
) -> Option<f64> {
    let data = (received + transmitted) / 1e3;
    if data == 0.0 {
        Some(0.0)
    } else {
        let energy_transfer = data * ratio * traffic_ratio;
        let energy_idle = idle_power * duration;
        Some(energy_transfer + energy_idle)
    }
}

/// Collection of network data consumption.
#[derive(Debug, Serialize)]
pub struct NetworkInterface {
    /// Interface Mac address.
    address_mac: Option<String>,
    /// Estimation of consumed energy according consumed data in W.
    energy_consumed: Option<f64>,
    /// Name of network interface.
    name: String,
    /// Type of network.
    network_type: NetworkType,
    /// Received network packages in MB.
    received: Option<f64>,
    /// Transmitted network packages in MB.
    transmitted: Option<f64>,
    /// Network errors received in MB.
    errors_received: Option<f64>,
    /// Network errors transmitted in MB.
    errors_transmitted: Option<f64>,
    /// Number of incoming packets in MB.
    packet_received: Option<f64>,
    /// Number of outcome packets in MB.
    packet_transmitted: Option<f64>,
}

impl NetworkInterface {
    /// Completes every parameters for each network interface.
    ///
    /// # Arguments
    ///
    /// - `name` : Network interface type name.
    /// - `network` :
    ///
    /// # Returns
    ///
    /// - Completed [`NetworkInterface`] structure with all network information per interface.
    /// - An error when no information about BIOS or Motherboard found.
    pub fn from_interface(name: &str, network: &sysinfo::NetworkData) -> Self {
        let received = network.total_received() as f64 / FACTOR;
        let transmitted = network.total_transmitted() as f64 / FACTOR;
        let packet_received = network.total_packets_received() as f64 / FACTOR;
        let packet_transmitted = network.total_packets_transmitted() as f64 / FACTOR;

        let network_type = get_network_type(name);
        let ratio = network_type.energy_ratio();
        let idle_power = network_type.idle_power();
        let traffic_type =
            get_traffic_type(received, transmitted, packet_received, packet_transmitted);
        let traffic_ratio = traffic_type.from_traffic();

        let delay = Duration::from_millis(100);
        let duration = delay.as_secs_f64();

        let energy_consumed = estimate_network_energy(
            received,
            transmitted,
            ratio,
            traffic_ratio,
            idle_power,
            duration,
        );

        NetworkInterface {
            address_mac: Some(network.mac_address().to_string()),
            energy_consumed,
            name: name.to_string(),
            network_type,
            received: Some(received),
            transmitted: Some(transmitted),
            errors_received: Some(network.total_errors_on_received() as f64 / FACTOR),
            errors_transmitted: Some(network.total_errors_on_transmitted() as f64 / FACTOR),
            packet_received: Some(packet_received),
            packet_transmitted: Some(packet_transmitted),
        }
    }

    /// Insert network interface parameters in the database.
    ///
    /// # Arguments
    ///
    /// - `conn` : Allow by a [`Connection`] constructor type the connection with an SQLite database.
    /// - `data` : [`NetworkInterface`] information to insert in database.
    /// - `timestamp`: Timestamp of the measurement.
    ///
    /// # Returns
    ///
    /// - Insert the [`NetworkInterface`] filled structure in an SQLite database.
    /// - Logs an error if the SQL insert request failed.
    pub fn insert_db(
        conn: &Connection,
        timestamp: &str,
        data: &Self,
    ) -> Result<(), Box<dyn Error>> {
        let type_name = data.network_type.get_name();
        conn.execute(
            "INSERT INTO network_data (
                timestamp,
                name,
                address_mac,
                network_type,
                received_MB,
                transmitted_MB,
                errors_received_MB,
                errors_transmitted_MB,
                packet_received_MB,
                packet_transmitted_MB,
                energy_consumed_W
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            params![
                timestamp,
                data.name,
                data.address_mac,
                type_name,
                data.received,
                data.transmitted,
                data.errors_received,
                data.errors_transmitted,
                data.packet_received,
                data.packet_transmitted,
                data.energy_consumed,
            ],
        )?;
        Ok(())
    }
}
