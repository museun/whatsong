use anyhow::Context as _;
use std::path::PathBuf;

pub fn timestamp() -> u64 {
    use std::time::SystemTime;
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}

pub struct Directories;

impl Directories {
    const SUBDIR: &'static str = "whatsong";

    pub fn db_path() -> anyhow::Result<PathBuf> {
        Self::data().map(|dir| dir.join("videos.db"))
    }

    pub fn config() -> anyhow::Result<PathBuf> {
        Self::get_and_make_dir(dirs::config_dir(), "config")
    }

    pub fn data() -> anyhow::Result<PathBuf> {
        Self::get_and_make_dir(dirs::config_dir(), "data")
    }

    pub fn get_and_make_dir(dir: Option<PathBuf>, kind: &str) -> anyhow::Result<PathBuf> {
        let path = dir
            .map(|dir| dir.join(Self::SUBDIR))
            .ok_or_else(|| anyhow::anyhow!("cannot get {} directory", kind))?;

        std::fs::create_dir_all(&path)
            .with_context(|| format!("cannot create {} directory: `{}`", kind, path.display()))?;

        Ok(path)
    }
}
