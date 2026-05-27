# pixiv-api Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a Rust library + CLI for the Pixiv App API (6.x) with full API parity with pixivpy.

**Architecture:** Composition with split `impl` blocks across domain files. One `PixivApi` struct, methods in separate files per API domain. Hybrid response models (`ApiResponse<T>`) carry both typed structs and raw JSON. SNI bypass behind `gfw-bypass` feature flag.

**Tech Stack:** reqwest, serde/serde_json, thiserror, tokio, chrono, md-5, clap

---

## File Structure

```
pixiv-api/                          # workspace root
├── Cargo.toml                      # workspace members
├── pixiv-api/                      # library crate
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs                  # re-exports PixivApi, ApiResponse, PixivError, Result
│       ├── api/
│       │   ├── mod.rs              # PixivApi struct definition
│       │   ├── auth.rs             # auth(), set_auth(), require_auth()
│       │   ├── user.rs             # 15 user endpoint methods
│       │   ├── illust.rs           # 10 illustration endpoint methods
│       │   ├── novel.rs            # 8 novel endpoint methods
│       │   ├── search.rs           # 4 search endpoint methods
│       │   ├── misc.rs             # ugoira_metadata, showcase_article
│       │   └── bypass.rs           # SNI bypass (feature-gated)
│       ├── models/
│       │   ├── mod.rs              # ApiResponse<T>
│       │   ├── illust.rs           # Illust, ImageUrls, MetaPage, IllustType, ...
│       │   ├── user.rs             # User, Profile, Workspace, PublicProfile, ...
│       │   ├── novel.rs            # Novel, NovelSeries, NovelTag, ...
│       │   ├── search.rs           # SearchSort, SearchDuration, SearchTarget enums
│       │   └── common.rs           # Tag, Pagination, Timestamps, ImageUrls
│       ├── downloader/
│       │   └── mod.rs              # DownloadManager
│       ├── error.rs                # PixivError enum
│       └── config.rs               # Config, ClientConfig
├── pixiv-dl/                       # CLI binary crate
│   ├── Cargo.toml
│   └── src/
│       └── main.rs                 # clap CLI
├── examples/
│   └── basic_usage.rs              # library usage example
└── tests/
    └── integration.rs              # integration tests
```

---

## Task 1: Workspace Setup

Convert the existing single-crate project into a Cargo workspace with two sub-crates.

**Files:**
- Modify: `Cargo.toml` (convert to workspace root)
- Create: `pixiv-api/Cargo.toml` (library crate manifest)
- Create: `pixiv-dl/Cargo.toml` (binary crate manifest)
- Move: `src/` → `pixiv-api/src/` (via git mv)

- [ ] **Step 1: Create workspace directory structure**

```bash
mkdir -p pixiv-api/src pixiv-dl/src
```

- [ ] **Step 2: Move existing source to library crate using git mv**

```bash
git mv src/api pixiv-api/src/api
git mv src/downloader pixiv-api/src/downloader
git mv src/utils pixiv-api/src/utils
git mv src/lib.rs pixiv-api/src/lib.rs
rmdir src
```

- [ ] **Step 3: Write pixiv-api/Cargo.toml**

```toml
[package]
name = "pixiv-api"
version = "0.1.0"
edition = "2024"

[dependencies]
reqwest = { version = "0.12", features = ["json", "rustls-tls"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
thiserror = "2"
tokio = { version = "1", features = ["full"] }
chrono = { version = "0.4", features = ["serde"] }
md-5 = "0.10"
hex = "0.4"
url = "2"

[features]
gfw-bypass = []
```

- [ ] **Step 4: Write pixiv-dl/Cargo.toml**

```toml
[package]
name = "pixiv-dl"
version = "0.1.0"
edition = "2024"

[dependencies]
pixiv-api = { path = "../pixiv-api" }
clap = { version = "4", features = ["derive"] }
tokio = { version = "1", features = ["full"] }
```

- [ ] **Step 5: Convert root Cargo.toml to workspace**

```toml
[workspace]
members = ["pixiv-api", "pixiv-dl"]
resolver = "3"
```

- [ ] **Step 6: Write minimal pixiv-dl/src/main.rs**

```rust
fn main() {
    println!("pixiv-dl: not yet implemented");
}
```

- [ ] **Step 7: Update pixiv-api/src/lib.rs to remove dead imports**

```rust
pub mod api;
pub mod downloader;
pub mod error;
pub mod models;

pub use api::PixivApi;
pub use error::PixivError;
pub type Result<T> = std::result::Result<T, PixivError>;
```

- [ ] **Step 8: Verify workspace builds**

```bash
cargo build --workspace
```

Expected: Compiles with warnings (empty modules, unused code).

- [ ] **Step 9: Commit**

```bash
git add -A
git commit -m "refactor: convert to workspace with pixiv-api lib and pixiv-dl bin"
```

---

## Task 2: Error Types

**Files:**
- Create: `pixiv-api/src/error.rs`
- Modify: `pixiv-api/src/lib.rs`

- [ ] **Step 1: Write error type tests**

Create `pixiv-api/src/error.rs`:

```rust
use reqwest::StatusCode;

#[derive(Debug, thiserror::Error)]
pub enum PixivError {
    #[error("authentication failed: {0}")]
    Auth(String),

    #[error("HTTP request failed: {0}")]
    Request(#[from] reqwest::Error),

    #[error("API returned status {0}")]
    Status(StatusCode),

    #[error("failed to parse response: {0}")]
    Parse(#[from] serde_json::Error),

    #[error("download failed: {0}")]
    Download(String),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("{0}")]
    Other(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_error_display() {
        let err = PixivError::Auth("bad token".into());
        assert_eq!(err.to_string(), "authentication failed: bad token");
    }

    #[test]
    fn test_status_error_display() {
        let err = PixivError::Status(StatusCode::NOT_FOUND);
        assert!(err.to_string().contains("404"));
    }

    #[test]
    fn test_other_error() {
        let err = PixivError::Other("custom".into());
        assert_eq!(err.to_string(), "custom");
    }
}
```

- [ ] **Step 2: Run tests to verify they pass**

```bash
cargo test -p pixiv-api error
```

Expected: 3 tests pass.

- [ ] **Step 3: Remove old utils/errors.rs**

```bash
rm pixiv-api/src/utils/errors.rs
```

- [ ] **Step 4: Commit**

```bash
git add pixiv-api/src/error.rs pixiv-api/src/lib.rs
git rm pixiv-api/src/utils/errors.rs
git commit -m "feat: add unified PixivError enum with thiserror"
```

---

## Task 3: Configuration

**Files:**
- Create: `pixiv-api/src/config.rs`
- Modify: `pixiv-api/src/lib.rs`

- [ ] **Step 1: Write config with tests**

Create `pixiv-api/src/config.rs`:

```rust
use std::time::Duration;

/// Pixiv API client credentials (well-known app values, not secrets).
#[derive(Debug, Clone)]
pub struct Config {
    pub client_id: &'static str,
    pub client_secret: &'static str,
    pub hash_secret: &'static str,
    pub host: &'static str,
    pub auth_host: &'static str,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            client_id: "MOBrBDS8blbauoSck0ZfDbtuzpyT",
            client_secret: "lsACyCD94FhDUtGTXi3QzcFE2uU1hqtDaKeqrdwj",
            hash_secret: "28c1fdd170a5204386cb1313c7077b34f83e4aaf4aa829ce78c231e05b0bae2c",
            host: "https://app-api.pixiv.net",
            auth_host: "https://oauth.secure.pixiv.net",
        }
    }
}

/// HTTP client configuration.
#[derive(Debug, Clone)]
pub struct ClientConfig {
    pub timeout: Duration,
    pub user_agent: String,
    pub proxy: Option<String>,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(30),
            user_agent: "PixivAndroidApp/5.0.234 (Android 11; Pixel 5)".into(),
            proxy: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_defaults() {
        let config = Config::default();
        assert_eq!(config.host, "https://app-api.pixiv.net");
        assert_eq!(config.auth_host, "https://oauth.secure.pixiv.net");
        assert!(!config.client_id.is_empty());
    }

    #[test]
    fn test_client_config_defaults() {
        let config = ClientConfig::default();
        assert_eq!(config.timeout, Duration::from_secs(30));
        assert!(config.proxy.is_none());
    }
}
```

- [ ] **Step 2: Run tests**

```bash
cargo test -p pixiv-api config
```

Expected: 2 tests pass.

- [ ] **Step 3: Add to lib.rs**

Add `pub mod config;` to `pixiv-api/src/lib.rs`.

