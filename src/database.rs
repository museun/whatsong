use std::path::PathBuf;

use once_cell::sync::Lazy;
pub static DB_PATH: Lazy<PathBuf> = Lazy::new(crate::get_db_path);

pub fn get_connection() -> rusqlite::Connection {
    if cfg!(test) {
        // TODO do something here
        panic!("no dbs in test")
    }

    rusqlite::Connection::open(&*DB_PATH).expect("connect to database")
}
