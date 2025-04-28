use clap::Parser;
use log::error;
use std::thread;

mod probes;
mod utils;

use crate::probes::{
    cpu_info, disk_info, gpu_info, load_info, motherboard_info, net_info, ram_info,
};

/// Structure de configuration des arguments CLI
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

fn main() {
    if let Err(e) = utils::init_logger() {
        println!("[LOGGER] INIT 'Failed to initialized error logger' : {e}");
        return;
    }

    let mut handles: Vec<thread::JoinHandle<()>> = vec![];
    let cli = Cli::parse();
    let map: Vec<(&str, fn())> = vec![
        ("cpu", cpu_info::get_cpu_info),
        ("disk", disk_info::get_disk_info),
        ("gpu", gpu_info::get_gpu_info),
        ("load", load_info::get_load_info),
        ("motherboard", motherboard_info::get_motherboard_info),
        ("net", net_info::get_net_info),
        ("ram", ram_info::get_ram_info),
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
                println!(" >> Available arguments : (cpu, disk, gpu, load, motherboard, net, ram)");
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
