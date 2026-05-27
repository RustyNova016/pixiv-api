use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::common::{ImageUrls, MetaPage, MetaSinglePage, Tag};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IllustType {
    Illust,
    Manga,
    Ugoira,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Illust {
    pub id: u64,
    pub title: String,
    #[serde(default)]
    pub r#type: Option<IllustType>,
    #[serde(default)]
    pub image_urls: Option<ImageUrls>,
    #[serde(default)]
    pub caption: Option<String>,
    #[serde(default)]
    pub user: Option<super::user::UserPreview>,
    #[serde(default)]
    pub tags: Option<Vec<Tag>>,
    #[serde(default)]
    pub tools: Option<Vec<String>>,
    #[serde(default)]
    pub create_date: Option<DateTime<Utc>>,
    #[serde(default)]
    pub page_count: Option<u32>,
    #[serde(default)]
    pub width: Option<u32>,
    #[serde(default)]
    pub height: Option<u32>,
    #[serde(default)]
    pub sanity_level: Option<i32>,
    #[serde(default)]
    pub x_restrict: Option<i32>,
    #[serde(default)]
    pub series: Option<SeriesRef>,
    #[serde(default)]
    pub meta_single_page: Option<MetaSinglePage>,
    #[serde(default)]
    pub meta_pages: Option<Vec<MetaPage>>,
    #[serde(default)]
    pub total_view: Option<u64>,
    #[serde(default)]
    pub total_bookmarks: Option<u64>,
    #[serde(default)]
    pub is_bookmarked: Option<bool>,
    #[serde(default)]
    pub visible: Option<bool>,
    #[serde(default)]
    pub is_muted: Option<bool>,
    #[serde(default)]
    pub total_comments: Option<u64>,
    #[serde(default)]
    pub restrict: Option<i32>,
    #[serde(default)]
    pub illust_ai_type: Option<i32>,
    #[serde(default)]
    pub illust_book_style: Option<i32>,
    #[serde(default)]
    pub event_banners: Option<Vec<serde_json::Value>>,
    #[serde(default)]
    pub request: Option<serde_json::Value>,
    #[serde(default)]
    pub seasonal_effect_animation_urls: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeriesRef {
    pub id: u64,
    pub title: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IllustComments {
    #[serde(default)]
    pub comments: Vec<Comment>,
    #[serde(default)]
    pub next_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Comment {
    pub id: u64,
    pub comment: String,
    #[serde(default)]
    pub date: Option<DateTime<Utc>>,
    #[serde(default)]
    pub user: Option<super::user::UserPreview>,
    #[serde(default)]
    pub has_replies: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UgoiraMetadata {
    #[serde(default)]
    pub zip_urls: Option<UgoiraZipUrls>,
    #[serde(default)]
    pub frames: Option<Vec<UgoiraFrame>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UgoiraZipUrls {
    #[serde(default)]
    pub medium: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UgoiraFrame {
    pub file: String,
    pub delay: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_illust_type_deserialize() {
        let json = r#""illust""#;
        let t: IllustType = serde_json::from_str(json).unwrap();
        assert!(matches!(t, IllustType::Illust));
    }

    #[test]
    fn test_illust_partial_deserialize() {
        let json = r#"{"id": 12345, "title": "Test Work", "page_count": 3}"#;
        let illust: Illust = serde_json::from_str(json).unwrap();
        assert_eq!(illust.id, 12345);
        assert_eq!(illust.title, "Test Work");
        assert_eq!(illust.page_count, Some(3));
        assert!(illust.user.is_none());
    }

    #[test]
    fn test_ugoira_frame() {
        let json = r#"{"file": "000000.jpg", "delay": 80}"#;
        let frame: UgoiraFrame = serde_json::from_str(json).unwrap();
        assert_eq!(frame.delay, 80);
    }
}
