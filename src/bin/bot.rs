use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use twitchchat::IntoChannel as _;
use twitchchat::{commands::PrivMsg, Writer};

#[derive(Debug, Serialize, Deserialize)]
struct Configuration {
    pub nickname: String,
    pub channel: String,
    pub commands: CommandAliases,
}

impl Default for Configuration {
    fn default() -> Self {
        Configuration {
            nickname: "shaken_bot".to_string(),
            channel: "museun".to_string(),
            commands: CommandAliases::default(),
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
    // let file = whatsong::get_port_file();
    // let addr = std::fs::read_to_string(&file).map_err(whatsong::Error::Io)?;
    let addr = "127.0.0.1:58810";
    let url = format!("http://{}/{}", addr, req.as_ep());
    log::trace!("getting: {}", url);

    attohttpc::get(&url)
        .send()
        .map_err(whatsong::Error::HttpRequest)?
        .json()
        .map_err(whatsong::Error::HttpResponse)
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct CommandAliases {
    pub previous: Vec<String>,
    pub current: Vec<String>,
    pub list: Vec<String>,
}

impl Default for CommandAliases {
    fn default() -> Self {
        Self {
            previous: vec!["!previous".to_string()],
            current: vec!["!current".to_string()],
            list: vec!["!songlist".to_string()],
        }
    }
}

impl CommandAliases {
    fn parse_user(&self, user: &str) -> Command {
        if self.previous.iter().any(|s| s == user) {
            return Command::Previous;
        }
        if self.current.iter().any(|s| s == user) {
            return Command::Current;
        }
        if self.list.iter().any(|s| s == user) {
            return Command::List;
        }
        Command::Unknown
    }
}

#[derive(Debug, Copy, Clone)]
enum Command {
    Previous,
    Current,
    List,
    Unknown,
}

fn handle_args(path: &std::path::Path) {
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
}

fn current_song() -> Option<whatsong::youtube::Song> {
    http_get(ReqType::Current).map_err(|err| dbg!(err)).ok()
}

fn previous_song() -> Option<whatsong::youtube::Song> {
    http_get(ReqType::Previous).map_err(|err| dbg!(err)).ok()
}

#[allow(dead_code)]
fn song_list() -> Option<whatsong::youtube::Song> {
    http_get(ReqType::List).map_err(|err| dbg!(err)).ok()
}

trait TryWrite {
    fn try_write<D: std::fmt::Display>(&mut self, msg: &PrivMsg, data: D);
}

impl TryWrite for Writer {
    fn try_write<D: std::fmt::Display>(&mut self, msg: &PrivMsg, data: D) {
        if let Err(err) = self.send(msg.channel(), data) {
            log::error!("cannot write to twitch: {}", err);
            std::process::exit(1);
        }
    }
}

fn handle_message(msg: PrivMsg, writer: &mut Writer, commands: &CommandAliases) {
    use chrono::prelude::*;
    let now = Utc::now();

    if let Some(cmd) = msg.message().split_whitespace().next() {
        let (style, song) = match commands.parse_user(cmd) {
            d @ Command::Previous => (d, previous_song()),
            d @ Command::Current => (d, current_song()),
            // d @ Command::List => (d, song_list()),
            // ignore the song list for now
            Command::Unknown | _ => return,
        };

        log::trace!("got a song: ({:?}) {:?}", style, song);

        match (style, song) {
            (Command::Previous, Some(song)) => {
                writer.try_write(
                    &msg,
                    format!(
                        "previous song: {} @ https://youtu.be/{}",
                        song.title, song.vid,
                    ),
                );
            }

            (Command::Current, Some(song)) => {
                let delta = calculate_offset(song.timestamp, song.duration, now);
                if delta == 0 {
                    writer.try_write(
                        &msg,
                        format!(
                            "current song: {} @ https://youtu.be/{}",
                            song.title, song.vid,
                        ),
                    );
                    return;
                }

                writer.try_write(
                    &msg,
                    format!(
                        "current song: {} @ https://youtu.be/{}?t={}",
                        song.title, song.vid, delta,
                    ),
                );
                return;
            }

            _ => writer.try_write(&msg, "I don't think any song is currently playing"),
        }
    }
}

fn calculate_offset(ts: i64, dur: i64, now: chrono::DateTime<chrono::Utc>) -> i64 {
    use chrono::prelude::*;
    let start = Utc.timestamp(ts, 0);
    let dur = chrono::Duration::from_std(std::time::Duration::from_secs(dur as u64)).unwrap();
    let time = dur - (now - start);
    (dur - time).num_seconds()
}

fn main() {
    flexi_logger::Logger::with_env_or_str("whatsong_bot=debug")
        .format(|write, now, record| {
            write!(
                write,
                "[{}] [{}] {}",
                now.now().format("%H:%M:%S%.6f"),
                record.level(),
                &record.args()
            )
        })
        .start()
        .unwrap();

    let path = whatsong::get_config_path().join("bot.toml");
    handle_args(&path);
    let config = Configuration::load(path);

    let active_channel: twitchchat::Channel =
        config.channel.clone().into_channel().unwrap_or_else(|err| {
            log::error!("invalid twitch channel: {}", err);
            std::process::exit(1);
        });

    let token = std::env::var("WHATSONG_TWITCH_PASSWORD").unwrap_or_else(|_| {
        log::error!("please set `WHATSONG_TWITCH_PASSWORD` to a valid twitch token");
        std::process::exit(1)
    });

    let user_config = twitchchat::UserConfig::with_caps()
        .nick(&config.nickname)
        .token(&token)
        .build()
        .unwrap();

    log::debug!("connecting to twitch");
    let client = twitchchat::connect(&user_config).unwrap_or_else(|err| {
        log::error!("cannot connect to twitch: {}", err);
        std::process::exit(1);
    });
    log::info!("connected to twitch");

    let commands = config.commands;

    let mut writer = client.writer();
    for event in client.filter::<PrivMsg>() {
        match event {
            twitchchat::Event::TwitchReady(local) => {
                log::info!(
                    "connected to twitch as: {} ({})",
                    local.display_name.as_ref().unwrap_or_else(|| &local.name),
                    local.user_id
                );
                let _ = writer.join(&config.channel);
            }
            twitchchat::Event::Message(twitchchat::Message::PrivMsg(msg)) => {
                if *msg.channel() != active_channel {
                    log::warn!(
                        "only one active channel is allowed. and its not {}, but rather {}",
                        msg.channel(),
                        active_channel
                    );
                    continue;
                }
                handle_message(msg, &mut writer, &commands)
            }

            twitchchat::Event::Error(error) => {
                log::error!("error from twitch: {}", error);
                std::process::exit(1);
            }
            _ => {}
        }
    }
}
