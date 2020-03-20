use super::{handlers, http};
use warp::{self, Filter, Rejection, Reply};

pub fn insert() -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    // TODO make this generic
    warp::path!("youtube")
        .and(warp::post())
        .and(http::limited_json_body(16 * 1024)) // 16K ought be enough for anyone
        .and_then(handlers::insert)
        .recover(http::recover)
}

pub fn current() -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::path!("current")
        .and(warp::get())
        .and_then(handlers::current)
        .recover(http::recover)
}

pub fn previous() -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::path!("previous")
        .and(warp::get())
        .and_then(handlers::previous)
        .recover(http::recover)
}

pub fn try_list() -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    // TODO make this generic
    warp::path!("list" / "youtube")
        .and(warp::get())
        .and(warp::path::end())
        .and_then(handlers::try_list)
        .recover(http::recover)
}
