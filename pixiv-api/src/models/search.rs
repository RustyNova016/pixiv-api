use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SearchSort {
    DateDesc,
    DateAsc,
    PopularDesc,
    PopularMaleDesc,
    PopularFemaleDesc,
}

impl SearchSort {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::DateDesc => "date_desc",
            Self::DateAsc => "date_asc",
            Self::PopularDesc => "popular_desc",
            Self::PopularMaleDesc => "popular_male_desc",
            Self::PopularFemaleDesc => "popular_female_desc",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SearchDuration {
    WithinLastDay,
    WithinLastWeek,
    WithinLastMonth,
    #[serde(rename = "")]
    None,
}

impl SearchDuration {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::WithinLastDay => "within_last_day",
            Self::WithinLastWeek => "within_last_week",
            Self::WithinLastMonth => "within_last_month",
            Self::None => "",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SearchTarget {
    PartialMatchForTags,
    ExactMatchForTags,
    TitleAndCaption,
    #[serde(rename = "keyword")]
    Keyword,
}

impl SearchTarget {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::PartialMatchForTags => "partial_match_for_tags",
            Self::ExactMatchForTags => "exact_match_for_tags",
            Self::TitleAndCaption => "title_and_caption",
            Self::Keyword => "keyword",
        }
    }
}

/// Response from search_illust endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchIllustResult {
    #[serde(default)]
    pub illusts: Vec<super::illust::Illust>,
    #[serde(default)]
    pub next_url: Option<String>,
    #[serde(default)]
    pub search_span_limit: Option<i32>,
    #[serde(default)]
    pub show_ai: Option<bool>,
}

/// Response from search_novel endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchNovelResult {
    #[serde(default)]
    pub novels: Vec<super::novel::Novel>,
    #[serde(default)]
    pub next_url: Option<String>,
    #[serde(default)]
    pub search_span_limit: Option<i32>,
    #[serde(default)]
    pub show_ai: Option<bool>,
}

/// Response from search_user endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchUserResult {
    #[serde(default)]
    pub user_previews: Vec<super::user::UserPreview>,
    #[serde(default)]
    pub next_url: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_sort_as_str() {
        assert_eq!(SearchSort::DateDesc.as_str(), "date_desc");
        assert_eq!(SearchSort::PopularDesc.as_str(), "popular_desc");
    }

    #[test]
    fn test_search_duration_none() {
        assert_eq!(SearchDuration::None.as_str(), "");
    }

    #[test]
    fn test_search_target_deserialize() {
        let json = r#""partial_match_for_tags""#;
        let target: SearchTarget = serde_json::from_str(json).unwrap();
        assert!(matches!(target, SearchTarget::PartialMatchForTags));
    }
}
