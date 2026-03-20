pub mod migrations;

use rusqlite::Connection;
use std::sync::Mutex;

/// Wrapper holding separate read and write SQLite connections for use as Tauri
/// managed state. SQLite WAL mode supports concurrent readers, so splitting
/// connections prevents the long-running sync/write operations from blocking
/// read-only queries.
pub struct DbState {
    pub writer: Mutex<Connection>,
    pub reader: Mutex<Connection>,
}

/// Initialise the database, creating the file and running migrations.
/// Returns two connections: one for writes and one for reads.
pub fn initialise(
    app_data_dir: &std::path::Path,
) -> Result<(Connection, Connection), rusqlite::Error> {
    // Ensure the data directory exists
    std::fs::create_dir_all(app_data_dir)
        .map_err(|_e| rusqlite::Error::InvalidPath(app_data_dir.to_path_buf()))?;

    let db_path = app_data_dir.join("pr_companion.db");

    // Writer connection — runs migrations and handles all mutations
    let writer = Connection::open(&db_path)?;
    writer.execute_batch("PRAGMA journal_mode = WAL; PRAGMA foreign_keys = ON;")?;
    writer.execute_batch("PRAGMA synchronous = NORMAL")?;
    writer.execute_batch("PRAGMA cache_size = -32000")?;
    writer.execute_batch("PRAGMA wal_autocheckpoint = 400;")?;

    // Run schema migrations on the writer connection
    migrations::run_migrations(&writer)?;

    // Reader connection — used for SELECT-only queries, avoids contention
    let reader = Connection::open(&db_path)?;
    reader.execute_batch("PRAGMA journal_mode = WAL; PRAGMA foreign_keys = ON;")?;
    reader.execute_batch("PRAGMA synchronous = NORMAL")?;
    reader.execute_batch("PRAGMA cache_size = -8000")?;

    Ok((writer, reader))
}
