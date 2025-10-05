use clap::Parser;
use env_logger::Env;
use mts_linkchats_launcher::apt;
use mts_linkchats_launcher::apt::Client;
use mts_linkchats_launcher::args::Args;
use mts_linkchats_launcher::config::ConfigFile;
use mts_linkchats_launcher::errors::*;
use mts_linkchats_launcher::extract;
use mts_linkchats_launcher::paths::{self, Paths, State};
use mts_linkchats_launcher::pkg;
use mts_linkchats_launcher::ui;
use std::path::Path;
use std::time::Duration;
use std::time::SystemTime;
use tokio::fs;
use tokio::process::Command;

async fn should_update(args: &Args, cf: &ConfigFile, state: Option<&State>) -> Result<bool> {
    if !args.check_update && !cf.launcher.check_update {
        return Ok(false);
    }

    if args.tar.is_some() {
        return Ok(true);
    }

    if let Some(state) = &state {
        let Ok(since_update) = SystemTime::now().duration_since(state.last_update_check) else {
            // if the last update time is somehow in the future, check for updates now
            return Ok(true);
        };

        let hours_since = since_update.as_secs() / 3600;
        let days_since = hours_since / 24;
        let hours_since = hours_since % 24;

        debug!(
            "Last update check was {} days and {} hours ago",
            days_since, hours_since
        );
        let interval: u64 = args
            .check_update_interval
            .unwrap_or(cf.launcher.check_update_interval)
            .try_into()?;
        Ok(since_update >= Duration::from_secs(interval))
    } else {
        Ok(true)
    }
}

fn print_tar_url() {
    println!("{}", pkg::DOWNLOAD_URL);
}

async fn update(
    args: &Args,
    state: Option<&State>,
    install_path: &Path,
    download_attempts: usize,
    paths: &Paths,
) -> Result<()> {
    let tar = if let Some(tar_path) = &args.tar {
        fs::read(tar_path)
            .await
            .with_context(|| anyhow!("Failed to read .tar.gz file from {:?}", tar_path))?
    } else {
        Client::new(args.timeout)?
            .download_tar(download_attempts)
            .await?
    };

    let version = pkg::parse_version(tar.as_slice())?;
    match state {
        Some(state) if state.version == version => {
            if args.tar.is_some() {
                info!(
                    "Latest version is already installed, but --tar options is passed. Force update..."
                );
                extract::pkg(tar.as_slice(), args, install_path, paths).await?;
            } else {
                info!("Latest version is already installed, skip...");
            }
        }
        _ => {
            extract::pkg(tar.as_slice(), args, install_path, paths).await?;
        }
    }

    debug!("Updating state file");
    let buf = toml::to_string(&paths::State {
        last_update_check: SystemTime::now(),
        version,
    })?;
    fs::write(&paths.state, buf)
        .await
        .context("Failed to write state file")?;

    Ok(())
}

async fn start(args: &Args, cf: &ConfigFile, install_path: &Path) -> Result<()> {
    let bin = install_path.join("linkchats");

    let mut exec_args = vec![];

    for arg in cf.mts_linkchats.extra_arguments.iter().cloned() {
        exec_args.push(arg);
    }

    debug!("Assembled command: {:?}", exec_args);

    if args.no_exec {
        info!("Skipping exec because --no-exec was used");
    } else {
        let mut child = Command::new(bin)
            .args(exec_args)
            .spawn()
            .with_context(|| anyhow!("Failed spawn `linkchats`"))?;

        let status_code = child
            .wait()
            .await
            .with_context(|| anyhow!("Failed wait `linkchats`"))?;
        debug!("`linkchats` is exited with code {status_code:?}");
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    let log_level = match args.verbose {
        0 => "info",
        1 => "info,mts_linkchats_launcher=debug",
        2 => "debug",
        _ => "trace",
    };
    env_logger::init_from_env(Env::default().default_filter_or(log_level));

    let paths = Paths::new()?;
    let cf = ConfigFile::load().context("Failed to load configuration")?;

    let install_path = if let Some(path) = &args.install_dir {
        path.as_path()
    } else {
        paths.install.as_path()
    };
    debug!("Using install path: {:?}", install_path);

    let download_attempts = args
        .download_attempts
        .or(cf.launcher.download_attempts)
        .unwrap_or(apt::DEFAULT_DOWNLOAD_ATTEMPTS);

    if args.print_tar_url {
        print_tar_url();
    } else {
        let state = paths::load_state_file(&paths).await?;
        if should_update(&args, &cf, state.as_ref()).await? {
            if let Err(err) = update(
                &args,
                state.as_ref(),
                install_path,
                download_attempts,
                &paths,
            )
            .await
            {
                error!("Update failed: {err:#}");
                ui::error(&err).await?;
            }
        } else {
            info!("No update needed");
        }
        start(&args, &cf, install_path).await?;
    }

    Ok(())
}
