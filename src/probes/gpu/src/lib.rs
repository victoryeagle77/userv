//! # Lib file for GPU data module
//!
//! This module provides functionalities to retrieve GPU data on Unix-based systems.

use chrono::{SecondsFormat, Utc};
use nvml_wrapper::Nvml;
use std::error::Error;

mod utils;
use core::core::init_db;
use utils::*;

const REQUEST: &str = "
    CREATE TABLE IF NOT EXISTS gpu_data (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        timestamp TEXT NOT NULL,
        architecture TEXT,
        bus_id TEXT NOT NULL,
        clock_graphic_MHz INTEGER,
        clock_memory_MHz INTEGER,
        clock_sm_MHz INTEGER,
        clock_video_MHz INTEGER,
        energy_consumption_J REAL,
        fan_speed TEXT,
        name TEXT,
        usage INTEGER,
        temperature_C INTEGER,
        memory_free_GB REAL,
        memory_stat INTEGER,
        memory_total_GB REAL,
        memory_usage REAL,
        pci_data_sent_KBs INTEGER,
        pci_data_received_KBs INTEGER,
        power_consumption_W REAL,
        power_limit_W REAL
    );
    CREATE TABLE IF NOT EXISTS gpu_process_data (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        indexation INTEGER NOT NULL,
        pid INTEGER NOT NULL,
        decoding INTEGER,
        encoding INTEGER,
        memory INTEGER,
        streaming_multiprocessor INTEGER,
        FOREIGN KEY(indexation) REFERENCES gpu_data(id)
        FOREIGN KEY(pid) REFERENCES gpu_data(bus_id)
    );
    ";

/// Public function used to send values in SQLite database.
/// Retrieves the various NVIDIA GPUs devices on the machine and their associated data.
///
/// # Returns
///
/// - `result` : Completed [`GpuMetrics`] and [`GpuProcessMetrics`] information for GPUs devices detected.
/// - An error when some important and critical metrics can't be retrieved.
pub fn get_gpu_info() -> Result<(), Box<dyn Error>> {
    let timestamp = Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true);
    let conn = init_db(REQUEST)?;

    let nvml = nvml_try("Failed to initialize NVML", Nvml::init)?;

    for index in 0..nvml_try("Failed to get GPU count", || nvml.device_count())? {
        let device = nvml_try("Failed to get device for GPU", || {
            nvml.device_by_index(index)
        })?;
        let metrics_gpu = GpuMetrics::from_device(&device);
        let id = GpuMetrics::insert_db(&conn, &timestamp, &metrics_gpu)?;

        if let Ok(utilization_stats) = nvml_try("Failed to get process utilization", || {
            device.process_utilization_stats(None)
        }) {
            for p in utilization_stats {
                let metrics_proc = GpuProcessMetrics::from_device(&p);
                GpuProcessMetrics::insert_db(&conn, id, &metrics_proc)?;
            }
        }
    }
    Ok(())
}
