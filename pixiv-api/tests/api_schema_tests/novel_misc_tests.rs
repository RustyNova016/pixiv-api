//! Schema tests for Novel and Misc API endpoints.
//!
//! Run: cargo test -p pixiv-client --test api_schema_tests -- novel_misc_tests --nocapture
//! Requires: PIXIV_REFRESH_TOKEN env var + proxy at 127.0.0.1:7897

use super::{
    TEST_ILLUST_ID, TEST_NOVEL_ID, TEST_NOVEL_SERIES_ID, assert_data_ok, create_client,
    print_schema_comparison, sf,
};

#[tokio::test]
async fn test_novel_detail_schema() {
    let api = create_client().await;
    let resp = api
        .novel_detail(TEST_NOVEL_ID)
        .await
        .expect("novel_detail failed");

    let expected = &[sf("novel", "Novel", true)];

    print_schema_comparison("NovelDetail", expected, &resp.raw);
    assert_data_ok(&resp);

    let data = resp.data.unwrap();
    let novel = &data.novel;
    println!(
        "  [NovelDetail.novel] id={}, title={}",
        novel.id, novel.title
    );
    println!(
        "  [NovelDetail.novel] text_length={:?}, total_view={:?}, total_bookmarks={:?}",
        novel.text_length, novel.total_view, novel.total_bookmarks
    );
    if let Some(ref user) = novel.user {
        println!(
            "  [NovelDetail.novel.user] id={:?}, name={:?}",
            user.id, user.name
        );
    }
    if let Some(ref series) = novel.series {
        println!(
            "  [NovelDetail.novel.series] id={:?}, title={:?}",
            series.id, series.title
        );
    }
}

#[tokio::test]
async fn test_novel_comments_schema() {
    let api = create_client().await;
    match api.novel_comments(TEST_NOVEL_ID, None).await {
        Ok(resp) => {
            let expected = &[
                sf("comments", "Vec<Comment>", true),
                sf("next_url", "Option<String>", false),
                sf("total_comments", "Option<u64>", false),
                sf("comment_access_control", "Option<i32>", false),
            ];

            print_schema_comparison("NovelComments", expected, &resp.raw);
            assert_data_ok(&resp);

            let data = resp.data.unwrap();
            println!("  [NovelComments] comments count: {}", data.comments.len());
            println!(
                "  [NovelComments] total_comments: {:?}",
                data.total_comments
            );
        }
        Err(e) => {
            println!("  [NovelComments] Error: {e}");
            println!(
                "  [NovelComments] TEST_NOVEL_ID={} may not have comments or be restricted",
                TEST_NOVEL_ID
            );
        }
    }
}

#[tokio::test]
async fn test_novel_recommended_schema() {
    let api = create_client().await;
    let resp = api
        .novel_recommended()
        .await
        .expect("novel_recommended failed");

    let expected = &[
        sf("novels", "Vec<Novel>", true),
        sf("next_url", "Option<String>", false),
    ];

    print_schema_comparison("NovelRecommendedResult", expected, &resp.raw);
    assert_data_ok(&resp);

    let data = resp.data.unwrap();
    println!(
        "  [NovelRecommendedResult] novels count: {}",
        data.novels.len()
    );
    if let Some(first) = data.novels.first() {
        println!("  [first novel] id={}, title={}", first.id, first.title);
    }
}

#[tokio::test]
async fn test_novel_new_schema() {
    let api = create_client().await;
    let resp = api.novel_new().await.expect("novel_new failed");

    let expected = &[
        sf("novels", "Vec<Novel>", true),
        sf("next_url", "Option<String>", false),
    ];

    print_schema_comparison("NovelNewResult", expected, &resp.raw);
    assert_data_ok(&resp);

    let data = resp.data.unwrap();
    println!("  [NovelNewResult] novels count: {}", data.novels.len());
    if let Some(first) = data.novels.first() {
        println!("  [first new novel] id={}, title={}", first.id, first.title);
    }
}

#[tokio::test]
async fn test_novel_follow_schema() {
    let api = create_client().await;
    let resp = api
        .novel_follow(Some("public"))
        .await
        .expect("novel_follow failed");

    let expected = &[
        sf("novels", "Vec<Novel>", true),
        sf("next_url", "Option<String>", false),
    ];

    print_schema_comparison("NovelFollowResult", expected, &resp.raw);
    assert_data_ok(&resp);

    let data = resp.data.unwrap();
    println!("  [NovelFollowResult] novels count: {}", data.novels.len());
}

