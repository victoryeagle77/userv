use clap::Parser;
use log::error;
use std::thread;

mod probes;
mod utils;

use crate::probes::{
    cpu_info, disk_info, gpu_info, motherboard_info, net_info, ram_info, system_info,
};

/// Configuration structure for CLI arguments
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Active specified components.
    #[arg(long, value_name = "COMPONENT")]
    active: Vec<String>,
    /// Active all components.
    #[arg(long)]
    all: bool,
}

fn get_cpu_info_wrapper() {
    if let Err(e) = cpu_info::get_cpu_info() {
        error!("[CPU] {e}");
    }
}
fn get_gpu_info_wrapper() {
    if let Err(e) = gpu_info::get_gpu_info() {
        error!("[GPU] {e}");
    }
}
fn get_motherboard_info_wrapper() {
    if let Err(e) = motherboard_info::get_motherboard_info() {
        error!("[MOTHERBOARD] {e}");
    }
}
fn get_net_info_wrapper() {
    if let Err(e) = net_info::get_net_info() {
        error!("[NET_DATA] {e}");
    }
}
fn get_ram_info_wrapper() {
    if let Err(e) = ram_info::get_ram_info() {
        error!("[RAM] {e}");
    }
}
fn get_system_info_wrapper() {
    if let Err(e) = system_info::get_system_info() {
        error!("[SYSTEM] {e}");
    }
}

fn main() {
    if let Err(e) = utils::init_logger() {
        println!("[LOGGER] INIT 'Failed to initialized error logger' : {e}");
        return;
    }

    let mut handles: Vec<thread::JoinHandle<()>> = vec![];
    let cli = Cli::parse();
    let map: Vec<(&str, fn())> = vec![
        ("cpu", get_cpu_info_wrapper),
        ("disk", disk_info::get_disk_info),
        ("gpu", get_gpu_info_wrapper),
        ("motherboard", get_motherboard_info_wrapper),
        ("net", get_net_info_wrapper),
        ("ram", get_ram_info_wrapper),
        ("system", get_system_info_wrapper),
    ];

    if cli.all {
        for (_, probe) in map {
            handles.push(thread::spawn(probe));
        }
    } else {
        for component in &cli.active {
            if let Some(&(_, probe)) = map.iter().find(|(name, _)| name == component) {
                handles.push(thread::spawn(probe));
            } else {
                println!("[MAIN] Arguments 'Unknown component' : {component}");
                println!(
                    " >> Available arguments : (cpu, disk, gpu, system, motherboard, net, ram)"
                );
            }
        }
    }

    for handle in handles {
        match handle.join() {
            Ok(_) => println!("Finished task with success"),
            Err(e) => error!("[MAIN] Process 'Failure in the thread' : {e:?}"),
        }
    }
}