- [ ] **Step 4: Remove old utils/config.rs**

```bash
rm pixiv-api/src/utils/config.rs
```

- [ ] **Step 5: Commit**

```bash
git add pixiv-api/src/config.rs pixiv-api/src/lib.rs
git rm pixiv-api/src/utils/config.rs
git commit -m "feat: add Config and ClientConfig with Pixiv app defaults"
```

---

## Task 4: Models — ApiResponse Wrapper

**Files:**
- Create: `pixiv-api/src/models/mod.rs`
- Modify: `pixiv-api/src/lib.rs`

- [ ] **Step 1: Write ApiResponse with tests**

Create `pixiv-api/src/models/mod.rs`:

```rust
use serde::Deserialize;

/// Hybrid response carrying both typed data and raw JSON.
///
/// If deserialization into `T` fails (e.g., due to API changes),
/// `data` will be `None` but `raw` is always available.
///
/// **Important:** Always write a raw JSON fallback route in your code.
/// Pixiv may change their API without notice.
#[derive(Debug, Clone)]
pub struct ApiResponse<T> {
    /// Parsed typed struct. None if deserialization failed.
    pub data: Option<T>,
    /// Raw JSON value. Always available regardless of parse success.
    pub raw: serde_json::Value,
}

impl<T: for<'de> Deserialize<'de>> ApiResponse<T> {
    /// Parse a JSON value into an ApiResponse.
    /// Tries to deserialize into T; falls back to None if it fails.
    pub fn from_json(raw: serde_json::Value) -> Self {
        let data = serde_json::from_value(raw.clone()).ok();
        Self { data, raw }
    }
}

impl<T> ApiResponse<T> {
    /// Get the typed data, panicking if missing.
    pub fn unwrap(self) -> T {
        self.data.expect("ApiResponse data was None")
    }

    /// Get the typed data with a default fallback.
    pub fn unwrap_or_default(self) -> T
    where
        T: Default,
    {
        self.data.unwrap_or_default()
    }

    /// Check if typed data is available.
    pub fn is_ok(&self) -> bool {
        self.data.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;

    #[derive(Debug, Deserialize, PartialEq)]
    struct TestItem {
        id: u64,
        name: String,
    }

    #[test]
    fn test_from_json_success() {
        let raw = serde_json::json!({"id": 1, "name": "test"});
        let resp: ApiResponse<TestItem> = ApiResponse::from_json(raw);
        assert!(resp.is_ok());
        let item = resp.unwrap();
        assert_eq!(item.id, 1);
        assert_eq!(item.name, "test");
    }

    #[test]
    fn test_from_json_failure_fallback() {
        let raw = serde_json::json!({"unexpected": "shape"});
        let resp: ApiResponse<TestItem> = ApiResponse::from_json(raw);
        assert!(!resp.is_ok());
        assert_eq!(resp.raw["unexpected"], "shape");
    }

    #[test]
    fn test_raw_always_available() {
        let raw = serde_json::json!({"foo": "bar"});
        let resp: ApiResponse<TestItem> = ApiResponse::from_json(raw.clone());
        assert_eq!(resp.raw, raw);
    }
}
```

- [ ] **Step 2: Run tests**

```bash
cargo test -p pixiv-api models
```

Expected: 3 tests pass.

- [ ] **Step 3: Add models module to lib.rs**

Ensure `pixiv-api/src/lib.rs` contains `pub mod models;`.

- [ ] **Step 4: Commit**

```bash
git add pixiv-api/src/models/
git commit -m "feat: add ApiResponse<T> hybrid typed+raw response wrapper"
```

---

## Task 5: Models — Common Types

**Files:**
- Create: `pixiv-api/src/models/common.rs`
- Modify: `pixiv-api/src/models/mod.rs`

- [ ] **Step 1: Write common model types with serialization tests**

