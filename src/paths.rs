use crate::errors::*;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::SystemTime;
use tokio::fs;

#[derive(Debug)]
pub struct Paths {
    pub install: PathBuf,
    pub new_install: PathBuf,
    pub state: PathBuf,
    pub cache: PathBuf,
}

impl Paths {
    pub fn new() -> Result<Self> {
        let data_dir = dirs::data_dir().context("Failed to detect data directory")?;
        let cache_dir = dirs::cache_dir().context("Failed to detect cache directory")?;

        Ok(Self {
            install: data_dir.join("install"),
            new_install: data_dir.join("install-new"),
            state: data_dir.join("state.toml"),
            cache: cache_dir,
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct State {
    pub version: String,
    pub last_update_check: SystemTime,
}

pub async fn load_state_file(paths: &Paths) -> Result<Option<State>> {
    let state_file_path = paths.state.as_path();
    if fs::metadata(&state_file_path).await.is_ok() {
        debug!("Reading state file from {:?}...", state_file_path);
        let buf = fs::read(&state_file_path).await.with_context(|| {
            anyhow!(
                "Failed to read mts-linkchats-launcher state file at {:?}",
                state_file_path
            )
        })?;
        let state = toml::from_slice::<State>(&buf);
        debug!("Loaded state: {:?}", state);
        Ok(state.ok())
    } else {
        debug!(
            "State file at {:?} does not exist, using empty state",
            state_file_path
        );
        Ok(None)
    }
}
