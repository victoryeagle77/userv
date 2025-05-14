//! # File utilities module
//!
//! This module provides functionality to handle or get data from files and folders.

use chrono::{SecondsFormat::Millis, Utc};
use log::LevelFilter;
use log4rs::{
    append::file::FileAppender,
    config::{Appender, Config, Root},
    encode::pattern::PatternEncoder,
    filter::threshold::ThresholdFilter,
};
use serde_json::{json, Value};
use std::{error::Error, fs::write, fs::OpenOptions, io::Write};

const LOGGER: &str = "log/error.log";

/// Initialization and formatting information logger to store messages concerning microservices behavior.
///
/// # Returns
///
/// IO error message if log writing failed
pub fn init_logger() -> Result<(), Box<dyn Error>> {
    let logfile = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new("{d} {m} {n}")))
        .build(LOGGER)
        .unwrap();

    let config = Config::builder()
        .appender(
            Appender::builder()
                .filter(Box::new(ThresholdFilter::new(LevelFilter::Error)))
                .build("logfile", Box::new(logfile)),
        )
        .build(
            Root::builder()
                .appender("logfile")
                .build(LevelFilter::Error),
        )?;

    write(LOGGER, "")?;
    log4rs::init_config(config)?;

    Ok(())
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
pub fn write_json_to_file<F>(generator: F, path: &str) -> Result<(), Box<dyn Error>>
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
