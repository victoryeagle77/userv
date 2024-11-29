//! # File utilities module
//!
//! This module provides functionality to handle or get data from files and folders.

use chrono::Local;
use std::fs::File;
use std::io::{BufRead, BufReader, Read};

const KILO: u64 = 1000;
const MEGA: u64 = 1000000;
const GIGA: u64 = 1000000000;

/// Public function to convert a integer value to an other representation, 
/// according to the International System norms :
/// 
/// * `Kilo` for 1e3 representation
/// * `Mega` for 1e6 representation
/// * `Giga` for 1e9 representation
/// 
/// # Arguments
/// 
/// * `unit` : The integer value to convert
/// 
/// # Returns
///
/// * Formatted string result
pub fn format_unit(unit: u64) -> f32 {
    if unit < KILO {
        unit as f32
    } else if unit < MEGA {
        unit as f32 / KILO as f32
    } else if unit < GIGA {
        unit as f32 / MEGA as f32
    } else {
        unit as f32 / GIGA as f32
    }
}

/// Public function calculate local date an hour.
///
/// # Returns
///
/// * Result in string with date and hour content
pub fn get_current_datetime() -> String {
    let now = Local::now();
    return now.format("%Y-%m-%d %H:%M:%S").to_string();
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
/// # Errors
///
/// If the file cannot be opened or read, an error message is printed to stderr,
/// including the file path and the specific error encountered. In this case,
/// the function returns `None`.
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
/// A `Vec<(String, String)>` where each tuple represents a key-value pair found in the file.
/// If the file cannot be opened or read, an empty vector is returned.
///
/// # Errors
///
/// If the file cannot be opened or read, an error message is printed to stderr,
/// but the function will still return an empty vector.
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
        }
        Err(e) => {
            eprintln!("<ERROR_3> File : '{}' ({})", path, e);
            Vec::new()
        }
    }
}
