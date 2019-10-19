pub type Result<T> = std::result::Result<T, Error>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("an i/o error: {0}")]
    Io(#[from] std::io::Error),

    #[error("an sql error: {0}")]
    Sql(#[from] rusqlite::Error),

    #[error("json deserialize error: {0}")]
    Deserialize(#[source] serde_json::Error),

    #[error("json serialize error: {0}")]
    Serialize(#[source] serde_json::Error),

    #[error("json serialize error: {0}")]
    HttpResponse(#[source] attohttpc::Error),

    #[error("json serialize error: {0}")]
    HttpRequest(#[source] attohttpc::Error),

    #[error("invalid youtube url: {0}")]
    InvalidYoutubeUrl(String),

    #[error("invalid youtube data")]
    InvalidYoutubeData, // context?

    #[error("invalid item version: expected: {expected}, got: {got}")]
    InvalidVersion { expected: u32, got: u32 },

    #[error("invalid item listing: {kind}")]
    InvalidListing { kind: String },
}
