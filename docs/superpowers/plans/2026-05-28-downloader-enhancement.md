# Downloader Enhancement Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Rewrite the download command to support multi-page illustrations, parallel downloads with progress display, and exponential backoff retry.

**Architecture:** Library provides `DownloadTask`, `resolve_download_tasks()`, and `download_all()` with progress callback. CLI handles `id[0,2,3]` parsing, parallel info fetching, and indicatif progress bars.

**Tech Stack:** Rust, tokio, reqwest, indicatif (new), futures (new)

---

## File Structure

| File | Responsibility |
|------|---------------|
| `pixiv-api/src/downloader/mod.rs` | `DownloadTask`, `ProgressEvent`, `resolve_download_tasks()`, `download_all()` |
| `pixiv-api/src/models/common.rs` | `ImageUrls`, `MetaPage`, `MetaSinglePage` (existing, unchanged) |
| `pixiv-api/src/models/illust.rs` | `Illust`, `IllustDetail` (existing, unchanged) |
| `pixiv-dl/src/main.rs` | CLI arg parsing for `id[0,2,3]`, parallel info fetch, indicatif progress |
| `pixiv-dl/Cargo.toml` | Add `indicatif`, `futures` deps |

---

### Task 1: Add DownloadTask and ProgressEvent types

**Files:**
- Modify: `pixiv-api/src/downloader/mod.rs`

- [ ] **Step 1: Write the failing test for DownloadTask and ProgressEvent**

Add to `pixiv-api/src/downloader/mod.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_download_task_creation() {
        let task = DownloadTask {
            url: "https://example.com/img.jpg".to_string(),
            filename: "12345_p0.jpg".to_string(),
        };
        assert_eq!(task.url, "https://example.com/img.jpg");
        assert_eq!(task.filename, "12345_p0.jpg");
    }

    #[test]
    fn test_download_manager_creation() {
        let client = reqwest::Client::new();
        let dm = DownloadManager::new(client, "./test_output");
        assert_eq!(dm.output_dir, PathBuf::from("./test_output"));
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p pixiv-client --lib downloader`
Expected: FAIL with "DownloadTask not found"

- [ ] **Step 3: Add DownloadTask and ProgressEvent structs**

Add these at the top of `pixiv-api/src/downloader/mod.rs`, before `DownloadManager`:

```rust
use std::path::PathBuf;
use tokio::fs;

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
    Finished {
        filename: String,
        path: PathBuf,
    },
    /// A download attempt failed (will be retried).
    Failed {
        filename: String,
        error: String,
        attempt: u32,
    },
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p pixiv-client --lib downloader`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add pixiv-api/src/downloader/mod.rs
git commit -m "feat(downloader): add DownloadTask and ProgressEvent types"
```

---

### Task 2: Add resolve_download_tasks function

**Files:**
- Modify: `pixiv-api/src/downloader/mod.rs`

- [ ] **Step 1: Write the failing tests**

Add to the test module in `pixiv-api/src/downloader/mod.rs`:

```rust
    #[test]
    fn test_resolve_single_page() {
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
            "image_urls": {"large": "https://img.example.com/99999_p0_master1200.jpg"},
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
            "image_urls": {"large": "https://img.example.com/99999_p0_master1200.jpg"},
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
        assert_eq!(tasks[0].url, "https://img.example.com/99999_p0_master1200.jpg");
        assert_eq!(tasks[1].filename, "99999_p2.jpg");
        assert_eq!(tasks[1].url, "https://img.example.com/99999_p2_master1200.jpg");
    }

    #[test]
    fn test_resolve_single_page_large_size() {
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
        let tasks = resolve_download_tasks(&illust, "large", None);
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].url, "https://img.example.com/12345_p0_master1200.jpg");
    }

    #[test]
    fn test_filename_ext_from_url() {
        assert_eq!(extract_ext("https://example.com/img.png"), "png");
        assert_eq!(extract_ext("https://example.com/img.jpg"), "jpg");
        assert_eq!(extract_ext("https://example.com/img.jpeg"), "jpg");
    }
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p pixiv-client --lib downloader`
Expected: FAIL with "resolve_download_tasks not found" and "extract_ext not found"

- [ ] **Step 3: Implement resolve_download_tasks and helpers**

Add these functions to `pixiv-api/src/downloader/mod.rs`, after the `DownloadManager` impl block:

```rust
use crate::models::common::ImageUrls;
use crate::models::illust::Illust;

