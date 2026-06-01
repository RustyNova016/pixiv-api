//! Schema tests for Illust API endpoints.
//!
//! Run: cargo test -p pixiv-client --test api_schema_tests -- illust_tests --nocapture
//! Requires: PIXIV_REFRESH_TOKEN env var + proxy at 127.0.0.1:7897

use super::{
    TEST_COMMENT_ILLUST_ID, TEST_ILLUST_ID, assert_data_ok, create_client, print_schema_comparison,
    sf,
};

#[tokio::test]
async fn test_illust_detail_schema() {
    let api = create_client().await;
    let resp = api
        .illust_detail(TEST_ILLUST_ID)
        .await
        .expect("illust_detail failed");

    let expected = &[sf("illust", "Illust", true)];

    print_schema_comparison("IllustDetail", expected, &resp.raw);
    assert_data_ok(&resp);

    let data = resp.data.unwrap();
    let illust = &data.illust;
    println!(
        "  [IllustDetail.illust] id={}, title={}",
        illust.id, illust.title
    );
    println!(
        "  [IllustDetail.illust] type={:?}, page_count={:?}, width={:?}, height={:?}",
        illust.r#type, illust.page_count, illust.width, illust.height
    );
    println!(
        "  [IllustDetail.illust] total_view={:?}, total_bookmarks={:?}, is_bookmarked={:?}",
        illust.total_view, illust.total_bookmarks, illust.is_bookmarked
    );
    println!(
        "  [IllustDetail.illust] sanity_level={:?}, x_restrict={:?}, illust_ai_type={:?}",
        illust.sanity_level, illust.x_restrict, illust.illust_ai_type
    );
    if let Some(ref user) = illust.user {
        println!(
            "  [IllustDetail.illust.user] id={:?}, name={:?}",
            user.id, user.name
        );
    }
    if let Some(ref tags) = illust.tags {
        println!(
            "  [IllustDetail.illust.tags] count={}, first={:?}",
            tags.len(),
            tags.first().map(|t| &t.name)
        );
    }
    if let Some(ref urls) = illust.image_urls {
        println!(
            "  [IllustDetail.illust.image_urls] square_medium={:?}, medium={:?}, large={:?}",
            urls.square_medium, urls.medium, urls.large
        );
    }

    let illust_expected = &[
        sf("id", "u64", true),
        sf("title", "String", true),
        sf("type", "Option<IllustType>", false),
        sf("image_urls", "Option<ImageUrls>", false),
        sf("caption", "Option<String>", false),
        sf("user", "Option<UserPreview>", false),
        sf("tags", "Option<Vec<Tag>>", false),
        sf("tools", "Option<Vec<String>>", false),
        sf("create_date", "Option<DateTime<Utc>>", false),
        sf("page_count", "Option<u32>", false),
        sf("width", "Option<u32>", false),
        sf("height", "Option<u32>", false),
        sf("sanity_level", "Option<i32>", false),
        sf("x_restrict", "Option<i32>", false),
        sf("series", "Option<SeriesRef>", false),
        sf("meta_single_page", "Option<MetaSinglePage>", false),
        sf("meta_pages", "Option<Vec<MetaPage>>", false),
        sf("total_view", "Option<u64>", false),
        sf("total_bookmarks", "Option<u64>", false),
        sf("is_bookmarked", "Option<bool>", false),
        sf("visible", "Option<bool>", false),
        sf("is_muted", "Option<bool>", false),
        sf("total_comments", "Option<u64>", false),
        sf("restrict", "Option<i32>", false),
        sf("illust_ai_type", "Option<i32>", false),
        sf("illust_book_style", "Option<i32>", false),
        sf("event_banners", "Option<Vec<Value>>", false),
        sf("request", "Option<Value>", false),
        sf("seasonal_effect_animation_urls", "Option<Value>", false),
        sf("restriction_attributes", "Option<Vec<String>>", false),
        sf("comment_access_control", "Option<i32>", false),
        sf("favorited_details", "Option<Value>", false),
    ];

    let illust_json = resp.raw.get("illust").unwrap_or(&serde_json::Value::Null);
    print_schema_comparison("Illust (inner)", illust_expected, illust_json);
}

#[tokio::test]
async fn test_illust_comments_schema() {
    let api = create_client().await;
    match api.illust_comments(TEST_COMMENT_ILLUST_ID, None).await {
        Ok(resp) => {
            let expected = &[
                sf("comments", "Vec<Comment>", true),
                sf("next_url", "Option<String>", false),
                sf("total_comments", "Option<u64>", false),
                sf("comment_access_control", "Option<i32>", false),
            ];

            print_schema_comparison("IllustCommentsResult", expected, &resp.raw);
            assert_data_ok(&resp);

            let data = resp.data.unwrap();
            println!(
                "  [IllustCommentsResult] comments count: {}",
                data.comments.len()
            );
            println!(
                "  [IllustCommentsResult] total_comments: {:?}",
                data.total_comments
            );
            if let Some(first) = data.comments.first() {
                println!(
                    "  [first comment] id={}, comment={:?}, has_replies={:?}",
                    first.id, first.comment, first.has_replies
                );
            }
        }
        Err(e) => {
            println!("  [IllustComments] Error: {e}");
            println!(
                "  [IllustComments] TEST_COMMENT_ILLUST_ID={} may not have comments",
                TEST_COMMENT_ILLUST_ID
            );
        }
    }
}

