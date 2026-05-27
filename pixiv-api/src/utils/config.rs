use reqwest::header::HeaderMap;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct Config {
    pub client_id: &'static str,
    pub client_secret: &'static str,
    pub hash_secret: &'static str,
    pub host: &'static str,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            client_id: "MOBrBDS8blbauoSck0ZfDbtuzpyT",
            client_secret: "lsACyCD94FhDUtGTXi3QzcFE2uU1hqtDaKeqrdwj",
            hash_secret: "28c1fdd170a5204386cb1313c7077b34f83e4aaf4aa829ce78c231e05b0bae2c",
            host: "https://app-api.pixiv.net",
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct ReqwestConfig {
    pub timeout: Option<Duration>,
    pub user_agent: Option<String>,
    pub proxy: Option<String>,
    pub headers: HeaderMap,
}
