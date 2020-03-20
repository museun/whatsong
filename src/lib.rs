use serde::{Deserialize, Serialize};

pub const CURRENT_API_VERSION: u32 = 1;

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

#[async_trait::async_trait]
pub trait Storage<T>
where
    T: FromRow,
{
    async fn insert(&self, item: Item) -> anyhow::Result<()>;
    async fn current(&self) -> anyhow::Result<T>;
    async fn previous(&self) -> anyhow::Result<T>;
    async fn all(&self) -> anyhow::Result<Vec<T>>;
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
pub use error::{Error, Result};

pub mod server;

pub mod youtube;
pub use youtube::Youtube;

pub fn init_api_keys_from_env() -> anyhow::Result<()> {
    youtube::initialize_api_key()
}
