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
    #[serde(default)]
    pub restriction_attributes: Option<Vec<String>>,
    #[serde(default)]
    pub comment_access_control: Option<i32>,
    #[serde(default)]
    pub favorited_details: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeriesRef {
    pub id: u64,
    pub title: String,
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
    #[serde(default)]
    pub parent_comment: Option<Box<serde_json::Value>>,
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

/// Response from illust_ranking endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IllustRankingResult {
    #[serde(default)]
    pub illusts: Vec<Illust>,
    #[serde(default)]
    pub next_url: Option<String>,
}

/// Response from illust_recommended endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IllustRecommendedResult {
    #[serde(default)]
    pub illusts: Vec<Illust>,
    #[serde(default)]
    pub ranking_illusts: Option<Vec<Illust>>,
    #[serde(default)]
    pub contest_exists: Option<bool>,
    #[serde(default)]
    pub next_url: Option<String>,
}

/// Response from illust_follow endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IllustFollowResult {
    #[serde(default)]
    pub illusts: Vec<Illust>,
    #[serde(default)]
    pub next_url: Option<String>,
}

/// Response from illust_new endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IllustNewResult {
    #[serde(default)]
    pub illusts: Vec<Illust>,
    #[serde(default)]
    pub next_url: Option<String>,
}

/// Response from user_illusts endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserIllustsResult {
    #[serde(default)]
    pub illusts: Vec<Illust>,
    #[serde(default)]
    pub next_url: Option<String>,
}

/// Response from user_bookmarks_illust endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserBookmarksIllustResult {
    #[serde(default)]
    pub illusts: Vec<Illust>,
    #[serde(default)]
    pub next_url: Option<String>,
}

/// Response from illust_related endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IllustRelatedResult {
    #[serde(default)]
    pub illusts: Vec<Illust>,
    #[serde(default)]
    pub next_url: Option<String>,
}

/// Response from illust_comments endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IllustCommentsResult {
    #[serde(default)]
    pub comments: Vec<Comment>,
    #[serde(default)]
    pub next_url: Option<String>,
    #[serde(default)]
    pub total_comments: Option<u64>,
    #[serde(default)]
    pub comment_access_control: Option<i32>,
}

/// Response from illust_bookmark_detail endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IllustBookmarkDetailResult {
    #[serde(default)]
    pub is_bookmarked: bool,
    #[serde(default)]
    pub restrict: Option<String>,
    #[serde(default)]
    pub tags: Vec<BookmarkTag>,
}

/// A tag as returned by the bookmark detail endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookmarkTag {
    pub name: String,
    #[serde(default)]
    pub is_registered: Option<bool>,
}

/// A trending tag with its associated illustration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendingTag {
    pub tag: super::common::Tag,
    #[serde(default)]
    pub illust: Option<Illust>,
}

/// Response from illust_detail endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IllustDetail {
    pub illust: Illust,
}

/// Response from trending_tags_illust endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendingTagsResult {
    #[serde(default)]
    pub trend_tags: Vec<TrendingTag>,
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