/// Extract file extension from a URL.
pub fn extract_ext(url: &str) -> &str {
    if url.contains(".png") {
        "png"
    } else {
        "jpg"
    }
}

/// Get the image URL from an ImageUrls struct based on size preference.
fn url_for_size(urls: &ImageUrls, size: &str) -> Option<&str> {
    match size {
        "original" => urls.original.as_deref()
            .or(urls.large.as_deref())
            .or(urls.medium.as_deref()),
        "large" => urls.large.as_deref()
            .or(urls.medium.as_deref()),
        "medium" => urls.medium.as_deref(),
        _ => urls.original.as_deref()
            .or(urls.large.as_deref())
            .or(urls.medium.as_deref()),
    }
}

/// Resolve an illustration into a list of download tasks.
///
/// - `illust`: the illustration detail
/// - `size`: "original", "large", or "medium"
/// - `pages`: optional page filter (e.g. Some(&[0, 2]) to download only pages 0 and 2)
pub fn resolve_download_tasks(
    illust: &Illust,
    size: &str,
    pages: Option<&[usize]>,
) -> Vec<DownloadTask> {
    let id = illust.id;
    let mut tasks = Vec::new();

    // Multi-page illustration
    if let Some(meta_pages) = &illust.meta_pages {
        if !meta_pages.is_empty() {
            let indices: Vec<usize> = if let Some(filter) = pages {
                filter.to_vec()
            } else {
                (0..meta_pages.len()).collect()
            };
            for &idx in &indices {
                if let Some(page) = meta_pages.get(idx) {
                    if let Some(urls) = &page.image_urls {
                        if let Some(url) = url_for_size(urls, size) {
                            let ext = extract_ext(url);
                            tasks.push(DownloadTask {
                                url: url.to_string(),
                                filename: format!("{id}_p{idx}.{ext}"),
                            });
                        }
                    }
                }
            }
            return tasks;
        }
    }

    // Single-page illustration
    let url = if size == "original" {
        illust.meta_single_page.as_ref()
            .and_then(|sp| sp.original_image_url.as_deref())
            .or_else(|| illust.image_urls.as_ref().and_then(|u| u.large.as_deref()))
            .or_else(|| illust.image_urls.as_ref().and_then(|u| u.medium.as_deref()))
    } else {
        illust.image_urls.as_ref().and_then(|u| url_for_size(u, size))
            .or_else(|| illust.meta_single_page.as_ref()
                .and_then(|sp| sp.original_image_url.as_deref()))
    };

    if let Some(url) = url {
        let ext = extract_ext(url);
        tasks.push(DownloadTask {
            url: url.to_string(),
            filename: format!("{id}_p0.{ext}"),
        });
    }

    tasks
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test -p pixiv-client --lib downloader`
Expected: PASS (all 10 tests)

- [ ] **Step 5: Commit**

```bash
git add pixiv-api/src/downloader/mod.rs
git commit -m "feat(downloader): add resolve_download_tasks for single/multi-page illustrations"
```

---

### Task 3: Add download_all with retry and progress callback

**Files:**
- Modify: `pixiv-api/src/downloader/mod.rs`

- [ ] **Step 1: Write the failing test**

Add to the test module:

```rust
    #[tokio::test]
    async fn test_download_all_with_retry_counts_attempts() {
        // This test verifies the retry logic structure.
        // We test with an invalid URL to trigger failures.
        let client = reqwest::Client::new();
        let dm = DownloadManager::new(client, "/tmp/pixiv-test-download-all");

        let tasks = vec![
            DownloadTask {
                url: "https://httpbin.org/status/404".to_string(),
                filename: "test_fail.jpg".to_string(),
            },
        ];

        let events = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let events_clone = events.clone();

        let results = dm.download_all(&tasks, 1, move |evt| {
            events_clone.lock().unwrap().push(format!("{:?}", evt));
        }).await;

        assert_eq!(results.len(), 1);
        assert!(results[0].is_err());

        let recorded = events.lock().unwrap();
        // Should have at least Started and Failed events
        let has_started = recorded.iter().any(|e| e.contains("Started"));
        let has_failed = recorded.iter().any(|e| e.contains("Failed"));
        assert!(has_started, "Should emit Started event");
        assert!(has_failed, "Should emit Failed event");
    }
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p pixiv-client --lib downloader::tests::test_download_all_with_retry_counts_attempts`
Expected: FAIL with "no method named `download_all`"

- [ ] **Step 3: Implement download_all**

Add this method to the `impl DownloadManager` block in `pixiv-api/src/downloader/mod.rs`:

```rust
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
                let _permit = sem.acquire().await
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

                    match download_with_progress(&client, &dir, &url, &filename, &*progress).await {
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
```

- [ ] **Step 4: Add the download_with_progress helper**

Add this function outside the impl block, before `resolve_download_tasks`:

```rust
/// Download a single file with chunk-level progress reporting.
async fn download_with_progress<F>(
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
```

- [ ] **Step 5: Add futures-util dependency**

In `pixiv-api/Cargo.toml`, add:

```toml
futures-util = "0.3"
```

- [ ] **Step 6: Run test to verify it passes**

Run: `cargo test -p pixiv-client --lib downloader::tests::test_download_all_with_retry_counts_attempts`
Expected: PASS

- [ ] **Step 7: Commit**

```bash
git add pixiv-api/src/downloader/mod.rs pixiv-api/Cargo.toml
git commit -m "feat(downloader): add download_all with retry and progress callback"
```

---

### Task 4: Parse ID syntax and add CLI args

**Files:**
- Modify: `pixiv-dl/src/main.rs`
- Modify: `pixiv-dl/Cargo.toml`

- [ ] **Step 1: Write the failing tests**

Add a new module at the bottom of `pixiv-dl/src/main.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_illust_input_bare_id() {
        let result = parse_illust_input("12345").unwrap();
        assert_eq!(result.id, 12345);
        assert_eq!(result.pages, None);
    }

    #[test]
    fn test_parse_illust_input_with_pages() {
        let result = parse_illust_input("12345[0,2,3]").unwrap();
        assert_eq!(result.id, 12345);
        assert_eq!(result.pages, Some(vec![0, 2, 3]));
    }

    #[test]
    fn test_parse_illust_input_single_page() {
        let result = parse_illust_input("99999[1]").unwrap();
        assert_eq!(result.id, 99999);
        assert_eq!(result.pages, Some(vec![1]));
    }

    #[test]
    fn test_parse_illust_input_invalid() {
        assert!(parse_illust_input("abc").is_err());
        assert!(parse_illust_input("123[").is_err());
        assert!(parse_illust_input("123[abc]").is_err());
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p pixiv-dl -- tests`
Expected: FAIL with "parse_illust_input not found"

- [ ] **Step 3: Implement IllustInput and parse_illust_input**

Add to `pixiv-dl/src/main.rs`, before the `Cli` struct:

```rust
#[derive(Debug, Clone)]
struct IllustInput {
    id: u64,
    pages: Option<Vec<usize>>,
}

fn parse_illust_input(s: &str) -> Result<IllustInput, String> {
    if let Some(bracket_start) = s.find('[') {
        let id_str = &s[..bracket_start];
        let id: u64 = id_str.parse().map_err(|_| format!("invalid illustration ID: {id_str}"))?;

        let rest = &s[bracket_start..];
        if !rest.ends_with(']') {
            return Err(format!("missing closing bracket in: {s}"));
        }
        let pages_str = &rest[1..rest.len() - 1];
        let pages: Vec<usize> = pages_str
            .split(',')
            .map(|p| p.trim().parse::<usize>())
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_| format!("invalid page number in: {s}"))?;

        Ok(IllustInput { id, pages: Some(pages) })
    } else {
        let id: u64 = s.parse().map_err(|_| format!("invalid illustration ID: {s}"))?;
        Ok(IllustInput { id, pages: None })
    }
}
```

- [ ] **Step 4: Update the Download command args**

Replace the `Download` variant in the `Commands` enum:

```rust
    /// Download illustrations by ID
    Download {
        /// Illustration IDs (e.g. 12345 or 12345[0,2,3])
        ids: Vec<String>,
        /// Output directory
        #[arg(short, long, default_value = "./images")]
        output: String,
        /// Image size: original, large, or medium
        #[arg(short, long, default_value = "original")]
        size: String,
        /// Max concurrent downloads
        #[arg(short = 'j', long, default_value = "4")]
        concurrency: usize,
    },
```

- [ ] **Step 5: Add deps to pixiv-dl/Cargo.toml**

```toml
indicatif = "0.17"
futures = "0.3"
```

- [ ] **Step 6: Run tests**

Run: `cargo test -p pixiv-dl -- tests`
Expected: PASS

- [ ] **Step 7: Commit**

```bash
git add pixiv-dl/src/main.rs pixiv-dl/Cargo.toml
git commit -m "feat(cli): add illust input parser and download command args"
```

---

### Task 5: Rewrite Download command with parallel fetch and progress

**Files:**
- Modify: `pixiv-dl/src/main.rs`

- [ ] **Step 1: Add use statements**

At the top of `pixiv-dl/src/main.rs`, add:

```rust
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use pixiv_client::downloader::{DownloadManager, DownloadTask, ProgressEvent, resolve_download_tasks};
use futures::future::join_all;
```

- [ ] **Step 2: Rewrite the Download command handler**

Replace the entire `Commands::Download` match arm in `main()`:

```rust
        Commands::Download {
            ids,
            output,
            size,
            concurrency,
        } => {
            let api = authenticated_api().await?;

            // Parse all inputs
            let inputs: Vec<IllustInput> = ids
                .iter()
                .map(|s| parse_illust_input(s))
                .collect::<Result<Vec<_>, _>>()
                .map_err(|e| pixiv_client::PixivError::Other(e))?;

            // Fetch all illustration details in parallel
            eprint!("Fetching details for {} illustration(s)...", inputs.len());
            let fetch_sem = std::sync::Arc::new(tokio::sync::Semaphore::new(concurrency));
            let fetch_handles: Vec<_> = inputs
                .iter()
                .map(|input| {
                    let api = &api;
                    let sem = fetch_sem.clone();
                    let id = input.id;
                    async move {
                        let _permit = sem.acquire().await.unwrap();
                        api.illust_detail(id).await
                    }
                })
                .collect();
            let details = join_all(fetch_handles).await;
            eprintln!(" done.");

            // Resolve all download tasks
            let mut all_tasks: Vec<DownloadTask> = Vec::new();
            for (input, detail_result) in inputs.iter().zip(details.iter()) {
                match detail_result {
                    Ok(resp) => {
                        if let Some(illust) = &resp.data {
                            let tasks = resolve_download_tasks(illust, &size, input.pages.as_deref());
                            if tasks.is_empty() {
                                eprintln!("  Warning: no downloadable images for {}", input.id);
                            }
                            all_tasks.extend(tasks);
                        } else {
                            // Fallback: try to parse from raw JSON
                            eprintln!("  Warning: typed parse failed for {}, trying raw", input.id);
                        }
                    }
                    Err(e) => {
                        eprintln!("  Error fetching {}: {}", input.id, e);
                    }
                }
            }

            if all_tasks.is_empty() {
                eprintln!("Nothing to download.");
                return Ok(());
            }

            // Set up progress bars
            let multi = MultiProgress::new();
            let overall = multi.add(ProgressBar::new(all_tasks.len() as u64));
            overall.set_style(
                ProgressStyle::default_bar()
                    .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})")
                    .unwrap()
                    .progress_chars("=>-"),
            );
            overall.set_prefix("Total");

            // Create file-level progress bars
            let file_bars: std::sync::Arc<std::sync::Mutex<std::collections::HashMap<String, ProgressBar>>> =
                std::sync::Arc::new(std::sync::Mutex::new(std::collections::HashMap::new()));

            let multi_clone = multi.clone();
            let file_bars_clone = file_bars.clone();
            let overall_clone = overall.clone();

            let dm = DownloadManager::new(reqwest::Client::new(), &output);

            let results = dm
                .download_all(&all_tasks, concurrency, move |evt| match evt {
                    ProgressEvent::Started { filename, total_bytes } => {
                        let pb = multi_clone.add(ProgressBar::new(total_bytes.unwrap_or(0)));
                        pb.set_style(
                            ProgressStyle::default_bar()
                                .template("  {prefix:.bold} [{bar:30}] {bytes}/{total_bytes} ({bytes_per_sec})")
                                .unwrap()
                                .progress_chars("=>-"),
                        );
                        pb.set_prefix(filename.clone());
                        file_bars_clone.lock().unwrap().insert(filename, pb);
                    }
                    ProgressEvent::Chunk { filename, bytes_downloaded } => {
                        if let Some(pb) = file_bars_clone.lock().unwrap().get(&filename) {
                            pb.set_position(bytes_downloaded);
                        }
                    }
                    ProgressEvent::Finished { filename, .. } => {
                        if let Some(pb) = file_bars_clone.lock().unwrap().remove(&filename) {
                            pb.finish_with_message("done");
                        }
                        overall_clone.inc(1);
                    }
                    ProgressEvent::Failed { filename, error, attempt } => {
                        if let Some(pb) = file_bars_clone.lock().unwrap().get(&filename) {
                            pb.set_prefix(format!("{} (retry {})", filename, attempt));
                        }
                        let _ = error; // logged by retry logic
                    }
                })
                .await;

            overall.finish_with_message("Complete");

            // Report results
            let mut succeeded = 0;
            let mut failed = 0;
            for result in &results {
                match result {
                    Ok(path) => {
                        succeeded += 1;
                        println!("  Saved: {}", path.display());
                    }
                    Err(e) => {
                        failed += 1;
                        eprintln!("  Failed: {e}");
                    }
                }
            }
            println!("\n{succeeded} succeeded, {failed} failed");
        }
```

- [ ] **Step 3: Build and fix any compilation errors**

Run: `cargo build -p pixiv-dl 2>&1`
Fix any type errors or missing imports.

- [ ] **Step 4: Run all tests**

Run: `cargo test -p pixiv-client -p pixiv-dl`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add pixiv-dl/src/main.rs
git commit -m "feat(cli): rewrite download command with parallel fetch and progress bars"
```

---

### Task 6: Integration test — verify CLI builds and help works

**Files:** None (verification only)

- [ ] **Step 1: Build release**

Run: `cargo build -p pixiv-dl --release`
Expected: SUCCESS

- [ ] **Step 2: Verify download help**

Run: `cargo run -p pixiv-dl -- download --help`
Expected output should show `--size`, `--concurrency` / `-j`, and the ID syntax.

- [ ] **Step 3: Verify input parser with edge cases**

Run: `cargo test -p pixiv-dl -- tests`
Expected: PASS

- [ ] **Step 4: Run clippy**

Run: `cargo clippy -p pixiv-dl -p pixiv-client`
Expected: no errors

- [ ] **Step 5: Run cargo fmt**

Run: `cargo fmt -p pixiv-dl -p pixiv-client`

- [ ] **Step 6: Final commit**

```bash
git add -A
git commit -m "feat: complete downloader enhancement with parallel download and progress"
```
