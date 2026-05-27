use crate::PixivApi;

/// DNS-over-HTTPS response from Cloudflare.
#[derive(serde::Deserialize)]
#[allow(non_snake_case)]
struct DnsResponse {
    #[serde(default)]
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
        if let Ok(ip) = self
            .resolve_via_doh("https://cloudflare-dns.com/dns-query", hostname)
            .await
        {
            return Ok(ip);
        }

        // Fallback to Google DoH
        self.resolve_via_doh("https://dns.google/resolve", hostname)
            .await
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
