//! # File utilities module
//!
//! This module provides functionalities to get specific data concerning RAM and SWAP memories on Unix-based systems.

use chrono::{SecondsFormat::Millis, Utc};
use log::error;
use serde_json::{json, Value};
use std::{
    error::Error,
    fs::OpenOptions,
    io::Write,
    process::Command,
    ptr::{read_volatile, write_volatile},
    time::{Duration, Instant},
};

pub const HEADER: &'static str = "RAM";
pub const LOGGER: &'static str = "log/ram_data.json";

const ARRAY_SIZE: &'static usize = &1_000_000_000;
pub const FACTOR: &'static u64 = &1_000_000;

/// Typical power consumption per GB for each RAM type, based on voltage specifications and average module datasheets.
///
/// # Sources
///
/// - [Wikipedia - SDRAM](https://en.wikipedia.org/wiki/Synchronous_dynamic_random-access_memory)
/// - [Crucial - DDR vs DDR2 vs DDR3 vs DDR4](https://www.crucial.fr/articles/about-memory/difference-between-ddr2-ddr3-ddr4)
/// - [Kingston - DDR2 vs DDR3](https://www.kingston.com/fr/blog/pc-performance/ddr2-vs-ddr3)
/// - [Crucial - DDR3 Power Consumption](https://www.crucial.com/articles/about-memory/power-consumption-of-ddr3)
/// - [FS.com - DDR3 vs DDR4 vs DDR5](https://community.fs.com/blog/ddr3-vs-ddr4-vs-ddr5.html)
/// - [Tom's Hardware - DDR5 vs DDR4 Power](https://www.tomshardware.com/news/ddr5-vs-ddr4-ram)
/// - [Micron - LPDDR2/LPDDR3 Power](https://www.micron.com/products/dram/lpdram)
/// - [Logic-fruit - DDR3 vs DDR4 vs LPDDR4](https://www.logic-fruit.com/blogs/ddr3-vs-ddr4-vs-lpddr4/)
/// - [Samsung - LPDDR5 Whitepaper](https://semiconductor.samsung.com/resources/white-paper/5th-generation-lpddr5/)
/// - [Micron - eMMC Power Consumption](https://media-www.micron.com/-/media/client/global/documents/products/technical-note/nand-flash/tn2961_emmc_power_consumption.pdf)
/// - [Kiatoo - DDR2/DDR3/DDR4/DDR5 Comparison (fr)](https://www.kiatoo.com/blog/ddr2-vs-ddr3-vs-ddr4-vs-ddr5/)
/// - [Granite River Labs - Overview DDR Standards](https://graniteriverlabs.com/technology/ddr/)
/// - [Reddit - Power consumption of RAM modules](https://www.reddit.com/r/buildapc/comments/7w3m2g/ram_power_consumption/)
///
/// Values are indicative and may vary depending on manufacturer, frequency, and module density.
///
/// | Type     | Voltage   | Typical for 8GB | W/GB |
/// |----------|-----------|-----------------|------|
/// | SDRAM    | 3.3V      | 5.5W            | 0.70 |
/// | DDR      | 2.5V      | 5W              | 0.62 |
/// | DDR2     | 1.8V      | 3.8W            | 0.48 |
/// | DDR3     | 1.5V      | 3–4W            | 0.45 |
/// | DDR4     | 1.2V      | 2–3W            | 0.32 |
/// | DDR5     | 1.1V      | 1.5–2.5W        | 0.25 |
/// | LPDDR2   | 1.2V      | 1.5W            | 0.19 |
/// | LPDDR3   | 1.2V      | 1.3W            | 0.16 |
/// | LPDDR4   | 1.1V      | 1–1.5W          | 0.16 |
/// | LPDDR5   | 1.05V     | 0.8–1.2W        | 0.12 |
/// | eMMC     | 3.3V/1.8V | < 0.8W          | 0.10 |
pub const RAM_TYPE_POWER: &[(&str, f64)] = &[
    ("SDRAM", 0.70),
    ("DDR", 0.60),
    ("DDR2", 0.48),
    ("DDR3", 0.45),
    ("DDR4", 0.32),
    ("DDR5", 0.25),
    ("LPDDR2", 0.19),
    ("LPDDR3", 0.16),
    ("LPDDR4", 0.16),
    ("LPDDR5", 0.12),
    ("eMMC", 0.10),
];

