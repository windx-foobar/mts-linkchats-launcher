use crate::errors::*;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use toml::Value;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct ConfigFile {
    #[serde(default)]
    pub mts_linkchats: MtsLinkchatsConfig,
    #[serde(default)]
    pub launcher: LauncherConfig,
}

impl ConfigFile {
    pub fn parse(s: &str) -> Result<ConfigFile> {
        let c_default = Self::default();
        let c_raw = toml::from_str::<Value>(s)?;

        let c_raw_launcher = c_raw.get("launcher");
        let check_update = c_raw_launcher
            .and_then(|value| value.get("check_update"))
            .and_then(|value| value.as_bool());
        let update_check_interval: Option<usize> = c_raw_launcher
            .and_then(|value| value.get("check_update_interval"))
            .and_then(|value| value.as_integer())
            .and_then(|value| value.try_into().ok());

        let mut c: Self = c_raw.try_into()?;

        if check_update.is_none() {
            c.launcher.check_update = c_default.launcher.check_update;
        }

        if update_check_interval.is_none() {
            c.launcher.check_update_interval = c_default.launcher.check_update_interval;
        }

        Ok(c)
    }

    pub fn load_from(path: &Path) -> Result<ConfigFile> {
        info!("Loading configuration file at {:?}", path);
        let buf = fs::read_to_string(path)
            .with_context(|| anyhow!("Failed to read config file at {:?}", path))?;
        Self::parse(&buf)
    }

    pub fn locate_file() -> Result<Option<PathBuf>> {
        for path in [dirs::config_dir(), Some(PathBuf::from("/etc/"))]
            .into_iter()
            .flatten()
        {
            let path = path.join("mts-linkchats.conf");
            debug!("Searching for configuration file at {:?}", path);
            if path.exists() {
                debug!("Found configuration file at {:?}", path);
                return Ok(Some(path));
            }
        }
        Ok(None)
    }

    pub fn load() -> Result<ConfigFile> {
        if let Some(path) = Self::locate_file()? {
            Self::load_from(&path)
        } else {
            info!("No configuration file found, using default config");
            Ok(Self::default())
        }
    }
}

impl Default for ConfigFile {
    fn default() -> Self {
        Self {
            mts_linkchats: Default::default(),
            launcher: LauncherConfig {
                check_update: true,
                check_update_interval: 3600 * 24,
                ..Default::default()
            },
        }
    }
}

#[derive(Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct MtsLinkchatsConfig {
    #[serde(default)]
    pub extra_arguments: Vec<String>,
    #[serde(default)]
    pub extra_env_vars: Vec<String>,
}

#[derive(Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct LauncherConfig {
    #[serde(default)]
    pub check_update: bool,
    #[serde(default)]
    pub check_update_interval: usize,
    pub download_attempts: Option<usize>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_config() -> Result<()> {
        let cf = ConfigFile::parse("")?;
        assert_eq!(cf, ConfigFile::default());
        Ok(())
    }

    #[test]
    fn test_empty_mts_linkchats_config() -> Result<()> {
        let cf = ConfigFile::parse("[mts-linkchats]")?;
        assert_eq!(cf, ConfigFile::default());
        Ok(())
    }

    #[test]
    fn test_empty_launcher_config() -> Result<()> {
        let cf = ConfigFile::parse("[launcher]")?;
        assert_eq!(cf, ConfigFile::default());
        Ok(())
    }

    #[test]
    fn test_check_update_and_check_update_interval_confg() -> Result<()> {
        let cf = ConfigFile::parse(
            r#"
[launcher]
check_update = false
check_update_interval = 0
        "#,
        )?;
        assert!(!cf.launcher.check_update);
        assert_eq!(cf.launcher.check_update_interval, 0);
        Ok(())
    }

    #[test]
    fn test_check_update_interval_negative_config() -> Result<()> {
        let cf = ConfigFile::parse(
            r#"
[launcher]
check_update_interval = -1
        "#,
        );
        assert!(cf.is_err());
        Ok(())
    }
}
