use std::path::PathBuf;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use twitchchat::{commands::PrivMsg, sync_adapters, Client};

#[derive(Debug, Serialize, Deserialize)]
struct Configuration {
    pub nickname: String,
    pub channel: String,
    pub commands: Command,
}

impl Default for Configuration {
    fn default() -> Self {
        Configuration {
            nickname: "shaken_bot".to_string(),
            channel: "museun".to_string(),
            commands: Command::default(),
        }
    }
}

impl Configuration {
    fn load(path: PathBuf) -> Self {
        std::fs::read_to_string(&path)
            .map_err(|_| {
                let s = toml::to_string_pretty(&Self::default()).expect("valid toml");
                std::fs::write(&path, &s).expect("valid HOME directory");
                log::error!("created a default configuration at: {}", path.display());
                log::error!("edit it and rerun this.");
                std::process::exit(1);
            })
            .and_then(|data| toml::from_str(&data))
            .map_err(|err| {
                log::error!("invalid toml because: {}.", err,);
                log::error!("please review it at: {}", path.display());
                std::process::exit(1);
            })
            .unwrap_or_default()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Command {
    pub previous: Vec<String>,
    pub current: Vec<String>,
    pub list: Vec<String>,
}

impl Default for Command {
    fn default() -> Self {
        Command {
            previous: vec!["!previous".to_string()],
            current: vec!["!current".to_string()],
            list: vec!["!songlist".to_string()],
        }
    }
}

trait LogUnwrapAndQuit<T> {
    fn unwrap_or_exit(self, msg: &str) -> T;
}

impl<T, E: std::fmt::Display> LogUnwrapAndQuit<T> for Result<T, E> {
    fn unwrap_or_exit(self, msg: &str) -> T {
        self.unwrap_or_else(|err| {
            log::error!("error: {}. because: {}", msg, err);
            std::process::exit(1);
        })
    }
}

struct CommandHandler {
    active_channel: twitchchat::Channel,
    router: CommandRouter,
    writer: twitchchat::Writer,
}

impl twitchchat::Handler for CommandHandler {
    fn on_priv_msg(&mut self, msg: Arc<PrivMsg>) {
        if *msg.channel() != self.active_channel {
            log::info!(
                "only one active channel is allowed. and its not {}, but rather {}",
                msg.channel(),
                &self.active_channel
            );
            return;
        }

        use chrono::prelude::*;
        let now = Utc::now();

        use chrono::Duration as CDur;
        use std::time::Duration as SDur;

        if let Some(cmd) = msg.message().split_whitespace().next() {
            let (style, song) = match self.router.parse(cmd) {
                d @ Commands::Previous => (d, self.previous_song()),
                d @ Commands::Current => (d, self.current_song()),
                // d @ Commands::List => (d, self.song_list()),
                // ignore the song list for now
                Commands::Unknown | _ => return,
            };

            log::trace!("got a song: ({:?}) {:?}", style, song);

            match (style, song) {
                (Commands::Previous, Some(song)) => {
                    let out = format!(
                        "previous song: {} @ https://youtu.be/{}",
                        song.title, song.vid,
                    );
                    let _ = self.writer.send(msg.channel(), out);
                }

                (Commands::Current, Some(song)) => {
                    let start = Utc.timestamp(song.timestamp as i64, 0);
                    let dur = CDur::from_std(SDur::from_secs(song.duration as u64)).unwrap();

                    let time = dur - (now - start);
                    let delta = (dur - time).num_seconds();

                    let out = if delta > 0 {
                        format!(
                            "current song: {} @ https://youtu.be/{}&t={}",
                            song.title, song.vid, delta,
                        )
                    } else {
                        format!(
                            "current song: {} @ https://youtu.be/{}",
                            song.title, song.vid,
                        )
                    };
                    let _ = self.writer.send(msg.channel(), out);
                }

                (..) => {
                    let _ = self
                        .writer
                        .send(msg.channel(), "I don't think any song is currently playing");
                }
            }
        }
    }
}

impl CommandHandler {
    fn current_song(&self) -> Option<whatsong::youtube::Song> {
        http_get(ReqType::Current).map_err(|err| dbg!(err)).ok()
    }

    fn previous_song(&self) -> Option<whatsong::youtube::Song> {
        http_get(ReqType::Previous).map_err(|err| dbg!(err)).ok()
    }

    #[allow(dead_code)]
    fn song_list(&self) -> Option<whatsong::youtube::Song> {
        http_get(ReqType::List).map_err(|err| dbg!(err)).ok()
    }
}

#[derive(Copy, Clone, Debug)]
enum ReqType {
    Current,
    Previous,
    #[allow(dead_code)]
    List,
}

impl ReqType {
    fn as_ep(&self) -> &str {
        match self {
            ReqType::Current => "current",
            ReqType::Previous => "previous",
            ReqType::List => "list/youtube",
        }
    }
}

fn http_get<T>(req: ReqType) -> Result<T, whatsong::Error>
where
    for<'de> T: Deserialize<'de>,
{
    let file = whatsong::get_port_file();
    let addr = std::fs::read_to_string(&file).map_err(whatsong::Error::Io)?;
    attohttpc::get(&format!("http://{}/{}", addr, req.as_ep()))
        .send()
        .map_err(whatsong::Error::HttpRequest)?
        .json()
        .map_err(whatsong::Error::HttpResponse)
}

#[derive(Debug, Copy, Clone)]
enum Commands {
    Previous,
    Current,
    List,
    Unknown,
}

struct CommandRouter {
    aliases: Command,
}

impl CommandRouter {
    fn parse(&self, left: &str) -> Commands {
        if self.aliases.previous.iter().any(|s| s == left) {
            return Commands::Previous;
        }
        if self.aliases.current.iter().any(|s| s == left) {
            return Commands::Current;
        }
        if self.aliases.list.iter().any(|s| s == left) {
            return Commands::List;
        }
        Commands::Unknown
    }
}

fn main() {
    flexi_logger::Logger::with_env_or_str("whatsong_bot=trace")
        .start()
        .unwrap();

    let path = whatsong::get_config_path().join("bot.toml");
    if let Some(cmd) = std::env::args().nth(1) {
        match cmd.as_str() {
            "config" => {
                println!("config: {}", path.display());
                std::process::exit(0);
            }
            "default" => {
                println!(
                    "{}",
                    toml::to_string_pretty(&Configuration::default()).unwrap()
                );
                std::process::exit(0);
            }
            "help" => {
                let name = std::env::current_exe().unwrap();
                let name = std::path::Path::new(&name)
                    .file_stem()
                    .unwrap()
                    .to_string_lossy();
                println!("commands for [{}]:", name);
                println!("    {} help    <-- prints out this message", name);
                println!("    {} config  <-- prints the config path location", name);
                println!(
                    "    {} default <-- prints out a default configuration",
                    name
                );
                std::process::exit(0);
            }
            _ => {}
        }
    }

    let config = Configuration::load(path);

    let token = std::env::var("WHATSONG_TWITCH_PASSWORD").unwrap_or_else(|_| {
        log::error!("please set `WHATSONG_TWITCH_PASSWORD` to a valid twitch token");
        std::process::exit(1)
    });

    use std::net::TcpStream;

    let (read, write) = match TcpStream::connect(twitchchat::TWITCH_IRC_ADDRESS) {
        Ok(conn) => (conn.try_clone().unwrap(), conn),
        Err(err) => {
            log::error!(
                "cannot connect to `{}` because: {}",
                twitchchat::TWITCH_IRC_ADDRESS,
                err
            );
            std::process::exit(1)
        }
    };

    log::info!("connected to twitch");
    let (read, write) = sync_adapters(read, write);
    let mut client = Client::new(read, write);

    let _ = client.handler(CommandHandler {
        active_channel: config.channel.clone().into(),
        router: CommandRouter {
            aliases: config.commands.clone(),
        },
        writer: client.writer(),
    });

    let user_config = twitchchat::UserConfig::with_caps()
        .nick(&config.nickname)
        .token(&token)
        .build()
        .unwrap();

    log::info!("registering");
    client
        .register(user_config)
        .unwrap_or_exit("cannot register with twitch. probably an invalid token");
    log::info!("registered");

    let _info = client.wait_for_ready();

    log::info!("joining: {}", &config.channel);
    client
        .writer()
        .join(config.channel.clone()) // hmm
        .unwrap_or_exit(&format!("cannot join channel `{}`", &config.channel));

    log::info!("starting run loop");
    client
        .run()
        .unwrap_or_exit("ran into a problem while running the bot")
}
