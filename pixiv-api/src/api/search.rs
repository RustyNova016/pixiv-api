use crate::PixivApi;
use crate::models::ApiResponse;
use crate::models::illust::TrendingTagsResult;
use crate::models::search::{
    SearchDuration, SearchIllustResult, SearchNovelResult, SearchSort, SearchTarget,
    SearchUserResult,
};
use reqwest::Method;
use urlencoding::encode;

impl PixivApi {
    /// Search illustrations.
    pub async fn search_illust(
        &self,
        word: &str,
        sort: Option<SearchSort>,
        duration: Option<SearchDuration>,
        search_target: Option<SearchTarget>,
        offset: Option<u32>,
    ) -> crate::Result<ApiResponse<SearchIllustResult>> {
        let encoded_word = encode(word);
        let mut path = format!("/v1/search/illust?word={encoded_word}");
        if let Some(s) = sort {
            path.push_str(&format!("&sort={}", s.as_str()));
        }
        if let Some(d) = duration {
            let ds = d.as_str();
            if !ds.is_empty() {
                path.push_str(&format!("&duration={ds}"));
            }
        }
        if let Some(t) = search_target {
            path.push_str(&format!("&search_target={}", t.as_str()));
        }
        if let Some(o) = offset {
            path.push_str(&format!("&offset={o}"));
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
