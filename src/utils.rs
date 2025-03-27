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
    fs::File,
    fs::OpenOptions,
    io::{BufRead, BufReader, Read, Write},
};

const LOGGER: &str = "log/error.log";

/// Initialization and formatting of logged information during execution.
///
/// # Returns
///
/// * IO error message if log writing failed
pub fn init_logger() -> Result<(), Box<dyn std::error::Error>> {
    let logfile: FileAppender = FileAppender::builder()
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

    std::fs::write(LOGGER, "")?;
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

/// Parses a file and extracts key-value pairs separated by a colon.
/// This function reads a file line by line and splits each line at the first colon (':').
/// Both the key and value are trimmed of whitespace.
///
/// # Arguments
///
/// * `path` - A string slice that holds the path to the file to be parsed.
/// * `seq` - Char or string identifier in file to determine data sequencing
///
/// # Returns
///
/// If the file cannot be opened or read, an empty vector is returned.
/// A `Vec<(String, String)>` where each tuple represents a key-value pair found in the file.
/// If the file cannot be opened or read, an error message is logged,
/// but the function will still return an empty vector.
pub fn parse_file_content(path: &str, seq: &str) -> Vec<(String, String)> {
    match File::open(path) {
        Ok(file) => {
            let reader: BufReader<File> = BufReader::new(file);
            reader
                .lines()
                .map_while(Result::ok)
                .filter_map(|line: String| {
                    line.split_once(seq)
                        .map(|(key, value)| (key.trim().to_string(), value.trim().to_string()))
                })
                .collect()
        }
        Err(e) => {
            error!("[FILE_ERROR] File '{path}' : {e}");
            Vec::new()
        }
    }
}

/// Writes JSON formatted data in a file
///
/// # Arguments
///
/// * `data` : JSON serialized collected metrics data to write
/// * `path` : File path use to writing data
/// * `header` : JSON element to define the type of retrieved and written data
///
/// # Return
///
/// - Custom error message if an error occurs during JSON data serialization or file handling.
pub fn write_json_to_file<F>(generator: F, path: &str, header: &str)
where
    F: FnOnce() -> Result<Value, Box<dyn Error>>,
{
    let result: Result<(), Box<dyn Error>> = (|| -> Result<(), Box<dyn Error>> {
        let mut data: Value = generator()?;
        let timestamp: Option<String> =
            Some(Utc::now().to_rfc3339_opts(Millis, true)).map_or_else(|| None, Some);

        if data.is_object() {
            data.as_object_mut()
                .unwrap()
                .insert("timestamp".to_string(), json!(timestamp));
        } else if data.is_array() {
            for item in data.as_array_mut().unwrap() {
                if item.is_object() {
                    item.as_object_mut()
                        .unwrap()
                        .insert("timestamp".to_string(), json!(timestamp));
                }
            }
        }

        let mut file: File = OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open(path)
            .map_err(|e| {
                error!("[{header}] File 'Failed to open file {path}' : {e}");
                e
            })?;

        let log_json_info: String = serde_json::to_string_pretty(&data).map_err(|e| {
            error!("[{header}] Data 'Failed to serialize JSON data' : {e}");
            e
        })?;

        file.write_all(log_json_info.as_bytes()).map_err(|e| {
            error!("[{header}] File 'Failed to write to file {path}' : {e}");
            e
        })?;

        Ok(())
    })();

    if let Err(e) = result {
        error!("[{header}] File 'Failed to write JSON to file {path}' : {e}");
    }
}
