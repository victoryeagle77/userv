use log::error;

mod cpu_info;
mod disk_info;
mod gpu_info;
mod load_info;
mod motherboard_info;
mod net_info;
mod ram_info;
mod utils;

/// Main program function
/// Sequentially executes each function of the module to obtain system data
fn main() {
    if let Err(e) = utils::init_logger() {
        error!("[INIT_LOGGER] Failed to initialize logger : {}", e);
        return;
    }

    cpu_info::get_cpu_info();
    gpu_info::get_gpu_info();
    load_info::get_load_info();
    motherboard_info::get_motherboard_info();
    net_info::get_net_info();
    ram_info::get_ram_info();
    disk_info::get_disk_info();
}
