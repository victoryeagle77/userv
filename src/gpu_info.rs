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
    /// GPU architecture.
    gpu_arch: Option<String>,
    /// GPU number.
    gpu_num: u32,
    /// GPU brand identification.
    gpu_brand: Option<String>,
    /// GPU model name.
    gpu_name: Option<String>,
    /// GPU usage in percentage.
    gpu_use: Option<u32>,
    /// GPU temperature in °C.
    gpu_tmp: Option<u32>,
    /// GPU electrical consumption in mW.
    pwr_energy: Option<f32>,
    /// GPU power limit in mW.
    pwr_limit: Option<f32>,
    /// Total GPU computing memory in GB.
    mem_tot: Option<f32>,
    /// Currently used computing memory in GB.
    mem_use: Option<f32>,
    /// Free available computing memory in GB.
    mem_free: Option<f32>,
    /// GPU computing memory usage in percentage.
    mem_stat: Option<u32>,
    /// Speed per fan.
    fan_speed: Vec<Option<u32>>,
    /// Streaming Multiprocessor utilization in percentage.
    sm_util: Option<u32>,
    /// Memory utilization in percentage.
    mem_util: Option<u32>,
    /// Encoder utilization in percentage.
    enc_util: Option<u32>,
    /// Decoder utilization in percentage.
    dec_util: Option<u32>,
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
/// - SM, memory, encoder, and decoder utilization percentages
fn collect_gpu_data() -> Result<Vec<GpuInfo>, Box<dyn std::error::Error>> {
    let nvml = Nvml::init()?;
    let gpus = nvml.device_count()?;

    let mut result = Vec::new();

    for index in 0..gpus {
        let device = nvml.device_by_index(index)?;

        let arch = device.architecture().ok().map(|a| format!("{:?}", a));
        let brand = device.brand().map(|b| format!("{:?}", b)).ok();
        let name = device.name().map(|n| n.to_string()).ok();
        let power = device.power_usage().ok();
        let limit = device.power_management_limit().ok();
        let memory = device.memory_info()?;
        let usage = device.utilization_rates()?;
        let temperature = device.temperature(TemperatureSensor::Gpu).ok();

        let mut fan_speeds = Vec::new();
        for i in 0..device.num_fans()? {
            let x = device.fan_speed(i).ok();
            fan_speeds.push(x);
        }

        let utilization = device.process_utilization_stats(None)?;
        let sm_util = utilization.iter().map(|p| p.sm_util).sum();
        let mem_util = utilization.iter().map(|p| p.mem_util).sum();
        let enc_util = utilization.iter().map(|p| p.enc_util).sum();
        let dec_util = utilization.iter().map(|p| p.dec_util).sum();

        let data = GpuInfo {
            gpu_arch: arch,
            gpu_num: index,
            gpu_brand: brand,
            gpu_name: name,
            gpu_use: Some(usage.gpu),
            pwr_energy: power.map(|p| format_unit(p.into())),
            pwr_limit: limit.map(|p| format_unit(p.into())),
            mem_tot: Some(format_unit(memory.total)),
            mem_use: Some(format_unit(memory.used)),
            mem_free: Some(format_unit(memory.free)),
            mem_stat: Some(usage.memory),
            fan_speed: fan_speeds,
            gpu_tmp: temperature,
            sm_util: Some(sm_util),
            mem_util: Some(mem_util),
            enc_util: Some(enc_util),
            dec_util: Some(dec_util),
        };

        result.push(data);
    }

    Ok(result)
}

/// Public function used to send JSON formatted values,
/// from `collect_gpu_data` function result.
pub fn get_gpu_info() {
    match collect_gpu_data() {
        Ok(values) => {
            let timestamp = Some(Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true))
                .map_or_else(|| None, |ts| Some(ts));
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
                            "power_energy": item.pwr_energy.unwrap_or(0.0),
                            "power_limit": item.pwr_limit.unwrap_or(0.0),
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

            if let Err(e) = write_json_to_file(data, LOGGER) {
                error!("[{}] {}", HEADER, e);
            }
        }
        Err(e) => {
            error!("[{}] {}", HEADER, e);
        }
    }
}
