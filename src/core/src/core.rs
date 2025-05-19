use rusqlite::Connection;
use std::{
    error::Error,
    path::Path,
    thread::sleep,
    time::{Duration, Instant},
};

/// SQLite database file path.
const DATABASE: &str = "log/data.db";

/// SMBIOS provides a structure called Entry Point Structure (EPS) that contains a pointer to the SMBIOS Structure Table and some additional information.
pub const ENTRY_BIN: &str = "/sys/firmware/dmi/tables/smbios_entry_point";
/// DMI table that contains a description of the system's hardware components.
pub const DMIDECODE_BIN: &str = "/sys/firmware/dmi/tables/DMI";

/// Initialize the SQLite database connection and create table if needed.
///
/// # Arguments
///
/// - `path` : File path to save the SQLite database.
/// - `request` : Request to use for database file.
///
/// # Returns
///
/// - A [`Connection`] constructor to initialize database parameters.
/// - An error if table creation or database initialization failed.
fn db_config<P: AsRef<Path>>(path: P, request: &'static str) -> Result<Connection, Box<dyn Error>> {
    let conn = Connection::open(path)?;
    conn.execute_batch(request)?;
    Ok(conn)
}

/// Call [`db_config`] function to initialize and create a SQLite database,
/// and using the static path [`DATABASE`] to save it.
///
/// # Arguments
///
/// - `request` : Request to use for database file.
///
/// # Returns
///
/// - A [`Connection`] constructor to initialize database parameters.
/// - An error if table creation or database initialization failed.
pub fn init_db(request: &'static str) -> Result<Connection, Box<dyn Error>> {
    db_config(DATABASE, request)
}

/// Measure the average variation of a value measurement on a given time interval.
/// We ignore previous data if is not exist.
///
/// # Arguments
///
/// - `measurement` : closure returning the current value of the measurement.
/// - `delay` : Time interval between 2 measures.
///
/// # Return
///
/// The calculated average difference for a measure.
pub fn measure_point<F>(measurement: F, delay: Duration) -> Option<f64>
where
    F: Fn() -> Option<f64>,
{
    let start_value = measurement();
    match start_value {
        None => measurement(),
        Some(next) => {
            let start_time = Instant::now();
            sleep(delay);
            let end_value = measurement();

            match end_value {
                None => None,
                Some(prev) => {
                    if prev < next {
                        None
                    } else {
                        let elapsed = start_time.elapsed().as_secs_f64();
                        Some((prev - next) / elapsed)
                    }
                }
            }
        }
    }
}

//----------------//
// UNIT CODE TEST //
//----------------//

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        sync::atomic::{AtomicUsize, Ordering},
        time::Duration,
    };
    use tempfile::NamedTempFile;

    // Test `init_db` function Appel vers la fonction interne avec chemin temporaire
    #[test]
    fn test_init_db_creates_table_tempfile() {
        let temp = NamedTempFile::new().expect("Temp file creation failed");
        let path = temp.path();

        let sql = "CREATE TABLE IF NOT EXISTS test_table (id INTEGER PRIMARY KEY, value TEXT);";
        let conn = db_config(path, sql).expect("init_db should succeed");

        conn.execute("INSERT INTO test_table (value) VALUES (?1)", &[&"hello"])
            .expect("Insert should succeed");

        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM test_table", [], |row| row.get(0))
            .expect("Select should succeed");

        assert_eq!(count, 1);
    }

    // Test `init_db` function
    #[test]
    fn test_init_db_invalid_sql() {
        let sql = "CREATE TABLE bad_syntax";
        let res = init_db(sql);
        assert!(res.is_err());
    }

    // Test `measure_point` function with increasing value value
    #[test]
    fn test_measure_point_increasing_value() {
        fn get_value() -> f64 {
            static VALUE: AtomicUsize = AtomicUsize::new(1);
            let v = VALUE.fetch_add(1, Ordering::SeqCst);
            v as f64
        }

        let measurement = || Some(get_value());
        let res = measure_point(measurement, Duration::from_millis(50));
        assert!(res.is_some());
        assert!(res.unwrap() > 0.0);
    }

    // Test `measure_point` function with no start value to use for measurement count
    #[test]
    fn test_measure_point_no_init() {
        static VALUE: AtomicUsize = AtomicUsize::new(0);

        let measurement = || {
            let count = VALUE.fetch_add(1, Ordering::SeqCst);
            if count == 0 { None } else { Some(42.0) }
        };

        let res = measure_point(measurement, Duration::from_millis(10));
        assert_eq!(res, Some(42.0));
    }

    // Test `measure_point` function with a previous value bigger than next value retrieved
    #[test]
    fn test_measure_point_error_diff() {
        static VALUE: AtomicUsize = AtomicUsize::new(0);

        let measurement = || {
            let count = VALUE.fetch_add(1, Ordering::SeqCst);
            match count {
                0 => Some(10.0),
                1 => Some(5.0),
                _ => None,
            }
        };

        let res = measure_point(measurement, Duration::from_millis(10));
        assert_eq!(res, None);
    }

    // Test `measure_point` function with count error
    #[test]
    fn test_measure_point_error() {
        static VALUE: AtomicUsize = AtomicUsize::new(0);

        let measurement = || {
            let count = VALUE.fetch_add(1, Ordering::SeqCst);
            if count == 0 { Some(10.0) } else { None }
        };

        let res = measure_point(measurement, Duration::from_millis(10));
        assert_eq!(res, None);
    }
}
