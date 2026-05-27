use crate::PixivApi;
use crate::error::PixivError;

impl PixivApi {
    /// Authenticate with a refresh token.
    ///
    /// This is the primary authentication method. Password-based auth
    /// is deprecated by Pixiv.
    pub async fn auth(&mut self, refresh_token: &str) -> crate::Result<()> {
        let (access, refresh, uid) =
            Self::fetch_tokens(&self.client, &self.config, refresh_token).await?;

        let mut tokens = self.tokens.lock().await;
        *tokens = (access, refresh, uid);
        Ok(())
    }

    /// Set authentication tokens manually (e.g., from a saved session).
    pub async fn set_auth(&self, access_token: &str, refresh_token: &str, user_id: u64) {
        let mut tokens = self.tokens.lock().await;
        *tokens = (
            Some(access_token.to_string()),
            Some(refresh_token.to_string()),
            Some(user_id),
        );
    }

    /// Get the current access token, if authenticated.
    pub async fn access_token(&self) -> Option<String> {
        let tokens = self.tokens.lock().await;
        tokens.0.clone()
    }

    /// Get the current refresh token, if set.
    pub async fn current_refresh_token(&self) -> Option<String> {
        let tokens = self.tokens.lock().await;
        tokens.1.clone()
    }

    /// Require authentication, returning an error if not authenticated.
    pub(crate) async fn require_auth(&self) -> crate::Result<()> {
        let tokens = self.tokens.lock().await;
        if tokens.0.is_none() {
            return Err(PixivError::Auth(
                "not authenticated. Call auth() or set_auth() first.".into(),
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_set_auth() {
        let api = PixivApi::new();
        assert!(!api.is_authenticated().await);

        api.set_auth("access_123", "refresh_456", 789).await;
        assert!(api.is_authenticated().await);
        assert_eq!(api.access_token().await.as_deref(), Some("access_123"));
        assert_eq!(
            api.current_refresh_token().await.as_deref(),
            Some("refresh_456")
        );
        assert_eq!(api.user_id().await, Some(789));
    }

    #[tokio::test]
    async fn test_require_auth_fails_without_token() {
        let api = PixivApi::new();
        assert!(api.require_auth().await.is_err());
    }

    #[tokio::test]
    async fn test_require_auth_succeeds_with_token() {
        let api = PixivApi::new();
        api.set_auth("token", "refresh", 1).await;
        assert!(api.require_auth().await.is_ok());
    }

    #[tokio::test]
    async fn test_auth_headers_contain_bearer() {
        use reqwest::header::AUTHORIZATION;

        let api = PixivApi::new();
        api.set_auth("my_token", "refresh", 1).await;
        let headers = api.auth_headers().await.unwrap();
        assert_eq!(
            headers.get(AUTHORIZATION).unwrap().to_str().unwrap(),
            "Bearer my_token"
        );
    }
}
