# pixiv-api

A Rust client library and CLI tool for the Pixiv App API (6.x), with full API parity with [pixivpy](https://github.com/upbit/pixivpy).

## Features

- **35+ API endpoints** — users, illustrations, novels, search, bookmarks, rankings
- **Hybrid response models** — typed Rust structs with raw JSON fallback for API change resilience
- **Async/await** — built on tokio and reqwest
- **Concurrent downloads** — semaphore-based download manager
- **SNI bypass** — DNS-over-HTTPS for users behind network restrictions (feature flag)
- **CLI tool** — `pixiv-dl` for quick searches and downloads from the terminal

## Installation

### As a library

```toml
[dependencies]
pixiv-api = "0.1"
```

With SNI bypass support (for users behind the Great Firewall):

```toml
[dependencies]
pixiv-api = { version = "0.1", features = ["gfw-bypass"] }
```

### As a CLI tool

```bash
cargo install --path pixiv-dl
```

## Quick Start

### Library

```rust
use pixiv_api::PixivApi;

#[tokio::main]
async fn main() -> pixiv_api::Result<()> {
    let mut api = PixivApi::new();
    api.auth("your_refresh_token").await?;

    // Search illustrations
    let results = api.search_illust("landscape", Some("popular_desc"), None, None, None).await?;

    // Access typed data (if parse succeeded)
    if let Some(data) = &results.data {
        println!("Got response: {}", serde_json::to_string(data).unwrap().len());
    }

    // Always access raw JSON (works even if Pixiv changes the API)
    println!("Raw: {}", serde_json::to_string_pretty(&results.raw).unwrap());

    Ok(())
}
```

### CLI

```bash
# Set your refresh token
export PIXIV_REFRESH_TOKEN=your_token_here

# Search
pixiv-dl search "landscape" --sort popular_desc

# View illustration details
pixiv-dl illust 12345

# Download illustrations
pixiv-dl download 12345 12346 -o ./images/
```

## Getting a Refresh Token

Run the included helper:

```bash
cargo run -p pixiv-api --example get_token
```

This will guide you through the OAuth2 PKCE flow:

1. Open a URL in your browser
2. Log in to Pixiv and authorize
3. Copy the redirect URL back to the terminal
4. Receive your refresh token

## API Reference

### Authentication

```rust
let mut api = PixivApi::new();

// Authenticate with refresh token
api.auth("your_refresh_token").await?;

// Or set tokens manually
api.set_auth("access_token", "refresh_token", user_id);

// Check auth status
assert!(api.is_authenticated());
assert_eq!(api.user_id(), Some(12345));
```

### User Endpoints

| Method | Description |
|---|---|
| `user_detail(user_id)` | Get user details |
| `user_illusts(user_id, type, offset)` | Get user's illustrations |
| `user_bookmarks_illust(user_id, restrict, max_id, tag)` | Get bookmarked illustrations |
| `user_bookmarks_novel(user_id, restrict, max_id)` | Get bookmarked novels |
| `user_related(user_id)` | Get related users |
| `user_recommended()` | Get recommended users |
| `user_following(user_id, restrict, offset)` | Get following list |
| `user_follower(user_id, offset)` | Get followers |
| `user_mypixiv(user_id, offset)` | Get Pixiv friends |
| `user_list(user_ids)` | Get users by IDs |
| `user_novels(user_id, offset)` | Get user's novels |
| `user_follow_add(user_id, restrict)` | Follow a user |
| `user_follow_delete(user_id)` | Unfollow a user |
| `user_bookmark_tags_illust(user_id, restrict)` | Get bookmark tags |
| `user_edit_ai_show_settings(ai_type)` | Edit AI show settings |

### Illustration Endpoints

| Method | Description |
|---|---|
| `illust_detail(illust_id)` | Get illustration details |
| `illust_comments(illust_id, offset)` | Get comments |
| `illust_related(illust_id)` | Get related illustrations |
| `illust_recommended()` | Get recommended illustrations |
| `illust_ranking(mode, date, offset)` | Get ranking |
| `illust_follow(restrict)` | Get followed artists' new works |
| `illust_new()` | Get newest illustrations |
| `illust_bookmark_detail(illust_id)` | Get bookmark status |
| `illust_bookmark_add(illust_id, restrict, tags)` | Add bookmark |
| `illust_bookmark_delete(illust_id)` | Remove bookmark |

### Novel Endpoints

| Method | Description |
|---|---|
| `novel_detail(novel_id)` | Get novel details |
| `novel_comments(novel_id, offset)` | Get comments |
| `novel_recommended()` | Get recommended novels |
| `novel_new()` | Get newest novels |
| `novel_follow(restrict)` | Get followed artists' new novels |
| `novel_series(series_id)` | Get series info |
| `novel_text(novel_id)` | Get novel text |
| `webview_novel(novel_id)` | Get novel via webview |

### Search Endpoints

| Method | Description |
|---|---|
| `search_illust(word, sort, duration, target, offset)` | Search illustrations |
| `search_novel(word, sort, target, offset)` | Search novels |
| `search_user(word, offset)` | Search users |
| `trending_tags_illust()` | Get trending tags |

### Misc Endpoints

| Method | Description |
|---|---|
| `ugoira_metadata(illust_id)` | Get UGOIRA animation metadata |
| `showcase_article(showcase_id)` | Get showcase article |

## Response Models

All API methods return `ApiResponse<T>` — a hybrid wrapper carrying both typed data and raw JSON:

```rust
pub struct ApiResponse<T> {
    pub data: Option<T>,         // Parsed typed struct (None if parse fails)
    pub raw: serde_json::Value,  // Raw JSON (always available)
}
```

**Important:** Pixiv may change their API without notice. Always write a raw JSON fallback route:

```rust
let resp = api.search_illust("keyword", None, None, None, None).await?;

// Try typed access first
if let Some(data) = &resp.data {
    // Use typed fields
}

// Always have a raw fallback
let raw = &resp.raw;
```

### Available Model Types

- `models::illust::Illust` — illustration with title, tags, image URLs, etc.
- `models::user::User` — user with profile, workspace, etc.
- `models::novel::Novel` — novel with text length, series, etc.
- `models::search::{SearchSort, SearchDuration, SearchTarget}` — search enums
- `models::common::{Tag, Pagination, ImageUrls, MetaPage}` — shared types

## Error Handling

All errors are wrapped in `PixivError`:

```rust
use pixiv_api::PixivError;

match api.illust_detail(12345).await {
    Ok(resp) => { /* ... */ }
    Err(PixivError::Auth(msg)) => eprintln!("Auth error: {msg}"),
    Err(PixivError::Status(code)) => eprintln!("HTTP {code}"),
    Err(PixivError::Request(e)) => eprintln!("Request failed: {e}"),
    Err(PixivError::Parse(e)) => eprintln!("Parse error: {e}"),
    Err(e) => eprintln!("Other error: {e}"),
}
```

## Downloader

```rust
use pixiv_api::downloader::DownloadManager;

let dm = DownloadManager::new(reqwest::Client::new(), "./images");

// Single download
let path = dm.download("https://...", "image.jpg").await?;

// Concurrent downloads
let items = vec![
    ("https://...1.jpg", "1.jpg"),
    ("https://...2.jpg", "2.jpg"),
];
let results = dm.download_many(&items, 3).await; // max 3 concurrent
```

## SNI Bypass (China/GFW)

Enable the `gfw-bypass` feature:

```toml
pixiv-api = { version = "0.1", features = ["gfw-bypass"] }
```

Use it to resolve Pixiv's real IP via DNS-over-HTTPS:

```rust
#[cfg(feature = "gfw-bypass")]
{
    let api = PixivApi::new();
    let ip = api.resolve_pixiv_ip().await?;
    println!("Pixiv real IP: {ip}");
}
```

## Configuration

```rust
use pixiv_api::config::{Config, ClientConfig};
use pixiv_api::PixivApi;

// Custom config
let config = Config {
    host: "https://custom.host",
    ..Default::default()
};

let client_config = ClientConfig {
    timeout: Duration::from_secs(60),
    proxy: Some("http://127.0.0.1:7890".into()),
    ..Default::default()
};

let api = PixivApi::with_config(config, client_config);
```

## License

Unlicense
