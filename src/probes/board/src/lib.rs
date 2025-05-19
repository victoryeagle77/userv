use chrono::{SecondsFormat, Utc};
use rusqlite::{Connection, params};
use std::{error::Error, fs::read};

mod dbms;
mod utils;

use core::core::{DMIDECODE_BIN, ENTRY_BIN, db_insert_query, db_table_query_creation, init_db};
use dbms::*;
use utils::*;

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
fn insert_db(conn: &Connection, timestamp: &str, data: &BoardInfo) -> Result<(), Box<dyn Error>> {
    let query = db_insert_query(TABLE_NAME, &field_descriptor())?;
    conn.execute(
        &query,
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

pub fn get_board_info() -> Result<(), Box<dyn Error>> {
    let entry_buf = read(ENTRY_BIN)?;
    let dmi_buf = read(DMIDECODE_BIN)?;

    let data = board_data_build(&entry_buf, &dmi_buf)?;
    if data.is_empty() {
        return Ok(());
    }

    let query = db_table_query_creation(TABLE_NAME, &field_descriptor())?;
    let conn = init_db(&query)?;

    let timestamp = Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true);
    insert_db(&conn, &timestamp, &data)?;

    Ok(())
}
