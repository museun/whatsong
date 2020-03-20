pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    Io { error: std::io::Error },
    Sql { error: rusqlite::Error },
    Deserialize { error: serde_json::Error },
    Serialize { error: serde_json::Error },
    HttpClient { error: reqwest::Error },
    InvalidYoutubeUrl(String),
    InvalidYoutubeData, // context?
    InvalidVersion { expected: u32, got: u32 },
    InvalidListing { kind: String },
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Self::Io { error }
    }
}

impl From<rusqlite::Error> for Error {
    fn from(error: rusqlite::Error) -> Self {
        Self::Sql { error }
    }
}

impl From<reqwest::Error> for Error {
    fn from(error: reqwest::Error) -> Self {
        Self::HttpClient { error }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io { error } => write!(f, "an i/o error: {}", error),
            Self::Sql { error } => write!(f, "an sql error: {}", error),
            Self::Deserialize { error } => write!(f, "json deserialize error: {}", error),
            Self::Serialize { error } => write!(f, "json serialize error: {}", error),
            Self::HttpClient { error } => write!(f, "http client error: {}", error),
            Self::InvalidYoutubeUrl(url) => write!(f, "invalid youtube url: {}", url),
            Self::InvalidYoutubeData => write!(f, "invalid youtube data"),
            Self::InvalidVersion { expected, got } => write!(
                f,
                "invalid item version: expected: {}, got: {}",
                expected, got
            ),
            Self::InvalidListing { kind } => write!(f, "invalid item listing: {}", kind),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io { error } => Some(error),
            Self::Sql { error } => Some(error),
            Self::Deserialize { error } => Some(error),
            Self::Serialize { error } => Some(error),
            Self::HttpClient { error } => Some(error),
            _ => None,
        }
    }
}
