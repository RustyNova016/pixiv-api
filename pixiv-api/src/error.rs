#[derive(Debug, thiserror::Error)]
pub enum PixivError {
    #[error("request failed: {0}")]
    Request(#[from] reqwest::Error),

    #[error("io failed: {0}")]
    Io(#[from] std::io::Error),
}
