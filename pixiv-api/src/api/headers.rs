use reqwest::header::{HeaderMap, HeaderName, HeaderValue};

use crate::PixivApi;

impl PixivApi {
    /// Set a custom header that will be included in all subsequent requests.
    ///
    /// If the header already exists, its value is replaced.
    pub async fn set_header(&self, name: HeaderName, value: &str) -> crate::Result<()> {
        let hv = HeaderValue::from_str(value)
            .map_err(|e| crate::PixivError::Other(format!("invalid header value: {e}")))?;
        let mut custom = self.custom_headers.lock().await;
        custom.insert(name, hv);
        Ok(())
    }

    /// Remove a custom header by name.
    pub async fn remove_header(&self, name: HeaderName) {
        let mut custom = self.custom_headers.lock().await;
        custom.remove(name);
    }

    /// Remove all custom headers.
    pub async fn clear_headers(&self) {
        let mut custom = self.custom_headers.lock().await;
        custom.clear();
    }

    /// Get a snapshot of the current custom headers.
    pub async fn custom_headers_snapshot(&self) -> HeaderMap {
        self.custom_headers.lock().await.clone()
    }
}

/// Generate a typed convenience setter for a well-known HTTP header.
macro_rules! custom_header_setter {
    ($method:ident, $header:expr, $doc:literal) => {
        #[doc = $doc]
        pub async fn $method(&self, value: &str) -> crate::Result<()> {
            self.set_header($header, value).await
        }
    };
}

impl PixivApi {
    custom_header_setter!(
        set_accept_lang,
        reqwest::header::ACCEPT_LANGUAGE,
        "Set the `Accept-Language` header for all requests."
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_set_and_remove_header() {
        let api = PixivApi::new();

        api.set_header(HeaderName::from_static("x-custom"), "test-value")
            .await
            .unwrap();

        let snap = api.custom_headers_snapshot().await;
        assert_eq!(
            snap.get("x-custom").unwrap().to_str().unwrap(),
            "test-value"
        );

        api.remove_header(HeaderName::from_static("x-custom")).await;
        let snap = api.custom_headers_snapshot().await;
        assert!(snap.is_empty());
    }

    #[tokio::test]
    async fn test_clear_headers() {
        let api = PixivApi::new();

        api.set_header(HeaderName::from_static("x-a"), "1")
            .await
            .unwrap();
        api.set_header(HeaderName::from_static("x-b"), "2")
            .await
            .unwrap();

        api.clear_headers().await;
        assert!(api.custom_headers_snapshot().await.is_empty());
    }

    #[tokio::test]
    async fn test_set_accept_lang() {
        let api = PixivApi::new();
        api.set_accept_lang("zh-CN").await.unwrap();

        let snap = api.custom_headers_snapshot().await;
        assert_eq!(
            snap.get(reqwest::header::ACCEPT_LANGUAGE)
                .unwrap()
                .to_str()
                .unwrap(),
            "zh-CN"
        );
    }

    #[tokio::test]
    async fn test_custom_headers_merged_into_auth_headers() {
        let api = PixivApi::new();
        api.set_accept_lang("ja").await.unwrap();
        api.set_auth("tok", "rt", 1).await;

        let headers = api.auth_headers().await.unwrap();
        assert_eq!(
            headers
                .get(reqwest::header::ACCEPT_LANGUAGE)
                .unwrap()
                .to_str()
                .unwrap(),
            "ja"
        );
        assert!(headers.get(reqwest::header::AUTHORIZATION).is_some());
    }

    #[tokio::test]
    async fn test_invalid_header_value_returns_error() {
        let api = PixivApi::new();
        let result = api
            .set_header(HeaderName::from_static("x-bad"), "\r\ninjection")
            .await;
        assert!(result.is_err());
    }
}
