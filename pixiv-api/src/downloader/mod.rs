use crate::error::PixivError;
use std::path::PathBuf;
use tokio::fs;

use crate::models::common::ImageUrls;
use crate::models::illust::Illust;

/// A single image download task.
#[derive(Debug, Clone)]
pub struct DownloadTask {
    pub url: String,
    pub filename: String,
}

/// Events emitted during downloads for progress tracking.
#[derive(Debug)]
pub enum ProgressEvent {
    /// A download attempt is starting.
    Started {
        filename: String,
        total_bytes: Option<u64>,
    },
    /// A chunk of data was downloaded.
    Chunk {
        filename: String,
        bytes_downloaded: u64,
    },
    /// A file was saved successfully.
    Finished { filename: String, path: PathBuf },
    /// A download attempt failed (will be retried).
    Failed {
        filename: String,
        error: String,
        attempt: u32,
    },
}

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

    /// Download multiple tasks with concurrency control, progress reporting, and retry.
    pub async fn download_all<F>(
        &self,
        tasks: &[DownloadTask],
        concurrency: usize,
        on_progress: F,
    ) -> Vec<crate::Result<PathBuf>>
    where
        F: Fn(ProgressEvent) + Send + Sync + 'static,
    {
        let semaphore = std::sync::Arc::new(tokio::sync::Semaphore::new(concurrency));
        let on_progress = std::sync::Arc::new(on_progress);
        let mut handles = Vec::new();

        for task in tasks {
            let sem = semaphore.clone();
            let url = task.url.clone();
            let filename = task.filename.clone();
            let client = self.client.clone();
            let dir = self.output_dir.clone();
            let progress = on_progress.clone();

            handles.push(tokio::spawn(async move {
                let _permit = sem
                    .acquire()
                    .await
                    .map_err(|e| PixivError::Download(e.to_string()))?;

                progress(ProgressEvent::Started {
                    filename: filename.clone(),
                    total_bytes: None,
                });

                let max_retries = 4u32;
                let mut last_err = String::new();

                for attempt in 0..max_retries {
                    if attempt > 0 {
                        let delay_ms = 1000 * 2u64.pow(attempt - 1);
                        tokio::time::sleep(std::time::Duration::from_millis(delay_ms)).await;
                    }

                    match download_file(&client, &dir, &url, &filename, &*progress).await {
                        Ok(path) => return Ok(path),
                        Err(e) => {
                            last_err = e.to_string();
                            progress(ProgressEvent::Failed {
                                filename: filename.clone(),
                                error: last_err.clone(),
                                attempt: attempt + 1,
                            });
                        }
                    }
                }

                Err(PixivError::Download(format!(
                    "{}: failed after {} attempts: {}",
                    filename, max_retries, last_err
                )))
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

/// Extract the file extension from a URL.
/// Returns `"png"` if the URL contains `.png`, otherwise returns `"jpg"`.
pub fn extract_ext(url: &str) -> &str {
    if url.contains(".png") { "png" } else { "jpg" }
}

/// Get the image URL for a requested size from `ImageUrls`, with fallback.
///
/// Fallback chains:
/// - `"original"` -> original > large > medium
/// - `"large"` -> large > medium
/// - `"medium"` -> medium only
/// - other -> same as original
pub fn url_for_size<'a>(urls: &'a ImageUrls, size: &str) -> Option<&'a str> {
    match size {
        "original" => urls
            .original
            .as_deref()
            .or(urls.large.as_deref())
            .or(urls.medium.as_deref()),
        "large" => urls.large.as_deref().or(urls.medium.as_deref()),
        "medium" => urls.medium.as_deref(),
        _ => urls
            .original
            .as_deref()
            .or(urls.large.as_deref())
            .or(urls.medium.as_deref()),
    }
}

/// Download a single file with chunk-level progress reporting.
async fn download_file<F>(
    client: &reqwest::Client,
    output_dir: &std::path::Path,
    url: &str,
    filename: &str,
    on_progress: &F,
) -> crate::Result<PathBuf>
where
    F: Fn(ProgressEvent) + Send + Sync,
{
    let resp = client
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

    fs::create_dir_all(output_dir).await?;
    let path = output_dir.join(filename);

    let mut file = tokio::fs::File::create(&path).await?;
    let mut downloaded: u64 = 0;

    use tokio::io::AsyncWriteExt;
    let mut stream = resp.bytes_stream();
    use futures_util::StreamExt;

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| PixivError::Download(e.to_string()))?;
        file.write_all(&chunk).await?;
        downloaded += chunk.len() as u64;
        on_progress(ProgressEvent::Chunk {
            filename: filename.to_string(),
            bytes_downloaded: downloaded,
        });
    }

    on_progress(ProgressEvent::Finished {
        filename: filename.to_string(),
        path: path.clone(),
    });

    Ok(path)
}

/// Resolve an illustration into a list of download tasks.
///
/// For multi-page illustrations (`meta_pages` is non-empty), each page becomes
/// a separate task with filename `{id}_p{index}.{ext}`. An optional `pages`
/// filter restricts which page indices are included.
///
/// For single-page illustrations, a single task is created using the
/// appropriate URL fallback chain for the requested size. Filename is
/// `{id}_p0.{ext}`.
pub fn resolve_download_tasks(
    illust: &Illust,
    size: &str,
    pages: Option<&[usize]>,
) -> Vec<DownloadTask> {
    let id = illust.id;

    // Multi-page: iterate over meta_pages if present and non-empty.
    if let Some(meta_pages) = &illust.meta_pages
        && !meta_pages.is_empty()
    {
        return meta_pages
            .iter()
            .enumerate()
            .filter(|(i, _)| match pages {
                Some(pages) => pages.contains(i),
                None => true,
            })
            .filter_map(|(i, page)| {
                let url = page
                    .image_urls
                    .as_ref()
                    .and_then(|urls| url_for_size(urls, size))?;
                let ext = extract_ext(url);
                Some(DownloadTask {
                    url: url.to_string(),
                    filename: format!("{}_p{}.{}", id, i, ext),
                })
            })
            .collect();
    }

    // Single-page: resolve URL based on size.
    let url = match size {
        "original" => illust
            .meta_single_page
            .as_ref()
            .and_then(|m| m.original_image_url.as_deref())
            .or_else(|| illust.image_urls.as_ref().and_then(|u| u.large.as_deref()))
            .or_else(|| illust.image_urls.as_ref().and_then(|u| u.medium.as_deref())),
        _ => illust
            .image_urls
            .as_ref()
            .and_then(|urls| url_for_size(urls, size))
            .or_else(|| {
                illust
                    .meta_single_page
                    .as_ref()
                    .and_then(|m| m.original_image_url.as_deref())
            }),
    };

    match url {
        Some(url) => {
            let ext = extract_ext(url);
            vec![DownloadTask {
                url: url.to_string(),
                filename: format!("{}_p0.{}", id, ext),
            }]
        }
        None => vec![],
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

    #[test]
    fn test_download_task_creation() {
        let task = DownloadTask {
            url: "https://example.com/image.jpg".to_string(),
            filename: "image.jpg".to_string(),
        };
        assert_eq!(task.url, "https://example.com/image.jpg");
        assert_eq!(task.filename, "image.jpg");
    }

    #[test]
    fn test_progress_event_started() {
        let event = ProgressEvent::Started {
            filename: "image.jpg".to_string(),
            total_bytes: Some(1024),
        };
        match event {
            ProgressEvent::Started {
                filename,
                total_bytes,
            } => {
                assert_eq!(filename, "image.jpg");
                assert_eq!(total_bytes, Some(1024));
            }
            _ => panic!("Expected Started variant"),
        }
    }

    #[test]
    fn test_progress_event_finished() {
        let event = ProgressEvent::Finished {
            filename: "image.jpg".to_string(),
            path: PathBuf::from("/tmp/image.jpg"),
        };
        match event {
            ProgressEvent::Finished { filename, path } => {
                assert_eq!(filename, "image.jpg");
                assert_eq!(path, PathBuf::from("/tmp/image.jpg"));
            }
            _ => panic!("Expected Finished variant"),
        }
    }

    #[test]
    fn test_progress_event_failed() {
        let event = ProgressEvent::Failed {
            filename: "image.jpg".to_string(),
            error: "network error".to_string(),
            attempt: 2,
        };
        match event {
            ProgressEvent::Failed {
                filename,
                error,
                attempt,
            } => {
                assert_eq!(filename, "image.jpg");
                assert_eq!(error, "network error");
                assert_eq!(attempt, 2);
            }
            _ => panic!("Expected Failed variant"),
        }
    }

    #[test]
    fn test_extract_ext() {
        assert_eq!(extract_ext("https://example.com/img.png"), "png");
        assert_eq!(extract_ext("https://example.com/img.jpg"), "jpg");
        assert_eq!(extract_ext("https://example.com/img.jpeg"), "jpg");
    }

    #[test]
    fn test_url_for_size_original() {
        let urls = ImageUrls {
            square_medium: None,
            medium: Some("medium_url".into()),
            large: Some("large_url".into()),
            original: Some("original_url".into()),
        };
        assert_eq!(url_for_size(&urls, "original"), Some("original_url"));
        assert_eq!(url_for_size(&urls, "large"), Some("large_url"));
        assert_eq!(url_for_size(&urls, "medium"), Some("medium_url"));
    }

    #[test]
    fn test_url_for_size_fallback() {
        let urls = ImageUrls {
            square_medium: None,
            medium: Some("medium_url".into()),
            large: None,
            original: None,
        };
        assert_eq!(url_for_size(&urls, "original"), Some("medium_url"));
        assert_eq!(url_for_size(&urls, "large"), Some("medium_url"));
    }

    #[test]
    fn test_resolve_single_page_original() {
        use crate::models::illust::Illust;
        let json = r#"{
            "id": 12345,
            "title": "Test",
            "page_count": 1,
            "image_urls": {"large": "https://img.example.com/12345_p0_master1200.jpg"},
            "meta_single_page": {"original_image_url": "https://img.example.com/12345_p0.jpg"},
            "meta_pages": []
        }"#;
        let illust: Illust = serde_json::from_str(json).unwrap();
        let tasks = resolve_download_tasks(&illust, "original", None);
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].url, "https://img.example.com/12345_p0.jpg");
        assert_eq!(tasks[0].filename, "12345_p0.jpg");
    }

    #[test]
    fn test_resolve_multi_page_all() {
        use crate::models::illust::Illust;
        let json = r#"{
            "id": 99999,
            "title": "Multi",
            "page_count": 3,
            "image_urls": null,
            "meta_single_page": null,
            "meta_pages": [
                {"image_urls": {"large": "https://img.example.com/99999_p0_master1200.jpg", "original": "https://img.example.com/99999_p0.jpg"}},
                {"image_urls": {"large": "https://img.example.com/99999_p1_master1200.jpg", "original": "https://img.example.com/99999_p1.jpg"}},
                {"image_urls": {"large": "https://img.example.com/99999_p2_master1200.jpg", "original": "https://img.example.com/99999_p2.jpg"}}
            ]
        }"#;
        let illust: Illust = serde_json::from_str(json).unwrap();
        let tasks = resolve_download_tasks(&illust, "original", None);
        assert_eq!(tasks.len(), 3);
        assert_eq!(tasks[0].filename, "99999_p0.jpg");
        assert_eq!(tasks[1].filename, "99999_p1.jpg");
        assert_eq!(tasks[2].filename, "99999_p2.jpg");
    }

    #[test]
    fn test_resolve_multi_page_filtered() {
        use crate::models::illust::Illust;
        let json = r#"{
            "id": 99999,
            "title": "Multi",
            "page_count": 3,
            "image_urls": null,
            "meta_single_page": null,
            "meta_pages": [
                {"image_urls": {"large": "https://img.example.com/99999_p0_master1200.jpg", "original": "https://img.example.com/99999_p0.jpg"}},
                {"image_urls": {"large": "https://img.example.com/99999_p1_master1200.jpg", "original": "https://img.example.com/99999_p1.jpg"}},
                {"image_urls": {"large": "https://img.example.com/99999_p2_master1200.jpg", "original": "https://img.example.com/99999_p2.jpg"}}
            ]
        }"#;
        let illust: Illust = serde_json::from_str(json).unwrap();
        let tasks = resolve_download_tasks(&illust, "large", Some(&[0, 2]));
        assert_eq!(tasks.len(), 2);
        assert_eq!(tasks[0].filename, "99999_p0.jpg");
        assert_eq!(
            tasks[0].url,
            "https://img.example.com/99999_p0_master1200.jpg"
        );
        assert_eq!(tasks[1].filename, "99999_p2.jpg");
    }

    #[test]
    fn test_resolve_empty_returns_nothing() {
        use crate::models::illust::Illust;
        let json = r#"{"id": 1, "title": "empty", "page_count": 0}"#;
        let illust: Illust = serde_json::from_str(json).unwrap();
        let tasks = resolve_download_tasks(&illust, "original", None);
        assert!(tasks.is_empty());
    }

    #[tokio::test]
    async fn test_download_all_with_invalid_url_has_retry_events() {
        let client = reqwest::Client::new();
        let dm = DownloadManager::new(client, std::env::temp_dir().join("pixiv-test-download-all"));

        let tasks = vec![DownloadTask {
            url: "https://httpbin.org/status/404".to_string(),
            filename: "test_fail.jpg".to_string(),
        }];

        let events = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let events_clone = events.clone();

        let results = dm
            .download_all(&tasks, 1, move |evt| {
                let s = format!("{:?}", evt);
                events_clone.lock().unwrap().push(s);
            })
            .await;

        assert_eq!(results.len(), 1);
        assert!(results[0].is_err());

        let recorded = events.lock().unwrap();
        let has_started = recorded.iter().any(|e| e.contains("Started"));
        let has_failed = recorded.iter().any(|e| e.contains("Failed"));
        assert!(has_started, "Should emit Started event");
        assert!(has_failed, "Should emit Failed event");
    }
}
