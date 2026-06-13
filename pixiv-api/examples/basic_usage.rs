use pixiv_client::PixivApi;
use pixiv_client::models::search::SearchOptions;
use pixiv_client::models::search::SearchSort;

#[tokio::main]
async fn main() -> Result<(), pixiv_client::PixivError> {
    let api = PixivApi::new();

    // Authenticate (requires PIXIV_REFRESH_TOKEN env var)
    let token =
        std::env::var("PIXIV_REFRESH_TOKEN").expect("Set PIXIV_REFRESH_TOKEN environment variable");
    api.auth(&token).await?;
    println!("Authenticated as user {:?}", api.user_id().await);

    // Search for illustrations
    // If the access token has expired, you'll get a 401 error.
    // Handle it explicitly by calling refresh_token() and retrying:
    //
    //   let results = match api.search_illust("landscape", None, None, None, None).await {
    //       Err(e) if e.is_auth_error() => {
    //           api.refresh_token().await?;
    //           api.search_illust("landscape", None, None, None, None).await?
    //       }
    //       other => other?,
    //   };

    let mut options = SearchOptions::default();
    options.sort = Some(SearchSort::PopularDesc);
    let results = api
        .search_illust("landscape", Some(options))
        .await?;

    if let Some(data) = &results.data {
        println!(
            "Got typed response: {} bytes",
            serde_json::to_string(data).unwrap().len()
        );
    }
    println!("Raw JSON available: {}", results.raw.is_object());

    let detail = api.illust_detail(12345).await?;
    println!(
        "Illustration raw: {}",
        serde_json::to_string_pretty(&detail.raw).unwrap()
    );

    Ok(())
}
