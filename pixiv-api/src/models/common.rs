use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tag {
    pub name: String,
    #[serde(default)]
    pub translated_name: Option<String>,
    #[serde(default)]
    pub added_by_uploaded_user: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pagination {
    #[serde(default)]
    pub next_url: Option<String>,
    #[serde(default)]
    pub prev_url: Option<String>,
}

/// Extract query parameters from a Pixiv next_url for pagination.
pub fn parse_next_url(url: &str) -> Option<std::collections::HashMap<String, String>> {
    let parsed = url::Url::parse(url).ok()?;
    Some(parsed.query_pairs().into_owned().collect())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageUrls {
    #[serde(default)]
    pub square_medium: Option<String>,
    #[serde(default)]
    pub medium: Option<String>,
    #[serde(default)]
    pub large: Option<String>,
    #[serde(default)]
    pub original: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetaPage {
    #[serde(default)]
    pub image_urls: Option<ImageUrls>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetaSinglePage {
    #[serde(default)]
    pub original_image_url: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tag_deserialize() {
        let json = r#"{"name": "landscape", "translated_name": null}"#;
        let tag: Tag = serde_json::from_str(json).unwrap();
        assert_eq!(tag.name, "landscape");
        assert!(tag.translated_name.is_none());
    }

    #[test]
    fn test_pagination_parse_next_url() {
        let url = "https://app-api.pixiv.net/v1/search/illust?word=test&offset=30";
        let params = parse_next_url(url).unwrap();
        assert_eq!(params["word"], "test");
        assert_eq!(params["offset"], "30");
    }

    #[test]
    fn test_image_urls_partial() {
        let json = r#"{"medium": "https://example.com/med.jpg"}"#;
        let urls: ImageUrls = serde_json::from_str(json).unwrap();
        assert_eq!(urls.medium.as_deref(), Some("https://example.com/med.jpg"));
        assert!(urls.original.is_none());
    }
}
