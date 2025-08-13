use thiserror::Error;

#[derive(Error, Debug)]
pub enum CopilotError {
    #[error("AI Provider error: {0}")]
    AIProvider(String),
    #[error("Function execution error: {0}")]
    FunctionExecution(String),
    #[error("Screen capture error: {0}")]
    ScreenCapture(String),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("Authentication error: {0}")]
    Authentication(String),
    #[error("Conversation loop exceeded max turns")]
    MaxTurnsExceeded,
    #[error("Missing function name in AI response")]
    MissingFunctionName,
    #[error("Invalid function call JSON: {0}")]
    InvalidFunctionCallJson(String),
    #[error("No text or function call in AI response")]
    NoAIResponseContent,
    #[error("No candidates in AI response")]
    NoAICandidates,
    #[error("API request failed: {0}")]
    APIRequest(String),
    #[error("Failed to parse API response: {0}")]
    APIResponseParse(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_copilot_error_display() {
        let error = CopilotError::AIProvider("Test error".to_string());
        assert!(error.to_string().contains("AI Provider error"));
    }

    #[test]
    fn test_copilot_error_from_serde() {
        let json_error = serde_json::from_str::<serde_json::Value>("invalid json");
        assert!(json_error.is_err());

        if let Err(e) = json_error {
            let copilot_error: CopilotError = e.into();
            assert!(matches!(copilot_error, CopilotError::Serialization(_)));
        }
    }
}
