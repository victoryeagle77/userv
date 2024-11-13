//! # System Load data Module
//!
//! This module provides functionality to retrieve system load data on Unix-based systems.

use std::fs::File;
use std::io::{BufRead, BufReader};
use std::time::{Duration};
use colored::Colorize;

const UPTIME: &'static str = "/proc/uptime";
const LOADAVG: &'static str = "/proc/loadavg";

/// Retrieves the system uptime from `/proc/uptime`.
///
/// # Returns
///
/// Returns a `Result` containing a `Duration` representing the system uptime
/// if successful, or an `std::io::Error` if an error occurs.
///
/// # Errors
///
/// This function will return an error if:
/// - The `/proc/uptime` file cannot be opened or read.
/// - The contents of the file cannot be parsed as expected.
///
fn get_uptime() -> Result<Duration, std::io::Error> {
    let uptime_file = File::open(UPTIME)?;
    let mut reader = BufReader::new(uptime_file);
    let mut data = String::new();
    reader.read_line(&mut data)?;

    let uptime_secs = data.split_whitespace()
        .next()
        .and_then(|s| s.parse::<f64>().ok())
        .unwrap_or(0.0);

    return Ok(Duration::from_secs_f64(uptime_secs));
}

/// Retrieves detailed system load data.
/// This function reads from `/proc/loadavg` to get load averages and process data.
///
/// # Returns
///
/// Returns `Ok(())` if the data was successfully retrieved,
/// or an `std::io::Error` if an error occurs.
///
/// # Errors
///
/// This function will return an error if:
/// - The `/proc/loadavg` file cannot be opened or read.
/// - The contents of the file cannot be parsed as expected.
///
/// # Output
///
/// The function retrieves the following data :
/// - 1, 5, and 15 minute load averages
/// - Number of currently running processes and total processes
/// - Last created process ID
/// - System uptime in days, hours, and minutes
///
pub fn get_load_info() -> Result<(), std::io::Error> {
    let file = File::open(LOADAVG)?;
    let mut reader = BufReader::new(file);
    let mut line = String::new();
    reader.read_line(&mut line)?;

    let parts: Vec<&str> = line.split_whitespace().collect();

    println!("{}", "\n[[ SYSTEM LOAD ]]\n".magenta().bold());

    // System uptime
    if let Ok(uptime) = get_uptime() {
        println!("{} {} days, {} hours, {} minutes", "System uptime:".bold(), 
                    uptime.as_secs() / 86400,
                    (uptime.as_secs() % 86400) / 3600,
                    (uptime.as_secs() % 3600) / 60);
    }

    if parts.len() >= 5 {
        println!("{} {}", "System load (1 min):".bold(), parts[0]);
        println!("{} {}", "System load (5 min):".bold(), parts[1]);
        println!("{} {}", "System load (15 min):".bold(), parts[2]);

        // Get processes data
        let running_processes: Vec<&str> = parts[3].split('/').collect();
        if running_processes.len() == 2 {
            println!("{} {}", "Running processes:".bold(), running_processes[0]);
            println!("{} {}", "Total processes:".bold(), running_processes[1]);
        }

        // ID of the last process created
        if let Ok(last_pid) = parts[4].parse::<u32>() {
            println!("{} {}", "Last process ID:".bold(), last_pid);
        }

    } else { println!("Unexpected format in {}", LOADAVG); }

    Ok(())
}
