use crate::PixivApi;
use crate::models::ApiResponse;
use crate::models::illust::TrendingTagsResult;
use crate::models::search::SearchOptions;
use crate::models::search::{
    SearchIllustResult, SearchNovelResult, SearchSort, SearchTarget, SearchUserResult,
};

use reqwest::Method;
use urlencoding::encode;

impl PixivApi {
    /// Search illustrations.
    pub async fn search_illust(
        &self,
        word: &str,
        options: Option<SearchOptions>,
    ) -> crate::Result<ApiResponse<SearchIllustResult>> {
        let encoded_word = encode(word);
        let mut path = format!("/v1/search/illust?word={encoded_word}");
        let options = options.unwrap_or_default();

        if let Some(s) = options.sort {
            path.push_str(&format!("&sort={}", s.as_str()));
        }

        if let Some(t) = options.target {
            path.push_str(&format!("&search_target={}", t.as_str()));
        }

        if let Some(o) = options.offset {
            path.push_str(&format!("&offset={o}"));
        }

        match options.include_translated_tag_results {
            Some(true) => path.push_str(&format!("&include_translated_tag_results=true")),
            Some(false) => path.push_str(&format!("&include_translated_tag_results=false")),
            None => {}
        }

        match options.merge_plain_keyword_results {
            Some(true) => path.push_str(&format!("&merge_plain_keyword_results=true")),
            Some(false) => path.push_str(&format!("&merge_plain_keyword_results=false")),
            None => {}
        }

        match options.include_potential_violation_works {
            Some(true) => path.push_str(&format!("&include_potential_violation_works=true")),
            Some(false) => path.push_str(&format!("&include_potential_violation_works=false")),
            None => {}
        }

        match options.search_ai_type {
            Some(true) => path.push_str(&format!("&search_ai_type=true")),
            Some(false) => path.push_str(&format!("&search_ai_type=false")),
            None => {}
        }

        if let Some(o) = options.start_date {
            path.push_str(&format!("&start_date={}", o.format("%Y-%m-%d").to_string()));
        }

        if let Some(o) = options.end_date {
            path.push_str(&format!("&end_date={}", o.format("%Y-%m-%d").to_string()));
        }

        if let Some(o) = options.content_type {
            path.push_str(&format!("&content_type={}", o.as_str()));
        }

        if let Some(o) = options.width_min {
            path.push_str(&format!("&width_min={o}"));
        }

        if let Some(o) = options.width_max {
            path.push_str(&format!("&width_max={o}"));
        }

        if let Some(o) = options.height_min {
            path.push_str(&format!("&height_min={o}"));
        }

        if let Some(o) = options.height_max {
            path.push_str(&format!("&height_max={o}"));
        }

        if let Some(o) = options.ratio_pattern {
            path.push_str(&format!("&ratio_pattern={}", o.as_str()));
        }

        if let Some(o) = options.tool {
            path.push_str(&format!("&tool={o}"));
        }

        self.request(Method::GET, &path).await
    }

    /// Search novels.
    pub async fn search_novel(
        &self,
        word: &str,
        sort: Option<SearchSort>,
        search_target: Option<SearchTarget>,
        offset: Option<u32>,
    ) -> crate::Result<ApiResponse<SearchNovelResult>> {
        let encoded_word = encode(word);
        let mut path = format!("/v1/search/novel?word={encoded_word}");
        if let Some(s) = sort {
            path.push_str(&format!("&sort={}", s.as_str()));
        }
        if let Some(t) = search_target {
            path.push_str(&format!("&search_target={}", t.as_str()));
        }
        if let Some(o) = offset {
            path.push_str(&format!("&offset={o}"));
        }
        self.request(Method::GET, &path).await
    }

    /// Search users.
    pub async fn search_user(
        &self,
        word: &str,
        offset: Option<u32>,
    ) -> crate::Result<ApiResponse<SearchUserResult>> {
        let encoded_word = encode(word);
        let mut path = format!("/v1/search/user?word={encoded_word}");
        if let Some(o) = offset {
            path.push_str(&format!("&offset={o}"));
        }
        self.request(Method::GET, &path).await
    }

    /// Get trending illustration tags.
    pub async fn trending_tags_illust(&self) -> crate::Result<ApiResponse<TrendingTagsResult>> {
        self.request(Method::GET, "/v1/trending-tags/illust").await
    }
}
