use crate::errors::*;
use crate::http;
use crate::pkg;
use crate::progress::ProgressBar;

pub const DEFAULT_DOWNLOAD_ATTEMPTS: usize = 5;

pub struct Client {
    client: http::Client,
}

impl Client {
    pub fn new(timeout: Option<u64>) -> Result<Client> {
        let client = http::Client::new(timeout)?;
        Ok(Client { client })
    }

    pub async fn download_tar(&self, download_attempts: usize) -> Result<Vec<u8>> {
        let filename = pkg::DOWNLOAD_URL
            .rsplit_once('/')
            .map(|(_, x)| x)
            .unwrap_or("???");

        info!("Downloading tar file for {:?}", filename);

        // download
        let mut pb = ProgressBar::spawn()?;
        let mut tar = Vec::new();
        let mut offset = None;

        let mut i: usize = 0;
        loop {
            // increast the counter until usize::MAX, but do not overflow
            i = i.saturating_add(1);
            if download_attempts > 0 && i > download_attempts {
                // number of download attempts exceeded
                break;
            }

            if i > 0 {
                info!("Retrying download...");
            }

            if let Err(err) = self
                .attempt_download(pkg::DOWNLOAD_URL, &mut tar, &mut pb, &mut offset)
                .await
            {
                warn!("Download has failed: {err:#}");
            } else {
                pb.close().await?;

                return Ok(tar);
            }
        }

        pb.close().await?;
        bail!("Exceeded number of retries for download");
    }

    async fn attempt_download(
        &self,
        url: &str,
        tar: &mut Vec<u8>,
        pb: &mut ProgressBar,
        offset: &mut Option<u64>,
    ) -> Result<()> {
        let mut dl = self.client.fetch_stream(url, *offset).await?;
        while let Some(chunk) = dl.chunk().await? {
            tar.extend(&chunk);
            *offset = Some(dl.progress);

            let progress = (dl.progress as f64 / dl.total as f64 * 100.0) as u64;
            pb.update(progress).await?;
            debug!(
                "Download progress: {}%, {}/{}",
                progress, dl.progress, dl.total
            );
        }
        Ok(())
    }
}
