use crate::PixivApi;
use crate::models::ApiResponse;
use crate::models::illust::{UserBookmarksIllustResult, UserIllustsResult};
use crate::models::novel::{UserBookmarksNovelResult, UserNovelsResult};
use crate::models::user::{
    UserDetail, UserFollowerResult, UserFollowingResult, UserListResult, UserMypixivResult,
};
use reqwest::Method;
use urlencoding::encode;

impl PixivApi {
    /// Get user details.
    pub async fn user_detail(&self, user_id: u64) -> crate::Result<ApiResponse<UserDetail>> {
        self.request(Method::GET, &format!("/v1/user/detail?user_id={user_id}"))
            .await
    }

    /// Get user's illustrations.
    pub async fn user_illusts(
        &self,
        user_id: u64,
        r#type: Option<&str>,
        offset: Option<u32>,
    ) -> crate::Result<ApiResponse<UserIllustsResult>> {
        let mut path = format!("/v1/user/illusts?user_id={user_id}");
        if let Some(t) = r#type {
            path.push_str(&format!("&type={t}"));
        }
        if let Some(o) = offset {
            path.push_str(&format!("&offset={o}"));
        }
        self.request(Method::GET, &path).await
    }

    /// Get user's bookmarked illustrations.
    pub async fn user_bookmarks_illust(
        &self,
        user_id: u64,
        restrict: Option<&str>,
        max_bookmark_id: Option<u64>,
        tag: Option<&str>,
    ) -> crate::Result<ApiResponse<UserBookmarksIllustResult>> {
        let mut path = format!("/v1/user/bookmarks/illust?user_id={user_id}");
        if let Some(r) = restrict {
            path.push_str(&format!("&restrict={r}"));
        }
        if let Some(m) = max_bookmark_id {
            path.push_str(&format!("&max_bookmark_id={m}"));
        }
        if let Some(t) = tag {
            let encoded_tag = encode(t);
            path.push_str(&format!("&tag={encoded_tag}"));
        }
        self.request(Method::GET, &path).await
    }

    /// Get user's bookmarked novels.
    pub async fn user_bookmarks_novel(
        &self,
        user_id: u64,
        restrict: Option<&str>,
        max_bookmark_id: Option<u64>,
    ) -> crate::Result<ApiResponse<UserBookmarksNovelResult>> {
        let mut path = format!("/v1/user/bookmarks/novel?user_id={user_id}");
        if let Some(r) = restrict {
            path.push_str(&format!("&restrict={r}"));
        }
        if let Some(m) = max_bookmark_id {
            path.push_str(&format!("&max_bookmark_id={m}"));
        }
        self.request(Method::GET, &path).await
    }

    /// Get users related to the given user.
    pub async fn user_related(&self, user_id: u64) -> crate::Result<ApiResponse<UserListResult>> {
        self.request(
            Method::GET,
            &format!("/v1/user/related?seed_user_id={user_id}"),
        )
        .await
    }

    /// Get recommended users.
    pub async fn user_recommended(&self) -> crate::Result<ApiResponse<UserListResult>> {
        self.request(Method::GET, "/v1/user/recommended").await
    }

    /// Get users the given user is following.
    pub async fn user_following(
        &self,
        user_id: u64,
        restrict: Option<&str>,
        offset: Option<u32>,
    ) -> crate::Result<ApiResponse<UserFollowingResult>> {
        let mut path = format!("/v1/user/following?user_id={user_id}");
        if let Some(r) = restrict {
            path.push_str(&format!("&restrict={r}"));
        }
        if let Some(o) = offset {
            path.push_str(&format!("&offset={o}"));
        }
        self.request(Method::GET, &path).await
    }

    /// Get user's followers.
    pub async fn user_follower(
        &self,
        user_id: u64,
        offset: Option<u32>,
    ) -> crate::Result<ApiResponse<UserFollowerResult>> {
        let mut path = format!("/v1/user/follower?user_id={user_id}");
        if let Some(o) = offset {
            path.push_str(&format!("&offset={o}"));
        }
        self.request(Method::GET, &path).await
    }

    /// Get user's Pixiv friends (mypixiv).
    pub async fn user_mypixiv(
        &self,
        user_id: u64,
        offset: Option<u32>,
    ) -> crate::Result<ApiResponse<UserMypixivResult>> {
        let mut path = format!("/v1/user/mypixiv?user_id={user_id}");
        if let Some(o) = offset {
            path.push_str(&format!("&offset={o}"));
        }
        self.request(Method::GET, &path).await
    }

    /// Get user list by IDs.
    pub async fn user_list(&self, user_ids: &[u64]) -> crate::Result<ApiResponse<UserListResult>> {
        let ids = user_ids
            .iter()
            .map(|id| id.to_string())
            .collect::<Vec<_>>()
            .join(",");
        self.request(Method::GET, &format!("/v2/user/list?user_ids={ids}"))
            .await
    }

    /// Get user's novels.
    pub async fn user_novels(
        &self,
        user_id: u64,
        offset: Option<u32>,
    ) -> crate::Result<ApiResponse<UserNovelsResult>> {
        let mut path = format!("/v1/user/novels?user_id={user_id}");
        if let Some(o) = offset {
            path.push_str(&format!("&offset={o}"));
        }
        self.request(Method::GET, &path).await
    }

    /// Follow a user.
    pub async fn user_follow_add(
        &self,
        user_id: u64,
        restrict: Option<&str>,
    ) -> crate::Result<ApiResponse<serde_json::Value>> {
        let mut params = vec![("user_id".to_string(), user_id.to_string())];
        if let Some(r) = restrict {
            params.push(("restrict".to_string(), r.to_string()));
        }
        self.post_form("/v1/user/follow/add", params).await
    }

    /// Unfollow a user.
    pub async fn user_follow_delete(
        &self,
        user_id: u64,
    ) -> crate::Result<ApiResponse<serde_json::Value>> {
        self.post_form(
            "/v1/user/follow/delete",
            vec![("user_id".to_string(), user_id.to_string())],
        )
        .await
    }

    /// Get user's bookmark tags for illustrations.
    pub async fn user_bookmark_tags_illust(
        &self,
        user_id: u64,
        restrict: Option<&str>,
    ) -> crate::Result<ApiResponse<serde_json::Value>> {
        let mut path = format!("/v1/user/bookmark-tags/illust?user_id={user_id}");
        if let Some(r) = restrict {
            path.push_str(&format!("&restrict={r}"));
        }
        self.request(Method::GET, &path).await
    }

    /// Edit user's AI show settings.
    pub async fn user_edit_ai_show_settings(
        &self,
        illust_ai_type: i32,
    ) -> crate::Result<ApiResponse<serde_json::Value>> {
        self.post_form(
            "/v1/user/edit-ai-show-settings",
            vec![("illust_ai_type".to_string(), illust_ai_type.to_string())],
        )
        .await
    }
}
