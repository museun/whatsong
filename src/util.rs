use std::path::PathBuf;

pub fn timestamp() -> u64 {
    use std::time::SystemTime;
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}

pub fn get_db_path() -> PathBuf {
    get_data_dir().join("videos.db")
}

pub fn get_port_file() -> PathBuf {
    get_data_dir().join("port")
}

fn get_data_dir() -> PathBuf {
    let dir = directories::ProjectDirs::from("com.github", "museun", "whatsong").unwrap();
    std::fs::create_dir_all(dir.data_dir()).unwrap();
    dir.data_dir().to_owned()
}

pub trait UnwrapOrAbort<T> {
    fn unwrap_or_abort(self, msg: &str) -> T;
}

impl<T, E> UnwrapOrAbort<T> for Result<T, E>
where
    E: std::fmt::Display,
{
    fn unwrap_or_abort(self, msg: &str) -> T {
        self.unwrap_or_else(|err| {
            let _ = std::fs::write("c:/dev/client.log", format!("error: {}, {}", msg, err));
            std::process::exit(1)
        })
    }
}

impl<T> UnwrapOrAbort<T> for Option<T> {
    fn unwrap_or_abort(self, msg: &str) -> T {
        self.unwrap_or_else(|| {
            let _ = std::fs::write("c:/dev/client.log", format!("error: {}", msg));
            std::process::exit(1)
        })
    }
}
