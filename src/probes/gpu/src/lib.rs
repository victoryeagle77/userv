//! # Lib file for GPU data module
//!
//! This module provides functionalities to retrieve GPU data on Unix-based systems.

use chrono::{SecondsFormat, Utc};
use log::error;
use nvml_wrapper::Nvml;
use rusqlite::{Connection, params};
use std::error::Error;

mod dbms;
mod utils;

use core::core::{db_insert_query, db_table_query_creation, init_db};
use dbms::*;
use utils::*;

const HEADER: &str = "GPU";

impl GpuMetrics {
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
    pub fn insert_db(
        conn: &Connection,
        timestamp: &str,
        data: &Self,
    ) -> Result<(), Box<dyn Error>> {
        let query = db_insert_query(TABLE_NAME[0], &field_descriptor_gpu())?;
        conn.execute(
            &query,
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
        Ok(())
    }
}

impl GpuProcessMetrics {
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
    pub fn insert_db(
        conn: &Connection,
        timestamp: &str,
        data: &Self,
    ) -> Result<(), Box<dyn Error>> {
        let query = db_insert_query(TABLE_NAME[1], &field_descriptor_process())?;
        conn.execute(
            &query,
            params![
                timestamp,
                data.gpu_bus_id,
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

/// Public function used to send values in SQLite database.
/// Retrieves the various NVIDIA GPUs devices on the machine and their associated data.
///
/// # Returns
///
/// - `result` : Completed [`GpuMetrics`] and [`GpuProcessMetrics`] information for GPUs devices detected.
/// - An error when some important and critical metrics can't be retrieved.
pub fn get_gpu_info() -> Result<(), Box<dyn Error>> {
    let nvml = Nvml::init()?;

    let query_gpu = db_table_query_creation(TABLE_NAME[0], &field_descriptor_gpu())?;
    let query_process = db_table_query_creation(TABLE_NAME[1], &field_descriptor_process())?;

    let conn = init_db(&query_gpu)?;
    conn.execute_batch(&query_process)?;

    for index in 0..nvml.device_count()? {
        let device = nvml.device_by_index(index)?;
        let gpu_bus_id = device.pci_info()?.bus_id.clone();

        let metrics_gpu = GpuMetrics::from_device(&device, gpu_bus_id.clone())?;
        let timestamp = Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true);

        GpuMetrics::insert_db(&conn, &timestamp, &metrics_gpu)?;

        match device.process_utilization_stats(None) {
            Ok(utilization_stats) => {
                for p in utilization_stats {
                    let metrics_process = GpuProcessMetrics::from_device(&p, gpu_bus_id.clone());
                    GpuProcessMetrics::insert_db(&conn, &timestamp, &metrics_process)?;
                }
            }
            Err(e) => {
                error!(
                    "[{HEADER}] Data : 'Impossible to get process utilization for {index} : {e}'"
                );
            }
        }
    }

    Ok(())
}
