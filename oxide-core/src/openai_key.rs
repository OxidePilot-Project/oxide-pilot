use keyring::Entry;
use thiserror::Error;

const OPENAI_AUTH_SERVICE_ID: &str = "oxide_pilot_openai";
const OPENAI_API_KEY_FIELD: &str = "api_key";

#[derive(Error, Debug)]
pub enum OpenAIKeyError {
    #[error("Keyring error: {0}")]
    Keyring(#[from] keyring::Error),
}

pub async fn store_api_key(api_key: &str) -> Result<(), OpenAIKeyError> {
    let entry = Entry::new(OPENAI_AUTH_SERVICE_ID, OPENAI_API_KEY_FIELD)?;
    entry.set_password(api_key)?;
    Ok(())
}

pub async fn get_api_key() -> Result<Option<String>, OpenAIKeyError> {
    // Environment variable takes precedence for non-interactive setups
    if let Ok(value) = std::env::var("OPENAI_API_KEY") {
        if !value.trim().is_empty() {
            return Ok(Some(value));
        }
    }

    let entry = Entry::new(OPENAI_AUTH_SERVICE_ID, OPENAI_API_KEY_FIELD)?;
    match entry.get_password() {
        Ok(v) => Ok(Some(v)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(e) => Err(e.into()),
    }
}

pub async fn clear_api_key() -> Result<(), OpenAIKeyError> {
    let entry = Entry::new(OPENAI_AUTH_SERVICE_ID, OPENAI_API_KEY_FIELD)?;
    match entry.delete_password() {
        Ok(_) | Err(keyring::Error::NoEntry) => Ok(()),
        Err(e) => Err(e.into()),
    }
}
