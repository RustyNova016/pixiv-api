mod config;

use clap::{Parser, Subcommand};
use futures::future::join_all;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use pixiv_client::PixivApi;
use pixiv_client::downloader::{DownloadManager, ProgressEvent, resolve_download_tasks};
use pixiv_client::models::search::SearchSort;
use std::collections::HashMap;

#[derive(Debug, Clone)]
struct IllustInput {
    id: u64,
    pages: Option<Vec<usize>>,
}

fn parse_illust_input(s: &str) -> Result<IllustInput, String> {
    if let Some(bracket_start) = s.find('[') {
        let id_str = &s[..bracket_start];
        let id: u64 = id_str
            .parse()
            .map_err(|_| format!("invalid illustration ID: {id_str}"))?;

        let rest = &s[bracket_start..];
        if !rest.ends_with(']') {
            return Err(format!("missing closing bracket in: {s}"));
        }
        let pages_str = &rest[1..rest.len() - 1];
        let pages: Vec<usize> = pages_str
            .split(',')
            .map(|p| p.trim().parse::<usize>())
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_| format!("invalid page number in: {s}"))?;

        Ok(IllustInput {
            id,
            pages: Some(pages),
        })
    } else {
        let id: u64 = s
            .parse()
            .map_err(|_| format!("invalid illustration ID: {s}"))?;
        Ok(IllustInput { id, pages: None })
    }
}

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
        /// Your Pixiv refresh token (omit value to paste via stdin)
        #[arg(short, long, num_args = 0..=1, default_missing_value = "")]
        token: Option<String>,

        /// Run the interactive OAuth2 PKCE flow to obtain a refresh token
        #[arg(short, long)]
        oauth: bool,
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
        /// Illustration IDs (e.g. 12345 or 12345[0,2,3])
        ids: Vec<String>,
        /// Output directory
        #[arg(short, long, default_value = "./images")]
        output: String,
        /// Image size: original, large, or medium
        #[arg(short, long, default_value = "original")]
        size: String,
        /// Max concurrent downloads
        #[arg(short = 'j', long, default_value = "4")]
        concurrency: usize,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Auth { token, oauth } => {
            let refresh_token = if oauth {
                oauth_login_flow().await?
            } else if let Some(t) = token {
                if t.is_empty() {
                    eprint!("Paste your refresh token: ");
                    read_line_trimmed()?
                } else {
                    t
                }
            } else {
                eprintln!("Usage: pixiv-dl auth --token [TOKEN] or pixiv-dl auth --oauth");
                return Ok(());
            };

            eprint!("Authenticating...");
            let api = PixivApi::new();
            api.auth(&refresh_token).await?;
            eprintln!(" done.");

            let cfg = config::Config {
                refresh_token: Some(refresh_token),
            };
            config::save(&cfg)?;

            if let Some(path) = config::config_path_display() {
                eprintln!("Credential saved to {path}");
            }

            println!(
                "Authenticated successfully. User ID: {:?}",
                api.user_id().await
            );
        }
        Commands::Search {
            keyword,
            sort,
            offset,
        } => {
            let api = authenticated_api().await?;
            let sort_enum: SearchSort = sort
                .parse()
                .map_err(|e: String| pixiv_client::PixivError::Other(e))?;
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
        Commands::Download {
            ids,
            output,
            size,
            concurrency,
        } => {
            let api = std::sync::Arc::new(authenticated_api().await?);

            // Parse all inputs
            let inputs: Vec<IllustInput> = ids
                .iter()
                .map(|s| parse_illust_input(s))
                .collect::<Result<Vec<_>, _>>()
                .map_err(pixiv_client::PixivError::Other)?;

            // Fetch all illustration details in parallel
            eprint!("Fetching details for {} illustration(s)...", inputs.len());
            let fetch_sem = std::sync::Arc::new(tokio::sync::Semaphore::new(concurrency));
            let fetch_handles: Vec<_> = inputs
                .iter()
                .map(|input| {
                    let sem = fetch_sem.clone();
                    let api = api.clone();
                    let id = input.id;
                    async move {
                        let _permit = sem.acquire().await.unwrap();
                        api.illust_detail(id).await
                    }
                })
                .collect();
            let details = join_all(fetch_handles).await;
            eprintln!(" done.");

            // Resolve all download tasks
            let mut all_tasks: Vec<pixiv_client::downloader::DownloadTask> = Vec::new();
            for (input, detail_result) in inputs.iter().zip(details.iter()) {
                match detail_result {
                    Ok(resp) => {
                        if let Some(illust) = &resp.data {
                            let tasks = resolve_download_tasks(
                                &illust.illust,
                                &size,
                                input.pages.as_deref(),
                            );
                            if tasks.is_empty() {
                                eprintln!("  Warning: no downloadable images for {}", input.id);
                            }
                            all_tasks.extend(tasks);
                        } else {
                            eprintln!("  Warning: typed parse failed for {}, using raw", input.id);
                            // Fallback: extract URLs from raw JSON
                            let raw = &resp.raw["illust"];
                            if let Some(meta_pages) = raw["meta_pages"].as_array() {
                                let indices: Vec<usize> = if let Some(filter) = &input.pages {
                                    filter.clone()
                                } else {
                                    (0..meta_pages.len()).collect()
                                };
                                for &idx in &indices {
                                    if let Some(page) = meta_pages.get(idx) {
                                        let url = page["image_urls"][&size]
                                            .as_str()
                                            .or(page["image_urls"]["large"].as_str())
                                            .or(page["image_urls"]["medium"].as_str());
                                        if let Some(url) = url {
                                            let ext =
                                                if url.contains(".png") { "png" } else { "jpg" };
                                            all_tasks.push(
                                                pixiv_client::downloader::DownloadTask {
                                                    url: url.to_string(),
                                                    filename: format!(
                                                        "{}_p{}.{}",
                                                        input.id, idx, ext
                                                    ),
                                                },
                                            );
                                        }
                                    }
                                }
                            } else {
                                let url = raw["image_urls"]["large"]
                                    .as_str()
                                    .or(raw["meta_single_page"]["original_image_url"].as_str());
                                if let Some(url) = url {
                                    let ext = if url.contains(".png") { "png" } else { "jpg" };
                                    all_tasks.push(pixiv_client::downloader::DownloadTask {
                                        url: url.to_string(),
                                        filename: format!("{}_p0.{}", input.id, ext),
                                    });
                                }
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("  Error fetching {}: {}", input.id, e);
                    }
                }
            }

            if all_tasks.is_empty() {
                eprintln!("Nothing to download.");
                return Ok(());
            }

            // Set up progress bars
            let multi = MultiProgress::new();
            let overall = multi.add(ProgressBar::new(all_tasks.len() as u64));
            overall.set_style(
                ProgressStyle::default_bar()
                    .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})")
                    .unwrap()
                    .progress_chars("=>-"),
            );
            overall.set_message("Total");

            let file_bars: std::sync::Arc<std::sync::Mutex<HashMap<String, ProgressBar>>> =
                std::sync::Arc::new(std::sync::Mutex::new(HashMap::new()));
            let multi_ref = multi.clone();
            let file_bars_clone = file_bars.clone();
            let overall_clone = overall.clone();

            let dm = DownloadManager::new(reqwest::Client::new(), &output);

            let results = dm
                .download_all(&all_tasks, concurrency, move |evt| match evt {
                    ProgressEvent::Started {
                        filename,
                        total_bytes,
                    } => {
                        let pb = multi_ref.add(ProgressBar::new(total_bytes.unwrap_or(0)));
                        pb.set_style(
                            ProgressStyle::default_bar()
                                .template("  {prefix:.bold} [{bar:30}] {bytes}/{total_bytes} ({bytes_per_sec})")
                                .unwrap()
                                .progress_chars("=>-"),
                        );
                        pb.set_prefix(filename.clone());
                        file_bars_clone.lock().unwrap().insert(filename, pb);
                    }
                    ProgressEvent::Chunk {
                        filename,
                        bytes_downloaded,
                    } => {
                        if let Some(pb) = file_bars_clone.lock().unwrap().get(&filename) {
                            pb.set_position(bytes_downloaded);
                        }
                    }
                    ProgressEvent::Finished { filename, .. } => {
                        if let Some(pb) = file_bars_clone.lock().unwrap().remove(&filename) {
                            pb.finish_with_message("done");
                        }
                        overall_clone.inc(1);
                    }
                    ProgressEvent::Failed {
                        filename,
                        error: _,
                        attempt,
                    } => {
                        if let Some(pb) = file_bars_clone.lock().unwrap().get(&filename) {
                            pb.set_message(format!("retry {}", attempt));
                        }
                    }
                })
                .await;

            overall.finish_with_message("Complete");

            // Report results
            let mut succeeded = 0;
            let mut failed = 0;
            for result in &results {
                match result {
                    Ok(path) => {
                        succeeded += 1;
                        println!("  Saved: {}", path.display());
                    }
                    Err(e) => {
                        failed += 1;
                        eprintln!("  Failed: {e}");
                    }
                }
            }
            println!("\n{} succeeded, {} failed", succeeded, failed);
        }
    }

    Ok(())
}

