use chrono::{DateTime, Duration, Utc};
use keyring::Entry;
use log::{error, info, warn};
use oauth2::basic::BasicClient;
use oauth2::{
    AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken, PkceCodeChallenge,
    PkceCodeVerifier, RedirectUrl, Scope, TokenResponse, TokenUrl,
};
use std::net::SocketAddr;
use thiserror::Error;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use url::Url;

#[derive(Error, Debug)]
pub enum OpenAIAuthError {
    #[error("Keyring error: {0}")]
    Keyring(#[from] keyring::Error),
    #[error("OAuth2 error: {0}")]
    OAuth2(
        #[from]
        oauth2::RequestTokenError<
            oauth2::reqwest::Error<reqwest::Error>,
            oauth2::StandardErrorResponse<oauth2::basic::BasicErrorResponseType>,
        >,
    ),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("URL parse error: {0}")]
    UrlParse(#[from] url::ParseError),
    #[error("HTTP error: {0}")]
    Http(#[from] http::Error),
    #[error("Missing authorization code in redirect URL")]
    MissingAuthCode,
    #[error("Missing state in redirect URL")]
    MissingState,
    #[error("CSRF token mismatch")]
    CsrfMismatch,
    #[error("Failed to bind TCP listener: {0}")]
    TcpBind(std::io::Error),
    #[error("No incoming connection")]
    NoIncomingConnection,
    #[error("No request line from browser")]
    NoRequestLine,
    #[error("Invalid redirect URL in request")]
    InvalidRedirectUrl,
    #[error("Failed to open URL in browser: {0}")]
    BrowserOpen(String),
    #[error("Missing client ID or secret in keyring")]
    MissingClientCredentials,
    #[error("Failed to refresh token: {0}")]
    TokenRefresh(String),
    #[error("Refresh token not found")]
    RefreshTokenNotFound,
}

const OPENAI_AUTH_SERVICE_ID: &str = "oxide_pilot_openai";
const OPENAI_CLIENT_ID_KEY: &str = "client_id";
const OPENAI_CLIENT_SECRET_KEY: &str = "client_secret";
const OPENAI_ACCESS_TOKEN_KEY: &str = "access_token";
const OPENAI_REFRESH_TOKEN_KEY: &str = "refresh_token";
const OPENAI_ACCESS_TOKEN_EXPIRY_KEY: &str = "access_token_expiry";

pub async fn store_client_credentials(
    client_id: &str,
    client_secret: &str,
) -> Result<(), OpenAIAuthError> {
    let client_id_entry = Entry::new(OPENAI_AUTH_SERVICE_ID, OPENAI_CLIENT_ID_KEY)?;
    client_id_entry.set_password(client_id)?;

    let client_secret_entry = Entry::new(OPENAI_AUTH_SERVICE_ID, OPENAI_CLIENT_SECRET_KEY)?;
    client_secret_entry.set_password(client_secret)?;
    Ok(())
}

pub async fn get_client_credentials() -> Result<(String, String), OpenAIAuthError> {
    let client_id_entry = Entry::new(OPENAI_AUTH_SERVICE_ID, OPENAI_CLIENT_ID_KEY)?;
    let client_id = client_id_entry.get_password()?;

    let client_secret_entry = Entry::new(OPENAI_AUTH_SERVICE_ID, OPENAI_CLIENT_SECRET_KEY)?;
    let client_secret = client_secret_entry.get_password()?;

    if client_id.is_empty() || client_secret.is_empty() {
        return Err(OpenAIAuthError::MissingClientCredentials);
    }
    Ok((client_id, client_secret))
}

pub async fn store_tokens(
    access_token: &str,
    refresh_token: Option<&str>,
    expires_in_secs: Option<u64>,
) -> Result<(), OpenAIAuthError> {
    let access_token_entry = Entry::new(OPENAI_AUTH_SERVICE_ID, OPENAI_ACCESS_TOKEN_KEY)?;
    access_token_entry.set_password(access_token)?;

    if let Some(rt) = refresh_token {
        let refresh_token_entry = Entry::new(OPENAI_AUTH_SERVICE_ID, OPENAI_REFRESH_TOKEN_KEY)?;
        refresh_token_entry.set_password(rt)?;
    }

    if let Some(expires_in) = expires_in_secs {
        let expiry_time = Utc::now() + Duration::seconds(expires_in as i64);
        let expiry_entry = Entry::new(OPENAI_AUTH_SERVICE_ID, OPENAI_ACCESS_TOKEN_EXPIRY_KEY)?;
        expiry_entry.set_password(&expiry_time.to_rfc3339())?;
    }
    Ok(())
}

pub async fn get_access_token() -> Result<Option<String>, OpenAIAuthError> {
    let access_token_entry = Entry::new(OPENAI_AUTH_SERVICE_ID, OPENAI_ACCESS_TOKEN_KEY)?;
    let expiry_entry = Entry::new(OPENAI_AUTH_SERVICE_ID, OPENAI_ACCESS_TOKEN_EXPIRY_KEY)?;

    match (
        access_token_entry.get_password(),
        expiry_entry.get_password(),
    ) {
        (Ok(token), Ok(expiry_str)) => {
            let expiry_time = DateTime::parse_from_rfc3339(&expiry_str)
                .map_err(|_e| OpenAIAuthError::Keyring(keyring::Error::NoEntry))?;

            if Utc::now() < expiry_time - Duration::minutes(5) {
                // Refresh 5 minutes before actual expiry
                Ok(Some(token))
            } else {
                info!("Access token expired or near expiry. Attempting to refresh.");
                match refresh_access_token().await {
                    Ok(new_token) => Ok(Some(new_token)),
                    Err(e) => {
                        error!("Failed to refresh token: {e}");
                        Err(e)
                    }
                }
            }
        }
        (Err(keyring::Error::NoEntry), _) | (_, Err(keyring::Error::NoEntry)) => Ok(None),
        (Err(e), _) | (_, Err(e)) => Err(e.into()),
    }
}

pub async fn get_refresh_token() -> Result<Option<String>, OpenAIAuthError> {
    let refresh_token_entry = Entry::new(OPENAI_AUTH_SERVICE_ID, OPENAI_REFRESH_TOKEN_KEY)?;
    match refresh_token_entry.get_password() {
        Ok(token) => Ok(Some(token)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(e) => Err(e.into()),
    }
}

pub async fn clear_auth() -> Result<(), OpenAIAuthError> {
    // Delete OpenAI OAuth tokens from keyring (ignore if missing)
    for key in [
        OPENAI_ACCESS_TOKEN_KEY,
        OPENAI_REFRESH_TOKEN_KEY,
        OPENAI_ACCESS_TOKEN_EXPIRY_KEY,
    ] {
        let entry = Entry::new(OPENAI_AUTH_SERVICE_ID, key)?;
        match entry.delete_password() {
            Ok(_) => {}
            Err(keyring::Error::NoEntry) => {}
            Err(e) => return Err(e.into()),
        }
    }
    Ok(())
}

pub async fn refresh_access_token() -> Result<String, OpenAIAuthError> {
    let (client_id_str, client_secret_str) = get_client_credentials().await?;
    let refresh_token_str = get_refresh_token()
        .await?
        .ok_or(OpenAIAuthError::RefreshTokenNotFound)?;

    let openai_client_id = ClientId::new(client_id_str);
    let openai_client_secret = ClientSecret::new(client_secret_str);
    let token_url = TokenUrl::new("https://auth.openai.com/oauth/token".to_string())
        .expect("Invalid token endpoint URL");

    let client = BasicClient::new(
        openai_client_id,
        Some(openai_client_secret),
        AuthUrl::new("https://auth.openai.com/authorize".to_string()).unwrap(), // AuthUrl not used for refresh
        Some(token_url),
    );

    let token_response = client
        .exchange_refresh_token(&oauth2::RefreshToken::new(refresh_token_str))
        .request_async(oauth2::reqwest::async_http_client)
        .await
        .map_err(|e| OpenAIAuthError::TokenRefresh(e.to_string()))?;

    let new_access_token = token_response.access_token().secret().to_string();
    let new_refresh_token = token_response
        .refresh_token()
        .map(|t| t.secret().to_string());
    let expires_in_secs = token_response.expires_in().map(|d| d.as_secs());

    store_tokens(
        &new_access_token,
        new_refresh_token.as_deref(),
        expires_in_secs,
    )
    .await?;

    info!("Access token refreshed successfully.");
    Ok(new_access_token)
}

pub async fn authenticate_openai() -> Result<String, OpenAIAuthError> {
    let (client_id_str, client_secret_str) = get_client_credentials().await?;

    let openai_client_id = ClientId::new(client_id_str);
    let openai_client_secret = ClientSecret::new(client_secret_str);
    let auth_url = AuthUrl::new("https://auth.openai.com/authorize".to_string())
        .expect("Invalid authorization endpoint URL");
    let token_url = TokenUrl::new("https://auth.openai.com/oauth/token".to_string())
        .expect("Invalid token endpoint URL");

    // Determine redirect listener port: env override -> try 8081 -> random
    let preferred_port = std::env::var("OPENAI_REDIRECT_PORT")
        .ok()
        .and_then(|s| s.parse::<u16>().ok());
    let listener = if let Some(port) = preferred_port {
        TcpListener::bind(("127.0.0.1", port))
            .await
            .map_err(OpenAIAuthError::TcpBind)?
    } else {
        match TcpListener::bind("127.0.0.1:8081").await {
            Ok(l) => l,
            Err(_) => {
                info!("Port 8081 is busy; falling back to a random localhost port for OAuth redirect. If using a Web application client ID, register the chosen redirect URI or switch to a Desktop client ID.");
                TcpListener::bind("127.0.0.1:0")
                    .await
                    .map_err(OpenAIAuthError::TcpBind)?
            }
        }
    };
    let local_addr: SocketAddr = listener
        .local_addr()
        .map_err(|e| OpenAIAuthError::TcpBind(std::io::Error::other(e)))?;
    let redirect_origin = format!("http://127.0.0.1:{}", local_addr.port());
    // Allow overriding the redirect path; default to "/callback-openai" to align with docs
    let redirect_path =
        std::env::var("OPENAI_REDIRECT_PATH").unwrap_or_else(|_| "/callback-openai".to_string());
    let redirect_url_full = format!("{redirect_origin}{redirect_path}");

    let client = BasicClient::new(
        openai_client_id,
        Some(openai_client_secret),
        auth_url,
        Some(token_url),
    )
    .set_redirect_uri(RedirectUrl::new(redirect_url_full.clone()).expect("Invalid redirect URL"));

    let (pkce_code_challenge, pkce_code_verifier) = PkceCodeChallenge::new_random_sha256();

    let (authorize_url, csrf_state) = client
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new("openid".to_string()))
        .add_scope(Scope::new("profile".to_string()))
        .add_scope(Scope::new("email".to_string()))
        .add_scope(Scope::new("offline_access".to_string()))
        .add_extra_param("access_type", "offline")
        .add_extra_param("prompt", "consent")
        .set_pkce_challenge(pkce_code_challenge)
        .url();

    // Support headless/no-browser mode for CI or restricted environments
    let no_browser = std::env::var("OPENAI_OAUTH_NO_BROWSER")
        .map(|v| matches!(v.as_str(), "1" | "true" | "TRUE" | "yes" | "YES"))
        .unwrap_or(false);
    if no_browser {
        info!(
            "NO-BROWSER mode: Please open the following URL manually to complete OpenAI authentication: {authorize_url}"
        );
    } else {
        info!("Opening browser for OpenAI authentication at {authorize_url}");
        if let Err(e) = webbrowser::open(authorize_url.as_str()) {
            return Err(OpenAIAuthError::BrowserOpen(e.to_string()));
        }
    }

    // Wait for the redirect
    let (stream, _) = listener
        .accept()
        .await
        .map_err(|_e| OpenAIAuthError::NoIncomingConnection)?;

    let access_token = handle_redirect(
        stream,
        csrf_state,
        client,
        pkce_code_verifier,
        &redirect_origin,
    )
    .await?;

    info!("OpenAI authentication successful!");
    Ok(access_token)
}

async fn handle_redirect(
    mut stream: TcpStream,
    csrf_state: CsrfToken,
    client: BasicClient,
    pkce_code_verifier: PkceCodeVerifier,
    redirect_origin: &str,
) -> Result<String, OpenAIAuthError> {
    let mut reader = tokio::io::BufReader::new(&mut stream);
    let mut request_line = String::new();
    reader.read_line(&mut request_line).await?;

    let redirect_url_str = request_line
        .split_whitespace()
        .nth(1)
        .ok_or(OpenAIAuthError::InvalidRedirectUrl)?;
    let url = Url::parse(&format!("{redirect_origin}{redirect_url_str}"))?;

    let code = url
        .query_pairs()
        .find_map(|(key, value)| if key == "code" { Some(value) } else { None })
        .ok_or(OpenAIAuthError::MissingAuthCode)?;
    let state = url
        .query_pairs()
        .find_map(|(key, value)| if key == "state" { Some(value) } else { None })
        .ok_or(OpenAIAuthError::MissingState)?;

    if state.as_ref() != csrf_state.secret() {
        return Err(OpenAIAuthError::CsrfMismatch);
    }

    let token_response = client
        .exchange_code(AuthorizationCode::new(code.to_string()))
        .set_pkce_verifier(pkce_code_verifier)
        .request_async(oauth2::reqwest::async_http_client)
        .await?;

    let access_token = token_response.access_token().secret().to_string();
    let refresh_token = token_response
        .refresh_token()
        .map(|t| t.secret().to_string());
    let expires_in_secs = token_response.expires_in().map(|d| d.as_secs());

    store_tokens(&access_token, refresh_token.as_deref(), expires_in_secs).await?;

    let response = "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\n\r\n<html><body><h1>Authentication successful! You can close this tab.</h1></body></html>";
    stream.write_all(response.as_bytes()).await?;

    Ok(access_token)
}

pub async fn get_auth_status() -> Result<String, OpenAIAuthError> {
    match get_access_token().await {
        Ok(Some(_)) => {
            // Check if token is still valid
            match get_refresh_token().await {
                Ok(Some(_)) => Ok("OAuth Token".to_string()),
                Ok(None) => Ok("OAuth Token (No Refresh)".to_string()),
                Err(_) => Ok("OAuth Token (Error checking refresh)".to_string()),
            }
        }
        Ok(None) => Ok("Not authenticated".to_string()),
        Err(e) => {
            warn!("Error checking OpenAI auth status: {e}");
            Ok("Auth Error".to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // These tests interact with the configured credential store (OS keyring).
    // They are ignored by default to avoid mutating developer machines unintentionally.
    // Run explicitly with: cargo test -p oxide-core --openai-auth -- --ignored

    #[ignore]
    #[tokio::test]
    async fn store_and_load_access_token_future_expiry() {
        // Store a token that expires in 1 hour
        store_tokens("TEST_ACCESS", Some("R"), Some(3600))
            .await
            .expect("store ok");
        let token = get_access_token().await.expect("get ok");
        assert_eq!(token, Some("TEST_ACCESS".to_string()));
    }

    #[ignore]
    #[tokio::test]
    async fn status_is_not_authenticated_when_cleared() {
        // Best-effort: try to clear by storing an already expired token without refresh
        let past = 1u64; // seconds, will be interpreted relative to now inside store_tokens
        store_tokens("EXPIRED", None, Some(past))
            .await
            .expect("store ok");
        let status = get_auth_status()
            .await
            .unwrap_or_else(|e| format!("Auth Error: {e}"));
        assert!(status.to_lowercase().contains("not") || status.to_lowercase().contains("error"));
    }
}
