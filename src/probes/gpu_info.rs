//! # GPU data Module
//!
//! This module provides functionality to retrieve GPU data on Unix-based systems.

use log::error;
use nvml_wrapper::{
    enum_wrappers::device::TemperatureSensor,
    enums::device::DeviceArchitecture,
    error::NvmlError,
    struct_wrappers::device::{MemoryInfo, ProcessUtilizationSample, Utilization},
    Nvml,
};
use serde::Serialize;
use serde_json::{json, Value};
use std::error::Error;

use crate::utils::write_json_to_file;

const HEADER: &str = "GPU";
const LOGGER: &str = "log/gpu_data.json";

/// Collection of collected GPU data
#[derive(Debug, Serialize)]
struct GpuInfo {
    /// GPU architecture.
    gpu_arch: Option<String>,
    /// GPU PCIe bus identification.
    gpu_bus_id: Option<String>,
    /// GPU energy consumption in J.
    gpu_energy_consumption: Option<u64>,
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
    /// GPU electrical consumption in W.
    gpu_power_consumption: Option<u32>,
    /// GPU power limit in W.
    gpu_power_limit: Option<u32>,
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

impl GpuInfo {
    /// Converts `GpuInfo` into a JSON object.
    fn to_json(&self) -> serde_json::Value {
        json!({
            "gpu_architecture": self.gpu_arch.as_deref().map(Some).unwrap_or(None),
            "gpu_bus_id": self.gpu_bus_id.as_deref().map(Some).unwrap_or(None),
            "gpu_energy_consumption_J": self.gpu_energy_consumption.unwrap_or(0) / 1_000,
            "gpu_fan_speeds_%": self.gpu_fan_speed.iter().map(|&speed| speed.unwrap_or(0)).collect::<Vec<u32>>(),
            "gpu_power_consumption_W": self.gpu_power_consumption.unwrap_or(0) / 1_000,
            "gpu_power_limit_W": self.gpu_power_limit.unwrap_or(0) / 1_000,
            "gpu_temperature_°C": self.gpu_temperature.map(Some).unwrap_or(None),
            "gpu_memory_free_GB": self.gpu_memory_free.map(Some).unwrap_or(None),
            "gpu_memory_stat_%": self.gpu_memory_stat.map(Some).unwrap_or(None),
            "gpu_memory_total_GB": self.gpu_memory_total.map(Some).unwrap_or(None),
            "gpu_memory_usage_GB": self.gpu_memory_usage.map(Some).unwrap_or(None),
            "gpu_name": self.gpu_name.as_deref().map(Some).unwrap_or(None),
            "gpu_usage_%": self.gpu_usage.map(Some).unwrap_or(None),
            "process_decoder_%": self.process_dec.map(Some).unwrap_or(None),
            "process_encoder_%": self.process_enc.map(Some).unwrap_or(None),
            "process_memory_%": self.process_mem.map(Some).unwrap_or(None),
            "process_pid": self.process_pid.map(Some).unwrap_or(None),
            "process_sm_%": self.process_sm.map(Some).unwrap_or(None),
        })
    }
}

/// Function that retrieves detailed GPU information.
///
/// # Return
///
/// `GpuInfo` : Completed `GpuInfo` structure with all GPU information:
/// - Number of GPUs.
/// - Name of GPU model.
/// - Power consumption.
/// - Total computing memory.
/// - Used computing memory.
/// - Free computing memory.
/// - GPU usage in percentage.
/// - GPU computing memory usage in percentage.
/// - Speed per fan for graphics card in percentage.
/// - Streaming Multiprocessor, memory, encoder, and decoder utilization in percentage.
fn collect_gpu_data() -> Result<Vec<GpuInfo>, Box<dyn Error>> {
    let nvml: Nvml = Nvml::init().map_err(|e: NvmlError| {
        error!("[{HEADER}] Library 'Failed to initialize NVML' : {e}");
        e
    })?;

    let gpus: u32 = nvml.device_count().map_err(|e: NvmlError| {
        error!("[{HEADER}] Data 'Failed to get GPU count' : {e}");
        e
    })?;

    if gpus == 0 {
        error!("[{HEADER}] Data 'No GPUs detected'");
        return Ok(Vec::new());
    }

    let mut result: Vec<GpuInfo> = Vec::new();

    for index in 0..gpus {
        let device: nvml_wrapper::Device<'_> =
            nvml.device_by_index(index).map_err(|e: NvmlError| {
                error!("[{HEADER}] Data 'Failed to get device for GPU {index}' : {e}");
                e
            })?;

        let mut data: GpuInfo = GpuInfo {
            gpu_bus_id: device.pci_info().ok().map(|pci| pci.bus_id.clone()),
            gpu_arch: device
                .architecture()
                .ok()
                .map(|a: DeviceArchitecture| format!("{:?}", a)),
            gpu_name: device.name().ok(),
            gpu_usage: device.utilization_rates().ok().map(|u: Utilization| u.gpu),

            gpu_fan_speed: (0..device.num_fans().unwrap_or(0))
                .map(|i: u32| device.fan_speed(i).ok())
                .collect(),
            gpu_temperature: device.temperature(TemperatureSensor::Gpu).ok(),

            gpu_energy_consumption: device.total_energy_consumption().ok(),
            gpu_power_consumption: device.power_usage().ok(),
            gpu_power_limit: device.power_management_limit().ok(),

            gpu_memory_free: device
                .memory_info()
                .ok()
                .map(|m: MemoryInfo| m.free as f32 / 1e9),
            gpu_memory_stat: device
                .utilization_rates()
                .ok()
                .map(|u: Utilization| u.memory),
            gpu_memory_total: device
                .memory_info()
                .ok()
                .map(|m: MemoryInfo| m.total as f32 / 1e9),
            gpu_memory_usage: device
                .memory_info()
                .ok()
                .map(|m: MemoryInfo| m.used as f32 / 1e9),

            process_dec: None,
            process_enc: None,
            process_mem: None,
            process_pid: None,
            process_sm: None,
        };

        if let Ok(utilization) = device.process_utilization_stats(None) {
            data.process_dec = Some(
                utilization
                    .iter()
                    .map(|p: &ProcessUtilizationSample| p.dec_util)
                    .sum(),
            );
            data.process_enc = Some(
                utilization
                    .iter()
                    .map(|p: &ProcessUtilizationSample| p.enc_util)
                    .sum(),
            );
            data.process_mem = Some(
                utilization
                    .iter()
                    .map(|p: &ProcessUtilizationSample| p.mem_util)
                    .sum(),
            );
            data.process_pid = Some(
                utilization
                    .iter()
                    .map(|p: &ProcessUtilizationSample| p.pid)
                    .sum(),
            );
            data.process_sm = Some(
                utilization
                    .iter()
                    .map(|p: &ProcessUtilizationSample| p.sm_util)
                    .sum(),
            );

            if data.process_dec.is_none()
                || data.process_enc.is_none()
                || data.process_mem.is_none()
                || data.process_pid.is_none()
                || data.process_sm.is_none()
            {
                error!("[{HEADER}] Data 'Failed to get process utilization stats for GPU {index}'");
            }
        } else {
            error!("[{HEADER}] Data 'Failed to get process utilization stats for GPU {index}'");
        }

        if data.gpu_name.is_none() {
            error!("[{HEADER}] Data 'Failed to get name for GPU {index}'");
        }
        if data.gpu_usage.is_none() {
            error!("[{HEADER}] Data 'Failed to get utilization rate for GPU {index}");
        }
        if data.gpu_temperature.is_none() {
            error!("[{HEADER}] Data 'Failed to get temperature for GPU {index}'");
        }
        if data.gpu_power_consumption.is_none() {
            error!("[{HEADER}] Data 'Failed to get power consumption for GPU {index}'");
        }
        if data.gpu_memory_total.is_none()
            || data.gpu_memory_usage.is_none()
            || data.gpu_memory_free.is_none()
        {
            error!("[{HEADER}] Data 'Failed to get memory info for GPU {index}'");
        }

        result.push(data);
    }

    Ok(result)
}

/// Public function used to send JSON formatted values,
/// from `collect_gpu_data` function result.
pub fn get_gpu_info() {
    let data = || -> Result<Value, Box<dyn Error>> {
        let values: Vec<Value> = collect_gpu_data()?
            .iter()
            .map(|item: &GpuInfo| json!({ HEADER: item.to_json() }))
            .collect::<Vec<_>>();

        Ok(json!(values))
    };

    write_json_to_file(data, LOGGER, HEADER);
}
