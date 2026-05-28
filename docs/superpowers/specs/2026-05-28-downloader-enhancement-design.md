# Downloader Enhancement Design

**Goal:** Rewrite the download command to support multi-page illustrations, parallel downloads with progress, and robust retry logic.

**Architecture:** Library handles orchestration (parallel fetch, retry, progress callback). CLI handles parsing, progress bar display (indicatif), and user interaction.

## CLI Input Syntax

```
pixiv-dl download 1234567 1234567[0,2,3] 9999[1] --size original --concurrency 4 -o ./images
```

- Bare ID (`1234567`) → download all pages
- ID with page selector (`1234567[0,2,3]`) → download pages 0, 2, 3 only
- `--size [original|large|medium]` — default `original`
- `--concurrency` / `-j` — default 4, controls both info-fetch and download parallelism

## New Types (pixiv-client)

```rust
pub struct DownloadTask {
    pub url: String,
    pub filename: String,
}

pub enum ProgressEvent {
    Started { filename: String, total_bytes: Option<u64> },
    Chunk { filename: String, bytes_downloaded: u64 },
    Finished { filename: String, path: PathBuf },
    Failed { filename: String, error: String, attempt: u32 },
}
```

## URL Resolution

New function in `pixiv-client::downloader`:

```rust
pub fn resolve_download_tasks(illust: &Illust, size: &str, pages: Option<&[usize]>) -> Vec<DownloadTask>
```

- Single-page: use `meta_single_page.original_image_url` (for `original` size) or `image_urls.large` (for `large`/`medium`)
- Multi-page: iterate `meta_pages`, filter by `pages` selector, extract URL from each `MetaPage.image_urls`
- Filename: `{id}_p{index}.{ext}` where ext is derived from URL

## DownloadManager Enhancement

Extend with:

```rust
pub async fn download_all<F>(
    &self,
    tasks: &[DownloadTask],
    concurrency: usize,
    on_progress: F,
) -> Vec<Result<PathBuf>>
where
    F: Fn(ProgressEvent) + Send + Sync + 'static
```

- Semaphore-limited concurrency (existing pattern)
- Exponential backoff retry: 4 attempts, delays 1s → 2s → 4s → 8s
- Calls `on_progress` for each event
- Streaming per-chunk progress via `resp.chunk()` instead of `resp.bytes()`

## Parallel Info Fetching

In the CLI, fetch multiple illust details concurrently:

```rust
let details = futures::future::join_all(
    ids.iter().map(|id| api.illust_detail(*id))
).await;
```

With semaphore limiting to `--concurrency` value.

## Progress Display (pixiv-dl)

Use `indicatif` crate:

- `MultiProgress` container
- One `ProgressBar` per file (shows filename, bytes, ETA)
- One overall `ProgressBar` at bottom (shows total files completed)
- Bars render to stderr

## Error Handling

- Per-file failures don't abort the batch
- Failed files printed to stderr with error details
- Exit code 0 if any files succeeded, 1 if all failed

## Dependencies

- `indicatif = "0.17"` — progress bars (pixiv-dl only)
- `futures = "0.3"` — join_all (pixiv-dl only)

## Files Changed

| File | Change |
|------|--------|
| `pixiv-api/src/downloader/mod.rs` | Add `DownloadTask`, `ProgressEvent`, `resolve_download_tasks()`, `download_all()` |
| `pixiv-dl/src/main.rs` | Rewrite `Commands::Download` with parallel fetch, new arg parsing, indicatif progress |
| `pixiv-dl/Cargo.toml` | Add `indicatif`, `futures` deps |
