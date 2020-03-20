use anyhow::Context as _;

fn get_address() -> anyhow::Result<std::net::SocketAddr> {
    // TODO make this port configurable
    const ADDRESS: &str = "127.0.0.1:58810";

    std::env::var("WHATSONG_SERVER_ADDRESS")
        .ok()
        .unwrap_or_else(|| ADDRESS.to_string())
        .parse()
        .with_context(|| "cannot parse the server address into a SocketAddr")
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    simple_env_load::load_env_from(&[".env"]);
    alto_logger::init(alto_logger::Style::MultiLine, Default::default())?;
    let address = get_address()?;

    whatsong::init_api_keys_from_env()?;
    whatsong::database::initialize_db_conn_string(
        whatsong::Directories::db_path()?
            .to_string_lossy()
            .to_string(),
    );
    whatsong::database::get_global_connection()?
        .execute_batch(include_str!("../sql/schema.sql"))
        .with_context(|| "must be able to create the initial table schema")?;

    whatsong::server::run_server(address).await
}
