use oauth2::{
    AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken, PkceCodeChallenge,
    RedirectUrl, Scope, TokenResponse, TokenUrl,
};
use oauth2::basic::BasicClient;
use oauth2::reqwest::async_http_client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use tiny_http::{Server, Response, Header};
use url::Url;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoogleOAuthConfig {
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uri: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthToken {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_in: Option<u64>,
    pub token_type: String,
}

pub struct GoogleOAuthManager {
    #[allow(dead_code)]
    config: GoogleOAuthConfig,
    client: BasicClient,
}

impl GoogleOAuthManager {
    pub fn new(config: GoogleOAuthConfig) -> Result<Self, Box<dyn std::error::Error>> {
        let client = BasicClient::new(
            ClientId::new(config.client_id.clone()),
            Some(ClientSecret::new(config.client_secret.clone())),
            AuthUrl::new("https://accounts.google.com/o/oauth2/v2/auth".to_string())?,
            Some(TokenUrl::new("https://oauth2.googleapis.com/token".to_string())?),
        )
        .set_redirect_uri(RedirectUrl::new(config.redirect_uri.clone())?);

        Ok(Self { config, client })
    }

    pub async fn authenticate(&self) -> Result<OAuthToken, Box<dyn std::error::Error>> {
        // Generate PKCE challenge
        let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

        // Generate authorization URL
        let (auth_url, _csrf_token) = self
            .client
            .authorize_url(CsrfToken::new_random)
            .add_scope(Scope::new("https://www.googleapis.com/auth/generative-language".to_string()))
            .add_scope(Scope::new("openid".to_string()))
            .add_scope(Scope::new("email".to_string()))
            .add_scope(Scope::new("profile".to_string()))
            .set_pkce_challenge(pkce_challenge)
            .url();

        println!("🔐 Oxide Pilot - Google Authentication");
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("📱 Opening browser for Google authentication...");
        println!("🌐 If browser doesn't open automatically, visit:");
        println!("   {auth_url}");
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

        // Open browser
        if let Err(e) = webbrowser::open(auth_url.as_str()) {
            eprintln!("⚠️  Failed to open browser: {e}");
            println!("Please manually open the URL above in your browser.");
        }

        // Start local server to receive callback
        let (tx, rx) = mpsc::channel();
        let server_port = 8080;
        let server_url = format!("127.0.0.1:{server_port}");

        thread::spawn(move || {
            let server = match Server::http(&server_url) {
                Ok(server) => server,
                Err(e) => {
                    eprintln!("Failed to start local server: {e}");
                    return;
                }
            };

            println!("🔄 Waiting for authentication callback...");

            for request in server.incoming_requests() {
                let url = format!("http://localhost{}", request.url());
                let parsed_url = match Url::parse(&url) {
                    Ok(url) => url,
                    Err(_) => continue,
                };

                let query_params: HashMap<String, String> = parsed_url
                    .query_pairs()
                    .into_owned()
                    .collect();

                if let Some(code) = query_params.get("code") {
                    let success_html = r#"
                        <!DOCTYPE html>
                        <html>
                        <head>
                            <title>Oxide Pilot - Authentication Success</title>
                            <style>
                                body { font-family: Arial, sans-serif; text-align: center; padding: 50px; background: #f0f0f0; }
                                .container { background: white; padding: 40px; border-radius: 10px; box-shadow: 0 4px 6px rgba(0,0,0,0.1); max-width: 500px; margin: 0 auto; }
                                .success { color: #28a745; font-size: 24px; margin-bottom: 20px; }
                                .message { color: #333; font-size: 16px; }
                            </style>
                        </head>
                        <body>
                            <div class="container">
                                <div class="success">✅ Authentication Successful!</div>
                                <div class="message">
                                    You have successfully authenticated with Google.<br>
                                    You can now close this window and return to Oxide Pilot.
                                </div>
                            </div>
                        </body>
                        </html>
                    "#;

                    let response = Response::from_string(success_html)
                        .with_header(Header::from_bytes(&b"Content-Type"[..], &b"text/html"[..]).unwrap());

                    let _ = request.respond(response);
                    let _ = tx.send(code.clone());
                    break;
                }

                if let Some(error) = query_params.get("error") {
                    let error_html = format!(r#"
                        <!DOCTYPE html>
                        <html>
                        <head>
                            <title>Oxide Pilot - Authentication Error</title>
                            <style>
                                body {{ font-family: Arial, sans-serif; text-align: center; padding: 50px; background: #f0f0f0; }}
                                .container {{ background: white; padding: 40px; border-radius: 10px; box-shadow: 0 4px 6px rgba(0,0,0,0.1); max-width: 500px; margin: 0 auto; }}
                                .error {{ color: #dc3545; font-size: 24px; margin-bottom: 20px; }}
                                .message {{ color: #333; font-size: 16px; }}
                            </style>
                        </head>
                        <body>
                            <div class="container">
                                <div class="error">❌ Authentication Failed</div>
                                <div class="message">
                                    Error: {error}<br>
                                    Please try again or contact support.
                                </div>
                            </div>
                        </body>
                        </html>
                    "#);

                    let response = Response::from_string(error_html)
                        .with_header(Header::from_bytes(&b"Content-Type"[..], &b"text/html"[..]).unwrap());

                    let _ = request.respond(response);
                    let _ = tx.send(format!("ERROR:{error}"));
                    break;
                }
            }
        });

        // Wait for callback with timeout
        let auth_code = match rx.recv_timeout(Duration::from_secs(300)) {
            Ok(code) => {
                if code.starts_with("ERROR:") {
                    return Err(format!("Authentication failed: {}", &code[6..]).into());
                }
                code
            }
            Err(_) => {
                return Err("Authentication timeout. Please try again.".into());
            }
        };

        println!("✅ Authentication code received, exchanging for token...");

        // Exchange authorization code for token
        let token_result = self
            .client
            .exchange_code(AuthorizationCode::new(auth_code))
            .set_pkce_verifier(pkce_verifier)
            .request_async(async_http_client)
            .await?;

        let oauth_token = OAuthToken {
            access_token: token_result.access_token().secret().clone(),
            refresh_token: token_result.refresh_token().map(|t| t.secret().clone()),
            expires_in: token_result.expires_in().map(|d| d.as_secs()),
            token_type: "Bearer".to_string(),
        };

        println!("🎉 Successfully authenticated with Google!");
        Ok(oauth_token)
    }

    pub async fn refresh_token(&self, refresh_token: &str) -> Result<OAuthToken, Box<dyn std::error::Error>> {
        let token_result = self
            .client
            .exchange_refresh_token(&oauth2::RefreshToken::new(refresh_token.to_string()))
            .request_async(async_http_client)
            .await?;

        let oauth_token = OAuthToken {
            access_token: token_result.access_token().secret().clone(),
            refresh_token: token_result.refresh_token().map(|t| t.secret().clone()),
            expires_in: token_result.expires_in().map(|d| d.as_secs()),
            token_type: "Bearer".to_string(),
        };

        Ok(oauth_token)
    }
}

// Default Google OAuth configuration for Gemini API
impl Default for GoogleOAuthConfig {
    fn default() -> Self {
        // Try to get OAuth credentials from environment variables
        let client_id = std::env::var("GOOGLE_OAUTH_CLIENT_ID")
            .unwrap_or_else(|_| "your-client-id.apps.googleusercontent.com".to_string());
        let client_secret = std::env::var("GOOGLE_OAUTH_CLIENT_SECRET")
            .unwrap_or_else(|_| "your-client-secret".to_string());
        let redirect_uri = std::env::var("GOOGLE_OAUTH_REDIRECT_URI")
            .unwrap_or_else(|_| "http://localhost:8080/callback".to_string());

        Self {
            client_id,
            client_secret,
            redirect_uri,
        }
    }
}
