pub mod common;
pub mod illust;
pub mod novel;
pub mod search;
pub mod user;

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
    ///
    /// # Panics
    ///
    /// Panics if `self.data` is `None` (i.e., deserialization failed).
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
