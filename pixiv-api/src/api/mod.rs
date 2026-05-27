pub mod auth;
pub mod illust;
pub mod misc;
pub mod novel;
pub mod search;
pub mod user;

#[cfg(feature = "gfw-bypass")]
pub mod bypass;

use crate::config::{ClientConfig, Config};
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
#[allow(dead_code)] // Fields will be used by auth, downloader, and endpoint modules
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

        if let Some(proxy_url) = &client_config.proxy
            && let Ok(proxy) = reqwest::Proxy::all(proxy_url)
        {
            builder = builder.proxy(proxy);
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
