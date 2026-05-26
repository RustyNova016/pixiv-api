use thiserror;

#[derive(Debug, thiserror::Error)]
pub enum DownloadError {
    #[error("request failed: {0}")]
    Request(#[from] reqwest::Error),

    #[error("io failed: {0}")]
    Io(#[from] std::io::Error),

    #[error("http status error: {0}")]
    Status(reqwest::StatusCode),
}