Create `pixiv-api/src/models/common.rs`:

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tag {
    pub name: String,
    #[serde(default)]
    pub translated_name: Option<String>,
    #[serde(default)]
    pub added_by_uploaded_user: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pagination {
    #[serde(default)]
    pub next_url: Option<String>,
    #[serde(default)]
    pub prev_url: Option<String>,
}

/// Extract query parameters from a Pixiv next_url for pagination.
pub fn parse_next_url(url: &str) -> Option<std::collections::HashMap<String, String>> {
    let parsed = url::Url::parse(url).ok()?;
    Some(parsed.query_pairs().into_owned().collect())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageUrls {
    #[serde(default)]
    pub square_medium: Option<String>,
    #[serde(default)]
    pub medium: Option<String>,
    #[serde(default)]
    pub large: Option<String>,
    #[serde(default)]
    pub original: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetaPage {
    #[serde(default)]
    pub image_urls: Option<ImageUrls>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetaSinglePage {
    #[serde(default)]
    pub original_image_url: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tag_deserialize() {
        let json = r#"{"name": "landscape", "translated_name": null}"#;
        let tag: Tag = serde_json::from_str(json).unwrap();
        assert_eq!(tag.name, "landscape");
        assert!(tag.translated_name.is_none());
    }

    #[test]
    fn test_pagination_parse_next_url() {
        let url = "https://app-api.pixiv.net/v1/search/illust?word=test&offset=30";
        let params = parse_next_url(url).unwrap();
        assert_eq!(params["word"], "test");
        assert_eq!(params["offset"], "30");
    }

    #[test]
    fn test_image_urls_partial() {
        let json = r#"{"medium": "https://example.com/med.jpg"}"#;
        let urls: ImageUrls = serde_json::from_str(json).unwrap();
        assert_eq!(urls.medium.as_deref(), Some("https://example.com/med.jpg"));
        assert!(urls.original.is_none());
    }
}
```

- [ ] **Step 2: Add common module to models/mod.rs**

Add `pub mod common;` at the top of `pixiv-api/src/models/mod.rs`.

- [ ] **Step 3: Run tests**

```bash
cargo test -p pixiv-api common
```

Expected: 3 tests pass.

- [ ] **Step 4: Commit**

```bash
git add pixiv-api/src/models/common.rs pixiv-api/src/models/mod.rs
git commit -m "feat: add common model types (Tag, Pagination, ImageUrls)"
```

---

## Task 6: Models — Illustration, User, Novel, Search Types

**Files:**
- Create: `pixiv-api/src/models/illust.rs`
- Create: `pixiv-api/src/models/user.rs`
- Create: `pixiv-api/src/models/novel.rs`
- Create: `pixiv-api/src/models/search.rs`
- Modify: `pixiv-api/src/models/mod.rs`

- [ ] **Step 1: Write illustration models**

Create `pixiv-api/src/models/illust.rs`:

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use super::common::{ImageUrls, MetaPage, MetaSinglePage, Tag};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IllustType {
    Illust,
    Manga,
    Ugoira,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Illust {
    pub id: u64,
    pub title: String,
    #[serde(default)]
    pub r#type: Option<IllustType>,
    #[serde(default)]
    pub image_urls: Option<ImageUrls>,
    #[serde(default)]
    pub caption: Option<String>,
    #[serde(default)]
    pub restrict: Option<i32>,
    #[serde(default)]
    pub user: Option<super::user::UserPreview>,
    #[serde(default)]
    pub tags: Option<Vec<Tag>>,
    #[serde(default)]
    pub tools: Option<Vec<String>>,
    #[serde(default)]
    pub create_date: Option<DateTime<Utc>>,
    #[serde(default)]
    pub page_count: Option<u32>,
    #[serde(default)]
    pub width: Option<u32>,
    #[serde(default)]
    pub height: Option<u32>,
    #[serde(default)]
    pub sanity_level: Option<i32>,
    #[serde(default)]
    pub x_restrict: Option<i32>,
    #[serde(default)]
    pub series: Option<SeriesRef>,
    #[serde(default)]
    pub meta_single_page: Option<MetaSinglePage>,
    #[serde(default)]
    pub meta_pages: Option<Vec<MetaPage>>,
    #[serde(default)]
    pub total_view: Option<u64>,
    #[serde(default)]
    pub total_bookmarks: Option<u64>,
    #[serde(default)]
    pub is_bookmarked: Option<bool>,
    #[serde(default)]
    pub visible: Option<bool>,
    #[serde(default)]
    pub is_muted: Option<bool>,
    #[serde(default)]
    pub total_comments: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeriesRef {
    pub id: u64,
    pub title: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IllustComments {
    #[serde(default)]
    pub comments: Vec<Comment>,
    #[serde(default)]
    pub next_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Comment {
    pub id: u64,
    pub comment: String,
    #[serde(default)]
    pub date: Option<DateTime<Utc>>,
    #[serde(default)]
    pub user: Option<super::user::UserPreview>,
    #[serde(default)]
    pub has_replies: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UgoiraMetadata {
    #[serde(default)]
    pub zip_urls: Option<UgoiraZipUrls>,
    #[serde(default)]
    pub frames: Option<Vec<UgoiraFrame>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UgoiraZipUrls {
    #[serde(default)]
    pub medium: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UgoiraFrame {
    pub file: String,
    pub delay: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_illust_type_deserialize() {
        let json = r#""illust""#;
        let t: IllustType = serde_json::from_str(json).unwrap();
        assert!(matches!(t, IllustType::Illust));
    }

    #[test]
    fn test_illust_partial_deserialize() {
        let json = r#"{"id": 12345, "title": "Test Work", "page_count": 3}"#;
        let illust: Illust = serde_json::from_str(json).unwrap();
        assert_eq!(illust.id, 12345);
        assert_eq!(illust.title, "Test Work");
        assert_eq!(illust.page_count, Some(3));
        assert!(illust.user.is_none());
    }

    #[test]
    fn test_ugoira_frame() {
        let json = r#"{"file": "000000.jpg", "delay": 80}"#;
        let frame: UgoiraFrame = serde_json::from_str(json).unwrap();
        assert_eq!(frame.delay, 80);
    }
}
```

- [ ] **Step 2: Write user models**

Create `pixiv-api/src/models/user.rs`:

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPreview {
    pub id: u64,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub account: Option<String>,
    #[serde(default)]
    pub profile_image_urls: Option<ProfileImageUrls>,
    #[serde(default)]
    pub is_followed: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileImageUrls {
    #[serde(default)]
    pub medium: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: u64,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub account: Option<String>,
    #[serde(default)]
    pub profile_image_urls: Option<ProfileImageUrls>,
    #[serde(default)]
    pub comment: Option<String>,
    #[serde(default)]
    pub is_followed: Option<bool>,
    #[serde(default)]
    pub profile: Option<Profile>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profile {
    #[serde(default)]
    pub webpage: Option<String>,
    #[serde(default)]
    pub gender: Option<String>,
    #[serde(default)]
    pub birth: Option<String>,
    #[serde(default)]
    pub birth_day: Option<String>,
    #[serde(default)]
    pub region: Option<String>,
    #[serde(default)]
    pub address_id: Option<u64>,
    #[serde(default)]
    pub country_code: Option<String>,
    #[serde(default)]
    pub job: Option<String>,
    #[serde(default)]
    pub job_id: Option<u64>,
    #[serde(default)]
    pub total_follow_users: Option<u64>,
    #[serde(default)]
    pub total_mypixiv_users: Option<u64>,
    #[serde(default)]
    pub total_illusts: Option<u64>,
    #[serde(default)]
    pub total_manga: Option<u64>,
    #[serde(default)]
    pub total_novels: Option<u64>,
    #[serde(default)]
    pub total_illust_bookmarks_public: Option<u64>,
    #[serde(default)]
    pub background_image_url: Option<String>,
    #[serde(default)]
    pub twitter_account: Option<String>,
    #[serde(default)]
    pub twitter_url: Option<String>,
    #[serde(default)]
    pub pawoo_url: Option<String>,
    #[serde(default)]
    pub is_premium: Option<bool>,
    #[serde(default)]
    pub is_using_custom_profile_image: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserDetail {
    pub user: User,
    #[serde(default)]
    pub profile: Option<Profile>,
    #[serde(default)]
    pub profile_publicity: Option<ProfilePublicity>,
    #[serde(default)]
    pub workspace: Option<Workspace>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfilePublicity {
    #[serde(default)]
    pub gender: Option<String>,
    #[serde(default)]
    pub region: Option<String>,
    #[serde(default)]
    pub birth_day: Option<String>,
    #[serde(default)]
    pub job: Option<String>,
    #[serde(default)]
    pub pawoo: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workspace {
    #[serde(default)]
    pub pc: Option<String>,
    #[serde(default)]
    pub monitor: Option<String>,
    #[serde(default)]
    pub tool: Option<String>,
    #[serde(default)]
    pub scanner: Option<String>,
    #[serde(default)]
    pub tablet: Option<String>,
    #[serde(default)]
    pub mouse: Option<String>,
    #[serde(default)]
    pub printer: Option<String>,
    #[serde(default)]
    pub desktop: Option<String>,
    #[serde(default)]
    pub music: Option<String>,
    #[serde(default)]
    pub desk: Option<String>,
    #[serde(default)]
    pub chair: Option<String>,
    #[serde(default)]
    pub comment: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_preview_partial() {
        let json = r#"{"id": 111, "name": "Artist"}"#;
        let user: UserPreview = serde_json::from_str(json).unwrap();
        assert_eq!(user.id, 111);
        assert_eq!(user.name.as_deref(), Some("Artist"));
    }

    #[test]
    fn test_user_detail_partial() {
        let json = r#"{"user": {"id": 222}}"#;
        let detail: UserDetail = serde_json::from_str(json).unwrap();
        assert_eq!(detail.user.id, 222);
    }
}
```

- [ ] **Step 3: Write novel models**

Create `pixiv-api/src/models/novel.rs`:

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use super::common::Tag;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Novel {
    pub id: u64,
    pub title: String,
    #[serde(default)]
    pub caption: Option<String>,
    #[serde(default)]
    pub restrict: Option<i32>,
    #[serde(default)]
    pub x_restrict: Option<i32>,
    #[serde(default)]
    pub is_original: Option<bool>,
    #[serde(default)]
    pub image_urls: Option<super::common::ImageUrls>,
    #[serde(default)]
    pub create_date: Option<DateTime<Utc>>,
    #[serde(default)]
    pub tags: Option<Vec<Tag>>,
    #[serde(default)]
    pub page_count: Option<u32>,
    #[serde(default)]
    pub text_length: Option<u64>,
    #[serde(default)]
    pub user: Option<super::user::UserPreview>,
    #[serde(default)]
    pub series: Option<NovelSeriesInfo>,
    #[serde(default)]
    pub is_bookmarked: Option<bool>,
    #[serde(default)]
    pub total_bookmarks: Option<u64>,
    #[serde(default)]
    pub total_view: Option<u64>,
    #[serde(default)]
    pub total_comments: Option<u64>,
    #[serde(default)]
    pub is_muted: Option<bool>,
    #[serde(default)]
    pub visible: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NovelSeriesInfo {
    pub id: u64,
    pub title: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NovelSeries {
    pub id: u64,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub caption: Option<String>,
    #[serde(default)]
    pub is_original: Option<bool>,
    #[serde(default)]
    pub is_concluded: Option<bool>,
    #[serde(default)]
    pub content_count: Option<u64>,
    #[serde(default)]
    pub total_character_count: Option<u64>,
    #[serde(default)]
    pub user: Option<super::user::UserPreview>,
    #[serde(default)]
    pub display_text: Option<String>,
    #[serde(default)]
    pub novel_ai_type: Option<i32>,
    #[serde(default)]
    pub cover_image_urls: Option<super::common::ImageUrls>,
    #[serde(default)]
    pub first_novel_id: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NovelText {
    #[serde(default)]
    pub novel_text: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_novel_partial() {
        let json = r#"{"id": 999, "title": "My Novel", "text_length": 5000}"#;
        let novel: Novel = serde_json::from_str(json).unwrap();
        assert_eq!(novel.id, 999);
        assert_eq!(novel.text_length, Some(5000));
    }

    #[test]
    fn test_novel_series() {
        let json = r#"{"id": 100, "title": "Series Name"}"#;
        let series: NovelSeries = serde_json::from_str(json).unwrap();
        assert_eq!(series.id, 100);
    }
}
```

- [ ] **Step 4: Write search models**

Create `pixiv-api/src/models/search.rs`:

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SearchSort {
    DateDesc,
    DateAsc,
    PopularDesc,
    PopularMaleDesc,
    PopularFemaleDesc,
}

impl SearchSort {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::DateDesc => "date_desc",
            Self::DateAsc => "date_asc",
            Self::PopularDesc => "popular_desc",
            Self::PopularMaleDesc => "popular_male_desc",
            Self::PopularFemaleDesc => "popular_female_desc",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SearchDuration {
    WithinLastDay,
    WithinLastWeek,
    WithinLastMonth,
    #[serde(rename = "")]
    None,
}

impl SearchDuration {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::WithinLastDay => "within_last_day",
            Self::WithinLastWeek => "within_last_week",
            Self::WithinLastMonth => "within_last_month",
            Self::None => "",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SearchTarget {
    PartialMatchForTags,
    ExactMatchForTags,
    TitleAndCaption,
    #[serde(rename = "keyword")]
    Keyword,
}

impl SearchTarget {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::PartialMatchForTags => "partial_match_for_tags",
            Self::ExactMatchForTags => "exact_match_for_tags",
            Self::TitleAndCaption => "title_and_caption",
            Self::Keyword => "keyword",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_sort_as_str() {
        assert_eq!(SearchSort::DateDesc.as_str(), "date_desc");
        assert_eq!(SearchSort::PopularDesc.as_str(), "popular_desc");
    }

    #[test]
    fn test_search_duration_none() {
        assert_eq!(SearchDuration::None.as_str(), "");
    }

    #[test]
    fn test_search_target_deserialize() {
        let json = r#""partial_match_for_tags""#;
        let target: SearchTarget = serde_json::from_str(json).unwrap();
        assert!(matches!(target, SearchTarget::PartialMatchForTags));
    }
}
```

- [ ] **Step 5: Register all model submodules in models/mod.rs**

Update `pixiv-api/src/models/mod.rs` to add:

```rust
pub mod common;
pub mod illust;
pub mod novel;
pub mod search;
pub mod user;
```

- [ ] **Step 6: Run all model tests**

```bash
cargo test -p pixiv-api models
```

Expected: All tests pass (9+ tests).

- [ ] **Step 7: Commit**

```bash
git add pixiv-api/src/models/
git commit -m "feat: add typed models for illust, user, novel, search with serde"
```

---

## Task 7: PixivApi Core Struct and Constructor

**Files:**
- Create: `pixiv-api/src/api/mod.rs`
- Modify: `pixiv-api/src/lib.rs`

- [ ] **Step 1: Write PixivApi struct with constructor test**

Create `pixiv-api/src/api/mod.rs`:

```rust
pub mod auth;
pub mod illust;
pub mod misc;
pub mod novel;
pub mod search;
pub mod user;

#[cfg(feature = "gfw-bypass")]
pub mod bypass;

use crate::config::{ClientConfig, Config};
use crate::error::PixivError;
use reqwest::Client;

/// Pixiv App API client.
///
/// # Example
/// ```rust,no_run
/// use pixiv_api::PixivApi;
///
/// # async fn example() -> Result<(), pixiv_api::PixivError> {
/// let mut api = PixivApi::new();
/// api.auth("your_refresh_token").await?;
/// # Ok(())
/// # }
/// ```
pub struct PixivApi {
    pub(crate) client: Client,
    pub(crate) access_token: Option<String>,
    pub(crate) refresh_token: Option<String>,
    pub(crate) user_id: Option<u64>,
    pub(crate) config: Config,
}

impl PixivApi {
    /// Create a new PixivApi client with default configuration.
    pub fn new() -> Self {
        Self::with_config(Config::default(), ClientConfig::default())
    }

    /// Create a new PixivApi client with custom configuration.
    pub fn with_config(config: Config, client_config: ClientConfig) -> Self {
        let mut builder = Client::builder()
            .timeout(client_config.timeout)
            .user_agent(&client_config.user_agent);

        if let Some(proxy_url) = &client_config.proxy {
            if let Ok(proxy) = reqwest::Proxy::all(proxy_url) {
                builder = builder.proxy(proxy);
            }
        }

        let client = builder.build().expect("failed to build HTTP client");

        Self {
            client,
            access_token: None,
            refresh_token: None,
            user_id: None,
            config,
        }
    }

    /// Check if the client is authenticated.
    pub fn is_authenticated(&self) -> bool {
        self.access_token.is_some()
    }

    /// Get the current user ID, if authenticated.
    pub fn user_id(&self) -> Option<u64> {
        self.user_id
    }
}

impl Default for PixivApi {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_has_defaults() {
        let api = PixivApi::new();
        assert!(!api.is_authenticated());
        assert!(api.user_id().is_none());
        assert_eq!(api.config.host, "https://app-api.pixiv.net");
    }

    #[test]
    fn test_custom_config() {
        let config = Config {
            host: "https://custom.host",
            ..Default::default()
        };
        let api = PixivApi::with_config(config, ClientConfig::default());
        assert_eq!(api.config.host, "https://custom.host");
    }

    #[test]
    fn test_default_trait() {
        let api = PixivApi::default();
        assert!(!api.is_authenticated());
    }
}
```

- [ ] **Step 2: Run tests**

```bash
cargo test -p pixiv-api api::tests
```

Expected: 3 tests pass.

- [ ] **Step 3: Update lib.rs to export from new structure**

Update `pixiv-api/src/lib.rs`:

```rust
pub mod api;
pub mod config;
pub mod downloader;
pub mod error;
pub mod models;

pub use api::PixivApi;
pub use error::PixivError;
pub use models::ApiResponse;

pub type Result<T> = std::result::Result<T, PixivError>;
```

- [ ] **Step 4: Remove old utils module**

```bash
rm -rf pixiv-api/src/utils
```

- [ ] **Step 5: Run full build**

```bash
cargo build -p pixiv-api
```

Expected: Compiles with warnings (empty endpoint modules).

- [ ] **Step 6: Commit**

```bash
git add pixiv-api/src/
git rm -r pixiv-api/src/utils
git commit -m "feat: add PixivApi core struct with constructor and config"
```

---

## Task 8: Authentication

**Files:**
- Create: `pixiv-api/src/api/auth.rs`

- [ ] **Step 1: Write auth module**

Create `pixiv-api/src/api/auth.rs`:

```rust
use crate::error::PixivError;
use crate::PixivApi;
use chrono::Utc;
use md5::{Digest, Md5};
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, REFERER};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct AuthResponse {
    access_token: String,
    refresh_token: String,
    user: AuthUser,
}

#[derive(Debug, Deserialize)]
struct AuthUser {
    id: String,
}

impl PixivApi {
    /// Authenticate with a refresh token.
    ///
    /// This is the primary authentication method. Password-based auth
    /// is deprecated by Pixiv.
    pub async fn auth(&mut self, refresh_token: &str) -> crate::Result<()> {
        let now = Utc::now().format("%Y-%m-%dT%H:%M:%S%z").to_string();
        let hash = {
            let mut hasher = Md5::new();
            hasher.update(format!("{}{}", now, self.config.hash_secret));
            format!("{:x}", hasher.finalize())
        };

        let mut headers = HeaderMap::new();
        headers.insert("x-client-time", HeaderValue::from_str(&now).unwrap());
        headers.insert("x-client-hash", HeaderValue::from_str(&hash).unwrap());
        headers.insert(
            REFERER,
            HeaderValue::from_static("https://app-api.pixiv.net/"),
        );

        let params = [
            ("client_id", self.config.client_id),
            ("client_secret", self.config.client_secret),
            ("grant_type", "refresh_token"),
            ("refresh_token", refresh_token),
        ];

        let url = format!("{}/auth/token", self.config.auth_host);
        let resp = self
            .client
            .post(&url)
            .headers(headers)
            .form(&params)
            .send()
            .await?;

        if !resp.status().is_success() {
            return Err(PixivError::Auth(format!(
                "token refresh failed with status {}",
                resp.status()
            )));
        }

        let auth_resp: AuthResponse = resp
            .json()
            .await
            .map_err(|e| PixivError::Auth(format!("failed to parse auth response: {e}")))?;

        self.access_token = Some(auth_resp.access_token);
        self.refresh_token = Some(auth_resp.refresh_token);
        self.user_id = auth_resp.user.id.parse().ok();

        Ok(())
    }

    /// Set authentication tokens manually (e.g., from a saved session).
    pub fn set_auth(&mut self, access_token: &str, refresh_token: &str, user_id: u64) {
        self.access_token = Some(access_token.to_string());
        self.refresh_token = Some(refresh_token.to_string());
        self.user_id = Some(user_id);
    }

    /// Get the current access token, if authenticated.
    pub fn access_token(&self) -> Option<&str> {
        self.access_token.as_deref()
    }

    /// Get the current refresh token, if set.
    pub fn current_refresh_token(&self) -> Option<&str> {
        self.refresh_token.as_deref()
    }

    /// Require authentication, returning an error if not authenticated.
    pub(crate) fn require_auth(&self) -> crate::Result<()> {
        if self.access_token.is_none() {
            return Err(PixivError::Auth(
                "not authenticated. Call auth() or set_auth() first.".into(),
            ));
        }
        Ok(())
    }

    /// Build default headers with Authorization bearer token.
    pub(crate) fn auth_headers(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(
            REFERER,
            HeaderValue::from_static("https://app-api.pixiv.net/"),
        );
        if let Some(token) = &self.access_token {
            headers.insert(
                AUTHORIZATION,
                HeaderValue::from_str(&format!("Bearer {token}")).unwrap(),
            );
        }
        headers
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set_auth() {
        let mut api = PixivApi::new();
        assert!(!api.is_authenticated());

        api.set_auth("access_123", "refresh_456", 789);
        assert!(api.is_authenticated());
        assert_eq!(api.access_token(), Some("access_123"));
        assert_eq!(api.current_refresh_token(), Some("refresh_456"));
        assert_eq!(api.user_id(), Some(789));
    }

    #[test]
    fn test_require_auth_fails_without_token() {
        let api = PixivApi::new();
        assert!(api.require_auth().is_err());
    }

    #[test]
    fn test_require_auth_succeeds_with_token() {
        let mut api = PixivApi::new();
        api.set_auth("token", "refresh", 1);
        assert!(api.require_auth().is_ok());
    }

    #[test]
    fn test_auth_headers_contain_bearer() {
        let mut api = PixivApi::new();
        api.set_auth("my_token", "refresh", 1);
        let headers = api.auth_headers();
        assert_eq!(
            headers.get(AUTHORIZATION).unwrap().to_str().unwrap(),
            "Bearer my_token"
        );
    }
}
```

- [ ] **Step 2: Run tests**

```bash
cargo test -p pixiv-api auth
```

Expected: 4 tests pass.

- [ ] **Step 3: Commit**

```bash
git add pixiv-api/src/api/auth.rs
git commit -m "feat: add OAuth2 refresh_token authentication"
```

---

## Task 9: Internal HTTP Helper

**Files:**
- Modify: `pixiv-api/src/api/mod.rs`

- [ ] **Step 1: Add internal request helper to PixivApi**

Add to the `impl PixivApi` block in `pixiv-api/src/api/mod.rs`:

```rust
use reqwest::Method;
use serde::de::DeserializeOwned;

impl PixivApi {
    /// Internal: make an authenticated API request and parse the response.
    pub(crate) async fn request<T: DeserializeOwned>(
        &self,
        method: Method,
        path: &str,
    ) -> crate::Result<crate::models::ApiResponse<T>> {
        self.require_auth()?;

        let url = format!("{}{path}", self.config.host);
        let resp = self
            .client
            .request(method, &url)
            .headers(self.auth_headers())
            .send()
            .await?;

        if !resp.status().is_success() {
            return Err(PixivError::Status(resp.status()));
        }

        let raw: serde_json::Value = resp.json().await?;
        Ok(crate::models::ApiResponse::from_json(raw))
    }

}
```

- [ ] **Step 2: Verify build**

```bash
cargo build -p pixiv-api
```

Expected: Compiles successfully.

- [ ] **Step 3: Commit**

```bash
git add pixiv-api/src/api/mod.rs
git commit -m "feat: add internal authenticated HTTP request helpers"
```

---

## Task 10: User Endpoints

**Files:**
- Create: `pixiv-api/src/api/user.rs`

- [ ] **Step 1: Write all user endpoint methods**

Create `pixiv-api/src/api/user.rs`:

```rust
use crate::models::ApiResponse;
use crate::PixivApi;
use reqwest::Method;

impl PixivApi {
    /// Get user details.
    pub async fn user_detail(&self, user_id: u64) -> crate::Result<ApiResponse<serde_json::Value>> {
        self.request(Method::GET, &format!("/v1/user/detail?user_id={user_id}"))
            .await
    }

    /// Get user's illustrations.
    pub async fn user_illusts(
        &self,
        user_id: u64,
        r#type: Option<&str>,
        offset: Option<u32>,
    ) -> crate::Result<ApiResponse<serde_json::Value>> {
        let mut path = format!("/v1/user/illusts?user_id={user_id}");
        if let Some(t) = r#type {
            path.push_str(&format!("&type={t}"));
        }
        if let Some(o) = offset {
            path.push_str(&format!("&offset={o}"));
        }
        self.request(Method::GET, &path).await
    }

    /// Get user's bookmarked illustrations.
    pub async fn user_bookmarks_illust(
        &self,
        user_id: u64,
        restrict: Option<&str>,
        max_bookmark_id: Option<u64>,
        tag: Option<&str>,
    ) -> crate::Result<ApiResponse<serde_json::Value>> {
        let mut path = format!("/v1/user/bookmarks/illust?user_id={user_id}");
        if let Some(r) = restrict {
            path.push_str(&format!("&restrict={r}"));
        }
        if let Some(m) = max_bookmark_id {
            path.push_str(&format!("&max_bookmark_id={m}"));
        }
        if let Some(t) = tag {
            path.push_str(&format!("&tag={t}"));
        }
        self.request(Method::GET, &path).await
    }

    /// Get user's bookmarked novels.
    pub async fn user_bookmarks_novel(
        &self,
        user_id: u64,
        restrict: Option<&str>,
        max_bookmark_id: Option<u64>,
    ) -> crate::Result<ApiResponse<serde_json::Value>> {
        let mut path = format!("/v1/user/bookmarks/novel?user_id={user_id}");
        if let Some(r) = restrict {
            path.push_str(&format!("&restrict={r}"));
        }
        if let Some(m) = max_bookmark_id {
            path.push_str(&format!("&max_bookmark_id={m}"));
        }
        self.request(Method::GET, &path).await
    }

    /// Get users related to the given user.
    pub async fn user_related(&self, user_id: u64) -> crate::Result<ApiResponse<serde_json::Value>> {
        self.request(Method::GET, &format!("/v1/user/related?seed_user_id={user_id}"))
            .await
    }

    /// Get recommended users.
    pub async fn user_recommended(&self) -> crate::Result<ApiResponse<serde_json::Value>> {
        self.request(Method::GET, "/v1/user/recommended").await
    }

    /// Get users the given user is following.
    pub async fn user_following(
        &self,
        user_id: u64,
        restrict: Option<&str>,
        offset: Option<u32>,
    ) -> crate::Result<ApiResponse<serde_json::Value>> {
        let mut path = format!("/v1/user/following?user_id={user_id}");
        if let Some(r) = restrict {
            path.push_str(&format!("&restrict={r}"));
        }
        if let Some(o) = offset {
            path.push_str(&format!("&offset={o}"));
        }
        self.request(Method::GET, &path).await
    }

    /// Get user's followers.
    pub async fn user_follower(
        &self,
        user_id: u64,
        offset: Option<u32>,
    ) -> crate::Result<ApiResponse<serde_json::Value>> {
        let mut path = format!("/v1/user/follower?user_id={user_id}");
        if let Some(o) = offset {
            path.push_str(&format!("&offset={o}"));
        }
        self.request(Method::GET, &path).await
    }

    /// Get user's Pixiv friends (mypixiv).
    pub async fn user_mypixiv(
        &self,
        user_id: u64,
        offset: Option<u32>,
    ) -> crate::Result<ApiResponse<serde_json::Value>> {
        let mut path = format!("/v1/user/mypixiv?user_id={user_id}");
        if let Some(o) = offset {
            path.push_str(&format!("&offset={o}"));
        }
        self.request(Method::GET, &path).await
    }

    /// Get user list by IDs.
    pub async fn user_list(
        &self,
        user_ids: &[u64],
    ) -> crate::Result<ApiResponse<serde_json::Value>> {
        let ids = user_ids
            .iter()
            .map(|id| id.to_string())
            .collect::<Vec<_>>()
            .join(",");
        self.request(Method::GET, &format!("/v2/user/list?user_ids={ids}"))
            .await
    }

    /// Get user's novels.
    pub async fn user_novels(
        &self,
        user_id: u64,
        offset: Option<u32>,
    ) -> crate::Result<ApiResponse<serde_json::Value>> {
        let mut path = format!("/v1/user/novels?user_id={user_id}");
        if let Some(o) = offset {
            path.push_str(&format!("&offset={o}"));
        }
        self.request(Method::GET, &path).await
    }

    /// Follow a user.
    pub async fn user_follow_add(
        &self,
        user_id: u64,
        restrict: Option<&str>,
    ) -> crate::Result<ApiResponse<serde_json::Value>> {
        self.require_auth()?;
        let url = format!("{}/v1/user/follow/add", self.config.host);
        let mut params = vec![("user_id", user_id.to_string())];
        if let Some(r) = restrict {
            params.push(("restrict".into(), r.into()));
        }
        let resp = self
            .client
            .post(&url)
            .headers(self.auth_headers())
            .form(&params)
            .send()
            .await?;
        if !resp.status().is_success() {
            return Err(crate::PixivError::Status(resp.status()));
        }
        let raw: serde_json::Value = resp.json().await?;
        Ok(crate::models::ApiResponse::from_json(raw))
    }

    /// Unfollow a user.
    pub async fn user_follow_delete(
        &self,
        user_id: u64,
    ) -> crate::Result<ApiResponse<serde_json::Value>> {
        self.require_auth()?;
        let url = format!("{}/v1/user/follow/delete", self.config.host);
        let params = vec![("user_id", user_id.to_string())];
        let resp = self
            .client
            .post(&url)
            .headers(self.auth_headers())
            .form(&params)
            .send()
            .await?;
        if !resp.status().is_success() {
            return Err(crate::PixivError::Status(resp.status()));
        }
        let raw: serde_json::Value = resp.json().await?;
        Ok(crate::models::ApiResponse::from_json(raw))
    }

    /// Get user's bookmark tags for illustrations.
    pub async fn user_bookmark_tags_illust(
        &self,
        user_id: u64,
        restrict: Option<&str>,
    ) -> crate::Result<ApiResponse<serde_json::Value>> {
        let mut path = format!("/v1/user/bookmark-tags/illust?user_id={user_id}");
        if let Some(r) = restrict {
            path.push_str(&format!("&restrict={r}"));
        }
        self.request(Method::GET, &path).await
    }

    /// Edit user's AI show settings.
    pub async fn user_edit_ai_show_settings(
        &self,
        illust_ai_type: i32,
    ) -> crate::Result<ApiResponse<serde_json::Value>> {
        self.require_auth()?;
        let url = format!("{}/v1/user/edit-ai-show-settings", self.config.host);
        let params = vec![("illust_ai_type", illust_ai_type.to_string())];
        let resp = self
            .client
            .post(&url)
            .headers(self.auth_headers())
            .form(&params)
            .send()
            .await?;
        if !resp.status().is_success() {
            return Err(crate::PixivError::Status(resp.status()));
        }
        let raw: serde_json::Value = resp.json().await?;
        Ok(crate::models::ApiResponse::from_json(raw))
    }
}
```

- [ ] **Step 2: Verify build**

```bash
cargo build -p pixiv-api
```

Expected: Compiles successfully.

- [ ] **Step 3: Commit**

```bash
git add pixiv-api/src/api/user.rs
git commit -m "feat: add 15 user API endpoint methods"
```

---

## Task 11: Illustration Endpoints

**Files:**
- Create: `pixiv-api/src/api/illust.rs`

- [ ] **Step 1: Write all illustration endpoint methods**

Create `pixiv-api/src/api/illust.rs`:

```rust
use crate::models::ApiResponse;
use crate::PixivApi;
use reqwest::Method;

impl PixivApi {
    /// Get illustration details.
    pub async fn illust_detail(&self, illust_id: u64) -> crate::Result<ApiResponse<serde_json::Value>> {
        self.request(Method::GET, &format!("/v1/illust/detail?illust_id={illust_id}"))
            .await
    }

    /// Get illustration comments.
    pub async fn illust_comments(
        &self,
        illust_id: u64,
        offset: Option<u32>,
    ) -> crate::Result<ApiResponse<serde_json::Value>> {
        let mut path = format!("/v1/illust/comments?illust_id={illust_id}");
        if let Some(o) = offset {
            path.push_str(&format!("&offset={o}"));
        }
        self.request(Method::GET, &path).await
    }

    /// Get related illustrations.
    pub async fn illust_related(&self, illust_id: u64) -> crate::Result<ApiResponse<serde_json::Value>> {
        self.request(Method::GET, &format!("/v2/illust/related?illust_id={illust_id}"))
            .await
    }

    /// Get recommended illustrations.
    pub async fn illust_recommended(&self) -> crate::Result<ApiResponse<serde_json::Value>> {
        self.request(Method::GET, "/v1/illust/recommended").await
    }

    /// Get illustration ranking.
    pub async fn illust_ranking(
        &self,
        mode: Option<&str>,
        date: Option<&str>,
        offset: Option<u32>,
    ) -> crate::Result<ApiResponse<serde_json::Value>> {
        let mut path = "/v1/illust/ranking?".to_string();
        if let Some(m) = mode {
            path.push_str(&format!("mode={m}&"));
        }
        if let Some(d) = date {
            path.push_str(&format!("date={d}&"));
        }
        if let Some(o) = offset {
            path.push_str(&format!("offset={o}"));
        }
        self.request(Method::GET, &path).await
    }

    /// Get illustrations from followed artists.
    pub async fn illust_follow(&self, restrict: Option<&str>) -> crate::Result<ApiResponse<serde_json::Value>> {
        let mut path = "/v2/illust/follow?".to_string();
        if let Some(r) = restrict {
            path.push_str(&format!("restrict={r}"));
        }
        self.request(Method::GET, &path).await
    }

    /// Get newest illustrations.
    pub async fn illust_new(&self) -> crate::Result<ApiResponse<serde_json::Value>> {
        self.request(Method::GET, "/v1/illust/new").await
    }

    /// Get bookmark detail for an illustration.
    pub async fn illust_bookmark_detail(
        &self,
        illust_id: u64,
    ) -> crate::Result<ApiResponse<serde_json::Value>> {
        self.request(Method::GET, &format!("/v2/illust/bookmark/detail?illust_id={illust_id}"))
            .await
    }

    /// Add an illustration bookmark.
    pub async fn illust_bookmark_add(
        &self,
        illust_id: u64,
        restrict: Option<&str>,
        tags: Option<&[&str]>,
    ) -> crate::Result<ApiResponse<serde_json::Value>> {
        self.require_auth()?;
        let url = format!("{}/v2/illust/bookmark/add", self.config.host);
        let mut params = vec![("illust_id", illust_id.to_string())];
        if let Some(r) = restrict {
            params.push(("restrict".into(), r.into()));
        }
        if let Some(t) = tags {
            params.push(("tags".into(), t.join(" ")));
        }
        let resp = self
            .client
            .post(&url)
            .headers(self.auth_headers())
            .form(&params)
            .send()
            .await?;
        if !resp.status().is_success() {
            return Err(crate::PixivError::Status(resp.status()));
        }
        let raw: serde_json::Value = resp.json().await?;
        Ok(crate::models::ApiResponse::from_json(raw))
    }

    /// Remove an illustration bookmark.
    pub async fn illust_bookmark_delete(
        &self,
        illust_id: u64,
    ) -> crate::Result<ApiResponse<serde_json::Value>> {
        self.require_auth()?;
        let url = format!("{}/v1/illust/bookmark/delete", self.config.host);
        let params = vec![("illust_id", illust_id.to_string())];
        let resp = self
            .client
            .post(&url)
            .headers(self.auth_headers())
            .form(&params)
            .send()
            .await?;
        if !resp.status().is_success() {
            return Err(crate::PixivError::Status(resp.status()));
        }
        let raw: serde_json::Value = resp.json().await?;
        Ok(crate::models::ApiResponse::from_json(raw))
    }
}
```

- [ ] **Step 2: Verify build**

```bash
cargo build -p pixiv-api
```

- [ ] **Step 3: Commit**

```bash
git add pixiv-api/src/api/illust.rs
git commit -m "feat: add 10 illustration API endpoint methods"
```

---

## Task 12: Novel Endpoints

**Files:**
- Create: `pixiv-api/src/api/novel.rs`

- [ ] **Step 1: Write all novel endpoint methods**

Create `pixiv-api/src/api/novel.rs`:

```rust
use crate::models::ApiResponse;
use crate::PixivApi;
use reqwest::Method;

impl PixivApi {
    /// Get novel details.
    pub async fn novel_detail(&self, novel_id: u64) -> crate::Result<ApiResponse<serde_json::Value>> {
        self.request(Method::GET, &format!("/v2/novel/detail?novel_id={novel_id}"))
            .await
    }

    /// Get novel comments.
    pub async fn novel_comments(
        &self,
        novel_id: u64,
        offset: Option<u32>,
    ) -> crate::Result<ApiResponse<serde_json::Value>> {
        let mut path = format!("/v1/novel/comments?novel_id={novel_id}");
        if let Some(o) = offset {
            path.push_str(&format!("&offset={o}"));
        }
        self.request(Method::GET, &path).await
    }

    /// Get recommended novels.
    pub async fn novel_recommended(&self) -> crate::Result<ApiResponse<serde_json::Value>> {
        self.request(Method::GET, "/v1/novel/recommended").await
    }

    /// Get newest novels.
    pub async fn novel_new(&self) -> crate::Result<ApiResponse<serde_json::Value>> {
        self.request(Method::GET, "/v1/novel/new").await
    }

    /// Get novels from followed artists.
    pub async fn novel_follow(&self, restrict: Option<&str>) -> crate::Result<ApiResponse<serde_json::Value>> {
        let mut path = "/v1/novel/follow?".to_string();
        if let Some(r) = restrict {
            path.push_str(&format!("restrict={r}"));
        }
        self.request(Method::GET, &path).await
    }

    /// Get novel series info.
    pub async fn novel_series(&self, series_id: u64) -> crate::Result<ApiResponse<serde_json::Value>> {
        self.request(Method::GET, &format!("/v2/novel/series?series_id={series_id}"))
            .await
    }

    /// Get novel text content.
    pub async fn novel_text(&self, novel_id: u64) -> crate::Result<ApiResponse<serde_json::Value>> {
        self.request(Method::GET, &format!("/v1/novel/text?novel_id={novel_id}"))
            .await
    }

    /// Get novel via webview (raw HTML extraction).
    pub async fn webview_novel(
        &self,
        novel_id: u64,
        raw: Option<bool>,
    ) -> crate::Result<ApiResponse<serde_json::Value>> {
        self.require_auth()?;
        let url = format!(
            "{}/webview/v2/novel?id={novel_id}&viewer_version=20221031",
            self.config.host
        );
        let resp = self
            .client
            .get(&url)
            .headers(self.auth_headers())
            .send()
            .await?;
        if !resp.status().is_success() {
            return Err(crate::PixivError::Status(resp.status()));
        }
        let raw_json: serde_json::Value = resp.json().await?;
        Ok(crate::models::ApiResponse::from_json(raw_json))
    }
}
```

- [ ] **Step 2: Verify build**

```bash
cargo build -p pixiv-api
```

- [ ] **Step 3: Commit**

```bash
git add pixiv-api/src/api/novel.rs
git commit -m "feat: add 8 novel API endpoint methods"
```

---

## Task 13: Search and Misc Endpoints

**Files:**
- Create: `pixiv-api/src/api/search.rs`
- Create: `pixiv-api/src/api/misc.rs`

- [ ] **Step 1: Write search endpoints**

Create `pixiv-api/src/api/search.rs`:

```rust
use crate::models::ApiResponse;
use crate::PixivApi;
use reqwest::Method;

impl PixivApi {
    /// Search illustrations.
    pub async fn search_illust(
        &self,
        word: &str,
        sort: Option<&str>,
        duration: Option<&str>,
        search_target: Option<&str>,
        offset: Option<u32>,
    ) -> crate::Result<ApiResponse<serde_json::Value>> {
        let mut path = format!("/v1/search/illust?word={word}");
        if let Some(s) = sort {
            path.push_str(&format!("&sort={s}"));
        }
        if let Some(d) = duration {
            path.push_str(&format!("&duration={d}"));
        }
        if let Some(t) = search_target {
            path.push_str(&format!("&search_target={t}"));
        }
        if let Some(o) = offset {
            path.push_str(&format!("&offset={o}"));
        }
        self.request(Method::GET, &path).await
    }

    /// Search novels.
    pub async fn search_novel(
        &self,
        word: &str,
        sort: Option<&str>,
        search_target: Option<&str>,
        offset: Option<u32>,
    ) -> crate::Result<ApiResponse<serde_json::Value>> {
        let mut path = format!("/v1/search/novel?word={word}");
        if let Some(s) = sort {
            path.push_str(&format!("&sort={s}"));
        }
        if let Some(t) = search_target {
            path.push_str(&format!("&search_target={t}"));
        }
        if let Some(o) = offset {
            path.push_str(&format!("&offset={o}"));
        }
        self.request(Method::GET, &path).await
    }

    /// Search users.
    pub async fn search_user(
        &self,
        word: &str,
        offset: Option<u32>,
    ) -> crate::Result<ApiResponse<serde_json::Value>> {
        let mut path = format!("/v1/search/user?word={word}");
        if let Some(o) = offset {
            path.push_str(&format!("&offset={o}"));
        }
        self.request(Method::GET, &path).await
    }

    /// Get trending illustration tags.
    pub async fn trending_tags_illust(&self) -> crate::Result<ApiResponse<serde_json::Value>> {
        self.request(Method::GET, "/v1/trending-tags/illust").await
    }
}
```

- [ ] **Step 2: Write misc endpoints**

Create `pixiv-api/src/api/misc.rs`:

```rust
use crate::models::ApiResponse;
use crate::PixivApi;
use reqwest::Method;

impl PixivApi {
    /// Get UGOIRA animation metadata.
    pub async fn ugoira_metadata(&self, illust_id: u64) -> crate::Result<ApiResponse<serde_json::Value>> {
        self.request(Method::GET, &format!("/v1/ugoira/metadata?illust_id={illust_id}"))
            .await
    }

    /// Get showcase article.
    pub async fn showcase_article(&self, showcase_id: &str) -> crate::Result<ApiResponse<serde_json::Value>> {
        self.request(
            Method::GET,
            &format!("/v1/showcase/article?showcase_id={showcase_id}"),
        )
        .await
    }
}
```

- [ ] **Step 3: Verify build**

```bash
cargo build -p pixiv-api
```

- [ ] **Step 4: Commit**

```bash
git add pixiv-api/src/api/search.rs pixiv-api/src/api/misc.rs
git commit -m "feat: add search (4) and misc (2) API endpoint methods"
```

---

## Task 14: Downloader

**Files:**
- Rewrite: `pixiv-api/src/downloader/mod.rs`

- [ ] **Step 1: Write DownloadManager with tests**

Rewrite `pixiv-api/src/downloader/mod.rs`:

```rust
use crate::error::PixivError;
use std::path::{Path, PathBuf};
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

        fs::create_dir_all(&self.output_dir)
            .await
            .map_err(|e| PixivError::Io(e))?;

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
                let _permit = sem.acquire().await.unwrap();
                let dm = DownloadManager::new(client, dir);
                dm.download(&url, &filename).await
            }));
        }

        let mut results = Vec::new();
        for handle in handles {
            results.push(handle.await.unwrap());
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
```

- [ ] **Step 2: Run tests**

```bash
cargo test -p pixiv-api downloader
```

Expected: 1 test passes.

- [ ] **Step 3: Commit**

```bash
git add pixiv-api/src/downloader/mod.rs
git commit -m "feat: add DownloadManager with concurrent download support"
```

---

## Task 15: SNI Bypass (Feature-Gated)

**Files:**
- Create: `pixiv-api/src/api/bypass.rs`

- [ ] **Step 1: Write bypass module**

Create `pixiv-api/src/api/bypass.rs`:

```rust
use crate::PixivApi;

/// DNS-over-HTTPS response from Cloudflare.
#[derive(serde::Deserialize)]
struct DnsResponse {
    Answer: Option<Vec<DnsAnswer>>,
}

#[derive(serde::Deserialize)]
struct DnsAnswer {
    data: String,
}

impl PixivApi {
    /// Resolve the real IP for app-api.pixiv.net via DNS-over-HTTPS.
    /// Uses Cloudflare DoH as primary, Google DoH as fallback.
    pub async fn resolve_pixiv_ip(&self) -> crate::Result<String> {
        let hostname = "app-api.pixiv.net";

        // Try Cloudflare DoH first
        if let Ok(ip) = self.resolve_via_doh("https://cloudflare-dns.com/dns-query", hostname).await {
            return Ok(ip);
        }

        // Fallback to Google DoH
        self.resolve_via_doh("https://dns.google/resolve", hostname).await
    }

    async fn resolve_via_doh(&self, endpoint: &str, hostname: &str) -> crate::Result<String> {
        let url = format!("{endpoint}?name={hostname}&type=A");
        let resp: DnsResponse = self
            .client
            .get(&url)
            .header("Accept", "application/dns-json")
            .send()
            .await
            .map_err(|e| crate::PixivError::Other(e.to_string()))?
            .json()
            .await
            .map_err(|e| crate::PixivError::Other(e.to_string()))?;

        resp.Answer
            .and_then(|answers| answers.first().map(|a| a.data.clone()))
            .ok_or_else(|| crate::PixivError::Other("no DNS answer".into()))
    }
}
```

- [ ] **Step 2: Verify build with feature flag**

```bash
cargo build -p pixiv-api --features gfw-bypass
```

Expected: Compiles successfully.

- [ ] **Step 3: Verify build without feature flag**

```bash
cargo build -p pixiv-api
```

Expected: Compiles (bypass module excluded by cfg).

- [ ] **Step 4: Commit**

```bash
git add pixiv-api/src/api/bypass.rs
git commit -m "feat: add SNI bypass via DNS-over-HTTPS (gfw-bypass feature)"
```

---

## Task 16: CLI Binary

**Files:**
- Rewrite: `pixiv-dl/src/main.rs`
- Modify: `pixiv-dl/Cargo.toml`

- [ ] **Step 1: Write CLI with clap**

Rewrite `pixiv-dl/src/main.rs`:

```rust
use clap::{Parser, Subcommand};
use pixiv_api::PixivApi;

#[derive(Parser)]
#[command(name = "pixiv-dl")]
#[command(version, about = "Pixiv illustration downloader")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Authenticate with a refresh token
    Auth {
        /// Your Pixiv refresh token
        #[arg(short, long)]
        token: String,
    },
    /// Search for illustrations
    Search {
        /// Search keyword
        keyword: String,
        /// Sort order (date_desc, date_asc, popular_desc, popular_male_desc, popular_female_desc)
        #[arg(short, long, default_value = "date_desc")]
        sort: String,
        /// Page offset
        #[arg(short, long, default_value = "0")]
        offset: u32,
    },
    /// Show illustration details
    Illust {
        /// Illustration ID
        id: u64,
    },
    /// Download illustrations by ID
    Download {
        /// Illustration IDs to download
        ids: Vec<u64>,
        /// Output directory
        #[arg(short, long, default_value = "./images")]
        output: String,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Auth { token } => {
            let mut api = PixivApi::new();
            api.auth(&token).await?;
            println!("Authenticated successfully.");
            println!("User ID: {:?}", api.user_id());
            println!("Access token: {}...", &api.access_token().unwrap_or("")[..8]);
        }
        Commands::Search { keyword, sort, offset } => {
            let api = authenticated_api().await?;
            let result = api
                .search_illust(&keyword, Some(&sort), None, None, Some(offset))
                .await?;
            if let Some(data) = &result.data {
                println!("Search results (raw JSON):");
                println!("{}", serde_json::to_string_pretty(data)?);
            } else {
                println!("Failed to parse response. Raw JSON:");
                println!("{}", serde_json::to_string_pretty(&result.raw)?);
            }
        }
        Commands::Illust { id } => {
            let api = authenticated_api().await?;
            let result = api.illust_detail(id).await?;
            if let Some(data) = &result.data {
                println!("Illustration details (raw JSON):");
                println!("{}", serde_json::to_string_pretty(data)?);
            } else {
                println!("Failed to parse response. Raw JSON:");
                println!("{}", serde_json::to_string_pretty(&result.raw)?);
            }
        }
        Commands::Download { ids, output } => {
            let api = authenticated_api().await?;
            for id in ids {
                let detail = api.illust_detail(id).await?;
                println!("Downloading illustration {id}...");
                // Extract image URL from raw JSON (works even if typed parse fails)
                let image_url = detail.raw["illust"]["image_urls"]["large"]
                    .as_str()
                    .or_else(|| detail.raw["illust"]["meta_single_page"]["original_image_url"].as_str());
                if let Some(url) = image_url {
                    let dm = pixiv_api::downloader::DownloadManager::new(
                        reqwest::Client::new(),
                        &output,
                    );
                    let ext = if url.contains(".png") { "png" } else { "jpg" };
                    let filename = format!("{id}.{ext}");
                    match dm.download(url, &filename).await {
                        Ok(path) => println!("  Saved to {}", path.display()),
                        Err(e) => eprintln!("  Failed: {e}"),
                    }
                } else {
                    eprintln!("  Could not find image URL for {id}");
                }
            }
        }
    }

    Ok(())
}

async fn authenticated_api() -> Result<PixivApi, Box<dyn std::error::Error>> {
    // Try environment variable first
    let token = std::env::var("PIXIV_REFRESH_TOKEN")
        .map_err(|_| "Set PIXIV_REFRESH_TOKEN env var or use 'pixiv-dl auth' first")?;

    let mut api = PixivApi::new();
    api.auth(&token).await?;
    Ok(api)
}
```

- [ ] **Step 2: Add serde_json dependency to pixiv-dl**

Update `pixiv-dl/Cargo.toml`:

```toml
[package]
name = "pixiv-dl"
version = "0.1.0"
edition = "2024"

[dependencies]
pixiv-api = { path = "../pixiv-api" }
clap = { version = "4", features = ["derive"] }
tokio = { version = "1", features = ["full"] }
serde_json = "1"
reqwest = "0.12"
```

- [ ] **Step 3: Verify build**

```bash
cargo build --workspace
```

Expected: Compiles successfully.

- [ ] **Step 4: Test help output**

```bash
cargo run -p pixiv-dl -- --help
```

Expected: Shows help with auth/search/illust/download subcommands.

- [ ] **Step 5: Commit**

```bash
git add pixiv-dl/
git commit -m "feat: add CLI binary with auth/search/illust/download commands"
```

---

## Task 17: Examples and Integration Test

**Files:**
- Create: `examples/basic_usage.rs`
- Create: `tests/integration.rs`

- [ ] **Step 1: Write basic usage example**

Create `examples/basic_usage.rs`:

```rust
use pixiv_api::PixivApi;

#[tokio::main]
async fn main() -> Result<(), pixiv_api::PixivError> {
    // Create a new client
    let mut api = PixivApi::new();

    // Authenticate (requires PIXIV_REFRESH_TOKEN env var)
    let token = std::env::var("PIXIV_REFRESH_TOKEN")
        .expect("Set PIXIV_REFRESH_TOKEN environment variable");
    api.auth(&token).await?;
    println!("Authenticated as user {:?}", api.user_id());

    // Search for illustrations (typed response with raw fallback)
    let results = api.search_illust("landscape", Some("popular_desc"), None, None, None).await?;
    if let Some(data) = &results.data {
        println!("Got typed response: {}", serde_json::to_string(data).unwrap().len());
    }
    // Always available regardless of parse success
    println!("Raw JSON available: {}", results.raw.is_object());

    // Get illustration detail
    let detail = api.illust_detail(12345).await?;
    println!("Illustration raw: {}", serde_json::to_string_pretty(&detail.raw).unwrap());

    Ok(())
}
```

- [ ] **Step 2: Write integration test**

Create `tests/integration.rs`:

```rust
use pixiv_api::{PixivApi, ApiResponse};

#[test]
fn test_client_creation() {
    let api = PixivApi::new();
    assert!(!api.is_authenticated());
}

#[test]
fn test_set_auth() {
    let mut api = PixivApi::new();
    api.set_auth("token", "refresh", 123);
    assert!(api.is_authenticated());
    assert_eq!(api.user_id(), Some(123));
}

#[test]
fn test_api_response_from_json() {
    let raw = serde_json::json!({"id": 1, "title": "test"});
    let resp: ApiResponse<serde_json::Value> = ApiResponse::from_json(raw);
    assert!(resp.is_ok());
    assert_eq!(resp.raw["id"], 1);
}

#[test]
fn test_api_response_parse_failure() {
    let raw = serde_json::json!({"unexpected": "shape"});
    // Even if typed parse fails, raw is always available
    let resp: ApiResponse<serde_json::Value> = ApiResponse::from_json(raw);
    assert!(resp.is_ok()); // Value always parses
    assert_eq!(resp.raw["unexpected"], "shape");
}
```

- [ ] **Step 3: Run all tests**

```bash
cargo test --workspace
```

Expected: All tests pass.

- [ ] **Step 4: Commit**

```bash
git add examples/ tests/
git commit -m "docs: add usage examples and integration tests"
```

---

## Task 18: Final Cleanup

- [ ] **Step 1: Remove stale files**

```bash
rm -f pixiv-api/src/utils/mod.rs
rmdir pixiv-api/src/utils 2>/dev/null || true
```

- [ ] **Step 2: Run full build with all features**

```bash
cargo build --workspace --all-features
```

- [ ] **Step 3: Run clippy**

```bash
cargo clippy --workspace --all-features -- -D warnings
```

Expected: No warnings or errors.

- [ ] **Step 4: Run all tests**

```bash
cargo test --workspace
```

Expected: All tests pass.

- [ ] **Step 5: Final commit**

```bash
git add -A
git commit -m "chore: cleanup stale files and verify full build"
```