#[tokio::test]
async fn test_illust_related_schema() {
    let api = create_client().await;
    let resp = api
        .illust_related(TEST_ILLUST_ID)
        .await
        .expect("illust_related failed");

    let expected = &[
        sf("illusts", "Vec<Illust>", true),
        sf("next_url", "Option<String>", false),
    ];

    print_schema_comparison("IllustRelatedResult", expected, &resp.raw);
    assert_data_ok(&resp);

    let data = resp.data.unwrap();
    println!(
        "  [IllustRelatedResult] illusts count: {}",
        data.illusts.len()
    );
    if let Some(first) = data.illusts.first() {
        println!("  [first related] id={}, title={}", first.id, first.title);
    }
}

#[tokio::test]
async fn test_illust_recommended_schema() {
    let api = create_client().await;
    let resp = api
        .illust_recommended()
        .await
        .expect("illust_recommended failed");

    let expected = &[
        sf("illusts", "Vec<Illust>", true),
        sf("ranking_illusts", "Option<Vec<Illust>>", false),
        sf("contest_exists", "Option<bool>", false),
        sf("next_url", "Option<String>", false),
    ];

    print_schema_comparison("IllustRecommendedResult", expected, &resp.raw);
    assert_data_ok(&resp);

    let data = resp.data.unwrap();
    println!(
        "  [IllustRecommendedResult] illusts count: {}",
        data.illusts.len()
    );
    println!(
        "  [IllustRecommendedResult] ranking_illusts: {:?}",
        data.ranking_illusts.as_ref().map(|v| v.len())
    );
    println!(
        "  [IllustRecommendedResult] contest_exists: {:?}",
        data.contest_exists
    );
    if let Some(first) = data.illusts.first() {
        println!(
            "  [first recommended] id={}, title={}",
            first.id, first.title
        );
    }
}

#[tokio::test]
async fn test_illust_ranking_schema() {
    let api = create_client().await;
    let resp = api
        .illust_ranking(Some("day"), None, None)
        .await
        .expect("illust_ranking failed");

    let expected = &[
        sf("illusts", "Vec<Illust>", true),
        sf("next_url", "Option<String>", false),
    ];

    print_schema_comparison("IllustRankingResult", expected, &resp.raw);
    assert_data_ok(&resp);

    let data = resp.data.unwrap();
    println!(
        "  [IllustRankingResult] illusts count: {}",
        data.illusts.len()
    );
    for (i, illust) in data.illusts.iter().take(3).enumerate() {
        println!(
            "  [ranking #{i}] id={}, title={}, total_view={:?}, total_bookmarks={:?}",
            illust.id, illust.title, illust.total_view, illust.total_bookmarks
        );
    }
}

#[tokio::test]
async fn test_illust_new_schema() {
    let api = create_client().await;
    let resp = api.illust_new().await.expect("illust_new failed");

    let expected = &[
        sf("illusts", "Vec<Illust>", true),
        sf("next_url", "Option<String>", false),
    ];

    print_schema_comparison("IllustNewResult", expected, &resp.raw);
    assert_data_ok(&resp);

    let data = resp.data.unwrap();
    println!("  [IllustNewResult] illusts count: {}", data.illusts.len());
    if let Some(first) = data.illusts.first() {
        println!("  [first new] id={}, title={}", first.id, first.title);
    }
}

#[tokio::test]
async fn test_illust_bookmark_detail_schema() {
    let api = create_client().await;
    let resp = api
        .illust_bookmark_detail(TEST_ILLUST_ID)
        .await
        .expect("illust_bookmark_detail failed");

    let expected = &[sf("bookmark_detail", "Option<BookmarkDetail>", true)];

    print_schema_comparison("IllustBookmarkDetailResult", expected, &resp.raw);
    assert_data_ok(&resp);

    let data = resp.data.unwrap();
    if let Some(ref detail) = data.bookmark_detail {
        println!("  [BookmarkDetail] is_bookmarked={}", detail.is_bookmarked);
        println!("  [BookmarkDetail] restrict={:?}", detail.restrict);
        println!("  [BookmarkDetail] tags count={}", detail.tags.len());
        for (i, tag) in detail.tags.iter().take(5).enumerate() {
            println!(
                "  [tag {i}] name={:?}, is_registered={:?}",
                tag.name, tag.is_registered
            );
        }
    } else {
        println!("  [IllustBookmarkDetailResult] bookmark_detail is None");
    }
}
