use crate::errors::*;
use serde::{Deserialize, Serialize};
use std::{
    path::{Path, PathBuf},
    time::SystemTime,
};
use tokio::fs;

#[derive(Debug, Serialize, Deserialize)]
pub struct State {
    pub version: String,
    pub last_update_check: SystemTime,
    pub pid: Option<u32>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            version: Default::default(),
            last_update_check: SystemTime::UNIX_EPOCH,
            pid: None,
        }
    }
}

pub struct StateFile {
    path: PathBuf,
    pub state: Option<State>,
}

impl StateFile {
    pub fn new(state: State, path: &Path) -> Self {
        Self {
            state: Some(state),
            path: path.to_path_buf(),
        }
    }

    pub async fn load(path: &Path) -> Result<Self> {
        if fs::metadata(path).await.is_ok() {
            debug!("Reading state file from {:?}...", path);
            let buf = fs::read(path).await.with_context(|| {
                anyhow!(
                    "Failed to read mts-linkchats-launcher state file at {:?}",
                    path
                )
            })?;
            let state = toml::from_slice::<State>(&buf);
            debug!("Loaded state: {:?}", state);
            Ok(Self {
                path: path.to_path_buf(),
                state: state.ok(),
            })
        } else {
            debug!(
                "State file at {:?} does not exist, create state file and fill default data",
                path
            );

            let state_file = Self::new(State::default(), path);
            state_file.save().await?;

            Ok(state_file)
        }
    }

    pub async fn save(&self) -> Result<()> {
        debug!("Save state in file");

        if let Some(state) = &self.state {
            let buf = toml::to_string(state)?;
            fs::write(&self.path, buf)
                .await
                .context("Failed to write state file")?;
        } else {
            debug!("State is empty. Skipping...");
        }

        Ok(())
    }
}
