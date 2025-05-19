use chrono::{SecondsFormat, Utc};
use once_cell::sync::Lazy;
use rusqlite::Connection;
use std::{error::Error, fs::read, sync::Mutex};

mod utils;
use utils::*;

use core::core::{DMIDECODE_BIN, ENTRY_BIN, init_db};

pub const REQUEST: &str = "CREATE TABLE IF NOT EXISTS board_data (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        timestamp TEXT NOT NULL,
        bios_date TEXT,
        bios_vendor TEXT,
        bios_version TEXT,
        board_name TEXT,
        board_serial TEXT,
        board_vendor TEXT,
        board_version TEXT
    )";

static DB_CONN: Lazy<Mutex<Connection>> = Lazy::new(|| {
    let conn = init_db(REQUEST).expect("DB initialization failed");
    Mutex::new(conn)
});

/// The main board information of device is static and produce static data.
/// We ensure to avoid the static or useless data replication, in comparing them with previous saved data.
/// Only different values ​​will be taken into account.
///
/// # Arguments
///
/// `conn`: The [`Connection`] to SQLite database.
///
/// # Returns
///
/// Use the [`BoardInfo`] filled structures for SQLite database comparison.
fn board_data_redundance(conn: &Connection) -> Option<BoardInfo> {
    let mut stmt = conn
        .prepare(
            "SELECT
                bios_date,
                bios_vendor,
                bios_version,
                board_name,
                board_serial,
                board_vendor,
                board_version
                FROM board_data ORDER BY id DESC LIMIT 1",
        )
        .ok()?;

    let mut rows = stmt.query([]).ok()?;
    rows.next().ok().flatten().map(|row| BoardInfo {
        bios_date: row.get(0).ok(),
        bios_vendor: row.get(1).ok(),
        bios_version: row.get(2).ok(),
        board_name: row.get(3).ok(),
        board_serial: row.get(4).ok(),
        board_vendor: row.get(5).ok(),
        board_version: row.get(6).ok(),
    })
}

pub fn get_board_info() -> Result<(), Box<dyn Error>> {
    let entry_buf = read(ENTRY_BIN)?;
    let dmi_buf = read(DMIDECODE_BIN)?;

    let data = board_data_build(&entry_buf, &dmi_buf)?;
    if data.is_empty() {
        return Ok(());
    }

    let conn = DB_CONN.lock().unwrap();

    if let Some(last) = board_data_redundance(&conn)
        && data == last
    {
        return Ok(());
    }

    let timestamp = Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true);
    BoardInfo::insert_db(&conn, &timestamp, &data)?;

    Ok(())
}

//----------------//
// UNIT CODE TEST //
//----------------//

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    // Mock function to simulate SQL database connection
    fn mock_db() -> Connection {
        let conn = Connection::open_in_memory().expect("Failed to open in-memory DB");
        conn.execute_batch(REQUEST).expect("Failed to create table");
        conn
    }

    // Test `board_info_redundance` function after insert new data and comparing to previous data
    #[test]
    fn test_board_info_redundance() {
        let conn = mock_db();

        let test_data = BoardInfo {
            bios_date: Some("1999-01-01".to_string()),
            bios_vendor: Some("TestVendor".to_string()),
            bios_version: Some("v1".to_string()),
            board_name: Some("TestBoard".to_string()),
            board_serial: Some("123456789".to_string()),
            board_vendor: Some("TestBoardVendor".to_string()),
            board_version: Some("1.0".to_string()),
        };

        let timestamp = Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true);
        BoardInfo::insert_db(&conn, &timestamp, &test_data).expect("Failed to insert test data");
        let res = board_data_redundance(&conn).expect("Expected Some after insert");
        assert_eq!(res.bios_date, test_data.bios_date);
    }
}
