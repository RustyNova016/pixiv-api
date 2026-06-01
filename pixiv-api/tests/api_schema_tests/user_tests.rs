//! Schema tests for User API endpoints.
//!
//! Run: cargo test -p pixiv-client --test api_schema_tests -- user_tests --nocapture
//! Requires: PIXIV_REFRESH_TOKEN env var + proxy at 127.0.0.1:7897

use super::{TEST_USER_ID, assert_data_ok, create_client, print_schema_comparison, sf};

#[tokio::test]
async fn test_user_detail_schema() {
    let api = create_client().await;
    let resp = api
        .user_detail(TEST_USER_ID)
        .await
        .expect("user_detail failed");

    let expected = &[
        sf("user", "User", true),
        sf("profile", "Option<Profile>", false),
        sf("workspace", "Option<Workspace>", false),
        sf("profile_publicity", "Option<ProfilePublicity>", false),
    ];

    print_schema_comparison("UserDetail", expected, &resp.raw);
    assert_data_ok(&resp);

    let data = resp.data.unwrap();
    println!(
        "  [UserDetail.user] id={}, name={:?}, account={:?}",
        data.user.id, data.user.name, data.user.account
    );
    if let Some(ref profile) = data.profile {
        println!(
            "  [UserDetail.profile] total_illusts={:?}, total_follow_users={:?}, is_premium={:?}",
            profile.total_illusts, profile.total_follow_users, profile.is_premium
        );
    }
    if let Some(ref pub_) = data.profile_publicity {
        println!(
            "  [UserDetail.profile_publicity] gender={:?}, region={:?}, job={:?}",
            pub_.gender, pub_.region, pub_.job
        );
    }
}

#[tokio::test]
async fn test_user_illusts_schema() {
    let api = create_client().await;
    let resp = api
        .user_illusts(TEST_USER_ID, None, None)
        .await
        .expect("user_illusts failed");

    let expected = &[
        sf("illusts", "Vec<Illust>", true),
        sf("next_url", "Option<String>", false),
    ];

    print_schema_comparison("UserIllustsResult", expected, &resp.raw);
    assert_data_ok(&resp);

    let data = resp.data.unwrap();
    println!(
        "  [UserIllustsResult] illusts count: {}",
        data.illusts.len()
    );
    if let Some(first) = data.illusts.first() {
        println!("  [first illust] id={}, title={}", first.id, first.title);
    }
}

#[tokio::test]
async fn test_user_bookmarks_illust_schema() {
    let api = create_client().await;
    let resp = api
        .user_bookmarks_illust(TEST_USER_ID, Some("public"), None, None)
        .await
        .expect("user_bookmarks_illust failed");

    let expected = &[
        sf("illusts", "Vec<Illust>", true),
        sf("next_url", "Option<String>", false),
    ];

    print_schema_comparison("UserBookmarksIllustResult", expected, &resp.raw);
    assert_data_ok(&resp);

    let data = resp.data.unwrap();
    println!(
        "  [UserBookmarksIllustResult] illusts count: {}",
        data.illusts.len()
    );
    if let Some(first) = data.illusts.first() {
        println!("  [first bookmark] id={}, title={}", first.id, first.title);
    }
}

#[tokio::test]
async fn test_user_bookmarks_novel_schema() {
    let api = create_client().await;
    let resp = api
        .user_bookmarks_novel(TEST_USER_ID, Some("public"), None)
        .await
        .expect("user_bookmarks_novel failed");

    let expected = &[
        sf("novels", "Vec<Novel>", true),
        sf("next_url", "Option<String>", false),
    ];

    print_schema_comparison("UserBookmarksNovelResult", expected, &resp.raw);
    assert_data_ok(&resp);

    let data = resp.data.unwrap();
    println!(
        "  [UserBookmarksNovelResult] novels count: {}",
        data.novels.len()
    );
    if let Some(first) = data.novels.first() {
        println!("  [first novel] id={}, title={}", first.id, first.title);
    }
}

#[tokio::test]
async fn test_user_related_schema() {
    let api = create_client().await;
    let resp = api
        .user_related(TEST_USER_ID)
        .await
        .expect("user_related failed");

    let expected = &[
        sf("user_previews", "Vec<UserPreview>", true),
        sf("next_url", "Option<String>", false),
    ];

    print_schema_comparison("UserListResult (user_related)", expected, &resp.raw);
    assert_data_ok(&resp);

    let data = resp.data.unwrap();
    println!(
        "  [UserListResult] user_previews count: {}",
        data.user_previews.len()
    );
    if let Some(first) = data.user_previews.first() {
        if let Some(ref user) = first.user {
            println!(
                "  [first user] id={}, name={:?}, account={:?}",
                user.id, user.name, user.account
            );
        } else {
            println!(
                "  [first user] id={:?}, name={:?}, account={:?}",
                first.id, first.name, first.account
            );
        }
    }
}

