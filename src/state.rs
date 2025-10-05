use crate::errors::*;
use serde::{Deserialize, Serialize};
use std::{path::Path, time::SystemTime};
use tokio::fs;

#[derive(Debug, Serialize, Deserialize)]
pub struct State {
    pub version: String,
    pub last_update_check: SystemTime,
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
}
