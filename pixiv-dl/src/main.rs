mod config;

use clap::{Parser, Subcommand};
use pixiv_client::PixivApi;
use pixiv_client::models::search::SearchSort;

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
                    let dm = pixiv_client::downloader::DownloadManager::new(
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
        let mut rng = rand::thread_rng();
        let bytes: Vec<u8> = (0..32).map(|_| rng.r#gen()).collect();
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
        format!("{:x}", hasher.finalize())
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
