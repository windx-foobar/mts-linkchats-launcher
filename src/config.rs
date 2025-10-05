use crate::args::Args;
use crate::errors::*;
use crate::paths::Paths;
use file::ConfigFile;
use std::path::PathBuf;

mod file;

#[derive(Debug)]
pub struct Config {
    pub install_path: PathBuf,
    pub new_intsall_path: PathBuf,
    pub state_path: PathBuf,
    pub cache_path: PathBuf,
    pub download_attempts: usize,
    pub check_update: bool,
    pub force_check_update: bool,
    pub check_update_interval: usize,
    pub extra_arguments: Vec<String>,
    pub tar_path: Option<PathBuf>,
    pub timeout: Option<usize>,
}

#[derive(Debug)]
pub struct ConfigBuilder<'a> {
    args: &'a Args,
    cf: Option<&'a ConfigFile>,
}

impl<'a> ConfigBuilder<'a> {
    pub fn new(args: &'a Args) -> Self {
        Self { args, cf: None }
    }

    pub fn config_file(&mut self, cf: &'a ConfigFile) -> &mut Self {
        self.cf = Some(cf);
        self
    }

    pub fn build(&self) -> Result<Config> {
        let cf = match self.cf {
            Some(cf) => cf,
            None => &ConfigFile::load().context("Failed load config file")?,
        };

        Config::new(self.args, cf)
    }
}

impl Config {
    pub fn builder<'a>(args: &'a Args) -> ConfigBuilder<'a> {
        ConfigBuilder::new(args)
    }

    pub fn new(args: &Args, cf: &ConfigFile) -> Result<Self> {
        let paths = Paths::new()?;

        Ok(Self {
            install_path: args.install_dir.clone().unwrap_or(paths.install),
            new_intsall_path: args.install_dir.clone().unwrap_or(paths.new_install),
            state_path: paths.state,
            cache_path: paths.cache,
            download_attempts: args
                .download_attempts
                .or(cf.launcher.download_attempts)
                .unwrap_or(5),
            check_update: if args.check_update {
                args.check_update
            } else {
                cf.launcher.check_update
            },
            force_check_update: args.tar.is_some(),
            check_update_interval: args
                .check_update_interval
                .unwrap_or(cf.launcher.check_update_interval),
            extra_arguments: cf.mts_linkchats.extra_arguments.clone(),
            tar_path: args.tar.clone(),
            timeout: args.timeout,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_default_args() -> Args {
        Args {
            check_update: false,
            timeout: None,
            tar: None,
            install_dir: None,
            check_update_interval: None,
            download_attempts: None,
            verbose: 0,
            print_tar_url: false,
            no_exec: true,
        }
    }

    #[test]
    fn check_overrided_args_over_config_file() -> Result<()> {
        let args = Args {
            check_update: true,
            install_dir: Some(dirs::data_dir().unwrap().join(".test")),
            download_attempts: Some(1),
            tar: Some(dirs::cache_dir().unwrap().join(".test.tar.gz")),
            ..get_default_args()
        };
        let cf = ConfigFile::parse(
            r#"
[launcher]
check_update = false
download_attempts = 2
        "#,
        )?;
        let config = Config::builder(&args).config_file(&cf).build()?;

        assert!(!cf.launcher.check_update);
        assert_eq!(cf.launcher.download_attempts, Some(2));

        assert_eq!(config.check_update, args.check_update);
        assert_eq!(config.download_attempts, args.download_attempts.unwrap());
        assert_eq!(config.install_path, *args.install_dir.as_ref().unwrap());
        assert_eq!(config.new_intsall_path, *args.install_dir.as_ref().unwrap());
        assert!(config.force_check_update);

        Ok(())
    }
}
