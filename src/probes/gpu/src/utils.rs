//! # File utilities module

use core::core::measure_point;
use std::time::Duration;

use nvml_wrapper::{
    Device,
    enum_wrappers::device::{Clock, ClockId, PcieUtilCounter, TemperatureSensor},
    error::NvmlError,
};
use serde::Serialize;

// Collection of collected GPU data.
#[derive(Serialize)]
pub struct GpuMetrics {
    /// GPU architecture.
    pub gpu_arch: Option<String>,
    /// GPU PCIe bus identification.
    pub gpu_bus_id: Option<String>,
    /// GPU graphic clock usage in MHz.
    pub gpu_clock_graphic: Option<u32>,
    /// GPU memory clock usage in MHz.
    pub gpu_clock_memory: Option<u32>,
    /// GPU streaming multiprocessor clock usage in MHz.
    pub gpu_clock_sm: Option<u32>,
    /// GPU video clock usage in MHz.
    pub gpu_clock_video: Option<u32>,
    /// GPU energy consumption in mJ.
    pub gpu_energy_consumption: Option<f64>,
    /// GPU model name.
    pub gpu_name: Option<String>,
    /// GPU usage in percentage.
    pub gpu_usage: Option<u32>,
    /// GPU temperature in °C.
    pub gpu_temperature: Option<u32>,
    /// Free available computing memory in Bytes.
    pub gpu_memory_free: Option<u64>,
    /// GPU computing memory usage in percentage.
    pub gpu_memory_stat: Option<u32>,
    /// Total GPU computing memory in Bytes.
    pub gpu_memory_total: Option<u64>,
    /// Currently used computing memory in Bytes.
    pub gpu_memory_usage: Option<u64>,
    /// PCI sent data consumption by GPU in Bytes/s.
    pub gpu_pci_data_sent: Option<u32>,
    /// PCI received data consumption by GPU in Bytes/s.
    pub gpu_pci_data_received: Option<u32>,
    /// GPU electrical consumption in mW.
    pub gpu_power_consumption: Option<u32>,
    /// GPU maximum electrical consumption accepted in W.
    pub gpu_power_ratio: Option<f32>,
}

/// Collection of collected running processes GPU data.
#[derive(Serialize)]
pub struct GpuProcessMetrics {
    pub gpu_bus_id: Option<String>,
    /// Process decoder utilization in percentage.
    pub process_dec: Option<u32>,
    /// Process encoder utilization in percentage.
    pub process_enc: Option<u32>,
    /// Process memory utilization by a process in percentage.
    pub process_mem: Option<u32>,
    /// Process PID.
    pub process_pid: Option<u32>,
    /// Streaming Multiprocessor utilization in percentage.
    pub process_sm: Option<u32>,
}

/// Check if an nvml feature is not supported by the current tested device.
fn error<T>(f: impl FnOnce() -> Result<T, NvmlError>) -> Option<T> {
    match f() {
        Ok(res) => Some(res),
        Err(NvmlError::NotSupported) => None,
        Err(_) => None,
    }
}

impl GpuMetrics {
    /// Collect all global hardware GPU metrics for a given device.
    ///
    /// # Arguments
    ///
    /// - `device` : The detected GPU device.
    ///
    /// # Returns
    ///
    /// Completed fields of [`GpuMetrics`].
    pub fn from_device(device: &Device, bus_id: Option<String>) -> Result<Self, NvmlError> {
        // Identifications
        let gpu_bus_id = bus_id;
        let gpu_arch = error(|| device.architecture().map(|a| a.to_string()));
        let gpu_name = error(|| device.name());

        // Memory and utilization
        let gpu_memory_info = error(|| device.memory_info());
        let gpu_utilization = error(|| device.utilization_rates());

        // Clocks
        let gpu_clock_id = ClockId::Current;
        let gpu_clock_graphic = error(|| device.clock(Clock::Graphics, gpu_clock_id));
        let gpu_clock_memory = error(|| device.clock(Clock::Memory, gpu_clock_id));
        let gpu_clock_sm = error(|| device.clock(Clock::SM, gpu_clock_id));
        let gpu_clock_video = error(|| device.clock(Clock::Video, gpu_clock_id));

        // Energy
        let gpu_energy_consumption = measure_point(
            || device.total_energy_consumption().ok().map(|val| val as f64),
            Duration::from_millis(100),
        );

        // Power
        let gpu_power_consumption = error(|| device.power_usage());
        let gpu_power_ratio = match (
            error(|| device.power_usage()),
            error(|| device.power_management_limit()),
        ) {
            (Some(p), Some(limit)) if limit > 0 => Some((p as f32 / limit as f32) * 100.0),
            _ => None,
        };

        // Thermal
        let gpu_temperature = error(|| device.temperature(TemperatureSensor::Gpu));

        // PCIe bus data
        let gpu_pci_data_sent = error(|| device.pcie_throughput(PcieUtilCounter::Send));
        let gpu_pci_data_received = error(|| device.pcie_throughput(PcieUtilCounter::Receive));

        // Memory stats
        let (gpu_memory_free, gpu_memory_total, gpu_memory_usage) = match gpu_memory_info {
            Some(info) => (Some(info.free), Some(info.total), Some(info.used)),
            None => (None, None, None),
        };

        // Utilization
        let (gpu_memory_stat, gpu_usage) = match gpu_utilization {
            Some(util) => (Some(util.memory), Some(util.gpu)),
            None => (None, None),
        };

        Ok(GpuMetrics {
            gpu_arch,
            gpu_name,
            gpu_bus_id,
            gpu_clock_graphic,
            gpu_clock_memory,
            gpu_clock_sm,
            gpu_clock_video,
            gpu_temperature,
            gpu_memory_free,
            gpu_memory_total,
            gpu_memory_usage,
            gpu_memory_stat,
            gpu_usage,
            gpu_pci_data_sent,
            gpu_pci_data_received,
            gpu_energy_consumption,
            gpu_power_consumption,
            gpu_power_ratio,
        })
    }
}

impl GpuProcessMetrics {
    /// Collect all metrics for a given process.
    ///
    /// # Arguments
    ///
    /// - `device` : The detected process.
    ///
    /// # Returns
    ///
    /// Completed fields of [`GpuProcessMetrics`].
    pub fn from_device(device: &Device, id: Option<String>) -> Result<Vec<Self>, NvmlError> {
        let mut results = Vec::new();

        if let Some(utilization_stats) = error(|| device.process_utilization_stats(None)) {
            for proc in utilization_stats {
                results.push(GpuProcessMetrics {
                    gpu_bus_id: id.clone(),
                    process_pid: Some(proc.pid),
                    process_dec: Some(proc.dec_util),
                    process_enc: Some(proc.enc_util),
                    process_mem: Some(proc.mem_util),
                    process_sm: Some(proc.sm_util),
                });
            }
        }

        Ok(results)
    }
}
