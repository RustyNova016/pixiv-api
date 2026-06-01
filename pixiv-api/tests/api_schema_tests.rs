//! Real-API schema tests for all Pixiv API endpoints.
//!
//! These tests hit the live Pixiv API through a proxy and verify that
//! the response JSON matches the expected Rust struct schemas.
//!
//! # Setup
//! ```bash
//! export PIXIV_REFRESH_TOKEN="your_token_here"
//! # Ensure proxy is running at 127.0.0.1:7897
//! ```
//!
//! # Run all schema tests
//! ```bash
//! cargo test -p pixiv-client --test api_schema_tests -- --nocapture
//! ```
//!
//! # Run by domain
//! ```bash
//! cargo test -p pixiv-client --test api_schema_tests -- search_tests --nocapture
//! cargo test -p pixiv-client --test api_schema_tests -- illust_tests --nocapture
//! cargo test -p pixiv-client --test api_schema_tests -- user_tests --nocapture
//! cargo test -p pixiv-client --test api_schema_tests -- novel_misc_tests --nocapture
//! ```

use pixiv_client::{ApiResponse, ClientConfig, PixivApi};
use std::time::Duration;

/// Proxy address for the local network environment.
const PROXY: &str = "http://127.0.0.1:7897";

/// Well-known public Pixiv user IDs for testing.
const TEST_USER_ID: u64 = 11;

/// Well-known illust ID for testing (popular work).
const TEST_ILLUST_ID: u64 = 122692092;

/// Well-known novel ID for testing (public, non-restricted).
const TEST_NOVEL_ID: u64 = 19152446;

/// Well-known novel series ID for testing.
const TEST_NOVEL_SERIES_ID: u64 = 1365710;

/// Well-known comment-bearing illust ID (popular, public).
const TEST_COMMENT_ILLUST_ID: u64 = 116565505;

/// Get refresh token from env var or pixiv-dl config file.
fn get_refresh_token() -> String {
    if let Ok(token) = std::env::var("PIXIV_REFRESH_TOKEN") {
        return token;
    }
    // Fallback: read from pixiv-dl config file
    // Windows: %APPDATA%/pixiv-dl/config.json
    // Linux/macOS: ~/.config/pixiv-dl/config.json
    let config_dir = if cfg!(target_os = "windows") {
        std::env::var("APPDATA").expect("APPDATA not set")
    } else {
        let home = std::env::var("HOME").expect("HOME not set");
        format!("{home}/.config")
    };
    let config_path = std::path::Path::new(&config_dir)
        .join("pixiv-dl")
        .join("config.json");
    let content = std::fs::read_to_string(&config_path)
        .unwrap_or_else(|e| panic!("cannot read {}: {e}", config_path.display()));
    let json: serde_json::Value = serde_json::from_str(&content).expect("invalid config json");
    json["refresh_token"]
        .as_str()
        .expect("no refresh_token in config")
        .to_string()
}

/// Create an authenticated PixivApi client with proxy configured.
async fn create_client() -> PixivApi {
    let refresh_token = get_refresh_token();

    let client_config = ClientConfig {
        timeout: Duration::from_secs(60),
        proxy: Some(PROXY.to_string()),
        ..Default::default()
    };

    let api = PixivApi::with_config(Default::default(), client_config);
    api.auth(&refresh_token)
        .await
        .expect("authentication failed");
    api
}

/// Print a side-by-side comparison of expected schema vs actual JSON keys.
fn print_schema_comparison(
    type_name: &str,
    expected_fields: &[(&str, &str, bool)],
    actual: &serde_json::Value,
) {
    let actual_keys: Vec<String> = match actual {
        serde_json::Value::Object(map) => map.keys().cloned().collect(),
        _ => vec!["(not an object)".to_string()],
    };

    let sep = "=".repeat(70);
    let dash35 = "-".repeat(35);
    let dash25 = "-".repeat(25);
    let dash8 = "-".repeat(8);

    println!("\n{sep}");
    println!("SCHEMA TEST: {type_name}");
    println!("{sep}");

    println!("\n--- Expected Schema (Rust struct fields) ---");
    println!("{:<35} {:<25} {}", "Field", "Type", "Required");
    println!("{dash35} {dash25} {dash8}");
    for (name, typ, required) in expected_fields {
        let req_str = if *required { "yes" } else { "no (Option)" };
        println!("{name:<35} {typ:<25} {req_str}");
    }

    println!("\n--- Actual Response Keys ---");
    for key in &actual_keys {
        let present_in_schema = expected_fields.iter().any(|(n, _, _)| n == key);
        let marker = if present_in_schema {
            "  ✓ matched"
        } else {
            "  ✗ (extra key)"
        };
        println!("  {key:<35}{marker}");
    }

    println!("\n--- Missing from Response ---");
    let mut missing = false;
    for (name, _, required) in expected_fields {
        if *required && !actual_keys.contains(&name.to_string()) {
            println!("  {name} (required but missing!)");
            missing = true;
        }
    }
    if !missing {
        println!("  (none)");
    }

    println!("\n--- Raw JSON (first 2000 chars) ---");
    let json_str = serde_json::to_string_pretty(actual).unwrap_or_default();
    let truncated: String = json_str.chars().take(2000).collect();
    println!("{truncated}");
    if json_str.len() > 2000 {
        println!("... (truncated, {} total chars)", json_str.len());
    }
    println!("{sep}\n");
}

/// Helper to build expected field tuples concisely.
/// Usage: `sf("illusts", "Vec<Illust>", true), sf("next_url", "Option<String>", false)`
const fn sf(
    name: &'static str,
    typ: &'static str,
    required: bool,
) -> (&'static str, &'static str, bool) {
    (name, typ, required)
}

/// Assert that an ApiResponse successfully deserialized (data is Some).
fn assert_data_ok<T: std::fmt::Debug>(resp: &ApiResponse<T>) {
    assert!(
        resp.data.is_some(),
        "ApiResponse data is None — deserialization failed!\nRaw JSON: {}",
        serde_json::to_string_pretty(&resp.raw).unwrap_or_default()
    );
}

// ── Submodules ──────────────────────────────────────────────────────

#[path = "api_schema_tests/search_tests.rs"]
mod search_tests;

#[path = "api_schema_tests/illust_tests.rs"]
mod illust_tests;

#[path = "api_schema_tests/user_tests.rs"]
mod user_tests;

#[path = "api_schema_tests/novel_misc_tests.rs"]
mod novel_misc_tests;
