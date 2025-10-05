use crate::args::Args;
use crate::errors::*;
use crate::paths::Paths;
use libflate::gzip::Decoder;
use std::io::Read;
use std::path::Path;
use tokio::fs;

enum AtomicSwapFallback {
    Atomic,
    Tokio,
}

async fn atomic_swap(src: &Path, target: &Path) -> Result<()> {
    info!(
        "Atomically swapping new directory at {:?} with {:?}...",
        src, target
    );
    fs::create_dir(target).await.ok();
    libxch::xch(src, target)?;
    Ok(())
}

async fn atomic_swap_with_fallback(src: &Path, target: &Path) -> Result<AtomicSwapFallback> {
    if let Err(err) = atomic_swap(src, target).await {
        warn!("Failed to swap {src:?} with {target:?}: {err:#}");
        debug!("Falling back to non-atomic swap, removing old directory...");
        fs::remove_dir_all(target)
            .await
            .context("Failed to delete old directory")?;
        debug!("Moving new directory in place...");
        fs::rename(src, target)
            .await
            .context("Failed to move new directory in place")?;

        Ok(AtomicSwapFallback::Tokio)
    } else {
        Ok(AtomicSwapFallback::Atomic)
    }
}

async fn extract_data<R: Read>(
    mut tar: tar::Archive<R>,
    args: &Args,
    install_path: &Path,
    paths: &Paths,
) -> Result<()> {
    let new_install_path = args.install_dir.as_ref().unwrap_or(&paths.new_install);
    let tmp = tempfile::tempdir_in(&paths.cache).context("Failed to create temporary directory")?;
    let prepare_path = tmp.path();

    info!("Extracting to {:?}...", prepare_path);
    tar.unpack(prepare_path)
        .context("Failed to extract mts-linkchats")?;

    debug!("Creating new install directory is not exists");
    fs::create_dir_all(&new_install_path)
        .await
        .context("Failed to create new install directory")?;

    if let Some(entry) = fs::read_dir(prepare_path)
        .await
        .context("Failed read prepare directory")?
        .next_entry()
        .await
        .ok()
        .flatten()
    {
        atomic_swap_with_fallback(&entry.path(), new_install_path).await?;
    } else {
        bail!("Failed get first entry in prepare directory");
    }

    if install_path != new_install_path
        && let AtomicSwapFallback::Atomic =
            atomic_swap_with_fallback(new_install_path, install_path).await?
    {
        debug!("Removing old directory...");
        if let Err(err) = fs::remove_dir_all(&new_install_path).await {
            warn!("Failed to delete old directory: {:#}", err);
        }
    }
    Ok(())
}

pub async fn pkg<R: Read>(tar: R, args: &Args, install_path: &Path, paths: &Paths) -> Result<()> {
    let decoder = Decoder::new(tar)?;
    let tar = tar::Archive::new(decoder);
    extract_data(tar, args, install_path, paths).await
}
