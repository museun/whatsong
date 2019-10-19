use std::path::PathBuf;

pub fn timestamp() -> u64 {
    use std::time::SystemTime;
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}

pub fn get_db_path() -> PathBuf {
    "videos.db".into()
    //get_data_dir().join("videos.db")
}

pub fn get_config_path() -> PathBuf {
    let dir = directories::ProjectDirs::from("com.github", "museun", "whatsong").unwrap();
    std::fs::create_dir_all(dir.config_dir()).unwrap();
    dir.config_dir().to_owned()
}

pub fn get_port_file() -> PathBuf {
    get_data_dir().join("port")
}

fn get_data_dir() -> PathBuf {
    let dir = directories::ProjectDirs::from("com.github", "museun", "whatsong").unwrap();
    std::fs::create_dir_all(dir.data_dir()).unwrap();
    dir.data_dir().to_owned()
}
