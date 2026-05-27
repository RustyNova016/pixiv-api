use serde::{Deserialize, Serialize};
use std::str::FromStr;

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

impl FromStr for SearchSort {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "date_desc" => Ok(Self::DateDesc),
            "date_asc" => Ok(Self::DateAsc),
            "popular_desc" => Ok(Self::PopularDesc),
            "popular_male_desc" => Ok(Self::PopularMaleDesc),
            "popular_female_desc" => Ok(Self::PopularFemaleDesc),
            _ => Err(format!("unknown SearchSort variant: {s}")),
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

impl FromStr for SearchDuration {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "within_last_day" => Ok(Self::WithinLastDay),
            "within_last_week" => Ok(Self::WithinLastWeek),
            "within_last_month" => Ok(Self::WithinLastMonth),
            "" => Ok(Self::None),
            _ => Err(format!("unknown SearchDuration variant: {s}")),
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

impl FromStr for SearchTarget {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "partial_match_for_tags" => Ok(Self::PartialMatchForTags),
            "exact_match_for_tags" => Ok(Self::ExactMatchForTags),
            "title_and_caption" => Ok(Self::TitleAndCaption),
            "keyword" => Ok(Self::Keyword),
            _ => Err(format!("unknown SearchTarget variant: {s}")),
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

    #[test]
    fn test_search_sort_from_str() {
        assert!(matches!(
            SearchSort::from_str("date_desc"),
            Ok(SearchSort::DateDesc)
        ));
        assert!(matches!(
            SearchSort::from_str("popular_female_desc"),
            Ok(SearchSort::PopularFemaleDesc)
        ));
        assert!(SearchSort::from_str("invalid").is_err());
    }

    #[test]
    fn test_search_duration_from_str() {
        assert!(matches!(
            SearchDuration::from_str("within_last_day"),
            Ok(SearchDuration::WithinLastDay)
        ));
        assert!(matches!(
            SearchDuration::from_str("within_last_week"),
            Ok(SearchDuration::WithinLastWeek)
        ));
        assert!(matches!(
            SearchDuration::from_str("within_last_month"),
            Ok(SearchDuration::WithinLastMonth)
        ));
        assert!(matches!(
            SearchDuration::from_str(""),
            Ok(SearchDuration::None)
        ));
        assert!(SearchDuration::from_str("invalid").is_err());
    }

    #[test]
    fn test_search_target_from_str() {
        assert!(matches!(
            SearchTarget::from_str("partial_match_for_tags"),
            Ok(SearchTarget::PartialMatchForTags)
        ));
        assert!(matches!(
            SearchTarget::from_str("exact_match_for_tags"),
            Ok(SearchTarget::ExactMatchForTags)
        ));
        assert!(matches!(
            SearchTarget::from_str("title_and_caption"),
            Ok(SearchTarget::TitleAndCaption)
        ));
        assert!(matches!(
            SearchTarget::from_str("keyword"),
            Ok(SearchTarget::Keyword)
        ));
        assert!(SearchTarget::from_str("invalid").is_err());
    }
}
