mod cpu_info;
mod load_info;
mod motherboard_info;
mod net_info;
mod ram_info;
mod rom_info;
mod utils;

/// # Function 
/// 
/// Main program function
/// Sequentially executes each function of the module to obtain system data
///
fn main() {
    cpu_info::get_cpu_info();
    load_info::get_load_info();
    motherboard_info::get_motherboard_info();
    net_info::get_net_info();
    ram_info::get_ram_info();
    rom_info::get_rom_info();
}
