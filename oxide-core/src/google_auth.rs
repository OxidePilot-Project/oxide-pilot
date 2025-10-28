use chrono::{DateTime, Duration, Utc};
use keyring::Entry;
use log::{error, info};
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
pub enum AuthError {
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

const GOOGLE_AUTH_SERVICE_ID: &str = "oxide_pilot_google_auth";
const GOOGLE_CLIENT_ID_KEY: &str = "client_id";
const GOOGLE_CLIENT_SECRET_KEY: &str = "client_secret";
const GOOGLE_ACCESS_TOKEN_KEY: &str = "access_token";
const GOOGLE_REFRESH_TOKEN_KEY: &str = "refresh_token";
const GOOGLE_ACCESS_TOKEN_EXPIRY_KEY: &str = "access_token_expiry";

pub async fn store_client_credentials(
    client_id: &str,
    client_secret: &str,
) -> Result<(), AuthError> {
    let client_id_entry = Entry::new(GOOGLE_AUTH_SERVICE_ID, GOOGLE_CLIENT_ID_KEY)?;
    client_id_entry.set_password(client_id)?;

    let client_secret_entry = Entry::new(GOOGLE_AUTH_SERVICE_ID, GOOGLE_CLIENT_SECRET_KEY)?;
    client_secret_entry.set_password(client_secret)?;
    Ok(())
}

pub async fn get_client_credentials() -> Result<(String, String), AuthError> {
    let client_id_entry = Entry::new(GOOGLE_AUTH_SERVICE_ID, GOOGLE_CLIENT_ID_KEY)?;
    let client_id = client_id_entry.get_password()?;

    let client_secret_entry = Entry::new(GOOGLE_AUTH_SERVICE_ID, GOOGLE_CLIENT_SECRET_KEY)?;
    let client_secret = client_secret_entry.get_password()?;

    if client_id.is_empty() || client_secret.is_empty() {
        return Err(AuthError::MissingClientCredentials);
    }
    Ok((client_id, client_secret))
}

pub async fn store_tokens(
    access_token: &str,
    refresh_token: Option<&str>,
    expires_in_secs: Option<u64>,
) -> Result<(), AuthError> {
    let access_token_entry = Entry::new(GOOGLE_AUTH_SERVICE_ID, GOOGLE_ACCESS_TOKEN_KEY)?;
    access_token_entry.set_password(access_token)?;

    if let Some(rt) = refresh_token {
        let refresh_token_entry = Entry::new(GOOGLE_AUTH_SERVICE_ID, GOOGLE_REFRESH_TOKEN_KEY)?;
        refresh_token_entry.set_password(rt)?;
    }

    if let Some(expires_in) = expires_in_secs {
        let expiry_time = Utc::now() + Duration::seconds(expires_in as i64);
        let expiry_entry = Entry::new(GOOGLE_AUTH_SERVICE_ID, GOOGLE_ACCESS_TOKEN_EXPIRY_KEY)?;
        expiry_entry.set_password(&expiry_time.to_rfc3339())?;
    }
    Ok(())
}

pub async fn get_access_token() -> Result<Option<String>, AuthError> {
    let access_token_entry = Entry::new(GOOGLE_AUTH_SERVICE_ID, GOOGLE_ACCESS_TOKEN_KEY)?;
    let expiry_entry = Entry::new(GOOGLE_AUTH_SERVICE_ID, GOOGLE_ACCESS_TOKEN_EXPIRY_KEY)?;

    match (
        access_token_entry.get_password(),
        expiry_entry.get_password(),
    ) {
        (Ok(token), Ok(expiry_str)) => {
            let expiry_time = DateTime::parse_from_rfc3339(&expiry_str)
                .map_err(|_e| AuthError::Keyring(keyring::Error::NoEntry))?;

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

pub async fn get_refresh_token() -> Result<Option<String>, AuthError> {
    let refresh_token_entry = Entry::new(GOOGLE_AUTH_SERVICE_ID, GOOGLE_REFRESH_TOKEN_KEY)?;
    match refresh_token_entry.get_password() {
        Ok(token) => Ok(Some(token)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(e) => Err(e.into()),
    }
}

pub async fn clear_auth() -> Result<(), AuthError> {
    // Delete Google OAuth tokens from keyring (ignore if missing)
    for key in [
        GOOGLE_ACCESS_TOKEN_KEY,
        GOOGLE_REFRESH_TOKEN_KEY,
        GOOGLE_ACCESS_TOKEN_EXPIRY_KEY,
    ] {
        let entry = Entry::new(GOOGLE_AUTH_SERVICE_ID, key)?;
        match entry.delete_password() {
            Ok(_) => {}
            Err(keyring::Error::NoEntry) => {}
            Err(e) => return Err(e.into()),
        }
    }
    Ok(())
}

pub async fn refresh_access_token() -> Result<String, AuthError> {
    let (client_id_str, client_secret_str) = get_client_credentials().await?;
    let refresh_token_str = get_refresh_token()
        .await?
        .ok_or(AuthError::RefreshTokenNotFound)?;

    let google_client_id = ClientId::new(client_id_str);
    let google_client_secret = ClientSecret::new(client_secret_str);
    let token_url = TokenUrl::new("https://oauth2.googleapis.com/token".to_string())
        .expect("Invalid token endpoint URL");

    let client = BasicClient::new(
        google_client_id,
        Some(google_client_secret),
        AuthUrl::new("https://accounts.google.com/o/oauth2/v2/auth".to_string()).unwrap(), // AuthUrl not used for refresh
        Some(token_url),
    );

    let token_response = client
        .exchange_refresh_token(&oauth2::RefreshToken::new(refresh_token_str))
        .request_async(oauth2::reqwest::async_http_client)
        .await
        .map_err(|e| AuthError::TokenRefresh(e.to_string()))?;

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

pub async fn authenticate_google() -> Result<String, AuthError> {
    let (client_id_str, client_secret_str) = get_client_credentials().await?;

    let google_client_id = ClientId::new(client_id_str);
    let google_client_secret = ClientSecret::new(client_secret_str);
    let auth_url = AuthUrl::new("https://accounts.google.com/o/oauth2/v2/auth".to_string())
        .expect("Invalid authorization endpoint URL");
    let token_url = TokenUrl::new("https://oauth2.googleapis.com/token".to_string())
        .expect("Invalid token endpoint URL");

    // Determine redirect listener port: env override -> try 8080 -> random
    let preferred_port = std::env::var("GOOGLE_REDIRECT_PORT")
        .ok()
        .and_then(|s| s.parse::<u16>().ok());
    let listener = if let Some(port) = preferred_port {
        TcpListener::bind(("127.0.0.1", port))
            .await
            .map_err(AuthError::TcpBind)?
    } else {
        match TcpListener::bind("127.0.0.1:8080").await {
            Ok(l) => l,
            Err(_) => {
                info!("Port 8080 is busy; falling back to a random localhost port for OAuth redirect. If using a Web application client ID, register the chosen redirect URI or switch to a Desktop client ID.");
                TcpListener::bind("127.0.0.1:0")
                    .await
                    .map_err(AuthError::TcpBind)?
            }
        }
    };
    let local_addr: SocketAddr = listener
        .local_addr()
        .map_err(|e| AuthError::TcpBind(std::io::Error::other(e)))?;
    let redirect_origin = format!("http://127.0.0.1:{}", local_addr.port());
    // Allow overriding the redirect path; default to "/callback" to align with docs
    let redirect_path =
        std::env::var("GOOGLE_REDIRECT_PATH").unwrap_or_else(|_| "/callback".to_string());
    let redirect_url_full = format!("{redirect_origin}{redirect_path}");

    let client = BasicClient::new(
        google_client_id,
        Some(google_client_secret),
        auth_url,
        Some(token_url),
    )
    .set_redirect_uri(RedirectUrl::new(redirect_url_full.clone()).expect("Invalid redirect URL"));

    let (pkce_code_challenge, pkce_code_verifier) = PkceCodeChallenge::new_random_sha256();

    let (authorize_url, csrf_state) = client
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new(
            "https://www.googleapis.com/auth/userinfo.email".to_string(),
        ))
        .add_scope(Scope::new(
            "https://www.googleapis.com/auth/userinfo.profile".to_string(),
        ))
        .add_scope(Scope::new(
            "https://www.googleapis.com/auth/drive.file".to_string(),
        ))
        .add_extra_param("access_type", "offline")
        .add_extra_param("prompt", "consent")
        .set_pkce_challenge(pkce_code_challenge)
        .url();

    // Support headless/no-browser mode for CI or restricted environments
    let no_browser = std::env::var("GOOGLE_OAUTH_NO_BROWSER")
        .map(|v| matches!(v.as_str(), "1" | "true" | "TRUE" | "yes" | "YES"))
        .unwrap_or(false);
    if no_browser {
        info!(
            "NO-BROWSER mode: Please open the following URL manually to complete Google authentication: {authorize_url}"
        );
    } else {
        info!("Opening browser for Google authentication at {authorize_url}");
        if let Err(e) = webbrowser::open(authorize_url.as_str()) {
            return Err(AuthError::BrowserOpen(e.to_string()));
        }
    }

    // Wait for the redirect
    let (stream, _) = listener
        .accept()
        .await
        .map_err(|_e| AuthError::NoIncomingConnection)?;

    let access_token = handle_redirect(
        stream,
        csrf_state,
        client,
        pkce_code_verifier,
        &redirect_origin,
    )
    .await?;

    info!("Google authentication successful!");
    Ok(access_token)
}

async fn handle_redirect(
    mut stream: TcpStream,
    csrf_state: CsrfToken,
    client: BasicClient,
    pkce_code_verifier: PkceCodeVerifier,
    redirect_origin: &str,
) -> Result<String, AuthError> {
    let mut reader = tokio::io::BufReader::new(&mut stream);
    let mut request_line = String::new();
    reader.read_line(&mut request_line).await?;

    let redirect_url_str = request_line
        .split_whitespace()
        .nth(1)
        .ok_or(AuthError::InvalidRedirectUrl)?;
    let url = Url::parse(&format!("{redirect_origin}{redirect_url_str}"))?;

    let code = url
        .query_pairs()
        .find_map(|(key, value)| if key == "code" { Some(value) } else { None })
        .ok_or(AuthError::MissingAuthCode)?;
    let state = url
        .query_pairs()
        .find_map(|(key, value)| if key == "state" { Some(value) } else { None })
        .ok_or(AuthError::MissingState)?;

    if state.as_ref() != csrf_state.secret() {
        return Err(AuthError::CsrfMismatch);
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
