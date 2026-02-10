use crate::{config::BIN_APP_NAME, errors::*};
use serde::{Deserialize, Serialize};
use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
    sync::LazyLock,
    time::SystemTime,
};
use sysinfo::{Pid, System};
use tokio::fs;

#[derive(Debug, Serialize, Deserialize)]
pub struct State {
    pub version: String,
    pub last_update_check: SystemTime,
    #[serde(skip)]
    pid: LazyLock<Option<Pid>>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            version: Default::default(),
            last_update_check: SystemTime::UNIX_EPOCH,
            pid: LazyLock::new(|| {
                let sys = System::new_all();

                sys.processes_by_name(OsStr::new(BIN_APP_NAME))
                    .next()
                    .map(|process| process.pid())
            }),
        }
    }
}

impl State {
    pub fn get_pid(&self) -> Option<Pid> {
        *self.pid
    }
}

pub struct StateFile {
    path: PathBuf,
    pub state: State,
}

impl StateFile {
    pub fn new(state: State, path: &Path) -> Self {
        Self {
            state,
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
            let state = toml::from_slice::<State>(&buf)?;
            debug!("Loaded state: {:?}", state);
            Ok(Self {
                path: path.to_path_buf(),
                state,
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

        let buf = toml::to_string(&self.state)?;
        fs::write(&self.path, buf)
            .await
            .context("Failed to write state file")?;

        Ok(())
    }
}
