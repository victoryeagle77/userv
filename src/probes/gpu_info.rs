//! # GPU data Module
//!
//! This module provides functionality to retrieve GPU data on Unix-based systems.

use log::error;
use nvml_wrapper::{
    enum_wrappers::device::{Clock, ClockId, PcieUtilCounter, TemperatureSensor},
    error::NvmlError,
    struct_wrappers::device::ProcessUtilizationSample,
    Device, Nvml,
};
use serde::Serialize;
use serde_json::{json, Value};
use std::error::Error;

use crate::utils::write_json_to_file;

const HEADER: &str = "GPU";
const LOGGER: &str = "log/gpu_data.json";

/// Helper for NVML error handling.
fn nvml_try<T, F>(context: &'static str, f: F) -> Result<T, NvmlError>
where
    F: FnOnce() -> Result<T, NvmlError>,
{
    match f() {
        Ok(val) => Ok(val),
        Err(e) => {
            error!("[{HEADER}] Data '{context}' : {e}");
            Err(e)
        }
    }
}

// Collection of collected GPU data.
#[derive(Serialize)]
struct GpuMetrics {
    /// GPU architecture.
    gpu_arch: Option<String>,
    /// GPU PCIe bus identification.
    gpu_bus_id: Option<String>,
    /// GPU graphic clock usage in MHz.
    gpu_clock_graphic: Option<u32>,
    /// GPU memory clock usage in MHz.
    gpu_clock_memory: Option<u32>,
    /// GPU streaming multiprocessor clock usage in MHz.
    gpu_clock_sm: Option<u32>,
    /// GPU video clock usage in MHz.
    gpu_clock_video: Option<u32>,
    /// GPU energy consumption in J.
    gpu_energy_consumption: Option<f32>,
    /// Speed per fan in percentage.
    gpu_fan_speed: Vec<Option<u32>>,
    /// GPU model name.
    gpu_name: Option<String>,
    /// GPU usage in percentage.
    gpu_usage: Option<u32>,
    /// GPU temperature in °C.
    gpu_temperature: Option<u32>,
    /// Free available computing memory in GB.
    gpu_memory_free: Option<f32>,
    /// GPU computing memory usage in percentage.
    gpu_memory_stat: Option<u32>,
    /// Total GPU computing memory in GB.
    gpu_memory_total: Option<f32>,
    /// Currently used computing memory in GB.
    gpu_memory_usage: Option<f32>,
    /// PCI sent data consumption by GPU in KB/s.
    gpu_pci_data_sent: Option<u32>,
    /// PCI received data consumption by GPU in KB/s.
    gpu_pci_data_received: Option<u32>,
    /// GPU electrical consumption in mW.
    gpu_power_consumption: Option<f32>,
    /// GPU maximum electrical consumption accepted in mW.
    gpu_power_limit: Option<f32>,
}

/// Collection of collected running processes GPU data.
#[derive(Serialize)]
struct GpuProcessMetrics {
    /// Process decoder utilization in percentage.
    process_dec: Option<u32>,
    /// Process encoder utilization in percentage.
    process_enc: Option<u32>,
    /// Process memory utilization by a process in percentage.
    process_mem: Option<u32>,
    /// Process PID.
    process_pid: Option<u32>,
    /// Streaming Multiprocessor utilization in percentage.
    process_sm: Option<u32>,
}

impl GpuMetrics {
    /// Converts [`GpuMetrics`] into a JSON object.
    fn to_json(&self) -> Value {
        json!({
            "gpu_architecture": self.gpu_arch.as_deref().map(Some).unwrap_or(None),
            "gpu_bus_id": self.gpu_bus_id.as_deref().map(Some).unwrap_or(None),
            "gpu_clock_graphic_MHz": self.gpu_clock_graphic.map(Some).unwrap_or(None),
            "gpu_clock_memory_MHz": self.gpu_clock_memory.map(Some).unwrap_or(None),
            "gpu_clock_sm_MHz": self.gpu_clock_sm.map(Some).unwrap_or(None),
            "gpu_clock_video_MHz": self.gpu_clock_video.map(Some).unwrap_or(None),
            "gpu_energy_consumption_J": self.gpu_energy_consumption.map(Some).unwrap_or(None),
            "gpu_fan_speeds_%": self.gpu_fan_speed.iter().map(|&s| s.unwrap_or(0)).collect::<Vec<u32>>(),
            "gpu_name": self.gpu_name.as_deref().map(Some).unwrap_or(None),
            "gpu_usage_%": self.gpu_usage.map(Some).unwrap_or(None),
            "gpu_temperature_°C": self.gpu_temperature.map(Some).unwrap_or(None),
            "gpu_memory_free_GB": self.gpu_memory_free.map(Some).unwrap_or(None),
            "gpu_memory_usage_%": self.gpu_memory_stat.map(Some).unwrap_or(None),
            "gpu_memory_total_GB": self.gpu_memory_total.map(Some).unwrap_or(None),
            "gpu_memory_usage_GB": self.gpu_memory_usage.map(Some).unwrap_or(None),
            "gpu_pci_data_sent_MB": self.gpu_pci_data_sent.map(Some).unwrap_or(None),
            "gpu_pci_data_received_MB": self.gpu_pci_data_received.map(Some).unwrap_or(None),
            "gpu_power_consumption_W": self.gpu_power_consumption.map(Some).unwrap_or(None),
            "gpu_power_limit_W": self.gpu_power_limit.map(Some).unwrap_or(None),
        })
    }

    /// Collect all global hardware GPU metrics for a given device.
    ///
    /// # Arguments
    ///
    /// - `device` : The detected GPU device.
    ///
    /// # Returns
    ///
    /// Completed fields of [`GpuMetrics`].
    fn from_device(device: &Device) -> Self {
        // Memory and utilization management
        let gpu_memory_info = nvml_try("Failed to get memory info", || device.memory_info()).ok();
        let gpu_utilization = nvml_try("Failed to get utilization rates", || {
            device.utilization_rates()
        })
        .ok();

        // Identifications
        let gpu_arch = nvml_try("Failed to get architecture type", || device.architecture())
            .ok()
            .map(|data| data.to_string());
        let gpu_bus_id = nvml_try("Failed to get GPU PCI bus identification", || {
            device.pci_info()
        })
        .ok()
        .map(|data| data.bus_id.clone());
        let gpu_name = nvml_try("Failed to get GPU name", || device.name()).ok();

        // Existing clock frequencies
        let gpu_clock_graphic = nvml_try("Failed to get graphic clock frequency", || {
            device.clock(Clock::Graphics, ClockId::Current)
        })
        .ok();
        let gpu_clock_memory = nvml_try("Failed to get memory clock frequency", || {
            device.clock(Clock::Memory, ClockId::Current)
        })
        .ok();
        let gpu_clock_sm = nvml_try(
            "Failed to get streaming multiprocessor clock frequency",
            || device.clock(Clock::SM, ClockId::Current),
        )
        .ok();
        let gpu_clock_video = nvml_try("Failed to get video clock frequency", || {
            device.clock(Clock::Video, ClockId::Current)
        })
        .ok();

        // Energy and power consumption
        let gpu_energy_consumption = nvml_try("Failed to get energy consumption", || {
            device.total_energy_consumption()
        })
        .ok()
        .map(|data| data as f32 / 1e3);
        let gpu_power_consumption =
            nvml_try("Failed to get power consumption", || device.power_usage())
                .ok()
                .map(|data| data as f32 / 1e3);
        let gpu_power_limit = nvml_try("Failed to get power management limit", || {
            device.power_management_limit()
        })
        .ok()
        .map(|data| data as f32 / 1e3);

        // Thermal information
        let gpu_fan_speed = (0..nvml_try("Failed to get fan number", || device.num_fans())
            .unwrap_or(0))
            .map(|data| nvml_try("Failed to get fan speed", || device.fan_speed(data)).ok())
            .collect();
        let gpu_temperature = nvml_try("Failed to get temperature(s)", || {
            device.temperature(TemperatureSensor::Gpu)
        })
        .ok();

        // PCIe bus data consumption
        let gpu_pci_data_sent = nvml_try("Failed to get PCI sent data consumed", || {
            device.pcie_throughput(PcieUtilCounter::Send)
        })
        .ok()
        .map(|data| data / 1_000);
        let gpu_pci_data_received = nvml_try("Failed to get PCI received data consumed", || {
            device.pcie_throughput(PcieUtilCounter::Receive)
        })
        .ok()
        .map(|data| data / 1_000);

        // GPU utilization and memory
        let gpu_memory_free = gpu_memory_info.as_ref().map(|m| m.free as f32 / 1e9);
        let gpu_memory_total = gpu_memory_info.as_ref().map(|m| m.total as f32 / 1e9);
        let gpu_memory_usage = gpu_memory_info.as_ref().map(|m| m.used as f32 / 1e9);
        let gpu_memory_stat = gpu_utilization.as_ref().map(|u| u.memory);
        let gpu_usage = gpu_utilization.as_ref().map(|u| u.gpu);

        GpuMetrics {
            gpu_arch,
            gpu_name,
            gpu_bus_id,

            gpu_clock_graphic,
            gpu_clock_memory,
            gpu_clock_sm,
            gpu_clock_video,

            gpu_fan_speed,
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
            gpu_power_limit,
        }
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
    fn from_device(proc: &ProcessUtilizationSample) -> Self {
        GpuProcessMetrics {
            process_pid: Some(proc.pid),
            process_mem: Some(proc.mem_util),
            process_sm: Some(proc.sm_util),
            process_enc: Some(proc.enc_util),
            process_dec: Some(proc.dec_util),
        }
    }

    /// Converts [`GpuProcessMetrics`] into a JSON object.
    fn to_json(&self) -> Value {
        json!({
            "process_pid": self.process_pid.map(Some).unwrap_or(None),
            "process_memory_%": self.process_mem.map(Some).unwrap_or(None),
            "process_sm_%": self.process_sm.map(Some).unwrap_or(None),
            "process_encoder_%": self.process_enc.map(Some).unwrap_or(None),
            "process_decoder_%": self.process_dec.map(Some).unwrap_or(None),
        })
    }
}

/// Collect all GPU process metric for a given device.
fn collect_processes(device: &Device) -> Vec<Value> {
    let mut processes = Vec::new();
    if let Ok(utilization_stats) = nvml_try("Failed to get process utilization", || {
        device.process_utilization_stats(None)
    }) {
        for p in utilization_stats {
            let process = GpuProcessMetrics::from_device(&p);
            processes.push(process.to_json());
        }
    }
    processes
}

/// Retrieves the various NVIDIA GPUs devices on the machine and their associated data.
///
/// # Returns
///
/// - `result` : Completed [`GpuMetrics`] and [`GpuProcessMetrics`] information for GPUs devices detected.
/// - An error when some important and critical metrics can't be retrieved.
fn collect_gpus_data() -> Result<Vec<Value>, Box<dyn Error>> {
    let nvml = nvml_try("Failed to initialize NVML", Nvml::init)?;
    let mut result = Vec::new();

    for index in 0..nvml_try("Failed to get GPU count", || nvml.device_count())? {
        let key = "gpu_".to_owned() + &index.to_string();
        let device = nvml_try("Failed to get device for GPU", || {
            nvml.device_by_index(index)
        })?;

        result.push(json!({
            key: {
                "device": GpuMetrics::from_device(&device).to_json(),
                "process": collect_processes(&device),
            }
        }));
    }

    Ok(result)
}

/// Public function used to send JSON formatted values,
/// from [`collect_gpus_data`] function result.
pub fn get_gpu_info() -> Result<(), Box<dyn Error>> {
    let data = collect_gpus_data()?;
    let values = json!({ HEADER: data });
    write_json_to_file(|| Ok(values), LOGGER)?;
    Ok(())
}
