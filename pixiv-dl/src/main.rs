use clap::{Parser, Subcommand};
use pixiv_api::PixivApi;
use pixiv_api::models::search::SearchSort;

#[derive(Parser)]
#[command(name = "pixiv-dl")]
#[command(version, about = "Pixiv illustration downloader")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Authenticate with a refresh token
    Auth {
        /// Your Pixiv refresh token
        #[arg(short, long)]
        token: String,
    },
    /// Search for illustrations
    Search {
        /// Search keyword
        keyword: String,
        /// Sort order (date_desc, date_asc, popular_desc, popular_male_desc, popular_female_desc)
        #[arg(short, long, default_value = "date_desc")]
        sort: String,
        /// Page offset
        #[arg(short, long, default_value = "0")]
        offset: u32,
    },
    /// Show illustration details
    Illust {
        /// Illustration ID
        id: u64,
    },
    /// Download illustrations by ID
    Download {
        /// Illustration IDs to download
        ids: Vec<u64>,
        /// Output directory
        #[arg(short, long, default_value = "./images")]
        output: String,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Auth { token } => {
            let mut api = PixivApi::new();
            api.auth(&token).await?;
            println!("Authenticated successfully.");
            println!("User ID: {:?}", api.user_id().await);
        }
        Commands::Search {
            keyword,
            sort,
            offset,
        } => {
            let api = authenticated_api().await?;
            let sort_enum: SearchSort = sort
                .parse()
                .map_err(|e: String| pixiv_api::PixivError::Other(e))?;
            let result = api
                .search_illust(&keyword, Some(sort_enum), None, None, Some(offset))
                .await?;
            println!("{}", serde_json::to_string_pretty(&result.raw)?);
        }
        Commands::Illust { id } => {
            let api = authenticated_api().await?;
            let result = api.illust_detail(id).await?;
            println!("{}", serde_json::to_string_pretty(&result.raw)?);
        }
        Commands::Download { ids, output } => {
            let api = authenticated_api().await?;
            for id in ids {
                let detail = api.illust_detail(id).await?;
                println!("Downloading illustration {id}...");
                let image_url = detail.raw["illust"]["image_urls"]["large"]
                    .as_str()
                    .or_else(|| {
                        detail.raw["illust"]["meta_single_page"]["original_image_url"].as_str()
                    });
                if let Some(url) = image_url {
                    let dm = pixiv_api::downloader::DownloadManager::new(
                        reqwest::Client::new(),
                        &output,
                    );
                    let ext = if url.contains(".png") { "png" } else { "jpg" };
                    let filename = format!("{id}.{ext}");
                    match dm.download(url, &filename).await {
                        Ok(path) => println!("  Saved to {}", path.display()),
                        Err(e) => eprintln!("  Failed: {e}"),
                    }
                } else {
                    eprintln!("  Could not find image URL for {id}");
                }
            }
        }
    }

    Ok(())
}

async fn authenticated_api() -> Result<PixivApi, Box<dyn std::error::Error>> {
    let token = std::env::var("PIXIV_REFRESH_TOKEN")
        .map_err(|_| "Set PIXIV_REFRESH_TOKEN env var or use 'pixiv-dl auth' first")?;

    let mut api = PixivApi::new();
    api.auth(&token).await?;
    Ok(api)
}
