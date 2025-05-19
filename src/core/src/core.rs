use rusqlite::Connection;
use std::{
    error::Error,
    path::Path,
    thread::sleep,
    time::{Duration, Instant},
};

const DATABASE: &str = "log/data.db";

/// Initialize the SQLite database and create table if needed.
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
    let conn = Connection::open(Path::new(DATABASE))?;
    conn.execute_batch(request)?;
    Ok(conn)
}

/// Measure the average variation of a value measurement on a given time interval.
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
    let start_value = measurement()?;
    let start_time = Instant::now();
    sleep(delay);
    let end_value = measurement()?;
    let elapsed = start_time.elapsed().as_secs_f64();
    Some((end_value - start_value) / elapsed)
}
