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
    /// Decoder utilization in percentage.
    dec_util: Option<u32>,
    /// Encoder utilization in percentage.
    enc_util: Option<u32>,
    /// GPU energy consumption in mJ.
    energy_conso: Option<u64>,
    /// Speed per fan in percentage.
    fan_speed: Vec<Option<u32>>,
    /// GPU architecture.
    gpu_arch: Option<String>,
    /// GPU model name.
    gpu_name: Option<String>,
    /// GPU number.
    gpu_num: u32,
    /// GPU usage in percentage.
    gpu_use: Option<u32>,
    /// GPU temperature in °C.
    gpu_tmp: Option<u32>,
    /// Free available computing memory in Bytes.
    mem_free: Option<f32>,
    /// GPU computing memory usage in percentage.
    mem_stat: Option<u32>,
    /// Total GPU computing memory in Bytes.
    mem_tot: Option<f32>,
    /// Currently used computing memory in Bytes.
    mem_use: Option<f32>,
    /// Memory utilization in percentage.
    mem_util: Option<u32>,
    /// GPU electrical consumption in mW.
    power_conso: Option<u32>,
    /// GPU power limit in mW.
    power_limit: Option<u32>,
    /// Streaming Multiprocessor utilization in percentage.
    sm_util: Option<u32>,
}

impl GpuInfo {
    /// Converts `GpuInfo` into a JSON object.
    fn to_json(&self) -> serde_json::Value {
        json!({
            "gpu_architecture": self.gpu_arch.as_deref().map(Some).unwrap_or(None),
            "gpu_number": self.gpu_num,
            "gpu_name": self.gpu_name.as_deref().map(Some).unwrap_or(None),
            "gpu_energy_consumption_J": self.energy_conso.unwrap_or(0) / 1_000,
            "gpu_power_consumption_W": self.power_conso.unwrap_or(0) as f64 / 1_000.0,
            "gpu_power_limit_W": self.power_limit.unwrap_or(0) as f64 / 1_000.0,
            "gpu_memory_total_GB": self.mem_tot.map(Some).unwrap_or(None),
            "gpu_memory_used_GB": self.mem_use.map(Some).unwrap_or(None),
            "gpu_memory_free_GB": self.mem_free.map(Some).unwrap_or(None),
            "gpu_used_%": self.gpu_use.map(Some).unwrap_or(None),
            "gpu_memory_stat_%": self.mem_stat.map(Some).unwrap_or(None),
            "gpu_fan_speeds_%": self.fan_speed.iter().map(|&speed| speed.unwrap_or(0)).collect::<Vec<u32>>(),
            "gpu_temperature_gpu_°C": self.gpu_tmp.map(Some).unwrap_or(None),
            "gpu_sm_util_%": self.sm_util.map(Some).unwrap_or(None),
            "gpu_memory_util_%": self.mem_util.map(Some).unwrap_or(None),
            "gpu_encoder_util_%": self.enc_util.map(Some).unwrap_or(None),
            "gpu_decoder_util_%": self.dec_util.map(Some).unwrap_or(None),
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
            gpu_arch: device
                .architecture()
                .ok()
                .map(|a: DeviceArchitecture| format!("{:?}", a)),
            gpu_num: index,
            gpu_name: device.name().ok(),
            gpu_use: device.utilization_rates().ok().map(|u: Utilization| u.gpu),
            gpu_tmp: device.temperature(TemperatureSensor::Gpu).ok(),
            power_conso: device.power_usage().ok(),
            power_limit: device.power_management_limit().ok(),
            energy_conso: device.total_energy_consumption().ok(),
            mem_tot: device
                .memory_info()
                .ok()
                .map(|m: MemoryInfo| m.total as f32 / 1e9),
            mem_use: device
                .memory_info()
                .ok()
                .map(|m: MemoryInfo| m.used as f32 / 1e9),
            mem_free: device
                .memory_info()
                .ok()
                .map(|m: MemoryInfo| m.free as f32 / 1e9),
            mem_stat: device
                .utilization_rates()
                .ok()
                .map(|u: Utilization| u.memory),
            fan_speed: (0..device.num_fans().unwrap_or(0))
                .map(|i| device.fan_speed(i).ok())
                .collect(),
            sm_util: None,
            mem_util: None,
            enc_util: None,
            dec_util: None,
        };

        if let Ok(utilization) = device.process_utilization_stats(None) {
            data.sm_util = Some(
                utilization
                    .iter()
                    .map(|p: &ProcessUtilizationSample| p.sm_util)
                    .sum(),
            );
            data.mem_util = Some(
                utilization
                    .iter()
                    .map(|p: &ProcessUtilizationSample| p.mem_util)
                    .sum(),
            );
            data.enc_util = Some(
                utilization
                    .iter()
                    .map(|p: &ProcessUtilizationSample| p.enc_util)
                    .sum(),
            );
            data.dec_util = Some(
                utilization
                    .iter()
                    .map(|p: &ProcessUtilizationSample| p.dec_util)
                    .sum(),
            );
        } else {
            error!("[{HEADER}] Data 'Failed to get process utilization stats for GPU {index}'");
        }

        if data.gpu_name.is_none() {
            error!("[{HEADER}] Data 'Failed to get name for GPU {index}'");
        }
        if data.gpu_use.is_none() {
            error!("[{HEADER}] Data 'Failed to get utilization rate for GPU {index}");
        }
        if data.gpu_tmp.is_none() {
            error!("[{HEADER}] Data 'Failed to get temperature for GPU {index}'");
        }
        if data.power_conso.is_none() {
            error!("[{HEADER}] Data 'Failed to get power consumption for GPU {index}'");
        }
        if data.mem_tot.is_none() || data.mem_use.is_none() || data.mem_free.is_none() {
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
