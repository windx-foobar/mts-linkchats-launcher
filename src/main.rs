use clap::Parser;
use env_logger::Env;
use mts_linkchats_launcher::{
    apt::Client,
    args::Args,
    config::{BIN_APP_NAME, Config},
    errors::*,
    extract, pkg,
    state::{State, StateFile},
    ui,
};
use std::time::{Duration, SystemTime};
use tokio::{fs, process::Command, signal};

async fn should_update(config: &Config, state: &State) -> Result<bool> {
    if config.force_check_update {
        Ok(true)
    } else if !config.check_update {
        Ok(false)
    } else {
        if state.get_pid().is_some() {
            return Ok(false);
        }

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
        let interval: u64 = config.check_update_interval.try_into()?;
        Ok(since_update >= Duration::from_secs(interval))
    }
}

fn print_tar_url() {
    println!("{}", pkg::DOWNLOAD_URL);
}

async fn update(config: &Config, state_file: &mut StateFile) -> Result<()> {
    let tar = if let Some(tar_path) = &config.tar_path {
        fs::read(tar_path)
            .await
            .with_context(|| anyhow!("Failed to read .tar.gz file from {:?}", tar_path))?
    } else {
        Client::new(config.timeout.and_then(|value| value.try_into().ok()))?
            .download_tar(config.download_attempts)
            .await?
    };

    let version = pkg::parse_version(tar.as_slice())?;
    let state = &mut state_file.state;

    if state.version != version {
        info!("Version not compared. Updating...");
        state.version = version;
        extract::pkg(tar.as_slice(), config).await?;
    } else if config.force_check_update {
        info!("Latest version is already installed, but --tar options is passed. Force update...");
        extract::pkg(tar.as_slice(), config).await?;
    } else {
        info!("Latest version is already installed, skip...");
    }
    state_file.save().await?;

    Ok(())
}

async fn start(args: &Args, config: &Config, state_file: &mut StateFile) -> Result<()> {
    let bin = config.install_path.join(BIN_APP_NAME);

    let exec_args = ["echo".into(), "--no-sandbox".into()]
        .iter()
        .chain(config.extra_arguments.iter())
        .cloned()
        .collect::<Vec<_>>();

    debug!("Assembled command: {} {:?}", bin.display(), exec_args);

    if args.no_exec {
        info!("Skipping exec because --no-exec was used");
    } else {
        let stub_desktop_file_path = dirs::data_local_dir()
            .unwrap()
            .join("applications/mtslink.desktop");
        let mut command = Command::new(bin);
        let command = command.args(exec_args);

        if fs::metadata(&stub_desktop_file_path).await.is_err() {
            fs::write(&stub_desktop_file_path, &[])
                .await
                .with_context(|| anyhow!("Failed create stub desktop file"))?;

            let mut stub_desktop_file_permissions = fs::metadata(&stub_desktop_file_path)
                .await
                .with_context(|| anyhow!("Failed get metadata opened stub desktop file"))?
                .permissions();
            stub_desktop_file_permissions.set_readonly(true);

            fs::set_permissions(&stub_desktop_file_path, stub_desktop_file_permissions)
                .await
                .with_context(|| anyhow!("Failed set permissions in stub desktop file"))?;
        }

        if state_file.state.get_pid().is_some() {
            debug!("`{}` already running, rerun without spawn", BIN_APP_NAME);

            command
                .output()
                .await
                .with_context(|| anyhow!("Failed output `{}`", BIN_APP_NAME))?;
        } else {
            use signal::unix::{SignalKind, signal};

            let mut child = command
                .spawn()
                .with_context(|| anyhow!("Failed spawn `{}`", BIN_APP_NAME))?;

            let mut sigterm = signal(SignalKind::terminate())?;

            tokio::select! {
                _ = signal::ctrl_c() => {
                    debug!("Exited by CTRL+C");
                },
                _ = sigterm.recv() => {
                    debug!("Exited by SIGTERM");
                }
                value = child.wait() => {
                    let status_code = value.with_context(|| anyhow!("Failed wait `{}`", BIN_APP_NAME))?;
                    debug!("`{}` is exited with code {status_code:?}", BIN_APP_NAME);
                }
            };
        }
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

    let config = Config::builder(&args).build()?;

    debug!("Using install path: {:?}", config.install_path);

    if args.print_tar_url {
        print_tar_url();
    } else {
        let mut state_file = StateFile::load(&config.state_path).await?;

        if should_update(&config, &state_file.state).await? {
            if let Err(err) = update(&config, &mut state_file).await {
                error!("Update failed: {err:#}");
                ui::error(&err).await?;
            }
        } else {
            info!("No update needed");
        }
        start(&args, &config, &mut state_file).await?;
    }

    Ok(())
}