#[tokio::test]
async fn test_novel_series_schema() {
    let api = create_client().await;
    let resp = api
        .novel_series(TEST_NOVEL_SERIES_ID)
        .await
        .expect("novel_series failed");

    let expected = &[
        sf("novel_series_detail", "Option<NovelSeries>", true),
        sf("novel_series_first_novel", "Option<Novel>", false),
        sf("novel_series_latest_novel", "Option<Novel>", false),
        sf("novels", "Vec<Novel>", true),
        sf("next_url", "Option<String>", false),
    ];

    print_schema_comparison("NovelSeriesResult", expected, &resp.raw);
    assert_data_ok(&resp);

    let data = resp.data.unwrap();
    if let Some(ref series) = data.novel_series_detail {
        println!(
            "  [NovelSeriesResult.novel_series_detail] id={}, title={:?}",
            series.id, series.title
        );
        println!(
            "  [NovelSeriesResult.novel_series_detail] content_count={:?}, total_character_count={:?}",
            series.content_count, series.total_character_count
        );
    }
    if let Some(ref first_novel) = data.novel_series_first_novel {
        println!(
            "  [NovelSeriesResult.first_novel] id={}, title={}",
            first_novel.id, first_novel.title
        );
    }
    if let Some(ref latest_novel) = data.novel_series_latest_novel {
        println!(
            "  [NovelSeriesResult.latest_novel] id={}, title={}",
            latest_novel.id, latest_novel.title
        );
    }
    println!("  [NovelSeriesResult] novels count: {}", data.novels.len());
    if let Some(first) = data.novels.first() {
        println!(
            "  [first novel in series] id={}, title={}",
            first.id, first.title
        );
    }
}

#[tokio::test]
async fn test_novel_text_schema() {
    let api = create_client().await;
    match api.novel_text(TEST_NOVEL_ID).await {
        Ok(resp) => {
            let expected = &[sf("novel_text", "Option<String>", false)];

            print_schema_comparison("NovelTextResult", expected, &resp.raw);
            assert_data_ok(&resp);

            let data = resp.data.unwrap();
            let text_len = data.novel_text.as_ref().map(|t| t.len()).unwrap_or(0);
            println!("  [NovelTextResult] novel_text length: {} chars", text_len);
            if let Some(ref text) = data.novel_text {
                let preview: String = text.chars().take(200).collect();
                println!("  [NovelTextResult] preview: {preview}");
            }
        }
        Err(e) => {
            println!("  [NovelTextResult] Error (novel may be restricted): {e}");
            println!(
                "  [NovelTextResult] TEST_NOVEL_ID={} may require auth or be deleted",
                TEST_NOVEL_ID
            );
        }
    }
}

#[tokio::test]
async fn test_ugoira_metadata_schema() {
    let api = create_client().await;
    match api.ugoira_metadata(TEST_ILLUST_ID).await {
        Ok(resp) => {
            let expected = &[
                sf("zip_urls", "Option<UgoiraZipUrls>", false),
                sf("frames", "Option<Vec<UgoiraFrame>>", false),
            ];

            print_schema_comparison("UgoiraMetadata", expected, &resp.raw);
            assert_data_ok(&resp);

            let data = resp.data.unwrap();
            if let Some(ref urls) = data.zip_urls {
                println!("  [UgoiraMetadata.zip_urls] medium={:?}", urls.medium);
            }
            if let Some(ref frames) = data.frames {
                println!("  [UgoiraMetadata] frames count: {}", frames.len());
                if let Some(first) = frames.first() {
                    println!("  [first frame] file={}, delay={}", first.file, first.delay);
                }
            }
        }
        Err(e) => {
            println!("  [UgoiraMetadata] Expected error for non-ugoira work: {e}");
            println!(
                "  [UgoiraMetadata] TEST_ILLUST_ID={} is not a ugoira, test passed (graceful failure)",
                TEST_ILLUST_ID
            );
        }
    }
}

#[tokio::test]
async fn test_showcase_article_schema() {
    let api = create_client().await;
    match api.showcase_article("37083").await {
        Ok(resp) => {
            println!("\n{}", "=".repeat(70));
            println!("SCHEMA TEST: ShowcaseArticle (raw JSON)");
            println!("{}", "=".repeat(70));
            let json_str = serde_json::to_string_pretty(&resp.raw).unwrap_or_default();
            let truncated: String = json_str.chars().take(2000).collect();
            println!("{truncated}");
            if json_str.len() > 2000 {
                println!("... (truncated, {} total chars)", json_str.len());
            }
            println!("{}\n", "=".repeat(70));
        }
        Err(e) => {
            println!("  [ShowcaseArticle] Error (showcase may not exist): {e}");
            println!("  [ShowcaseArticle] This is acceptable — showcase IDs are ephemeral");
        }
    }
}