async fn authenticated_api() -> Result<PixivApi, Box<dyn std::error::Error>> {
    // Try env var first, then saved config
    let token = std::env::var("PIXIV_REFRESH_TOKEN")
        .ok()
        .or_else(|| {
            let cfg = config::load();
            cfg.refresh_token
        })
        .ok_or(
            "Not authenticated. Run 'pixiv-dl auth --token <TOKEN>' or set PIXIV_REFRESH_TOKEN",
        )?;

    eprint!("Authenticating...");
    let api = PixivApi::new();
    api.auth(&token).await?;
    eprintln!(" done.");
    Ok(api)
}

/// Read a single line from stdin, trimmed.
fn read_line_trimmed() -> Result<String, Box<dyn std::error::Error>> {
    let mut line = String::new();
    std::io::stdin().read_line(&mut line)?;
    Ok(line.trim().to_string())
}

/// Run the OAuth2 PKCE flow to obtain a refresh token.
async fn oauth_login_flow() -> Result<String, Box<dyn std::error::Error>> {
    use base64::Engine;
    use sha2::Digest;

    const LOGIN_URL: &str = "https://app-api.pixiv.net/web/v1/login";
    const AUTH_TOKEN_URL: &str = "https://oauth.secure.pixiv.net/auth/token";
    const CLIENT_ID: &str = "MOBrBDS8blbauoSck0ZfDbtuzpyT";
    const CLIENT_SECRET: &str = "lsACyCD94FhDUtGTXi3QzcFE2uU1hqtDaKeqrdwj";
    const HASH_SECRET: &str = "28c1fdd170a5204386cb1313c7077b34f83e4aaf4aa829ce78c231e05b0bae2c";
    const REDIRECT_URI: &str = "https://app-api.pixiv.net/web/v1/users/auth/pixiv/callback";

    // Generate PKCE challenge
    let code_verifier = {
        use rand::Rng;
        let mut rng = rand::rng();
        let bytes: Vec<u8> = (0..32).map(|_| rng.random::<u8>()).collect();
        base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(&bytes)
    };
    let code_challenge = {
        let mut hasher = sha2::Sha256::new();
        hasher.update(code_verifier.as_bytes());
        base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(hasher.finalize())
    };

    let login_params = format!(
        "code_challenge={}&code_challenge_method=S256&client=pixiv-android",
        code_challenge
    );
    let url = format!("{}?{}", LOGIN_URL, login_params);

    println!("=== Pixiv OAuth2 PKCE Login ===\n");
    println!("1. Open this URL in your browser:\n");
    println!("   {}\n", url);
    println!("2. Log in to Pixiv and authorize the app");
    println!("3. After login, open F12 dev tools -> Network tab (check \"Preserve log\")");
    println!("4. Look for a request containing \"callback?code=\" or a pixiv:// URL");
    println!("   It will look like one of these:");
    println!("     https://app-api.pixiv.net/.../callback?state=...&code=XXXXX");
    println!("     pixiv://account/login?code=XXXXX");
    println!("5. Copy the full URL and paste it below\n");

    let redirect_url = read_line_trimmed()?;

    // Extract the code from the redirect URL
    let code = extract_code(&redirect_url).ok_or(
        "Could not extract 'code' from the URL. Make sure you copied the full callback URL",
    )?;

    println!("\nExchanging code for tokens...");

    // Build auth headers
    let now = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S%z").to_string();
    let hash = {
        use md5::Digest;
        let mut hasher = md5::Md5::new();
        hasher.update(format!("{}{}", now, HASH_SECRET).as_bytes());
        hex::encode(hasher.finalize())
    };

    let client = reqwest::Client::new();
    let resp = client
        .post(AUTH_TOKEN_URL)
        .header("x-client-time", &now)
        .header("x-client-hash", &hash)
        .header("Referer", "https://app-api.pixiv.net/")
        .form(&[
            ("client_id", CLIENT_ID),
            ("client_secret", CLIENT_SECRET),
            ("code", &code),
            ("code_verifier", &code_verifier),
            ("grant_type", "authorization_code"),
            ("include_policy", "true"),
            ("redirect_uri", REDIRECT_URI),
        ])
        .send()
        .await?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(format!("OAuth token exchange failed: HTTP {status}\n{body}").into());
    }

    let json: serde_json::Value = resp.json().await?;

    let refresh_token = json["refresh_token"]
        .as_str()
        .ok_or("No refresh_token in response")?
        .to_string();

    println!("\n=== Refresh Token Obtained ===\n");
    println!("{}", refresh_token);
    println!();

    Ok(refresh_token)
}

fn extract_code(url: &str) -> Option<String> {
    let query_start = url.find('?')?;
    let query = &url[query_start + 1..];
    for pair in query.split('&') {
        let mut parts = pair.splitn(2, '=');
        if let (Some(key), Some(value)) = (parts.next(), parts.next())
            && key == "code"
        {
            return Some(value.to_string());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_illust_input_bare_id() {
        let result = parse_illust_input("12345").unwrap();
        assert_eq!(result.id, 12345);
        assert_eq!(result.pages, None);
    }

    #[test]
    fn test_parse_illust_input_with_pages() {
        let result = parse_illust_input("12345[0,2,3]").unwrap();
        assert_eq!(result.id, 12345);
        assert_eq!(result.pages, Some(vec![0, 2, 3]));
    }

    #[test]
    fn test_parse_illust_input_single_page() {
        let result = parse_illust_input("99999[1]").unwrap();
        assert_eq!(result.id, 99999);
        assert_eq!(result.pages, Some(vec![1]));
    }

    #[test]
    fn test_parse_illust_input_invalid() {
        assert!(parse_illust_input("abc").is_err());
        assert!(parse_illust_input("123[").is_err());
        assert!(parse_illust_input("123[abc]").is_err());
    }
}
