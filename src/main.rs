//! # Main File
//!
//! This file provides call the necessary to handle each probe,
//! separately or simultaneously in threaded tasks.

use clap::Parser;
use log::error;
use std::{
    process::exit,
    thread::{sleep, spawn},
    time::Duration,
};

mod utils;
use utils::*;
//use gui_web::web;

/// Data defining arguments to active or not a probe to retrieve component data.
#[derive(Parser, Debug)]
struct Arg {
    /// List of [`Component`] to active.
    #[arg(long, value_enum, value_delimiter = ',', conflicts_with = "all")]
    active: Vec<Component>,
    /// Activation state of a probe.
    #[arg(long, conflicts_with = "active")]
    all: bool,
    /// Interval in seconds between each probe run. If not set, probes run once.
    #[arg(long, default_value_t = 0)]
    freq: u64,
}

/// Main function of `userv` program that run in threading tasks each probes
/// to retrieve all data concerning component of a machine.
fn main() {
    if let Err(e) = init_logger() {
        eprintln!("[{HEADER}] INIT 'Failed to initialize error logger' : {e}");
        return;
    }

    let arg = Arg::parse();
    if !arg.all && arg.active.is_empty() {
        error!("[{HEADER}] Arguments 'No probe specified'");
        eprintln!(
            "[{HEADER}] Arguments : No probe specified !\n\
            --all : Active all probes\n\
            --active <probe>"
        );
        exit(1);
    }

    let components = if arg.all {
        vec![
            Component::Board,
            Component::Cpu,
            Component::Gpu,
            Component::Net,
            Component::Memory,
            Component::Storage,
            Component::System,
        ]
    } else {
        arg.active
    };

    if arg.freq == 0 {
        let mut handles = Vec::new();
        for component in components {
            let probe = Probe::get_probe(&component);
            handles.push(spawn(move || Probe::run_probe(probe)));
        }
        for handle in handles {
            match handle.join() {
                Ok(_) => println!("Finished task with success"),
                Err(e) => error!("[{HEADER}] Process 'Failure in the thread' : {e:?}"),
            }
        }
    } else {
        loop {
            let mut handles = Vec::new();
            for component in &components {
                let probe = Probe::get_probe(component);
                handles.push(spawn(move || Probe::run_probe(probe)));
            }
            for handle in handles {
                match handle.join() {
                    Ok(_) => println!("Finished task with success"),
                    Err(e) => error!("[{HEADER}] Process 'Failure in the thread' : {e:?}"),
                }
            }
            sleep(Duration::from_secs(arg.freq));
        }
    }

    //web();
}
