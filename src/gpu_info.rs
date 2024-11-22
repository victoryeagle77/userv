use nvml_wrapper::Nvml;
use serde::{Serialize, Deserialize};

use crate::utils::format_unit;

/// Collection of collected GPU data
#[derive(Serialize, Deserialize)]
struct GpuInfo {
    /// GPU model name.
    gpu_nme: String,
    /// GPU electrical consumption in mW.
    pwr_erg: f32,
    /// Total GPU computing memory.
    mem_tot: f32,
    /// Currently used computing memory.
    mem_use: f32,
    /// Free available computing memory.
    mem_fre: f32,
    /// GPU usage in percentage.
    use_gpu: u32,
    /// GPU Memory usage in percentage.
    use_mem: u32,
    /// Speed per fan.
    fan_spd: Vec<u32>,
}

pub fn get_gpu_info() -> Result<(), Box<dyn std::error::Error>> {
    let nvml = Nvml::init()?;
    let device = nvml.device_by_index(0)?;
    let name = device.name()?;

    let power = device.power_usage()?;
    let memory = device.memory_info()?;
    let usage = device.utilization_rates()?;

    let mut fan_speeds = Vec::new();
    for i in 0..device.num_fans()? {
        let fan_speed = device.fan_speed(i)?;
        fan_speeds.push(fan_speed);
    }

    let gpu_info = GpuInfo {
        gpu_nme: name.to_string(),
        pwr_erg: format_unit(power.into()),
        mem_tot: format_unit(memory.total),
        mem_use: format_unit(memory.used),
        mem_fre: format_unit(memory.free),
        use_gpu: usage.gpu,
        use_mem: usage.memory,
        fan_spd: fan_speeds,
    };

    println!("\n[ GPU ]\n");
    println!("{}", serde_json::to_string_pretty(&gpu_info)?);

    Ok(())
}