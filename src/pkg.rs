use crate::errors::*;
use libflate::gzip::Decoder;
use std::io::Read;
use tar::Archive;

pub const DOWNLOAD_URL: &str = "https://apps.webinar.ru/weteams/linkchats-desktop.tar.gz";

pub fn parse_version<R: Read>(data: R) -> Result<String> {
    let archive = Decoder::new(data).context("Failed to decode tar archive")?;
    let mut archive = Archive::new(archive);

    if let Some(entry) = archive
        .entries()
        .context("Failed get entries from archive")?
        .next()
    {
        let entry = entry.context("Failed get entry from archive")?;
        let path = entry.path().context("Failed get entry path from archive")?;
        let path = path.to_string_lossy();

        if let Some(version) = path.split("-").nth(2)
            && version.split(".").count() == 3
        {
            Ok(version.to_owned())
        } else {
            bail!("Failed to get version from first entry archive");
        }
    } else {
        bail!("Failed to get first entry from archive");
    }
}
