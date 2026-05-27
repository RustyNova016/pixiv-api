pub mod api;
pub mod config;
pub mod downloader;
pub mod error;
pub mod models;

pub use api::PixivApi;
pub use error::PixivError;
pub type Result<T> = std::result::Result<T, PixivError>;
