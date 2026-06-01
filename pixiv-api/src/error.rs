use reqwest::StatusCode;

#[derive(Debug, thiserror::Error)]
pub enum PixivError {
    #[error("authentication failed: {0}")]
    Auth(String),

    #[error("HTTP request failed: {0}")]
    Request(#[from] reqwest::Error),

    #[error("API returned status {0}")]
    Status(StatusCode),

    #[error("failed to parse response: {0}")]
    Parse(#[from] serde_json::Error),

    #[error("download failed: {0}")]
    Download(String),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("{0}")]
    Other(String),
}

impl PixivError {
    /// Returns `true` if this error indicates an authentication failure (HTTP 401).
    ///
    /// Useful for deciding whether to call [`PixivApi::refresh_token()`](crate::PixivApi::refresh_token).
    pub fn is_auth_error(&self) -> bool {
        matches!(self, PixivError::Status(s) if *s == reqwest::StatusCode::UNAUTHORIZED)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_error_display() {
        let err = PixivError::Auth("bad token".into());
        assert_eq!(err.to_string(), "authentication failed: bad token");
    }

    #[test]
    fn test_status_error_display() {
        let err = PixivError::Status(StatusCode::NOT_FOUND);
        assert!(err.to_string().contains("404"));
    }

    #[test]
    fn test_other_error() {
        let err = PixivError::Other("custom".into());
        assert_eq!(err.to_string(), "custom");
    }

    #[test]
    fn test_is_auth_error() {
        let err = PixivError::Status(StatusCode::UNAUTHORIZED);
        assert!(err.is_auth_error());

        let err = PixivError::Status(StatusCode::NOT_FOUND);
        assert!(!err.is_auth_error());

        let err = PixivError::Auth("bad".into());
        assert!(!err.is_auth_error());
    }
}
