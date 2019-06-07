use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub struct Item {
    pub kind: ItemKind,
    pub ts: i64,
    pub version: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ItemKind {
    Youtube(String),
}

pub trait Storage<T>
where
    T: FromRow,
{
    fn insert(&self, item: &Item) -> Result<()>;
    fn current(&self) -> Result<T>;
    fn previous(&self) -> Result<T>;
    fn all(&self) -> Result<Vec<T>>;
}

pub trait FromRow {
    fn from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Self>
    where
        Self: Sized;
    fn timestamp(&self) -> i64;
}

pub mod database;

mod util;
pub use util::*;

mod error;
pub use error::*;

pub mod youtube;
pub use youtube::Youtube;
