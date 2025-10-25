use reqwest::Client;
use serde::{Deserialize, Serialize};
use log::{error, info};
use thiserror::Error;
use crate::{openai_auth, openai_key};

#[derive(Error, Debug)]
pub enum OpenAIClientError {
    #[error("Authentication error: {0}")]
    Auth(String),
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("API error: {0}")]
    Api(String),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("No response from API")]
    NoResponse,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Serialize, Debug)]
struct ChatCompletionRequest {
    model: String,
    messages: Vec<ChatMessage>,
    temperature: Option<f32>,
    max_tokens: Option<u32>,
}

#[derive(Deserialize, Debug)]
struct ChatCompletionResponse {
    choices: Vec<Choice>,
}

#[derive(Deserialize, Debug)]
struct Choice {
    message: ChatMessage,
}

/// Send a chat completion request to OpenAI API using OAuth bearer token
pub async fn chat_completion(
    model: &str,
    messages: Vec<ChatMessage>,
    temperature: Option<f32>,
    max_tokens: Option<u32>,
) -> Result<String, OpenAIClientError> {
    // Resolve credential: prefer API key (env or keyring), fallback to OAuth access token
    let bearer = match openai_key::get_api_key().await {
        Ok(Some(api_key)) => {
            info!("Using OpenAI API Key from env/keyring");
            api_key
        },
        Ok(None) => {
            info!("OpenAI API Key not found; falling back to OAuth access token");
            openai_auth::get_access_token()
                .await
                .map_err(|e| OpenAIClientError::Auth(e.to_string()))?
                .ok_or_else(|| OpenAIClientError::Auth("No OpenAI API key or OAuth token configured".to_string()))?
        },
        Err(e) => {
            error!("Failed to read OpenAI API key: {e}");
            openai_auth::get_access_token()
                .await
                .map_err(|e| OpenAIClientError::Auth(e.to_string()))?
                .ok_or_else(|| OpenAIClientError::Auth("No OpenAI API key or OAuth token configured".to_string()))?
        }
    };

    // Default base URL (can be overridden via env for enterprise tenants)
    let base_url = std::env::var("OPENAI_API_BASE")
        .unwrap_or_else(|_| "https://api.openai.com/v1".to_string());
    let url = format!("{base_url}/chat/completions");

    let client = Client::new();
    let request_body = ChatCompletionRequest {
        model: model.to_string(),
        messages,
        temperature,
        max_tokens,
    };

    let response = client
        .post(&url)
        .bearer_auth(&bearer)
        .json(&request_body)
        .send()
        .await?;

    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_default();
        error!("OpenAI API request failed: {status} - {error_text}");
        return Err(OpenAIClientError::Api(format!("{status} - {error_text}")));
    }

    let completion_response: ChatCompletionResponse = response.json().await?;

    if let Some(choice) = completion_response.choices.first() {
        Ok(choice.message.content.clone())
    } else {
        Err(OpenAIClientError::NoResponse)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_sample_chat_response() {
        let sample = r#"{
          "choices": [
            { "message": { "role": "assistant", "content": "{\\"risk_score\\": 50, \\"confidence\\\": 0.8}" } }
          ]
        }"#;

        let parsed: ChatCompletionResponse = serde_json::from_str(sample).expect("should parse");
        assert_eq!(parsed.choices.len(), 1);
        let content = parsed.choices[0].message.content.clone();
        assert!(content.contains("risk_score"));
    }

    #[test]
    fn parse_empty_choices_is_noresponse() {
        let sample = r#"{ "choices": [] }"#;
        let parsed = serde_json::from_str::<ChatCompletionResponse>(sample).expect("should parse");
        assert!(parsed.choices.is_empty());
    }

    #[test]
    fn invalid_json_is_serialization_error() {
        let bad = "{ not-json }";
        let err = serde_json::from_str::<ChatCompletionResponse>(bad).expect_err("should error");
        // Ensure we get a serde error kind
        let _ = err;
    }
}