#[tokio::test]
async fn test_user_recommended_schema() {
    let api = create_client().await;
    let resp = api
        .user_recommended()
        .await
        .expect("user_recommended failed");

    let expected = &[
        sf("user_previews", "Vec<UserPreview>", true),
        sf("next_url", "Option<String>", false),
    ];

    print_schema_comparison("UserListResult (user_recommended)", expected, &resp.raw);
    assert_data_ok(&resp);

    let data = resp.data.unwrap();
    println!(
        "  [UserListResult] user_previews count: {}",
        data.user_previews.len()
    );
    if let Some(first) = data.user_previews.first() {
        println!("  [first user] id={:?}, name={:?}", first.id, first.name);
    }
}

#[tokio::test]
async fn test_user_following_schema() {
    let api = create_client().await;
    let resp = api
        .user_following(TEST_USER_ID, Some("public"), None)
        .await
        .expect("user_following failed");

    let expected = &[
        sf("user_previews", "Vec<UserPreview>", true),
        sf("next_url", "Option<String>", false),
    ];

    print_schema_comparison("UserFollowingResult", expected, &resp.raw);
    assert_data_ok(&resp);

    let data = resp.data.unwrap();
    println!(
        "  [UserFollowingResult] user_previews count: {}",
        data.user_previews.len()
    );
    if let Some(first) = data.user_previews.first() {
        println!(
            "  [first following] id={:?}, name={:?}",
            first.id, first.name
        );
    }
}

#[tokio::test]
async fn test_user_follower_schema() {
    let api = create_client().await;
    let resp = api
        .user_follower(TEST_USER_ID, None)
        .await
        .expect("user_follower failed");

    let expected = &[
        sf("user_previews", "Vec<UserPreview>", true),
        sf("next_url", "Option<String>", false),
    ];

    print_schema_comparison("UserFollowerResult", expected, &resp.raw);
    assert_data_ok(&resp);

    let data = resp.data.unwrap();
    println!(
        "  [UserFollowerResult] user_previews count: {}",
        data.user_previews.len()
    );
    if let Some(first) = data.user_previews.first() {
        println!(
            "  [first follower] id={:?}, name={:?}",
            first.id, first.name
        );
    }
}

#[tokio::test]
async fn test_user_mypixiv_schema() {
    let api = create_client().await;
    let resp = api
        .user_mypixiv(TEST_USER_ID, None)
        .await
        .expect("user_mypixiv failed");

    let expected = &[
        sf("user_previews", "Vec<UserPreview>", true),
        sf("next_url", "Option<String>", false),
    ];

    print_schema_comparison("UserMypixivResult", expected, &resp.raw);
    assert_data_ok(&resp);

    let data = resp.data.unwrap();
    println!(
        "  [UserMypixivResult] user_previews count: {}",
        data.user_previews.len()
    );
}

#[tokio::test]
async fn test_user_list_schema() {
    let api = create_client().await;
    match api.user_list(&[TEST_USER_ID]).await {
        Ok(resp) => {
            let expected = &[
                sf("user_previews", "Vec<UserPreview>", true),
                sf("next_url", "Option<String>", false),
            ];

            print_schema_comparison("UserListResult", expected, &resp.raw);
            assert_data_ok(&resp);

            let data = resp.data.unwrap();
            println!(
                "  [UserListResult] user_previews count: {}",
                data.user_previews.len()
            );
        }
        Err(e) => {
            println!("  [UserList] Error: {e}");
            println!("  [UserList] user_list endpoint may require multiple valid user IDs");
        }
    }
}

#[tokio::test]
async fn test_user_novels_schema() {
    let api = create_client().await;
    let resp = api
        .user_novels(TEST_USER_ID, None)
        .await
        .expect("user_novels failed");

    let expected = &[
        sf("novels", "Vec<Novel>", true),
        sf("next_url", "Option<String>", false),
    ];

    print_schema_comparison("UserNovelsResult", expected, &resp.raw);
    assert_data_ok(&resp);

    let data = resp.data.unwrap();
    println!("  [UserNovelsResult] novels count: {}", data.novels.len());
    if let Some(first) = data.novels.first() {
        println!("  [first novel] id={}, title={}", first.id, first.title);
    }
}

#[tokio::test]
async fn test_user_bookmark_tags_illust_schema() {
    let api = create_client().await;
    let resp = api
        .user_bookmark_tags_illust(TEST_USER_ID, Some("public"))
        .await
        .expect("user_bookmark_tags_illust failed");

    assert!(
        resp.raw.is_object() || resp.raw.is_array(),
        "Expected object or array, got: {}",
        resp.raw
    );

    println!("\n{}", "=".repeat(70));
    println!("SCHEMA TEST: UserBookmarkTagsIllust (raw JSON)");
    println!("{}", "=".repeat(70));
    let json_str = serde_json::to_string_pretty(&resp.raw).unwrap_or_default();
    let truncated: String = json_str.chars().take(2000).collect();
    println!("{truncated}");
    println!("{}\n", "=".repeat(70));
}
