//! # File utilities module

use serde::Serialize;

const FACTOR: &'static f64 = &1e6;

/// Enumeration of network type according radiocommunication protocol.
#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum NetworkType {
    /// Wired ethernet network interface.
    Ethernet,
    /// Specific HPC wired network interface.
    InfiniBand,
    /// Wireless network interface.
    Wifi,
    /// Cellular network interface.
    Cellular2G,
    Cellular3G,
    Cellular4G,
    Cellular5G,
    /// Network interface used for machine internal communication.
    Loopback,
    /// Network interface used for containers and virtualization.
    Virtual,
    /// Network interface not detected or unknown.
    Unknown,
}

impl NetworkType {
    /// Average value in Wh/GB for the ratio associated to a radiocommunication technology used.
    fn energy_ratio(&self) -> f64 {
        match self {
            NetworkType::Ethernet => 0.2,
            NetworkType::InfiniBand => 0.1,
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

    /// Idle power consumption of the interface in W (for context/load modeling).
    fn idle_power(&self) -> f64 {
        match self {
            NetworkType::Ethernet => 2.0,
            NetworkType::InfiniBand => 1.5,
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
    /// Optimized and little packets, voice.
    VoIP,
    /// Streaming data (music, video, ...).
    Video,
    /// Not recognized transfer type.
    Unknown,
}

impl TrafficType {
    /// Energy ratio according the traffic type (Wh/Go)
    fn traffic_ratio(&self) -> f64 {
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
/// The string identifying the network interface.
fn guess_network_type(interface_name: &str) -> NetworkType {
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
        NetworkType::InfiniBand
    } else if name.starts_with("wlan") || name.starts_with("wlp") || name.starts_with("wlx") {
        NetworkType::Wifi
    } else if name.starts_with("wwan") || name.starts_with("ppp") || name.starts_with("rmnet") {
        NetworkType::Cellular4G
    } else if name.starts_with("wwan") || name.starts_with("ppp") || name.starts_with("rmnet") {
        NetworkType::Cellular5G
    } else {
        NetworkType::Unknown
    }
}

fn guess_traffic_type(
    received: f64,
    transmitted: f64,
    packet_received: f64,
    packet_transmitted: f64,
) -> TrafficType {
    let total = received + transmitted;
    let total_packets = packet_received + packet_transmitted;

    // Heuristique simple : beaucoup de paquets pour peu de données => VoIP ou Interactive
    if total < 1.0 && total_packets > 0.5 {
        TrafficType::VoIP
    }
    // Beaucoup de données, peu de paquets => Bulk (transfert fichiers)
    else if total > 10.0 && total_packets < 0.1 * total {
        TrafficType::Bulk
    }
    // Débit intermédiaire, nombre de paquets intermédiaire => Video
    else if total > 5.0 && total_packets > 0.2 * total {
        TrafficType::Video
    }
    // Faible trafic => Interactive
    else if total < 2.0 {
        TrafficType::Interactive
    }
    // Sinon, inconnu
    else {
        TrafficType::Unknown
    }
}

/// Calculates an estimation of consumed energy (Wh) and average power (W).
/// according a transferred data volume (in MB) for a network interface.
///
/// # Arguments
/// - `received` : Received network data in MB.
/// - `transmitted` : Transmitted network data in MB.
/// - `ratio` : Ratio of the network interface (Wh/GB).
/// - `traffic_ratio` : Ratio according to the traffic type.
/// - `idle_power` : Idle power of the interface (W).
///
/// # Returns
///
/// - total energy consumed in Wh.
/// - Average power consumed in W.
fn estimate_network_energy(
    received: f64,
    transmitted: f64,
    ratio: f64,
    traffic_ratio: f64,
    idle_power: f64,
) -> (f64, f64) {
    let duration = 1.0 / 60.0;
    let data = (received + transmitted) / 1e3;
    let energy_transfer = data * ratio * traffic_ratio;
    let energy_idle = idle_power * duration;
    let total_energy = energy_transfer + energy_idle;
    let average_power = if duration > 0.0 {
        total_energy / duration
    } else {
        0.0
    };
    (total_energy, average_power)
}

/// Collection of network data consumption.
#[derive(Debug, Serialize)]
pub struct NetworkInterface {
    /// Interface Mac address.
    pub address_mac: Option<String>,
    /// Average power consumption according consumed data in W.
    pub average_power: Option<f64>,
    /// Estimation of consumed energy according consumed data in Wh.
    pub estimated_energy: Option<f64>,
    /// Name of network interface.
    pub name: String,
    /// Type of network.
    pub network_type: NetworkType,
    /// Received network packages in MB.
    pub received: Option<f64>,
    /// Transmitted network packages in MB.
    pub transmitted: Option<f64>,
    /// Network errors received in MB.
    pub errors_received: Option<f64>,
    /// Network errors transmitted in MB.
    pub errors_transmitted: Option<f64>,
    /// Number of incoming packets in MB.
    pub packet_received: Option<f64>,
    /// Number of outcome packets in MB.
    pub packet_transmitted: Option<f64>,
}

impl NetworkInterface {
    pub fn from_interface(name: &str, network: &sysinfo::NetworkData) -> Self {
        let received = network.total_received() as f64 / FACTOR;
        let transmitted = network.total_transmitted() as f64 / FACTOR;
        let packet_received = network.total_packets_received() as f64 / FACTOR;
        let packet_transmitted = network.total_packets_transmitted() as f64 / FACTOR;

        let network_type = guess_network_type(name);
        let ratio = network_type.energy_ratio();
        let idle_power = network_type.idle_power();
        let traffic_type =
            guess_traffic_type(received, transmitted, packet_received, packet_transmitted);
        let traffic_ratio = traffic_type.traffic_ratio();

        let (estimated_energy, average_power) =
            estimate_network_energy(received, transmitted, ratio, traffic_ratio, idle_power);

        NetworkInterface {
            address_mac: Some(network.mac_address().to_string()),
            average_power: Some(average_power),
            estimated_energy: Some(estimated_energy),
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
}
