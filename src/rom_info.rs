use std::fs::{File, read_dir};
use std::io::{BufRead, BufReader};
use colored::Colorize;

const PARTITIONS: &'static str = "/proc/partitions";

fn read_file(path: &str) -> Result<String, std::io::Error> {
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);
    let mut contents = String::new();
    reader.read_line(&mut contents)?;
    Ok(contents)
}

pub fn get_rom_info() -> Result<(), std::io::Error> {
    println!("{}", "[[ ROM ]]\n".magenta().bold());

    let file = File::open(PARTITIONS)?;
    let reader = BufReader::new(file);

    for line in reader.lines().skip(2) {
        let line = line?;
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() == 4 && !parts[3].chars().any(|c| c.is_ascii_digit()) {

            let device = parts[3];
            println!("Device: /dev/{}", device);

            // Taille du disque
            if let Ok(size) = read_file(&format!("/sys/block/{}/size", device)) {
                let size_bytes = size.trim().parse::<u64>().unwrap_or(0) * 512;
                println!("Size: {} GB", size_bytes / 1_073_741_824); // Conversion en Go
            }
            // Modele du disque
            if let Ok(model) = read_file(&format!("/sys/block/{}/device/model", device)) {
                println!("Model: {}", model.trim());
            }
            // Vendor
            if let Ok(vendor) = read_file(&format!("/sys/block/{}/device/vendor", device)) {
                println!("Vendor: {}", vendor.trim());
            }
            // Type de disque (HDD/SSD)
            if let Ok(rotational) = read_file(&format!("/sys/block/{}/queue/rotational", device)) {
                let disk_type = if rotational.trim() == "1" { "HDD" } else { "SSD" };
                println!("Type: {}", disk_type);
            }
            // Informations sur les partitions
            if let Ok(entries) = read_dir(format!("/sys/block/{}", device)) {
                for entry in entries {
                    if let Ok(entry) = entry {
                        let name = entry.file_name();
                        if name.to_str().map_or(false, |s| s.starts_with(device)) {
                            if let Ok(size) = read_file(&format!("/sys/block/{}/{}/size", device, name.to_str().unwrap())) {
                                let size_bytes = size.trim().parse::<u64>().unwrap_or(0) * 512;
                                println!("Partition {}: {:.2} GB", name.to_str().unwrap(), size_bytes / 1_073_741_824);
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

/*
use std::ffi::CString;
use std::ptr;
use libc::{c_void, open, read, close};

const DEVICE_PATH: &str = "/dev/sda"; // Remplacez par le chemin de votre disque

fn main() {
    // Ouvrir le périphérique
    let device = CString::new(DEVICE_PATH).expect("CString::new failed");
    let fd = unsafe { open(device.as_ptr(), 0) }; // Ouvrir en mode lecture

    if fd < 0 {
        eprintln!("Erreur lors de l'ouverture du périphérique");
        return;
    }

    let mut buffer: [u8; 512] = [0; 512]; // Buffer pour les données SMART

    // Envoyer la commande SMART
    let result = unsafe { read(fd, buffer.as_mut_ptr() as *mut c_void, buffer.len()) };

    if result < 0 {
        eprintln!("Erreur lors de la lecture des données SMART");
        unsafe { close(fd) };
        return;
    }

    // Extraire les informations spécifiques
    extract_smart_info(&buffer[..result as usize]);

    // Fermer le périphérique
    unsafe { close(fd) };
}

fn extract_smart_info(buffer: &[u8]) {
    // Assurez-vous que le buffer contient suffisamment de données
    if buffer.len() < 512 {
        eprintln!("Le buffer ne contient pas assez de données.");
        return;
    }

    // Exemple d'extraction des heures de mise sous tension (index hypothétique)
    let power_on_hours_index = 9; // Hypothétique index pour Power-On Hours
    let power_on_hours = buffer[power_on_hours_index];

    println!("Nombre d'heures de fonctionnement : {}", power_on_hours);

    // Extraction du nombre de secteurs réalloués
    let reallocated_sectors_index = 5; // Hypothétique index pour Reallocated Sectors Count
    let reallocated_sectors = buffer[reallocated_sectors_index];

    if reallocated_sectors > 0 {
        println!("État de santé : Mauvais (secteurs réalloués : {})", reallocated_sectors);
    } else {
        println!("État de santé : Bon");
    }

    // Extraction du Current Pending Sector Count
    let pending_sectors_index = 196; // Index typique pour Current Pending Sector Count
    let current_pending_sectors = buffer[pending_sectors_index];

    println!("Nombre de secteurs en attente : {}", current_pending_sectors);

    // Extraction de la température (index hypothétique)
    let temperature_index = 194; // Index typique pour la température
    let temperature = buffer[temperature_index];

    println!("Température du disque : {} °C", temperature);
}
*/