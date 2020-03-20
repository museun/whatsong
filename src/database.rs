use anyhow::Context as _;
use once_cell::sync::OnceCell;
use rusqlite::{Connection, OpenFlags};

static DB_CONN_STRING: OnceCell<String> = OnceCell::new();

pub fn initialize_db_conn_string(conn_str: impl ToString) {
    DB_CONN_STRING.get_or_init(|| conn_str.to_string());
}

pub fn get_global_connection() -> anyhow::Result<Connection> {
    if cfg!(test) {
        // TODO do something here
        panic!("no dbs in test")
    }

    DB_CONN_STRING
        .get()
        .ok_or_else(|| anyhow::anyhow!("DB_CONN_STRING is not set"))
        .and_then(|conn| get_connection(conn.as_str()))
}

fn get_connection(conn_str: &str) -> anyhow::Result<Connection> {
    Connection::open_with_flags(
        conn_str,
        OpenFlags::SQLITE_OPEN_SHARED_CACHE
            | OpenFlags::SQLITE_OPEN_READ_WRITE
            | OpenFlags::SQLITE_OPEN_CREATE,
    )
    .with_context(|| anyhow::anyhow!("cannot open db, conn_string: {}", conn_str.escape_debug()))
}
