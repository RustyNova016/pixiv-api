/// Search for illustrations and download them.
///
/// Run: cargo run -p pixiv-client --example download_illusts
use pixiv_client::PixivApi;
use pixiv_client::downloader::DownloadManager;
use pixiv_client::models::search::SearchOptions;
use pixiv_client::models::search::SearchSort;

#[tokio::main]
async fn main() -> Result<(), pixiv_client::PixivError> {
    let api = PixivApi::new();

    let token =
        std::env::var("PIXIV_REFRESH_TOKEN").expect("Set PIXIV_REFRESH_TOKEN environment variable");
    api.auth(&token).await?;
    println!("Authenticated as user {:?}\n", api.user_id().await);

    // Search for illustrations
    let keyword = "風景"; // "landscape" in Japanese
    println!("Searching for '{}'...", keyword);
    let mut options = SearchOptions::default();
    options.sort = Some(SearchSort::PopularDesc);
    let results = api.search_illust(keyword, Some(options)).await?;

    let Some(data) = &results.data else {
        println!("Failed to parse search results, raw JSON:");
        println!("{}", serde_json::to_string_pretty(&results.raw)?);
        return Ok(());
    };

    println!("Found {} illustrations\n", data.illusts.len());

    // Download first 3 illustrations
    let dm = DownloadManager::new(reqwest::Client::new(), "./downloads");

    for illust in data.illusts.iter().take(3) {
        let id = illust.id;
        let title = &illust.title;

        // Get the best available image URL
        let image_url = illust
            .meta_single_page
            .as_ref()
            .and_then(|p| p.original_image_url.as_deref())
            .or(illust.image_urls.as_ref().and_then(|u| u.large.as_deref()));

        if let Some(url) = image_url {
            let ext = if url.contains(".png") { "png" } else { "jpg" };
            let filename = format!("{id}.{ext}");
            println!("Downloading: {} ({})", title, id);

            match dm.download(url, &filename).await {
                Ok(path) => println!("  Saved to: {}\n", path.display()),
                Err(e) => eprintln!("  Failed: {}\n", e),
            }
        } else {
            println!("Skipping {} ({}) - no image URL found\n", title, id);
        }
    }

    Ok(())
}
