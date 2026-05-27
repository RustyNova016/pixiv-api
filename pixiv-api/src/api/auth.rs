use crate::PixivApi;
use crate::error::PixivError;
use chrono::Utc;
use md5::{Digest, Md5};
use reqwest::header::{AUTHORIZATION, HeaderMap, HeaderValue, REFERER};
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
    #[allow(dead_code)] // Will be used by endpoint modules in later tasks
    pub(crate) fn require_auth(&self) -> crate::Result<()> {
        if self.access_token.is_none() {
            return Err(PixivError::Auth(
                "not authenticated. Call auth() or set_auth() first.".into(),
            ));
        }
        Ok(())
    }

    /// Build default headers with Authorization bearer token.
    #[allow(dead_code)] // Will be used by endpoint modules in later tasks
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
