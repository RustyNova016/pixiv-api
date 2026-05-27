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
