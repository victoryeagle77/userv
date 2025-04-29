//! # File utilities module
//!
//! This module provides functionality to handle or get data from files and folders.

use chrono::{SecondsFormat::Millis, Utc};
use log::{error, LevelFilter};
use log4rs::{
    append::file::FileAppender,
    config::{Appender, Config, Root},
    encode::pattern::PatternEncoder,
    filter::threshold::ThresholdFilter,
};
use serde_json::{json, Value};
use std::{
    error::Error,
    fs::write,
    fs::File,
    fs::OpenOptions,
    io::{Read, Write},
};

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

/// Reads the entire contents of a file into a string.
/// This function attempts to open the file at the given path and read its entire contents
/// into a `String`. If successful, it returns the contents wrapped in `Some`. If any error
/// occurs during the file opening or reading process, it returns `None`.
///
/// # Arguments
///
/// * `path` - A string slice that holds the path to the file to be read.
///
/// # Returns
///
/// * `Some(String)` containing the entire contents of the file if successful.
/// * `None` if any error occurs during file opening or reading.
///
/// If the file cannot be opened or read, an error message is printed to stderr,
/// including the file path and the specific error encountered. In this case,
/// the function returns `None`.
pub fn read_file_content(path: &str) -> Option<String> {
    let mut content = String::new();
    match File::open(path).and_then(|mut file| file.read_to_string(&mut content)) {
        Ok(_) => Some(content),
        Err(e) => {
            error!("[FILE_ERROR] File '{path}' : {e}");
            None
        }
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
