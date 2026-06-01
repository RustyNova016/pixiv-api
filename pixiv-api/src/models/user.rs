use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPreview {
    /// The user object (present in user_related/recommended/list responses).
    #[serde(default)]
    pub user: Option<User>,
    /// Preview illustrations (present in user_related/recommended/list responses).
    #[serde(default)]
    pub illusts: Option<Vec<super::illust::Illust>>,
    /// Preview novels (present in user_related/recommended/list responses).
    #[serde(default)]
    pub novels: Option<Vec<super::novel::Novel>>,
    /// Simple user fields (present in simple preview contexts, e.g. illust.user).
    #[serde(default)]
    pub id: Option<u64>,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub account: Option<String>,
    #[serde(default)]
    pub profile_image_urls: Option<ProfileImageUrls>,
    #[serde(default)]
    pub is_followed: Option<bool>,
    #[serde(default)]
    pub is_accept_request: Option<bool>,
    #[serde(default)]
    pub is_muted: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileImageUrls {
    #[serde(default)]
    pub medium: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: u64,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub account: Option<String>,
    #[serde(default)]
    pub profile_image_urls: Option<ProfileImageUrls>,
    #[serde(default)]
    pub comment: Option<String>,
    #[serde(default)]
    pub is_followed: Option<bool>,
    #[serde(default)]
    pub profile: Option<Profile>,
    #[serde(default)]
    pub is_access_blocking_user: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profile {
    #[serde(default)]
    pub webpage: Option<String>,
    #[serde(default)]
    pub gender: Option<String>,
    #[serde(default)]
    pub birth: Option<String>,
    #[serde(default)]
    pub birth_day: Option<String>,
    #[serde(default)]
    pub region: Option<String>,
    #[serde(default)]
    pub country_code: Option<String>,
    #[serde(default)]
    pub job: Option<String>,
    #[serde(default)]
    pub total_follow_users: Option<u64>,
    #[serde(default)]
    pub total_mypixiv_users: Option<u64>,
    #[serde(default)]
    pub total_illusts: Option<u64>,
    #[serde(default)]
    pub total_manga: Option<u64>,
    #[serde(default)]
    pub total_novels: Option<u64>,
    #[serde(default)]
    pub total_illust_bookmarks_public: Option<u64>,
    #[serde(default)]
    pub background_image_url: Option<String>,
    #[serde(default)]
    pub twitter_account: Option<String>,
    #[serde(default)]
    pub twitter_url: Option<String>,
    #[serde(default)]
    pub is_premium: Option<bool>,
    #[serde(default)]
    pub birth_year: Option<i32>,
    #[serde(default)]
    pub address_id: Option<i32>,
    #[serde(default)]
    pub job_id: Option<i32>,
    #[serde(default)]
    pub total_illust_series: Option<u64>,
    #[serde(default)]
    pub total_novel_series: Option<u64>,
    #[serde(default)]
    pub pawoo_url: Option<String>,
    #[serde(default)]
    pub is_using_custom_profile_image: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfilePublicity {
    #[serde(default)]
    pub gender: Option<String>,
    #[serde(default)]
    pub region: Option<String>,
    #[serde(default)]
    pub birth_day: Option<String>,
    #[serde(default)]
    pub birth_year: Option<String>,
    #[serde(default)]
    pub job: Option<String>,
    #[serde(default)]
    pub pawoo: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserDetail {
    pub user: User,
    #[serde(default)]
    pub profile: Option<Profile>,
    #[serde(default)]
    pub workspace: Option<Workspace>,
    #[serde(default)]
    pub profile_publicity: Option<ProfilePublicity>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workspace {
    #[serde(default)]
    pub pc: Option<String>,
    #[serde(default)]
    pub monitor: Option<String>,
    #[serde(default)]
    pub tool: Option<String>,
    #[serde(default)]
    pub tablet: Option<String>,
    #[serde(default)]
    pub mouse: Option<String>,
    #[serde(default)]
    pub printer: Option<String>,
    #[serde(default)]
    pub desktop: Option<String>,
    #[serde(default)]
    pub music: Option<String>,
    #[serde(default)]
    pub desk: Option<String>,
    #[serde(default)]
    pub chair: Option<String>,
    #[serde(default)]
    pub comment: Option<String>,
    #[serde(default)]
    pub scanner: Option<String>,
    #[serde(default)]
    pub workspace_image_url: Option<String>,
}

/// Response from user_following endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserFollowingResult {
    #[serde(default)]
    pub user_previews: Vec<UserPreview>,
    #[serde(default)]
    pub next_url: Option<String>,
}

/// Response from user_follower endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserFollowerResult {
    #[serde(default)]
    pub user_previews: Vec<UserPreview>,
    #[serde(default)]
    pub next_url: Option<String>,
}

/// Response from user_mypixiv endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserMypixivResult {
    #[serde(default)]
    pub user_previews: Vec<UserPreview>,
    #[serde(default)]
    pub next_url: Option<String>,
}

/// Response from user_list endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserListResult {
    #[serde(default)]
    pub user_previews: Vec<UserPreview>,
    #[serde(default)]
    pub next_url: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_preview_partial() {
        let json = r#"{"id": 111, "name": "Artist"}"#;
        let user: UserPreview = serde_json::from_str(json).unwrap();
        assert_eq!(user.id, Some(111));
        assert_eq!(user.name.as_deref(), Some("Artist"));
    }

    #[test]
    fn test_user_detail_partial() {
        let json = r#"{"user": {"id": 222}}"#;
        let detail: UserDetail = serde_json::from_str(json).unwrap();
        assert_eq!(detail.user.id, 222);
    }
}
