//! # File utilities module
use serde::Serialize;
use std::time::Duration;
use sysinfo::Networks;

const FACTOR: f64 = 1e6;

/// Existing network interface available.
#[derive(Debug, Clone, Copy, Serialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum NetworkType {
    Ethernet,
    Infiniband,
    Wifi,
    Loopback,
    Virtual,
    Unknown,
}

impl NetworkType {
    /// Get the name of the type of a network interface.
    ///
    /// # Returns
    ///
    /// The network interface type name.
    pub fn get_name(&self) -> &'static str {
        match self {
            NetworkType::Ethernet => "ETHERNET",
            NetworkType::Infiniband => "INFINIBAND",
            NetworkType::Wifi => "WIFI",
            NetworkType::Loopback => "LOOPBACK",
            NetworkType::Virtual => "VIRTUAL",
            NetworkType::Unknown => "UNKNOWN",
        }
    }

    /// Reference energy ratio in Wh/GB,
    /// according ARCEP, CNRS, ADEME and HPC documentations.
    ///
    /// # Returns
    ///
    /// The energy ratio per interface.
    fn energy_ratio(&self) -> f64 {
        match self {
            NetworkType::Ethernet => 0.2,
            NetworkType::Infiniband => 0.1,
            NetworkType::Wifi => 0.4,
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
    /// Energy ratio according the traffic type (Wh/GB).
    fn traffic(&self) -> f64 {
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
        || name.starts_with("vnet")
        || name.starts_with("bridge")
    {
        NetworkType::Virtual
    } else if name.starts_with("eth") || name.starts_with("enp") || name.starts_with("eno") {
        NetworkType::Ethernet
    } else if name.starts_with("ib") || name.starts_with("infiniband") {
        NetworkType::Infiniband
    } else if name.starts_with("wlan") || name.starts_with("wlp") || name.starts_with("wlx") {
        NetworkType::Wifi
    } else {
        NetworkType::Unknown
    }
}

/// Evaluation of the traffic type based on data flows.
///
/// # Arguments
///
/// - `received` : Received network data in MB.
/// - `transmitted` : Transmitted network data in MB.
/// - `packet_received` : Received network data packets in millions.
/// - `packet_transmitted` : Transmitted network data packets in millions.
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

/// Estimate CPU overhead energy due to virtualization network interface.
///
/// # Arguments
///
/// - `received` : Received network data in MB.
/// - `transmitted` : Transmitted network data in MB.
/// - `packets_received` : Number of packets received in millions.
/// - `packets_transmitted` : Number of packets transmitted in millions.
///
/// # Returns
///
/// CPU overhead energy in Watts.
fn estimate_cpu_overhead(
    received: f64,
    transmitted: f64,
    packets_received: f64,
    packets_transmitted: f64,
) -> f64 {
    let data_overhead = 0.05 * (received + transmitted);
    let packets_overhead = 0.1 * (packets_received + packets_transmitted);
    data_overhead + packets_overhead
}

/// Calculates an estimation of consumed energy (Wh) and average power (W)
/// according a transferred data volume (in MB) for a network interface.
///
/// # Arguments
///
/// - `received` : Received network data in MB.
/// - `transmitted` : Transmitted network data in MB.
/// - `ratio` : Ratio of the network interface (Wh/GB).
/// - `traffic_ratio` : Ratio according to the traffic type.
/// - `idle_power` : Idle power of the interface (W).
/// - `duration` : Duration of the measurement period (in seconds).
///
/// # Returns
///
/// The total energy consumed in Wh.
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
    pub address_mac: Option<String>,
    /// Estimation of consumed energy according consumed data in Wh.
    pub energy_consumed: Option<f64>,
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
    /// Number of incoming packets in millions.
    pub packet_received: Option<f64>,
    /// Number of outgoing packets in millions.
    pub packet_transmitted: Option<f64>,
}

impl NetworkInterface {
    /// Completes every parameter for each network interface.
    ///
    /// # Arguments
    ///
    /// - `name` : Network interface name.
    /// - `network` : sysinfo NetworkData reference.
    ///
    /// # Returns
    ///
    /// - Completed [`NetworkInterface`] with all information per interface.
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

        let traffic_ratio = traffic_type.traffic();
        let delay = Duration::from_millis(100);
        let duration = delay.as_secs_f64();

        let energy_consumed = if network_type == NetworkType::Virtual {
            let cpu_overhead_energy =
                estimate_cpu_overhead(received, transmitted, packet_received, packet_transmitted)
                    * duration;
            Some(cpu_overhead_energy)
        } else {
            estimate_network_energy(
                received,
                transmitted,
                ratio,
                traffic_ratio,
                idle_power,
                duration,
            )
        };

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
}

/// Retrieves information about each network interface of an IT equipment.
///
/// # Arguments
///
/// - networks : [`Networks`] information available.
///
/// # Returns
///
/// Tuple of completed [`NetworkInterface`] structure with all network information per interface.
pub fn collect_network_data(networks: &Networks) -> Vec<NetworkInterface> {
    networks
        .iter()
        .filter(|(name, _)| !name.trim().is_empty())
        .map(|(name, network)| NetworkInterface::from_interface(name, network))
        .collect()
}