/// Function that calculates the writing and reading speed of RAM,
/// allocating a wide range [`ARRAY_SIZE`] of test data in memory.
///
/// # Return
///
/// - `write_bandwidth` : Write bandwidth test result in MB/s.
/// - `read_bandwidth` : Read bandwidth test result in MB/s.
pub fn get_ram_test() -> Result<(Option<f64>, Option<f64>), Box<dyn Error>> {
    let mut space_area = vec![0u8; *ARRAY_SIZE];

    let write_start = Instant::now();
    for (i, item) in space_area.iter_mut().enumerate() {
        *item = (i % 256) as u8;
    }
    let write_duration = write_start.elapsed();

    let read_start = Instant::now();
    let mut sum = 0u64;
    for &value in space_area.iter() {
        sum = sum.wrapping_add(value as u64);
    }
    unsafe {
        write_volatile(&mut sum as *mut u64, sum);
        let _ = read_volatile(&sum as *const u64);
    }
    let read_duration: Duration = read_start.elapsed();

    let result = *ARRAY_SIZE as f64;
    let write_bandwidth = result / write_duration.as_secs_f64() / 1e6;
    let read_bandwidth = result / read_duration.as_secs_f64() / 1e6;

    if write_bandwidth.is_nan()
        || read_bandwidth.is_nan()
        || write_bandwidth <= 0.0
        || read_bandwidth <= 0.0
    {
        return Err("Data 'Invalid bandwidth calculation'".to_string().into());
    }

    Ok((Some(write_bandwidth), Some(read_bandwidth)))
}

/// Parse the `dmidecode` command output to get detected RAM types.
///
/// # Returns
///
/// - A tuple of RAM type values if at least one correct type is found.
/// - An error if no values are available.
///
/// # Operating
///
/// Root privileges are required.
pub fn get_ram_types() -> Result<Option<Vec<String>>, Box<dyn Error>> {
    let output = Command::new("dmidecode").args(["-t", "memory"]).output()?;

    if !output.status.success() {
        return Err(format!(
            "Data 'dmidecode command failed with status : {}'",
            output.status
        )
        .into());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut result = Vec::new();

    for line in stdout.lines() {
        if let Some(rest) = line.trim_start().strip_prefix("Type:") {
            let types = rest.trim();

            if types != "Unknown"
                && types != "Other"
                && types != "DRAM"
                && !result.contains(&types.to_string())
            {
                result.push(types.to_string());
            }
        }
    }

    if result.is_empty() {
        Err("Data 'Failed to identifying the RAM type'".into())
    } else {
        Ok(Some(result))
    }
}

/// Estimation of power consumption by RAM in W.
/// Base on the typical power consumption per GB based on the RAM type defined in [`RAM_TYPE_POWER`].
///
/// # Returns
///
/// - Returns the estimated RAM power consumption in W.
/// - None if RAM type is unknown or total RAM is zero.
pub fn ram_power_consumption(ram_total: u64, ram_used: u64, ram_type: &str) -> Option<f64> {
    let power = RAM_TYPE_POWER
        .iter()
        .find(|&&(t, _)| t == ram_type)
        .map(|&(_, w)| w);

    if power.is_none() {
        error!("[{HEADER}] Data 'Failed to determine the RAM power classification'");
    }

    let power = power?;
    let ram_total_gb = ram_total as f64 / 1e3;
    let ram_used_gb = ram_used as f64 / 1e3;
    if ram_total_gb > 0.0 {
        Some((ram_total_gb * power) * (ram_used_gb / ram_total_gb))
    } else {
        error!("[{HEADER}] Data 'Failed to estimate the RAM power consumption'");
        None
    }
}

/// Writes JSON formatted data in a file
///
/// # Arguments
///
/// * `data` : JSON serialized collected metrics data to write
/// * `path` : File path use to writing data
///
/// # Return
///
/// - Custom error message if an error occurs during JSON data serialization or file handling.
pub fn write_json_to_file<F>(generator: F, path: &'static str) -> Result<(), Box<dyn Error>>
where
    F: FnOnce() -> Result<Value, Box<dyn Error>>,
{
    let mut data: Value = generator()?;

    // Timestamp implementation in JSON object
    let timestamp = Some(Utc::now().to_rfc3339_opts(Millis, true));

    // Format data to JSON object
    if data.is_object() {
        data.as_object_mut()
            .unwrap()
            .insert("timestamp".to_owned(), json!(timestamp));
    } else if data.is_array() {
        for item in data.as_array_mut().unwrap() {
            if item.is_object() {
                item.as_object_mut()
                    .unwrap()
                    .insert("timestamp".to_owned(), json!(timestamp));
            }
        }
    }

    let mut file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(path)?;
    let log = serde_json::to_string_pretty(&data)?;

    file.write_all(log.as_bytes())?;

    Ok(())
}
