use warp::{
    self,
    http::StatusCode,
    reply::{Json, WithStatus},
    Filter, Rejection, Reply,
};

use super::error::{ServerError, UserError};

pub async fn recover(err: Rejection) -> Result<impl Reply, Rejection> {
    if let Some(err) = err
        .find::<UserError>()
        .and_then(|err| err.as_reply())
        .or_else(|| err.find::<ServerError>().and_then(|err| err.as_reply()))
    {
        return Ok(warp::reply::with_status(
            err,
            warp::http::StatusCode::INTERNAL_SERVER_ERROR,
        ));
    }

    Err(err)
}

pub fn user_error(error: crate::Error) -> Rejection {
    warp::reject::custom(UserError { error })
}

pub fn server_error(error: crate::Error) -> Rejection {
    warp::reject::custom(ServerError { error })
}

pub fn okay<T>(item: T) -> WithStatus<Json>
where
    T: serde::Serialize,
{
    warp::reply::with_status(warp::reply::json(&item), StatusCode::OK)
}

pub fn limited_json_body<T>(max: u64) -> impl Filter<Extract = (T,), Error = Rejection> + Clone
where
    for<'de> T: Send + serde::Deserialize<'de>,
{
    warp::body::content_length_limit(max).and(warp::body::json())
}
