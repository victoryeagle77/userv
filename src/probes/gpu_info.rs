//! # GPU data Module
//!
//! This module provides functionality to retrieve GPU data on Unix-based systems.

use chrono::{SecondsFormat, Utc};
use log::error;
use nvml_wrapper::{enum_wrappers::device::TemperatureSensor, Nvml};
use serde::Serialize;
use serde_json::json;

use crate::utils::{format_unit, write_json_to_file};

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
    energy_conso: Option<f32>,
    /// Speed per fan.
    fan_speed: Vec<Option<u32>>,
    /// GPU architecture.
    gpu_arch: Option<String>,
    /// GPU brand identification.
    gpu_brand: Option<String>,
    /// GPU model name.
    gpu_name: Option<String>,
    /// GPU number.
    gpu_num: u32,
    /// GPU usage in percentage.
    gpu_use: Option<u32>,
    /// GPU temperature in °C.
    gpu_tmp: Option<u32>,
    /// Free available computing memory in GB.
    mem_free: Option<f32>,
    /// GPU computing memory usage in percentage.
    mem_stat: Option<u32>,
    /// Total GPU computing memory in GB.
    mem_tot: Option<f32>,
    /// Currently used computing memory in GB.
    mem_use: Option<f32>,
    /// Memory utilization in percentage.
    mem_util: Option<u32>,
    /// GPU electrical consumption in mW.
    power_conso: Option<f32>,
    /// GPU power limit in mW.
    power_limit: Option<f32>,
    /// Streaming Multiprocessor utilization in percentage.
    sm_util: Option<u32>,
}

/// Function that retrieves detailed GPU information.
///
/// # Return
///
/// `GpuInfo` : Completed `GpuInfo` structure with all GPU information:
/// - Number of GPUs
/// - Name of GPU model
/// - Power consumption
/// - Total computing memory
/// - Used computing memory
/// - Free computing memory
/// - GPU usage in percentage
/// - GPU computing memory usage in percentage
/// - Speed per fan for graphics card
/// - Streaming Multiprocessor, memory, encoder, and decoder utilization percentages
fn collect_gpu_data() -> Result<Vec<GpuInfo>, Box<dyn std::error::Error>> {
    let nvml = Nvml::init().map_err(|e| {
        error!("[{HEADER}] Failed to initialize NVML : {e}");
        e
    })?;

    let gpus = nvml.device_count().map_err(|e| {
        error!("[{HEADER}] Failed to get GPU count : {e}");
        e
    })?;

    if gpus == 0 {
        error!("[{HEADER}] No GPUs detected");
        return Ok(Vec::new());
    }

    let mut result = Vec::new();

    for index in 0..gpus {
        let device = nvml.device_by_index(index)?;

        let mut data = GpuInfo {
            gpu_arch: device.architecture().ok().map(|a| format!("{:?}", a)),
            gpu_brand: device.brand().ok().map(|b| format!("{:?}", b)),
            gpu_num: index,
            gpu_name: device.name().ok().map(String::from),
            gpu_use: device.utilization_rates().ok().map(|u| u.gpu),
            gpu_tmp: device.temperature(TemperatureSensor::Gpu).ok(),
            power_conso: device.power_usage().ok().map(|p| format_unit(p.into())),
            power_limit: device
                .power_management_limit()
                .ok()
                .map(|p| format_unit(p.into())),
            energy_conso: device.total_energy_consumption().ok().map(format_unit),
            mem_tot: device.memory_info().ok().map(|m| format_unit(m.total)),
            mem_use: device.memory_info().ok().map(|m| format_unit(m.used)),
            mem_free: device.memory_info().ok().map(|m| format_unit(m.free)),
            mem_stat: device.utilization_rates().ok().map(|u| u.memory),
            fan_speed: (0..device.num_fans().unwrap_or(0))
                .map(|i| device.fan_speed(i).ok())
                .collect(),
            sm_util: None,
            mem_util: None,
            enc_util: None,
            dec_util: None,
        };

        if let Ok(utilization) = device.process_utilization_stats(None) {
            data.sm_util = Some(utilization.iter().map(|p| p.sm_util).sum());
            data.mem_util = Some(utilization.iter().map(|p| p.mem_util).sum());
            data.enc_util = Some(utilization.iter().map(|p| p.enc_util).sum());
            data.dec_util = Some(utilization.iter().map(|p| p.dec_util).sum());
        }

        result.push(data);
    }

    Ok(result)
}

/// Public function used to send JSON formatted values,
/// from `collect_gpu_data` function result.
pub fn get_gpu_info() {
    let data_generator = || -> Result<serde_json::Value, Box<dyn std::error::Error>> {
        let values = collect_gpu_data()?;
        let timestamp = Some(Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true))
            .map_or_else(|| None, Some);

        let data: Vec<serde_json::Value> = values
            .iter()
            .map(|item| {
                json!({
                    HEADER: {
                        "timestamp": timestamp,
                        "gpu_architecture": item.gpu_arch.as_ref().unwrap_or(&"NULL".to_string()),
                        "gpu_number": item.gpu_num,
                        "gpu_brand": item.gpu_brand.as_ref().unwrap_or(&"NULL".to_string()),
                        "gpu_name": item.gpu_name.as_ref().unwrap_or(&"NULL".to_string()),
                        "energy_consumption": item.energy_conso.unwrap_or(0.0),
                        "power_consumption": item.power_conso.unwrap_or(0.0),
                        "power_limit": item.power_limit.unwrap_or(0.0),
                        "memory_total": item.mem_tot.unwrap_or(0.0),
                        "memory_used": item.mem_use.unwrap_or(0.0),
                        "memory_free": item.mem_free.unwrap_or(0.0),
                        "gpu_used": item.gpu_use.unwrap_or(0),
                        "memory_stat": item.mem_stat.unwrap_or(0),
                        "fan_speeds": item.fan_speed.iter().map(|&speed| speed.unwrap_or(0)).collect::<Vec<u32>>(),
                        "temperature_gpu": item.gpu_tmp.unwrap_or(0),
                        "sm_util": item.sm_util.unwrap_or(0),
                        "memory_util": item.mem_util.unwrap_or(0),
                        "encoder_util": item.enc_util.unwrap_or(0),
                        "decoder_util": item.dec_util.unwrap_or(0),
                    }
                })
            })
            .collect();

        Ok(json!(data))
    };

    write_json_to_file(data_generator, LOGGER, HEADER);
}
