//! # File utilities module

use dmidecode::{EntryPoint, Structure};
use log::error;
use serde::Serialize;
use std::error::Error;

const HEADER: &str = "BOARD";

/// Collection of collected motherboard data.
#[derive(Debug, Serialize, PartialEq, Default)]
pub struct BoardInfo {
    /// BIOS release date version.
    pub bios_date: Option<String>,
    /// BIOS software version.
    pub bios_version: Option<String>,
    /// BIOS vendor name.
    pub bios_vendor: Option<String>,
    /// Main board (or motherboard) full name.
    pub board_name: Option<String>,
    /// Main board (or motherboard) serial number.
    pub board_serial: Option<String>,
    /// Main board (or motherboard) vendor name.
    pub board_vendor: Option<String>,
    /// Main board (or motherboard) hardware version.
    pub board_version: Option<String>,
}

/// Parse the `dmidecode` command output to get data on detected main board data.
///
/// # Returns
///
/// - A tuple of data concerning device mother board or main board.
/// - An error if no values are available.
///
/// # Operating
///
/// Root privileges are required.
pub fn board_data_build(entry_buf: &[u8], dmi_buf: &[u8]) -> Result<BoardInfo, Box<dyn Error>> {
    let entry = EntryPoint::search(entry_buf).map_err(|e| {
        error!("[{HEADER}] Data 'EntryPoint search error': {e:?}");
        Box::new(e) as Box<dyn Error>
    })?;

    let mut data = BoardInfo::default();

    for table in entry.structures(dmi_buf).filter_map(Result::ok) {
        if let Structure::Bios(bios) = &table {
            data.bios_date = Some(bios.bios_release_date.to_string());
            data.bios_version = Some(bios.bios_version.to_string());
            data.bios_vendor = Some(bios.vendor.to_string());
        } else if let Structure::BaseBoard(board) = &table {
            data.board_name = Some(board.product.to_string());
            data.board_serial = Some(board.serial.to_string());
            data.board_vendor = Some(board.product.to_string());
            data.board_version = Some(board.version.to_string());
        }
    }

    Ok(data)
}

//----------------//
// UNIT CODE TEST //
//----------------//

#[cfg(test)]
mod tests {
    use super::*;

    // Test `board_data_build` function with invalid data reading
    #[test]
    fn test_board_data_build_error() {
        let invalid_entry_buf: &[u8] = b"invalid data";
        let dmi_buf: &[u8] = &[];
        let res = board_data_build(invalid_entry_buf, dmi_buf);
        assert!(res.is_err());
    }
}
