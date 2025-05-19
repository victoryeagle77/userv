//! # File utilities module

use dmidecode::{EntryPoint, Structure};
use log::error;
use rusqlite::{Connection, params};
use serde::Serialize;
use std::error::Error;

pub const HEADER: &str = "BOARD";

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

impl BoardInfo {
    pub fn is_empty(&self) -> bool {
        self.bios_date.is_none()
            && self.bios_version.is_none()
            && self.bios_vendor.is_none()
            && self.board_name.is_none()
            && self.board_serial.is_none()
            && self.board_vendor.is_none()
            && self.board_version.is_none()
    }

    /// Insert main board parameters in database.
    ///
    /// # Arguments
    ///
    /// - `conn` : Allow by a [`Connection`] constructor type the connection with an SQLite database.
    /// - `timestamp` : Date trace for the history identification.
    /// - `data` : [`BoardInfo`] information to insert in database.
    ///
    /// # Returns
    ///
    /// - Insert the [`BoardInfo`] filled structure in an SQLite database.
    /// - Logs an error if the SQL insert request failed.
    pub fn insert_db(
        conn: &Connection,
        timestamp: &str,
        data: &Self,
    ) -> Result<(), Box<dyn Error>> {
        conn.execute(
            "INSERT INTO board_data (
                timestamp,
                bios_date,
                bios_vendor,
                bios_version,
                board_name,
                board_serial,
                board_vendor,
                board_version
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                timestamp,
                data.bios_date,
                data.bios_vendor,
                data.bios_version,
                data.board_name,
                data.board_serial,
                data.board_vendor,
                data.board_version
            ],
        )?;
        Ok(())
    }
}

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
    use crate::REQUEST;

    const TIMESTAMP: &'static str = "2025-08-15T14:00:00Z";

    // Mock function to simulate SQL database connection
    fn mock_data(schema: &str) -> Result<Connection, Box<dyn Error>> {
        let conn = Connection::open_in_memory().expect("Failed to open in-memory DB");
        conn.execute_batch(schema)
            .expect("Failed to insert data in table");
        Ok(conn)
    }

    // Test `insert_db` function with data pushed in SQL table
    #[test]
    fn test_insert_db() {
        let conn = mock_data(REQUEST).expect("DB init failed");

        let data = BoardInfo {
            bios_date: Some("2025-08-24".to_string()),
            bios_vendor: Some("bios_vendor".to_string()),
            bios_version: Some("v1.1".to_string()),
            board_name: Some("board".to_string()),
            board_serial: Some("123456789".to_string()),
            board_vendor: Some("board_vendor".to_string()),
            board_version: Some("v1.2".to_string()),
        };

        BoardInfo::insert_db(&conn, TIMESTAMP, &data).expect("Insert into DB failed");

        let mut stmt = conn
            .prepare(
                "SELECT timestamp, bios_date, board_name FROM board_data WHERE board_serial = ?1",
            )
            .unwrap();
        let mut rows = stmt.query(&[&data.board_serial]).unwrap();

        if let Some(row) = rows.next().unwrap() {
            let ts: String = row.get(0).unwrap();
            let bios_date: Option<String> = row.get(1).unwrap();
            assert_eq!(ts, TIMESTAMP);
            assert_eq!(bios_date, data.bios_date);
        } else {
            panic!("No row found in DB after insert");
        }
    }

    // Test `is_empty` function of `BoardInfo` implementation
    #[test]
    fn test_is_empty() {
        let res = BoardInfo {
            bios_date: None,
            bios_version: None,
            bios_vendor: None,
            board_name: None,
            board_serial: None,
            board_vendor: None,
            board_version: None,
        };
        assert!(res.is_empty());
    }

    // Test `board_data_build` function with invalid data reading
    #[test]
    fn test_board_data_build_error() {
        let invalid_entry_buf: &[u8] = b"invalid data";
        let dmi_buf: &[u8] = &[];
        let res = board_data_build(invalid_entry_buf, dmi_buf);
        assert!(res.is_err());
    }
}
