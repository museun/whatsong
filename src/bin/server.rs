use std::net::ToSocketAddrs;
use std::path::PathBuf;
use std::sync::mpsc::channel;
use std::sync::Arc;

use regex::Regex;
use serde::Serialize;
use whatsong::Error;
use whatsong::Storage as _;

type Result<T> = std::result::Result<T, Error>;

struct Server {
    server: tiny_http::Server,
    addr: String,
}

impl Server {
    pub fn create<A>(addr: A) -> Result<Self>
    where
        A: ToSocketAddrs + std::fmt::Display + std::fmt::Debug + Clone,
    {
        let server = tiny_http::Server::http(addr.clone()).map_err(|err| {
            log::error!("cannot bind http server at {}: {}", addr, err);
            Error::BindHttp(addr.to_string())
        })?;

        let addr = match server.server_addr() {
            std::net::SocketAddr::V4(addr) => addr.to_string(),
            std::net::SocketAddr::V6(addr) => addr.to_string(),
        };
        log::info!("started whatsong server at: {}", addr);

        Ok(Self { server, addr })
    }

    pub fn run(&self, port_file: PathBuf) {
        let re = Regex::new(r#"/list/(?P<ty>\w.*?)(/|$)"#).expect("compileregex");
        std::fs::write(port_file, &self.addr).expect("write port file");
        for req in self.server.incoming_requests() {
            if let Err(err) = Self::handle(req, &re) {
                log::error!("processing request failed: {}", err);
            }
        }
    }

    pub fn stop(&self) {
        self.server.unblock()
    }

    fn handle(mut req: tiny_http::Request, re: &Regex) -> Result<()> {
        log::trace!("{} {}", req.method(), req.url());

        macro_rules! err {
            ($req:expr) => {{
                log::debug!("unknown {} on {}", $req.method(), req.url());
                return $req
                    .respond(tiny_http::Response::empty(400))
                    .map_err(Error::Io);
            }};
        }

        use tiny_http::Method::*;
        use whatsong::Youtube;

        match (req.method(), req.url()) {
            (Get, "/current") => Self::respond((Youtube.current()?, req)),
            (Get, "/previous") => Self::respond((Youtube.previous()?, req)),
            (Get, other) => {
                let ns = re
                    .captures(other)
                    .and_then(|c| c.name("ty"))
                    .map(|s| s.as_str().to_lowercase());
                match ns.unwrap_or_default().as_str() {
                    "youtube" => Self::respond(Self::check(Youtube.all(), req)?),
                    _ => err!(req),
                }
            }
            (Post, "/youtube") => {
                let item: whatsong::Item =
                    serde_json::from_reader(req.as_reader()).map_err(Error::Serialize)?;
                if item.version != 1 {
                    return req
                        .respond(tiny_http::Response::empty(400))
                        .map_err(Error::Io);
                }

                // if let whatsong::ItemKind::Youtube(..) = item.kind {
                Youtube.insert(&item)
                // }
            }
            _ => err!(req),
        }
    }

    fn check<T>(res: Result<T>, req: tiny_http::Request) -> Result<(T, tiny_http::Request)> {
        match res {
            Ok(ok) => Ok((ok, req)),
            Err(err) => req
                .respond(tiny_http::Response::empty(500))
                .map_err(Error::Io)
                .and_then(|_| Err(err)),
        }
    }

    fn respond<T: Serialize>((res, req): (T, tiny_http::Request)) -> Result<()> {
        match serde_json::to_vec(&res).map_err(whatsong::Error::Serialize) {
            Ok(data) => req
                .respond(tiny_http::Response::from_data(data))
                .map_err(Error::Io),
            Err(err) => req
                .respond(tiny_http::Response::empty(400))
                .map_err(Error::Io)
                .and_then(|_| Err(err)),
        }
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        let port_file = whatsong::get_port_file();
        std::fs::remove_file(port_file).expect("delete port file")
    }
}

fn main() {
    flexi_logger::Logger::with_env_or_str("whatsong_server=trace")
        .start()
        .unwrap();

    let (tx, rx) = channel::<()>();
    ctrlc::set_handler(move || {
        let _ = tx.send(());
    })
    .unwrap();

    let port_file = whatsong::get_port_file();
    if let Ok(..) = port_file.metadata() {
        log::error!("port file already exists at: {}", port_file.display());
        log::error!("- check for any whatsong_server running");
        log::error!("- kill any instances");
        log::error!("- then make sure the file is deleted");
        std::process::exit(1);
    };

    whatsong::database::get_connection()
        .execute_batch(include_str!("../sql/schema.sql"))
        .expect("create tables");

    let server = match Server::create("localhost:0") {
        Ok(server) => Arc::new(server),
        Err(err) => {
            log::error!("cannot start server: {}", err);
            std::process::exit(1);
        }
    };

    let clone = Arc::clone(&server);
    let handle = std::thread::spawn(move || clone.run(port_file));
    let _ = rx.recv();

    server.stop();
    handle.join().unwrap();
}
