pub mod api;
pub mod config;
pub mod downloader;
pub mod error;
pub mod models;

pub use api::PixivApi;
pub use config::{ClientConfig, Config};
pub use error::PixivError;
pub use models::ApiResponse;

pub type Result<T> = std::result::Result<T, PixivError>;
