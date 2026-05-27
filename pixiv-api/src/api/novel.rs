use crate::PixivApi;
use crate::models::ApiResponse;
use crate::models::novel::{
    NovelComments, NovelDetail, NovelFollowResult, NovelNewResult, NovelRecommendedResult,
    NovelSeriesResult, NovelTextResult,
};
use reqwest::Method;

impl PixivApi {
    /// Get novel details.
    pub async fn novel_detail(&self, novel_id: u64) -> crate::Result<ApiResponse<NovelDetail>> {
        self.request(
            Method::GET,
            &format!("/v2/novel/detail?novel_id={novel_id}"),
        )
        .await
    }

    /// Get novel comments.
    pub async fn novel_comments(
        &self,
        novel_id: u64,
        offset: Option<u32>,
    ) -> crate::Result<ApiResponse<NovelComments>> {
        let mut path = format!("/v1/novel/comments?novel_id={novel_id}");
        if let Some(o) = offset {
            path.push_str(&format!("&offset={o}"));
        }
        self.request(Method::GET, &path).await
    }

    /// Get recommended novels.
    pub async fn novel_recommended(&self) -> crate::Result<ApiResponse<NovelRecommendedResult>> {
        self.request(Method::GET, "/v1/novel/recommended").await
    }

    /// Get newest novels.
    pub async fn novel_new(&self) -> crate::Result<ApiResponse<NovelNewResult>> {
        self.request(Method::GET, "/v1/novel/new").await
    }

    /// Get novels from followed artists.
    pub async fn novel_follow(
        &self,
        restrict: Option<&str>,
    ) -> crate::Result<ApiResponse<NovelFollowResult>> {
        let mut path = "/v1/novel/follow?".to_string();
        if let Some(r) = restrict {
            path.push_str(&format!("restrict={r}"));
        }
        self.request(Method::GET, &path).await
    }

    /// Get novel series info.
    pub async fn novel_series(
        &self,
        series_id: u64,
    ) -> crate::Result<ApiResponse<NovelSeriesResult>> {
        self.request(
            Method::GET,
            &format!("/v2/novel/series?series_id={series_id}"),
        )
        .await
    }

    /// Get novel text content.
    pub async fn novel_text(&self, novel_id: u64) -> crate::Result<ApiResponse<NovelTextResult>> {
        self.request(Method::GET, &format!("/v1/novel/text?novel_id={novel_id}"))
            .await
    }

    /// Get novel via webview (raw HTML extraction).
    pub async fn webview_novel(
        &self,
        novel_id: u64,
    ) -> crate::Result<ApiResponse<serde_json::Value>> {
        self.request(
            Method::GET,
            &format!("/webview/v2/novel?id={novel_id}&viewer_version=20221031"),
        )
        .await
    }
}
