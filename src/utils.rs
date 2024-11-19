//! # File utilities module
//!
//! This module provides functionality to handle or get data from files and folders.

use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use chrono::Local;

const KB: &'static u64 = &1000;
const MB: &'static u64 = &1000000;
const GB: &'static u64 = &1000000000;

/// # Function
/// 
/// Public function `format_unit` provides conversion
/// 
/// # Returns
///
/// * Formatted string result
///
pub fn format_unit(bytes: u64) -> String {
    if bytes < *KB {
        format!("{} octets", bytes)
    } else if bytes < *MB {
        format!("{:.2} KB", bytes as f64 / *KB as f64)
    } else if bytes < *GB {
        format!("{:.2} MB", bytes as f64 / *MB as f64)
    } else {
        format!("{:.2} GB", bytes as f64 / *GB as f64)
    }
}

/// # Function
/// 
/// Public function `get_current_datetime` calculate local date an hour.
/// 
/// # Returns
///
/// * Result in string with date and hour content
///
pub fn get_current_datetime() -> String {
    let now = Local::now();
    return now.format("%Y-%m-%d %H:%M:%S").to_string();
}

/// Function `read_file_content`
/// Reads the entire contents of a file into a string.
///
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
/// # Errors
///
/// If the file cannot be opened or read, an error message is printed to stderr,
/// including the file path and the specific error encountered. In this case,
/// the function returns `None`.
///
pub fn read_file_content(path: &str) -> Option<String> {
    let mut content = String::new();
    match File::open(path).and_then(|mut file| file.read_to_string(&mut content)) {
        Ok(_) => Some(content),
        Err(e) => {
            eprintln!("<ERROR_3> File : '{}' ({})", path, e);
            None
        }
    }
}

/// Function `parse_file_content`
/// Parses a file and extracts key-value pairs separated by a colon.
///
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
/// A `Vec<(String, String)>` where each tuple represents a key-value pair found in the file.
/// If the file cannot be opened or read, an empty vector is returned.
///
/// # Errors
///
/// If the file cannot be opened or read, an error message is printed to stderr,
/// but the function will still return an empty vector.
///
pub fn parse_file_content(path: &str, seq: &str) -> Vec<(String, String)> {
    match File::open(path) {
        Ok(file) => {
            let reader = BufReader::new(file);
            let mut data = Vec::new();

            for line in reader.lines().flatten() {
                if let Some((key, value)) = line.split_once(seq) {
                    data.push((key.trim().to_string(), value.trim().to_string()));
                }
            }
            data
        },
        Err(e) => {
            eprintln!("<ERROR_3> File : '{}' ({})", path, e);
            Vec::new()
        }
    }
}