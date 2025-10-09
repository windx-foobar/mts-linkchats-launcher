use crate::errors::*;
use serde::{Deserialize, Serialize};
use std::{path::Path, time::SystemTime};
use tokio::fs;

#[derive(Debug, Serialize, Deserialize)]
pub struct State {
    pub version: String,
    pub last_update_check: SystemTime,
    pub pid: Option<u32>,
}

impl State {
    pub async fn load(path: &Path) -> Result<Option<Self>> {
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
            Ok(state.ok())
        } else {
            debug!("State file at {:?} does not exist, using empty state", path);
            Ok(None)
        }
    }

    pub async fn write_pid(&mut self, pid: Option<u32>, state_path: &Path) -> Result<()> {
        self.pid = pid;
        debug!("Updating state pid");

        let buf = toml::to_string(&self)?;
        fs::write(state_path, buf)
            .await
            .context("Failed to write state file")
    }
}
