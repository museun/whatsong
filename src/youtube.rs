use super::*;

use anyhow::Context as _;
use once_cell::sync::{Lazy, OnceCell};
use regex::Regex;
use serde::{Deserialize, Serialize};

static PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"(?-u)(:?^(:?http?.*?youtu(:?\.be|be.com))(:?/|.*?v=))(?P<id>[A-Za-z0-9_-]{11})"#)
        .expect("valid regex")
});

static API_KEY: OnceCell<String> = OnceCell::new();

pub(crate) fn initialize_api_key() -> anyhow::Result<()> {
    const YOUTUBE_API_KEY: &str = "YOUTUBE_API_KEY";

    let key = std::env::var(YOUTUBE_API_KEY)
        .with_context(|| format!("environment var `{}` must be set", YOUTUBE_API_KEY))?;

    API_KEY
        .set(key)
        .map_err(|err| anyhow::anyhow!("existing value: {}", err))
        .with_context(|| format!("`{}` was already set", YOUTUBE_API_KEY))
}

#[derive(Default)]
pub struct Youtube;

#[async_trait::async_trait]
impl Storage<Song> for Youtube {
    async fn insert(&self, item: super::Item) -> anyhow::Result<()> {
        let ItemKind::Youtube(url) = item.kind;
        let ts = item.ts;

        let id: String = PATTERN
            .captures(&url)
            .and_then(|s| s.name("id"))
            .map(|s| s.as_str().to_string())
            .ok_or_else(|| Error::InvalidYoutubeUrl(url.to_string()))?;

        let info = Item::fetch(&id).await?;

        tokio::task::spawn_blocking(move || {
            database::get_global_connection().and_then(|conn| {
                conn.execute_named(
                    include_str!("sql/youtube/add_video.sql"),
                    &[
                        (":vid", &id),
                        (":ts", &ts),
                        (":duration", &info.duration),
                        (":title", &info.title),
                    ],
                )
                .map_err(|error| Error::Sql { error })
                .map_err(Into::into)
                .map(|_| ())
            })
        })
        .await
        .unwrap()
    }

    async fn current(&self) -> anyhow::Result<Song> {
        tokio::task::spawn_blocking(move || {
            database::get_global_connection().and_then(|conn| {
                conn.query_row(
                    include_str!("sql/youtube/get_current.sql"),
                    rusqlite::NO_PARAMS,
                    Song::from_row,
                )
                .map_err(|error| Error::Sql { error })
                .map_err(Into::into)
            })
        })
        .await
        .unwrap()
    }

    async fn previous(&self) -> anyhow::Result<Song> {
        tokio::task::spawn_blocking(move || {
            database::get_global_connection().and_then(|conn| {
                conn.query_row(
                    include_str!("sql/youtube/get_previous.sql"),
                    rusqlite::NO_PARAMS,
                    Song::from_row,
                )
                .map_err(|error| Error::Sql { error })
                .map_err(Into::into)
            })
        })
        .await
        .unwrap()
    }

    async fn all(&self) -> anyhow::Result<Vec<Song>> {
        tokio::task::spawn_blocking(move || {
            database::get_global_connection().and_then(|conn| {
                Ok(conn
                    .prepare(include_str!("sql/youtube/get_all.sql"))?
                    .query_map(rusqlite::NO_PARAMS, Song::from_row)
                    .map_err(|error| Error::Sql { error })?
                    .flatten()
                    .collect::<Vec<_>>())
            })
        })
        .await
        .unwrap()
    }
}

pub struct Item {
    pub title: String,
    pub duration: i64,
}

impl Item {
    pub async fn fetch(id: &str) -> Result<Self> {
        #[derive(Serialize)]
        struct Query<'a> {
            id: &'a str,
            part: &'a str,
            fields: &'a str,
            key: &'a str,
        }

        #[derive(Deserialize)]
        struct Response {
            items: Vec<Item>,
        }

        #[derive(Deserialize)]
        struct Item {
            snippet: Snippet,
            #[serde(rename = "contentDetails")]
            details: ContentDetails,
        }

        #[derive(Deserialize)]
        struct Snippet {
            title: String,
        }

        #[derive(Deserialize)]
        struct ContentDetails {
            duration: String,
        }

        let resp: Response = reqwest::Client::new()
            .get("https://www.googleapis.com/youtube/v3/videos")
            .query(&Query {
                part: "snippet,contentDetails",
                fields: "items(id, snippet(title), contentDetails(duration))",
                key: API_KEY.get().expect("api key must be set").as_str(),
                id,
            })
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;

        resp.items
            .get(0)
            // TODO context
            .ok_or_else(|| Error::InvalidYoutubeData)
            .map(|item| Self {
                title: item.snippet.title.to_string(),
                duration: from_iso8601(&item.details.duration),
            })
            .map_err(Into::into)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Song {
    pub id: i64,
    pub vid: String,
    pub timestamp: i64,
    pub duration: i64,
    pub title: String,
}

impl FromRow for Song {
    fn from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Self> {
        Ok(Self {
            id: row.get(0)?,
            vid: row.get(1)?,
            timestamp: row.get(2)?,
            duration: row.get(3)?,
            title: row.get(4)?,
        })
    }

    fn timestamp(&self) -> i64 {
        self.timestamp
    }
}

#[inline]
fn from_iso8601(period: &str) -> i64 {
    let parse = |s, e| period[s + 1..e].parse::<i64>().unwrap_or_default();
    period
        .chars()
        .enumerate()
        .fold((0, 0), |(a, p), (i, c)| match c {
            c if c.is_numeric() => (a, p),
            'H' => (a + parse(p, i) * 60 * 60, i),
            'M' => (a + parse(p, i) * 60, i),
            'S' => (a + parse(p, i), i),
            'P' | 'T' | _ => (a, i),
        })
        .0
}
