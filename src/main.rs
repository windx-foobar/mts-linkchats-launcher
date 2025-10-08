use clap::Parser;
use env_logger::Env;
use mts_linkchats_launcher::apt::Client;
use mts_linkchats_launcher::args::Args;
use mts_linkchats_launcher::config::Config;
use mts_linkchats_launcher::errors::*;
use mts_linkchats_launcher::extract;
use mts_linkchats_launcher::pkg;
use mts_linkchats_launcher::state::State;
use mts_linkchats_launcher::ui;
use std::time::Duration;
use std::time::SystemTime;
use tokio::fs;
use tokio::process::Command;

async fn should_update(config: &Config, state: Option<&State>) -> Result<bool> {
    if config.force_check_update {
        Ok(true)
    } else if !config.check_update {
        Ok(false)
    } else if let Some(state) = &state {
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
    } else {
        Ok(true)
    }
}

fn print_tar_url() {
    println!("{}", pkg::DOWNLOAD_URL);
}

async fn update(config: &Config, state: Option<&State>) -> Result<()> {
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
    match state {
        Some(state) if state.version == version => {
            if config.force_check_update {
                info!(
                    "Latest version is already installed, but --tar options is passed. Force update..."
                );
                extract::pkg(tar.as_slice(), config).await?;
            } else {
                info!("Latest version is already installed, skip...");
            }
        }
        _ => {
            extract::pkg(tar.as_slice(), config).await?;
        }
    }

    debug!("Updating state file");
    let buf = toml::to_string(&State {
        last_update_check: SystemTime::now(),
        version,
    })?;
    fs::write(&config.state_path, buf)
        .await
        .context("Failed to write state file")?;

    Ok(())
}

async fn start(args: &Args, config: &Config) -> Result<()> {
    let bin = config.install_path.join("linkchats.bin");

    let exec_args = ["echo".into(), "--no-sandbox".into()]
        // let exec_args = []
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
            .join("applications/linkchats.desktop");

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

        let mut child = Command::new(bin)
            .args(exec_args)
            .spawn()
            .with_context(|| anyhow!("Failed spawn `linkchats.bin`"))?;

        let status_code = child
            .wait()
            .await
            .with_context(|| anyhow!("Failed wait `linkchats.bin`"))?;

        debug!("`linkchats.bin` is exited with code {status_code:?}");
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
        let state = State::load(&config.state_path).await?;
        let state = state.as_ref();

        if should_update(&config, state).await? {
            if let Err(err) = update(&config, state).await {
                error!("Update failed: {err:#}");
                ui::error(&err).await?;
            }
        } else {
            info!("No update needed");
        }
        start(&args, &config).await?;
    }

    Ok(())
}
