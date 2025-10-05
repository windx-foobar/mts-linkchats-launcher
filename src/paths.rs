use crate::errors::*;
use std::path::PathBuf;

#[derive(Debug)]
pub struct Paths {
    pub install: PathBuf,
    pub new_install: PathBuf,
    pub state: PathBuf,
    pub cache: PathBuf,
}

impl Paths {
    pub fn new() -> Result<Self> {
        let data_dir = dirs::data_dir()
            .context("Failed to detect data directory")?
            .join("mts-linkchats-launcher");
        let cache_dir = dirs::cache_dir().context("Failed to detect cache directory")?;

        Ok(Self {
            install: data_dir.join("install"),
            new_install: data_dir.join("install-new"),
            state: data_dir.join("state.toml"),
            cache: cache_dir,
        })
    }
}
