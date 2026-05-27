use pixiv_api::PixivApi;
use pixiv_api::models::search::SearchSort;

#[tokio::main]
async fn main() -> Result<(), pixiv_api::PixivError> {
    // Create a new client
    let mut api = PixivApi::new();

    // Authenticate (requires PIXIV_REFRESH_TOKEN env var)
    let token =
        std::env::var("PIXIV_REFRESH_TOKEN").expect("Set PIXIV_REFRESH_TOKEN environment variable");
    api.auth(&token).await?;
    println!("Authenticated as user {:?}", api.user_id().await);

    // Search for illustrations (typed response with raw fallback)
    let results = api
        .search_illust("landscape", Some(SearchSort::PopularDesc), None, None, None)
        .await?;
    if let Some(data) = &results.data {
        println!(
            "Got typed response: {} bytes",
            serde_json::to_string(data).unwrap().len()
        );
    }
    // Always available regardless of parse success
    println!("Raw JSON available: {}", results.raw.is_object());

    // Get illustration detail
    let detail = api.illust_detail(12345).await?;
    println!(
        "Illustration raw: {}",
        serde_json::to_string_pretty(&detail.raw).unwrap()
    );

    Ok(())
}
