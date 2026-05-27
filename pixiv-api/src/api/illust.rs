use crate::PixivApi;
use crate::models::ApiResponse;
use crate::models::illust::{
    IllustBookmarkDetailResult, IllustCommentsResult, IllustDetail, IllustFollowResult,
    IllustNewResult, IllustRankingResult, IllustRecommendedResult, IllustRelatedResult,
};
use reqwest::Method;

impl PixivApi {
    /// Get illustration details.
    pub async fn illust_detail(&self, illust_id: u64) -> crate::Result<ApiResponse<IllustDetail>> {
        self.request(
            Method::GET,
            &format!("/v1/illust/detail?illust_id={illust_id}"),
        )
        .await
    }

    /// Get illustration comments.
    pub async fn illust_comments(
        &self,
        illust_id: u64,
        offset: Option<u32>,
    ) -> crate::Result<ApiResponse<IllustCommentsResult>> {
        let mut path = format!("/v1/illust/comments?illust_id={illust_id}");
        if let Some(o) = offset {
            path.push_str(&format!("&offset={o}"));
        }
        self.request(Method::GET, &path).await
    }

    /// Get related illustrations.
    pub async fn illust_related(
        &self,
        illust_id: u64,
    ) -> crate::Result<ApiResponse<IllustRelatedResult>> {
        self.request(
            Method::GET,
            &format!("/v2/illust/related?illust_id={illust_id}"),
        )
        .await
    }

    /// Get recommended illustrations.
    pub async fn illust_recommended(&self) -> crate::Result<ApiResponse<IllustRecommendedResult>> {
        self.request(Method::GET, "/v1/illust/recommended").await
    }

    /// Get illustration ranking.
    pub async fn illust_ranking(
        &self,
        mode: Option<&str>,
        date: Option<&str>,
        offset: Option<u32>,
    ) -> crate::Result<ApiResponse<IllustRankingResult>> {
        let mut path = "/v1/illust/ranking?".to_string();
        if let Some(m) = mode {
            path.push_str(&format!("mode={m}&"));
        }
        if let Some(d) = date {
            path.push_str(&format!("date={d}&"));
        }
        if let Some(o) = offset {
            path.push_str(&format!("offset={o}"));
        }
        self.request(Method::GET, &path).await
    }

    /// Get illustrations from followed artists.
    pub async fn illust_follow(
        &self,
        restrict: Option<&str>,
    ) -> crate::Result<ApiResponse<IllustFollowResult>> {
        let mut path = "/v2/illust/follow?".to_string();
        if let Some(r) = restrict {
            path.push_str(&format!("restrict={r}"));
        }
        self.request(Method::GET, &path).await
    }

    /// Get newest illustrations.
    pub async fn illust_new(&self) -> crate::Result<ApiResponse<IllustNewResult>> {
        self.request(Method::GET, "/v1/illust/new").await
    }

    /// Get bookmark detail for an illustration.
    pub async fn illust_bookmark_detail(
        &self,
        illust_id: u64,
    ) -> crate::Result<ApiResponse<IllustBookmarkDetailResult>> {
        self.request(
            Method::GET,
            &format!("/v2/illust/bookmark/detail?illust_id={illust_id}"),
        )
        .await
    }

    /// Add an illustration bookmark.
    pub async fn illust_bookmark_add(
        &self,
        illust_id: u64,
        restrict: Option<&str>,
        tags: Option<&[&str]>,
    ) -> crate::Result<ApiResponse<serde_json::Value>> {
        let mut params = vec![("illust_id".to_string(), illust_id.to_string())];
        if let Some(r) = restrict {
            params.push(("restrict".to_string(), r.to_string()));
        }
        if let Some(t) = tags {
            params.push(("tags".to_string(), t.join(" ")));
        }
        self.post_form("/v2/illust/bookmark/add", params).await
    }

    /// Remove an illustration bookmark.
    pub async fn illust_bookmark_delete(
        &self,
        illust_id: u64,
    ) -> crate::Result<ApiResponse<serde_json::Value>> {
        self.post_form(
            "/v1/illust/bookmark/delete",
            vec![("illust_id".to_string(), illust_id.to_string())],
        )
        .await
    }
}
