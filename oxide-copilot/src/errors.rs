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
