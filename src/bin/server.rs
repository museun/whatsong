use futures::sync::oneshot;
use warp::{self, path, Filter};

use whatsong::Error;
use whatsong::Storage as _;

fn insert(item: whatsong::Item) -> Result<impl warp::Reply, warp::Rejection> {
    log::info!("got an item: {:?}", item);
    if item.version != 1 {
        log::warn!("invalid version: {}. only '1' is suppported", item.version);
        return Err(warp::reject::custom(Error::InvalidVersion {
            got: item.version,
            expected: 1,
        }));
    }

    whatsong::Youtube
        .insert(&item)
        .map(|_| http::StatusCode::ACCEPTED)
        .map_err(warp::reject::custom)
}

fn serialize_ok<T>(item: T) -> impl warp::Reply
where
    T: for<'a> serde::Serialize,
{
    let json = warp::reply::json(&item);
    warp::reply::with_status(json, http::StatusCode::OK)
}

fn current() -> Result<impl warp::Reply, warp::Rejection> {
    whatsong::Youtube
        .current()
        .map(serialize_ok)
        .map_err(warp::reject::custom)
}

fn previous() -> Result<impl warp::Reply, warp::Rejection> {
    whatsong::Youtube
        .previous()
        .map(serialize_ok)
        .map_err(warp::reject::custom)
}

fn try_list() -> Result<impl warp::Reply, warp::Rejection> {
    whatsong::Youtube
        .all()
        .map(serialize_ok)
        .map_err(warp::reject::custom)
}

fn map_err(rejection: warp::Rejection) -> Result<impl warp::Reply, warp::Rejection> {
    match rejection.find_cause::<Error>() {
        Some(Error::InvalidYoutubeUrl(url)) => Ok(warp::reply::with_status(
            warp::reply::json(&serde_json::json!({
                "message": "invalid youtube url",
                "url": url
            })),
            http::StatusCode::NOT_ACCEPTABLE,
        )),

        Some(Error::InvalidVersion { expected, got }) => Ok(warp::reply::with_status(
            warp::reply::json(&serde_json::json!({
                "message": "invalid item version",
                "expected": expected,
                "got": got
            })),
            http::StatusCode::NOT_ACCEPTABLE,
        )),

        Some(Error::InvalidYoutubeData) => Ok(warp::reply::with_status(
            warp::reply::json(&serde_json::json!({
                "message": "invalid youtube data",

            })),
            http::StatusCode::NOT_ACCEPTABLE,
        )),

        Some(Error::InvalidListing { kind }) => Ok(warp::reply::with_status(
            warp::reply::json(&serde_json::json!({
                "message": "invalid item listing",
                "kind": kind

            })),
            http::StatusCode::NOT_ACCEPTABLE,
        )),

        Some(_) => unreachable!(),
        None => Err(rejection),
    }
}

fn main() {
    // TODO make this configurable
    const ADDRESS: &str = "127.0.0.1:58810";

    flexi_logger::Logger::with_env_or_str("whatsong_server=trace")
        .start()
        .unwrap();

    // let port_file = whatsong::get_port_file();
    // if let Ok(..) = port_file.metadata() {
    //     log::error!("port file already exists at: {}", port_file.display());
    //     log::error!("- check for any whatsong_server running");
    //     log::error!("- kill any instances");
    //     log::error!("- then make sure the file is deleted");
    //     std::process::exit(1);
    // };

    whatsong::database::get_connection()
        .execute_batch(include_str!("../sql/schema.sql"))
        .expect("create tables");

    let (tx, rx) = oneshot::channel();

    tokio::run(futures::future::lazy(move || {
        let youtube = warp::path("youtube")
            .and(warp::body::content_length_limit(1024 * 4))
            .and(warp::body::json())
            .and_then(insert)
            .recover(map_err);

        let current = warp::path("current").and_then(current).recover(map_err);

        let previous = warp::path("previous").and_then(previous).recover(map_err);

        let list = path!("list" / "youtube")
            .and(warp::path::end())
            .and_then(try_list)
            // TODO:
            // .or_else(|_| { /* listing not allowed */ })
            .recover(map_err);

        let posts = warp::post2().and(youtube);
        let gets = warp::get2().and(current).or(previous).or(list);

        let route = posts.or(gets);

        let (addr, server) = warp::serve(route).bind_with_graceful_shutdown(
            ADDRESS
                .parse::<std::net::SocketAddr>()
                .expect("valid bind name"),
            rx,
        );
        log::info!("running on: {}", addr);

        warp::spawn(server);
        Ok(())
    }));

    let _ = tx.send(());
}
