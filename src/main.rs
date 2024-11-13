use std::fmt::Display;

mod ram_info;
mod rom_info;
mod cpu_info;
mod load_info;
mod motherboard_info;
mod utils;

/// Executes a given function and handles potential errors.
///
/// This function takes a closure as an argument, executes it, and handles any errors that may occur.
/// If an error occurs, it is printed to stderr. Otherwise, an empty line is printed to stdout.
///
/// # Arguments
///
/// * `func` - Closure which return `Result<(), E>` où `E` implémente `Display`.
///
fn execute_and_handle_error<F, E>(func: F)
where
    F: FnOnce() -> Result<(), E>,
    E: Display,
{
    if let Err(e) = func() { eprintln!("Error : {}\n", e); } 
    else { println!("\n"); }
}

/// Function main
/// Main program function
///
/// Sequentially executes each function of the module to obtain data from the system
///
fn main() {
    execute_and_handle_error(rom_info::get_rom_info);
    execute_and_handle_error(load_info::get_load_info);
    ram_info::get_ram_info();
    cpu_info::get_cpu_info();
    motherboard_info::get_motherboard_info();
}
