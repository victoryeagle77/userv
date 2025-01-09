//! # GPU data Module
//!
//! This module provides functionality to retrieve GPU data on Unix-based systems.

use chrono::Utc;
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
    /// GPU number.
    gpu_num: u32,
    /// GPU model name.
    gpu_nme: Option<String>,
    /// GPU electrical consumption in mW.
    pwr_erg: Option<f32>,
    /// Total GPU computing memory in GB.
    mem_tot: Option<f32>,
    /// Currently used computing memory in GB.
    mem_use: Option<f32>,
    /// Free available computing memory in GB.
    mem_fre: Option<f32>,
    /// GPU usage in percentage.
    use_gpu: Option<u32>,
    /// GPU computing memory usage in percentage.
    use_mem: Option<u32>,
    /// Speed per fan.
    fan_spd: Vec<Option<u32>>,
    /// GPU temperature.
    tmp_gpu: Option<u32>,
}

/// Function that retrieves detailed GPU information.
///
/// # Return
///
/// ``CpuInfo`` : Completed `GpuInfo` structure with all gpu information
/// - Number of GPU
/// - Name of GPU model
/// - GPU power consumption
/// - Total computing memory in mW
/// - Used computing memory
/// - Free computing memory
/// - GPU usage in percentage
/// - GPU computing memory usage in percentage
/// - Speed per fan for graphics card
fn collect_gpu_data() -> Result<Vec<GpuInfo>, Box<dyn std::error::Error>> {
    let nvml = Nvml::init()?;
    let gpus = nvml.device_count()?;

    let mut result = Vec::new();

    for index in 0..gpus {
        let device = nvml.device_by_index(index)?;

        let name = device.name().map(|n| n.to_string()).ok();
        let power = device.power_usage().ok();
        let memory = device.memory_info()?;
        let usage = device.utilization_rates()?;
        let temperature = device.temperature(TemperatureSensor::Gpu).ok();

        let mut fan_speeds = Vec::new();
        for i in 0..device.num_fans()? {
            let fan_speed = device.fan_speed(i).ok();
            fan_speeds.push(fan_speed);
        }

        let data = GpuInfo {
            gpu_num: index,
            gpu_nme: name,
            pwr_erg: power.map(|p| format_unit(p.into())),
            mem_tot: Some(format_unit(memory.total)),
            mem_use: Some(format_unit(memory.used)),
            mem_fre: Some(format_unit(memory.free)),
            use_gpu: Some(usage.gpu),
            use_mem: Some(usage.memory),
            fan_spd: fan_speeds,
            tmp_gpu: temperature,
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
            let data: Vec<serde_json::Value> = values
                .iter()
                .map(|item| {
                    json!({
                        HEADER: {
                            "timestamp": Utc::now().to_rfc3339(),
                            "gpu_num": item.gpu_num,
                            "gpu_nme": item.gpu_nme.as_ref().unwrap_or(&"NULL".to_string()),
                            "pwr_erg": item.pwr_erg.unwrap_or(0.0),
                            "mem_tot": item.mem_tot.unwrap_or(0.0),
                            "mem_use": item.mem_use.unwrap_or(0.0),
                            "mem_fre": item.mem_fre.unwrap_or(0.0),
                            "use_gpu": item.use_gpu.unwrap_or(0),
                            "use_mem": item.use_mem.unwrap_or(0),
                            "fan_spd": item.fan_spd.iter().map(|&speed| speed.unwrap_or(0)).collect::<Vec<u32>>(),
                            "tmp_gpu": item.tmp_gpu.unwrap_or(0),
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
