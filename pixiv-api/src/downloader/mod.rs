use crate::error::PixivError;
use std::path::PathBuf;
use tokio::fs;

/// Download manager for Pixiv images.
pub struct DownloadManager {
    client: reqwest::Client,
    output_dir: PathBuf,
}

impl DownloadManager {
    /// Create a new DownloadManager.
    pub fn new(client: reqwest::Client, output_dir: impl Into<PathBuf>) -> Self {
        Self {
            client,
            output_dir: output_dir.into(),
        }
    }

    /// Download a single image from a URL to the output directory.
    pub async fn download(&self, url: &str, filename: &str) -> crate::Result<PathBuf> {
        let resp = self
            .client
            .get(url)
            .header("Referer", "https://app-api.pixiv.net/")
            .send()
            .await
            .map_err(|e| PixivError::Download(e.to_string()))?;

        if !resp.status().is_success() {
            return Err(PixivError::Download(format!(
                "HTTP {} for {}",
                resp.status(),
                url
            )));
        }

        fs::create_dir_all(&self.output_dir).await?;

        let path = self.output_dir.join(filename);
        let bytes = resp
            .bytes()
            .await
            .map_err(|e| PixivError::Download(e.to_string()))?;

        fs::write(&path, bytes).await?;
        Ok(path)
    }

    /// Download multiple images concurrently.
    pub async fn download_many(
        &self,
        items: &[(&str, &str)],
        concurrency: usize,
    ) -> Vec<crate::Result<PathBuf>> {
        let semaphore = std::sync::Arc::new(tokio::sync::Semaphore::new(concurrency));
        let mut handles = Vec::new();

        for &(url, filename) in items {
            let sem = semaphore.clone();
            let url = url.to_string();
            let filename = filename.to_string();
            let client = self.client.clone();
            let dir = self.output_dir.clone();

            handles.push(tokio::spawn(async move {
                let _permit = match sem.acquire().await {
                    Ok(permit) => permit,
                    Err(e) => return Err(PixivError::Download(e.to_string())),
                };
                let dm = DownloadManager::new(client, dir);
                dm.download(&url, &filename).await
            }));
        }

        let mut results = Vec::new();
        for handle in handles {
            match handle.await {
                Ok(result) => results.push(result),
                Err(e) => results.push(Err(PixivError::Download(e.to_string()))),
            }
        }
        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_download_manager_creation() {
        let client = reqwest::Client::new();
        let dm = DownloadManager::new(client, "./test_output");
        assert_eq!(dm.output_dir, PathBuf::from("./test_output"));
    }
}
