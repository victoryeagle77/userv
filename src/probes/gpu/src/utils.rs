//! # File utilities module

use core::core::measure_point;
use std::time::Duration;

use log::error;
use nvml_wrapper::{
    Device,
    enum_wrappers::device::{Clock, ClockId, PcieUtilCounter, TemperatureSensor},
    error::NvmlError,
    struct_wrappers::device::ProcessUtilizationSample,
};
use rusqlite::{Connection, params};
use serde::Serialize;

pub const HEADER: &str = "GPU";

// Collection of collected GPU data.
#[derive(Serialize)]
pub struct GpuMetrics {
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
    gpu_energy_consumption: Option<f64>,
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
    /// GPU electrical consumption in W.
    gpu_power_consumption: Option<f32>,
    /// GPU maximum electrical consumption accepted in W.
    gpu_power_limit: Option<f32>,
}

/// Collection of collected running processes GPU data.
#[derive(Serialize)]
pub struct GpuProcessMetrics {
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

/// Helper for NVML error handling.
pub fn nvml_try<T, F>(context: &'static str, f: F) -> Result<T, NvmlError>
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
    pub fn from_device(device: &Device) -> Self {
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
        let gpu_energy_consumption = measure_point(
            || {
                nvml_try("Failed to get energy consumption", || {
                    device.total_energy_consumption()
                })
                .ok()
                .map(|data| data as f64 / 1e3)
            },
            Duration::from_millis(100),
        );
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

    /// Insert GPU parameters in database.
    ///
    /// # Arguments
    ///
    /// - `conn` : Allow by a [`Connection`] constructor type the connection with an SQLite database.
    /// - `timestamp` : Date trace for the history identification.
    /// - `data` : [`GpuMetrics`] information to insert in database.
    ///
    /// # Returns
    ///
    /// - Insert the [`GpuMetrics`] filled structure in an SQLite database.
    /// - Logs an error if the SQL insert request failed.
    pub fn insert_db(conn: &Connection, timestamp: &str, data: &Self) -> rusqlite::Result<i64> {
        conn.execute(
            "INSERT INTO gpu_data (
                timestamp,
                architecture,
                bus_id,
                clock_graphic_MHz,
                clock_memory_MHz,
                clock_sm_MHz,
                clock_video_MHz,
                energy_consumption_J,
                fan_speed,
                name,
                usage,
                temperature_C,
                memory_free_GB,
                memory_stat,
                memory_total_GB,
                memory_usage,
                pci_data_sent_KBs,
                pci_data_received_KBs,
                power_consumption_W,
                power_limit_W
            ) VALUES (
                ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11,
                ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20
            )",
            params![
                timestamp,
                data.gpu_arch,
                data.gpu_bus_id,
                data.gpu_clock_graphic,
                data.gpu_clock_memory,
                data.gpu_clock_sm,
                data.gpu_clock_video,
                data.gpu_energy_consumption,
                serde_json::to_string(&data.gpu_fan_speed).ok(),
                data.gpu_name,
                data.gpu_usage,
                data.gpu_temperature,
                data.gpu_memory_free,
                data.gpu_memory_stat,
                data.gpu_memory_total,
                data.gpu_memory_usage,
                data.gpu_pci_data_sent,
                data.gpu_pci_data_received,
                data.gpu_power_consumption,
                data.gpu_power_limit,
            ],
        )?;
        Ok(conn.last_insert_rowid())
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
    pub fn from_device(proc: &ProcessUtilizationSample) -> Self {
        GpuProcessMetrics {
            process_pid: Some(proc.pid),
            process_dec: Some(proc.dec_util),
            process_enc: Some(proc.enc_util),
            process_mem: Some(proc.mem_util),
            process_sm: Some(proc.sm_util),
        }
    }

    /// Insert GPU processes parameters in database.
    ///
    /// # Arguments
    ///
    /// - `conn` : Allow by a [`Connection`] constructor type the connection with an SQLite database.
    /// - `timestamp` : Date trace for the history identification.
    /// - `data` : [`GpuProcessMetrics`] information to insert in database.
    ///
    /// # Returns
    ///
    /// - Insert the [`GpuProcessMetrics`] filled structure in an SQLite database.
    /// - Logs an error if the SQL insert request failed.
    pub fn insert_db(conn: &Connection, id: i64, data: &Self) -> rusqlite::Result<()> {
        conn.execute(
            "INSERT INTO gpu_process_data (
                indexation,
                pid,
                decoding,
                encoding,
                memory,
                streaming_multiprocessor
            ) VALUES (
                ?1, ?2, ?3, ?4, ?5, ?6
            )",
            params![
                id,
                data.process_pid,
                data.process_dec,
                data.process_enc,
                data.process_mem,
                data.process_sm,
            ],
        )?;
        Ok(())
    }
}
