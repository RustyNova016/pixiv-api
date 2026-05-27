/// Helper to obtain a Pixiv refresh token via OAuth2 PKCE flow.
///
/// Run: cargo run -p pixiv-api --example get_token
///
/// Steps:
/// 1. This script prints a URL — open it in your browser
/// 2. Log in to Pixiv and authorize the app
/// 3. After login, open F12 dev tools → Network tab (check "Preserve log")
/// 4. Look for a request to `callback?code=...` or a `pixiv://` URL
/// 5. Copy the full URL and paste it here when prompted
/// 6. The script prints your refresh token
use sha2::Digest;

const LOGIN_URL: &str = "https://app-api.pixiv.net/web/v1/login";
const AUTH_TOKEN_URL: &str = "https://oauth.secure.pixiv.net/auth/token";
const CLIENT_ID: &str = "MOBrBDS8blbauoSck0ZfDbtuzpyT";
const CLIENT_SECRET: &str = "lsACyCD94FhDUtGTXi3QzcFE2uU1hqtDaKeqrdwj";
const HASH_SECRET: &str = "28c1fdd170a5204386cb1313c7077b34f83e4aaf4aa829ce78c231e05b0bae2c";
const REDIRECT_URI: &str = "https://app-api.pixiv.net/web/v1/users/auth/pixiv/callback";

#[tokio::main]
async fn main() {
    // Generate PKCE challenge
    let code_verifier = generate_code_verifier();
    let code_challenge = {
        let mut hasher = sha2::Sha256::new();
        hasher.update(code_verifier.as_bytes());
        base64_encode(&hasher.finalize())
    };

    let login_params = format!(
        "code_challenge={}&code_challenge_method=S256&client=pixiv-android",
        code_challenge
    );
    let url = format!("{}?{}", LOGIN_URL, login_params);

    println!("=== Pixiv Refresh Token Helper ===\n");
    println!("1. Open this URL in your browser:\n");
    println!("   {}\n", url);
    println!("2. Log in to Pixiv and authorize the app");
    println!("3. After login, open F12 dev tools -> Network tab (check \"Preserve log\")");
    println!("4. Look for a request containing \"callback?code=\" or a pixiv:// URL");
    println!("   It will look like one of these:");
    println!("     https://app-api.pixiv.net/.../callback?state=...&code=XXXXX");
    println!("     pixiv://account/login?code=XXXXX");
    println!("5. Copy the full URL and paste it below (empty lines are ignored)\n");

    // Read the redirect URL from user input, skipping empty lines
    let redirect_url = loop {
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();
        let trimmed = input.trim();
        if !trimmed.is_empty() {
            break trimmed.to_string();
        }
    };

    // Extract the code from the redirect URL
    let code = extract_code(&redirect_url).expect(
        "Could not extract 'code' from the URL. Make sure you copied the full callback URL",
    );

    println!("\nExchanging code for tokens...");

    // Get auth headers
    let now = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S%z").to_string();
    let hash = {
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
        .await
        .unwrap();

    if !resp.status().is_success() {
        eprintln!("Error: HTTP {}", resp.status());
        eprintln!("Response: {}", resp.text().await.unwrap());
        std::process::exit(1);
    }

    let json: serde_json::Value = resp.json().await.unwrap();

    if let Some(refresh_token) = json["refresh_token"].as_str() {
        println!("\n=== Your Refresh Token ===\n");
        println!("{}", refresh_token);
        println!("\nSave this! Use it with:");
        println!("  export PIXIV_REFRESH_TOKEN={}", refresh_token);
        println!("  pixiv-dl search \"landscape\"");
    } else {
        eprintln!("Failed to get refresh token. Response:");
        eprintln!("{}", serde_json::to_string_pretty(&json).unwrap());
    }
}

fn generate_code_verifier() -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let bytes: Vec<u8> = (0..32).map(|_| rng.r#gen()).collect();
    base64_encode(&bytes)
}

fn base64_encode(data: &[u8]) -> String {
    use base64::Engine;
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(data)
}

fn extract_code(url: &str) -> Option<String> {
    // Handle callback URL: https://app-api.pixiv.net/.../callback?code=XXX
    // Also handle pixiv://...?code=XXX (older flow)
    let query_start = url.find('?')?;
    let query = &url[query_start + 1..];
    for pair in query.split('&') {
        let mut parts = pair.splitn(2, '=');
        if let (Some(key), Some(value)) = (parts.next(), parts.next()) {
            if key == "code" {
                return Some(value.to_string());
            }
        }
    }
    None
}
