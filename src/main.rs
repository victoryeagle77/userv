use clap::{Parser, ValueEnum};
use log::error;
use probes::{
    board_info::get_board_info, cpu_info::get_cpu_info, disk_info::get_disk_info,
    gpu_info::get_gpu_info, net_info::get_net_info, ram_info::get_ram_info,
    system_info::get_system_info,
};
use std::{process::exit, thread};
use utils::init_logger;

mod probes;
mod utils;

const HEADER: &str = "MAIN";

/// Enumeration of available arguments corresponding to a component
#[derive(Debug, Clone, ValueEnum)]
enum Component {
    /// CPU probe data.
    Cpu,
    /// Storage device probe data.
    Disk,
    /// GPU device probe data.
    Gpu,
    /// Motherboard or principal system board probe data.
    Board,
    /// Network probe data.
    Net,
    /// Computing and SWAP memory probe data.
    Ram,
    /// Operating system probe data.
    System,
}

/// Data defining arguments to active or not a probe to retrieve component data.
#[derive(Parser, Debug)]
struct Arg {
    /// List of [`Component`] to active.
    #[arg(long, value_enum, value_delimiter = ',')]
    active: Vec<Component>,
    /// Activation state of a probe.
    #[arg(long)]
    all: bool,
}

/// Parameters of probe that analyzing and retrieves data about a component.
struct Probe {
    /// Identification header for information loggers about a probe.
    label: &'static str,
    /// Function concerning data retrieves by a probe.
    func: fn() -> Result<(), Box<dyn std::error::Error>>,
}

impl Probe {
    /// Define the probe and the label associated to a component,
    /// and check if it is selected.
    ///
    /// # Arguments
    ///
    /// - `component` : The component that we want retrieves data.
    ///
    /// # Returns
    ///
    /// The selected component via [`Probe`] information.
    fn get_probe(component: &Component) -> Probe {
        match component {
            Component::Cpu => Probe {
                label: "CPU",
                func: get_cpu_info,
            },
            Component::Disk => Probe {
                label: "STORAGE",
                func: get_disk_info,
            },
            Component::Gpu => Probe {
                label: "GPU",
                func: get_gpu_info,
            },
            Component::Board => Probe {
                label: "MOTHERBOARD",
                func: get_board_info,
            },
            Component::Net => Probe {
                label: "NETWORK",
                func: get_net_info,
            },
            Component::Ram => Probe {
                label: "RAM",
                func: get_ram_info,
            },
            Component::System => Probe {
                label: "SYSTEM",
                func: get_system_info,
            },
        }
    }

    /// Run a probe to retrieve information about a component.
    /// If component's data can't be retrieved, we log the error returned.
    ///
    /// # Arguments
    ///
    /// - `probe` : Concerning component with [`Probe`].
    fn run_probe(probe: Probe) {
        if let Err(e) = (probe.func)() {
            error!("[{}] {e}", probe.label);
        }
    }
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
        eprintln!(
            "[{HEADER}] Arguments : No probe specified !\n\
            --all : Active all probes\n\
            --active <probe>"
        );
        exit(1);
    }

    let components = if arg.all {
        vec![
            Component::Cpu,
            Component::Disk,
            Component::Gpu,
            Component::Board,
            Component::Net,
            Component::Ram,
            Component::System,
        ]
    } else {
        arg.active
    };

    let mut handles = Vec::new();
    for component in components {
        let probe = Probe::get_probe(&component);
        handles.push(thread::spawn(move || Probe::run_probe(probe)));
    }

    for handle in handles {
        match handle.join() {
            Ok(_) => println!("Finished task with success"),
            Err(e) => error!("[{HEADER}] Process 'Failure in the thread' : {e:?}"),
        }
    }
}
