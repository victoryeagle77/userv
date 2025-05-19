use chrono::{SecondsFormat, Utc};
use rusqlite::{Connection, ToSql};
use std::{error::Error, fs::read};

mod dbms;
mod utils;

use core::core::{DMIDECODE_BIN, ENTRY_BIN, db_insert_unique, db_table_query_creation, init_db};
use dbms::*;
use utils::*;

impl BoardInfo {
    /// Insert only one time main board and BIOS parameters in database.
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
        conn: &mut Connection,
        timestamp: &str,
        data: &Self,
    ) -> Result<(), Box<dyn Error>> {
        let conflict_param = ["board_serial"];
        let update_param = ["id", "timestamp"];

        let (create_index_sql, insert_stmt_sql) = db_insert_unique(
            TABLE_NAME,
            &field_descriptor(),
            &conflict_param,
            &update_param,
        )?;

        if let Some(index) = create_index_sql {
            conn.execute_batch(&index)?;
        }

        let mut stmt = conn.prepare(&insert_stmt_sql)?;

        let values: Vec<&dyn ToSql> = vec![
            &timestamp,
            &data.bios_date,
            &data.bios_vendor,
            &data.bios_version,
            &data.board_name,
            &data.board_serial,
            &data.board_vendor,
            &data.board_version,
        ];
        stmt.execute(&*values)?;

        Ok(())
    }
}

/// Push in SQLite memory database the data retrieve by [`BoardInfo`].
///
/// # Returns
///
/// Failure if we can't retrieve information or push it in database.
pub fn get_board_info() -> Result<(), Box<dyn Error>> {
    let entry_buf = read(ENTRY_BIN)?;
    let dmi_buf = read(DMIDECODE_BIN)?;

    let data = board_data_build(&entry_buf, &dmi_buf)?;
    let query = db_table_query_creation(TABLE_NAME, &field_descriptor())?;
    let mut conn = init_db(&query)?;

    let timestamp = Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true);
    BoardInfo::insert_db(&mut conn, &timestamp, &data)?;

    Ok(())
}
