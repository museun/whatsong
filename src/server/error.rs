#[derive(Debug)]
pub struct UserError {
    pub error: crate::Error,
}

impl warp::reject::Reject for UserError {}

impl UserError {
    pub fn as_reply(&self) -> Option<warp::reply::Json> {
        let error = match self.error {
            crate::Error::InvalidYoutubeUrl(..) => serde_json::json!({
                "error": "invalid_youtube_url",
                "message": self.error.to_string(),
            }),
            crate::Error::InvalidYoutubeData => serde_json::json!({
                "error": "invalid_youtube_data",
                "message": self.error.to_string(),
            }),
            crate::Error::InvalidVersion { .. } => serde_json::json!({
                "error": "invalid_version",
                "message": self.error.to_string(),
            }),
            crate::Error::InvalidListing { .. } => serde_json::json!({
                "error": "invalid_listing",
                "message": self.error.to_string(),
            }),
            _ => return None,
        };

        Some(warp::reply::json(&error))
    }
}

#[derive(Debug)]
pub struct ServerError {
    pub error: crate::Error,
}

impl ServerError {
    pub fn as_reply(&self) -> Option<warp::reply::Json> {
        let error = match self.error {
            crate::Error::Io { .. } => serde_json::json!({
                "error": "io",
                "message": self.error.to_string(),
            }),
            crate::Error::Sql { .. } => serde_json::json!({
                "error": "sql",
                "message": self.error.to_string(),
            }),
            crate::Error::Deserialize { .. } => serde_json::json!({
                "error": "deserialize",
                "message": self.error.to_string(),
            }),
            crate::Error::Serialize { .. } => serde_json::json!({
                "error": "serialize",
                "message": self.error.to_string(),
            }),
            crate::Error::HttpClient { .. } => serde_json::json!({
                "error": "http_client",
                "message": self.error.to_string(),
            }),
            _ => return None,
        };

        Some(warp::reply::json(&error))
    }
}

impl warp::reject::Reject for ServerError {}
