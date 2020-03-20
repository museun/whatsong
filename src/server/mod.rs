use warp::Filter as _;

mod error;
mod handlers;
mod http;
mod routes;

pub async fn run_server(address: std::net::SocketAddr) -> anyhow::Result<()> {
    let youtube = routes::insert()
        .or(routes::current())
        .or(routes::previous())
        .or(routes::try_list());

    let routes = youtube.with(warp::log("whatsong"));

    warp::serve(routes).run(address).await;
    Ok(())
}
