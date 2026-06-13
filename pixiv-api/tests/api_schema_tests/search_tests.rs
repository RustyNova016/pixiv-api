//! Schema tests for Search API endpoints.
//!
//! Run: cargo test -p pixiv-client --test api_schema_tests -- search_tests --nocapture
//! Requires: PIXIV_REFRESH_TOKEN env var + proxy at 127.0.0.1:7897

use super::{assert_data_ok, create_client, print_schema_comparison, sf};
use pixiv_client::models::search::SearchOptions;
use pixiv_client::models::search::{SearchSort, SearchTarget};

#[tokio::test]
async fn test_search_illust_schema() {
    let api = create_client().await;
    let mut options = SearchOptions::default();
    options.sort = Some(SearchSort::DateDesc);
    options.target = Some(SearchTarget::PartialMatchForTags);
    let resp = api
        .search_illust(
            "初音ミク",
            Some(options),
        )
        .await
        .expect("search_illust failed");

    let expected = &[
        sf("illusts", "Vec<Illust>", true),
        sf("next_url", "Option<String>", false),
        sf("search_span_limit", "Option<i32>", false),
        sf("show_ai", "Option<bool>", false),
    ];

    print_schema_comparison("SearchIllustResult", expected, &resp.raw);
    assert_data_ok(&resp);

    let data = resp.data.unwrap();
    println!(
        "  [SearchIllustResult] illusts count: {}",
        data.illusts.len()
    );
    if let Some(first) = data.illusts.first() {
        println!("  [first illust] id={}, title={}", first.id, first.title);
    }
    println!("  [next_url] {:?}", data.next_url);
}

#[tokio::test]
async fn test_search_novel_schema() {
    let api = create_client().await;
    let resp = api
        .search_novel(
            "初音ミク",
            Some(SearchSort::DateDesc),
            Some(SearchTarget::PartialMatchForTags),
            None,
        )
        .await
        .expect("search_novel failed");

    let expected = &[
        sf("novels", "Vec<Novel>", true),
        sf("next_url", "Option<String>", false),
        sf("search_span_limit", "Option<i32>", false),
        sf("show_ai", "Option<bool>", false),
    ];

    print_schema_comparison("SearchNovelResult", expected, &resp.raw);
    assert_data_ok(&resp);

    let data = resp.data.unwrap();
    println!("  [SearchNovelResult] novels count: {}", data.novels.len());
    if let Some(first) = data.novels.first() {
        println!("  [first novel] id={}, title={}", first.id, first.title);
    }
}

#[tokio::test]
async fn test_search_user_schema() {
    let api = create_client().await;
    let resp = api
        .search_user("初音ミク", None)
        .await
        .expect("search_user failed");

    let expected = &[
        sf("user_previews", "Vec<UserPreview>", true),
        sf("next_url", "Option<String>", false),
    ];

    print_schema_comparison("SearchUserResult", expected, &resp.raw);
    assert_data_ok(&resp);

    let data = resp.data.unwrap();
    println!(
        "  [SearchUserResult] user_previews count: {}",
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
async fn test_trending_tags_illust_schema() {
    let api = create_client().await;
    let resp = api
        .trending_tags_illust()
        .await
        .expect("trending_tags_illust failed");

    let expected = &[sf("trend_tags", "Vec<TrendingTag>", true)];

    print_schema_comparison("TrendingTagsResult", expected, &resp.raw);
    assert_data_ok(&resp);

    let data = resp.data.unwrap();
    println!(
        "  [TrendingTagsResult] trend_tags count: {}",
        data.trend_tags.len()
    );
    for (i, tt) in data.trend_tags.iter().take(5).enumerate() {
        println!(
            "  [tag {i}] tag={:?}, translated={:?}, has_illust={}",
            tt.tag,
            tt.translated_name,
            tt.illust.is_some()
        );
    }
}
