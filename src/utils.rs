//! # File utilities

use board::get_board_info;
use cpu::get_cpu_info;
use gpu::get_gpu_info;
use memory::get_mem_info;
use network::get_net_info;
use storage::get_storage_info;
use system::get_system_info;

use clap::ValueEnum;
use log::{LevelFilter, error};
use log4rs::{
    append::file::FileAppender,
    config::{Appender, Config, Root},
    encode::pattern::PatternEncoder,
    filter::threshold::ThresholdFilter,
    init_config,
};
use std::{
    error::Error,
    fs::{create_dir_all, write},
    path::Path,
};

const LOGGER: &str = "log/error.log";
pub const HEADER: &str = "MAIN";

/// Enumeration of available arguments corresponding to a component
#[derive(Debug, Clone, ValueEnum)]
pub enum Component {
    /// Motherboard or principal system board probe data.
    Board,
    /// CPU probe data.
    Cpu,
    /// GPU device probe data.
    Gpu,
    /// Network probe data.
    Net,
    /// Computing memory probe data.
    Memory,
    /// Storage device probe data.
    Storage,
    /// Operating system probe data.
    System,
}

/// Parameters of probe that analyzing and retrieves data about a component.
pub struct Probe {
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
    pub fn get_probe(component: &Component) -> Probe {
        match component {
            Component::Board => Probe {
                label: "BOARD",
                func: get_board_info,
            },
            Component::Cpu => Probe {
                label: "CPU",
                func: get_cpu_info,
            },
            Component::Gpu => Probe {
                label: "GPU",
                func: get_gpu_info,
            },
            Component::Net => Probe {
                label: "NETWORK",
                func: get_net_info,
            },
            Component::Memory => Probe {
                label: "MEMORY",
                func: get_mem_info,
            },
            Component::Storage => Probe {
                label: "STORAGE",
                func: get_storage_info,
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
    pub fn run_probe(probe: Probe) {
        if let Err(e) = (probe.func)() {
            error!("[{}] {e}", probe.label);
        }
    }
}

/// Initialization and formatting information logger to store messages concerning microservices behavior.
///
/// # Returns
///
/// Writing the error in the log file.
/// Print IO error message if log writing failed.
pub fn init_logger() -> Result<(), Box<dyn Error>> {
    if let Some(parent) = Path::new(LOGGER).parent() {
        create_dir_all(parent)?;
    }

    let logfile = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new("{d} {m} {n}")))
        .build(LOGGER)?;

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

    let _ = write(LOGGER, "");
    init_config(config)?;

    Ok(())
}
