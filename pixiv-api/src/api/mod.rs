pub mod auth;
pub mod headers;
pub mod illust;
pub mod misc;
pub mod novel;
pub mod search;
pub mod user;

#[cfg(feature = "gfw-bypass")]
pub mod bypass;

use crate::config::{ClientConfig, Config};
use crate::error::PixivError;
use reqwest::header::{AUTHORIZATION, HeaderMap, HeaderValue, REFERER};
use reqwest::{Client, Method};
use serde::de::DeserializeOwned;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Pixiv App API client.
///
/// # Example
/// ```rust,no_run
/// use pixiv_client::PixivApi;
///
/// # async fn example() -> Result<(), pixiv_client::PixivError> {
/// let api = PixivApi::new();
/// api.auth("your_refresh_token").await?;
/// # Ok(())
/// # }
/// ```
pub struct PixivApi {
    pub(crate) client: Client,
    pub(crate) tokens: Mutex<(Option<String>, Option<String>, Option<u64>)>,
    pub(crate) config: Config,
    pub(crate) custom_headers: Arc<Mutex<HeaderMap>>,
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

        if let Some(proxy_url) = &client_config.proxy
            && let Ok(proxy) = reqwest::Proxy::all(proxy_url)
        {
            builder = builder.proxy(proxy);
        }

        let client = builder.build().expect("failed to build HTTP client");

        Self {
            client,
            tokens: Mutex::new((None, None, None)),
            config,
            custom_headers: Arc::new(Mutex::new(HeaderMap::new())),
        }
    }

    /// Check if the client is authenticated.
    pub async fn is_authenticated(&self) -> bool {
        let tokens = self.tokens.lock().await;
        tokens.0.is_some()
    }

    /// Get the current user ID, if authenticated.
    pub async fn user_id(&self) -> Option<u64> {
        let tokens = self.tokens.lock().await;
        tokens.2
    }

    /// Refresh the access token using the stored refresh token.
    ///
    /// Call this explicitly when you receive a [`PixivError::Status(401)`]
    /// to obtain fresh credentials. The new tokens are stored internally
    /// and will be used by subsequent API calls.
    ///
    /// Returns `Err` if no refresh token is stored or the refresh request fails.
    pub async fn refresh_token(&self) -> crate::Result<()> {
        let rt = {
            let tokens = self.tokens.lock().await;
            tokens.1.clone()
        };
        let rt = rt.ok_or_else(|| PixivError::Auth("no refresh token available".into()))?;

        let (new_access, new_refresh, new_uid) =
            Self::fetch_tokens(&self.client, &self.config, &rt).await?;

        let mut tokens = self.tokens.lock().await;
        *tokens = (new_access, new_refresh, new_uid);
        Ok(())
    }

    /// Internal: exchange a refresh token for new tokens (stateless, no locking).
    async fn fetch_tokens(
        client: &Client,
        config: &Config,
        refresh_token: &str,
    ) -> crate::Result<(Option<String>, Option<String>, Option<u64>)> {
        use chrono::Utc;
        use md5::{Digest, Md5};

        let now = Utc::now().format("%Y-%m-%dT%H:%M:%S%z").to_string();
        let hash = {
            let mut hasher = Md5::new();
            hasher.update(format!("{}{}", now, config.hash_secret));
            hex::encode(hasher.finalize())
        };

        let mut headers = HeaderMap::new();
        headers.insert(
            "x-client-time",
            HeaderValue::from_str(&now)
                .map_err(|e| PixivError::Other(format!("invalid header: {e}")))?,
        );
        headers.insert(
            "x-client-hash",
            HeaderValue::from_str(&hash)
                .map_err(|e| PixivError::Other(format!("invalid header: {e}")))?,
        );
        headers.insert(
            REFERER,
            HeaderValue::from_static("https://app-api.pixiv.net/"),
        );

        let params = [
            ("client_id", config.client_id),
            ("client_secret", config.client_secret),
            ("grant_type", "refresh_token"),
            ("refresh_token", refresh_token),
        ];

        let url = format!("{}/auth/token", config.auth_host);
        let resp = client
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

        #[derive(serde::Deserialize)]
        struct AuthResp {
            access_token: String,
            refresh_token: String,
            user: AuthUserResp,
        }
        #[derive(serde::Deserialize)]
        struct AuthUserResp {
            id: String,
        }

        let auth_resp: AuthResp = resp
            .json()
            .await
            .map_err(|e| PixivError::Auth(format!("failed to parse auth response: {e}")))?;

        Ok((
            Some(auth_resp.access_token),
            Some(auth_resp.refresh_token),
            auth_resp.user.id.parse().ok(),
        ))
    }

    /// Internal: build auth headers from current token, merged with custom headers.
    pub(crate) async fn auth_headers(&self) -> crate::Result<HeaderMap> {
        let mut headers = HeaderMap::new();
        headers.insert(
            REFERER,
            HeaderValue::from_static("https://app-api.pixiv.net/"),
        );
        let tokens = self.tokens.lock().await;
        if let Some(token) = &tokens.0 {
            headers.insert(
                AUTHORIZATION,
                HeaderValue::from_str(&format!("Bearer {token}"))
                    .map_err(|e| PixivError::Other(format!("invalid auth header: {e}")))?,
            );
        }
        drop(tokens);

        let custom = self.custom_headers.lock().await;
        for (name, value) in custom.iter() {
            headers.insert(name.clone(), value.clone());
        }

        Ok(headers)
    }

    /// Make an authenticated API request.
    pub(crate) async fn request<T: DeserializeOwned>(
        &self,
        method: Method,
        path: &str,
    ) -> crate::Result<crate::models::ApiResponse<T>> {
        let url = format!("{}{path}", self.config.host);

        let resp = self
            .client
            .request(method, &url)
            .headers(self.auth_headers().await?)
            .send()
            .await?;

        if !resp.status().is_success() {
            return Err(crate::PixivError::Status(resp.status()));
        }

        let raw: serde_json::Value = resp.json().await?;
        Ok(crate::models::ApiResponse::from_json(raw))
    }

    /// Make an authenticated POST request with form parameters.
    pub(crate) async fn post_form<T: DeserializeOwned, K: Into<String>, V: Into<String>>(
        &self,
        path: &str,
        params: Vec<(K, V)>,
    ) -> crate::Result<crate::models::ApiResponse<T>> {
        let url = format!("{}{path}", self.config.host);
        let string_params: Vec<(String, String)> = params
            .into_iter()
            .map(|(k, v)| (k.into(), v.into()))
            .collect();

        let resp = self
            .client
            .post(&url)
            .headers(self.auth_headers().await?)
            .form(&string_params)
            .send()
            .await?;

        if !resp.status().is_success() {
            return Err(crate::PixivError::Status(resp.status()));
        }

        let raw: serde_json::Value = resp.json().await?;
        Ok(crate::models::ApiResponse::from_json(raw))
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

    #[tokio::test]
    async fn test_new_has_defaults() {
        let api = PixivApi::new();
        assert!(!api.is_authenticated().await);
        assert!(api.user_id().await.is_none());
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

    #[tokio::test]
    async fn test_default_trait() {
        let api = PixivApi::default();
        assert!(!api.is_authenticated().await);
    }
}
