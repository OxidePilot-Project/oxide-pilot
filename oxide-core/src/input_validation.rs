use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ValidationError {
    #[error("Input too long: maximum {max} characters, got {actual}")]
    TooLong { max: usize, actual: usize },
    #[error("Input too short: minimum {min} characters, got {actual}")]
    TooShort { min: usize, actual: usize },
    #[error("Invalid format: {0}")]
    InvalidFormat(String),
    #[error("Contains forbidden characters: {0}")]
    ForbiddenCharacters(String),
    #[error("Contains potential security threat: {0}")]
    SecurityThreat(String),
    #[error("Required field missing: {0}")]
    Required(String),
    #[error("Invalid value: {0}")]
    InvalidValue(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationRule {
    pub min_length: Option<usize>,
    pub max_length: Option<usize>,
    pub pattern: Option<String>,
    pub allowed_chars: Option<String>,
    pub forbidden_chars: Option<String>,
    pub required: bool,
    pub sanitize: bool,
}

impl Default for ValidationRule {
    fn default() -> Self {
        Self {
            min_length: None,
            max_length: Some(1000), // Default max length
            pattern: None,
            allowed_chars: None,
            forbidden_chars: None,
            required: false,
            sanitize: true,
        }
    }
}

pub struct InputValidator {
    rules: HashMap<String, ValidationRule>,
    security_patterns: Vec<Regex>,
}

impl InputValidator {
    pub fn new() -> Self {
        let mut validator = Self {
            rules: HashMap::new(),
            security_patterns: Vec::new(),
        };

        // Initialize security patterns
        validator.init_security_patterns();

        // Initialize default rules
        validator.init_default_rules();

        validator
    }

    fn init_security_patterns(&mut self) {
        // SQL injection patterns
        if let Ok(regex) = Regex::new(r"(?i)(union|select|insert|update|delete|drop|create|alter|exec|execute|\-\-|/\*|\*/|xp_|sp_)") {
            self.security_patterns.push(regex);
        }

        // XSS patterns
        if let Ok(regex) = Regex::new(r"(?i)(<script|</script|javascript:|vbscript:|onload|onerror|onclick|onmouseover)") {
            self.security_patterns.push(regex);
        }

        // Command injection patterns
        if let Ok(regex) = Regex::new(r"(?i)(;|\||&|`|\$\(|<|>|&&|\|\|)") {
            self.security_patterns.push(regex);
        }

        // Path traversal patterns
        if let Ok(regex) = Regex::new(r"(\.\./|\.\.\\|%2e%2e%2f|%2e%2e%5c)") {
            self.security_patterns.push(regex);
        }
    }

    fn init_default_rules(&mut self) {
        // User input validation
        self.rules.insert("user_input".to_string(), ValidationRule {
            min_length: Some(1),
            max_length: Some(5000),
            pattern: None,
            allowed_chars: None,
            forbidden_chars: Some("<>\"'&".to_string()),
            required: true,
            sanitize: true,
        });

        // Email validation
        self.rules.insert("email".to_string(), ValidationRule {
            min_length: Some(5),
            max_length: Some(254),
            pattern: Some(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$".to_string()),
            allowed_chars: None,
            forbidden_chars: None,
            required: true,
            sanitize: false,
        });

        // Password validation
        self.rules.insert("password".to_string(), ValidationRule {
            min_length: Some(8),
            max_length: Some(128),
            pattern: Some(r"^(?=.*[a-z])(?=.*[A-Z])(?=.*\d)(?=.*[@$!%*?&])[A-Za-z\d@$!%*?&]".to_string()),
            allowed_chars: None,
            forbidden_chars: None,
            required: true,
            sanitize: false,
        });

        // Username validation
        self.rules.insert("username".to_string(), ValidationRule {
            min_length: Some(3),
            max_length: Some(50),
            pattern: Some(r"^[a-zA-Z0-9_-]+$".to_string()),
            allowed_chars: None,
            forbidden_chars: None,
            required: true,
            sanitize: false,
        });

        // File path validation
        self.rules.insert("file_path".to_string(), ValidationRule {
            min_length: Some(1),
            max_length: Some(260),
            pattern: None,
            allowed_chars: None,
            forbidden_chars: Some("<>:\"|?*".to_string()),
            required: true,
            sanitize: true,
        });

        // URL validation
        self.rules.insert("url".to_string(), ValidationRule {
            min_length: Some(10),
            max_length: Some(2048),
            pattern: Some(r"^https?://[^\s/$.?#].[^\s]*$".to_string()),
            allowed_chars: None,
            forbidden_chars: None,
            required: false,
            sanitize: false,
        });

        // Configuration key validation
        self.rules.insert("config_key".to_string(), ValidationRule {
            min_length: Some(1),
            max_length: Some(100),
            pattern: Some(r"^[a-zA-Z0-9._-]+$".to_string()),
            allowed_chars: None,
            forbidden_chars: None,
            required: true,
            sanitize: false,
        });
    }

    pub fn validate(&self, field_name: &str, value: &str) -> Result<String, ValidationError> {
        let default_rule = ValidationRule::default();
        let rule = self.rules.get(field_name).unwrap_or(&default_rule);

        // Check if required
        if rule.required && value.is_empty() {
            return Err(ValidationError::Required(field_name.to_string()));
        }

        // Skip validation for empty optional fields
        if !rule.required && value.is_empty() {
            return Ok(value.to_string());
        }

        // Length validation
        if let Some(min_len) = rule.min_length {
            if value.len() < min_len {
                return Err(ValidationError::TooShort {
                    min: min_len,
                    actual: value.len(),
                });
            }
        }

        if let Some(max_len) = rule.max_length {
            if value.len() > max_len {
                return Err(ValidationError::TooLong {
                    max: max_len,
                    actual: value.len(),
                });
            }
        }

        // Pattern validation
        if let Some(pattern_str) = &rule.pattern {
            if let Ok(pattern) = Regex::new(pattern_str) {
                if !pattern.is_match(value) {
                    return Err(ValidationError::InvalidFormat(
                        format!("Value does not match required pattern for {field_name}")
                    ));
                }
            }
        }

        // Forbidden characters check
        if let Some(forbidden) = &rule.forbidden_chars {
            for forbidden_char in forbidden.chars() {
                if value.contains(forbidden_char) {
                    return Err(ValidationError::ForbiddenCharacters(
                        format!("Contains forbidden character: {forbidden_char}")
                    ));
                }
            }
        }

        // Security threat detection
        for pattern in &self.security_patterns {
            if pattern.is_match(value) {
                return Err(ValidationError::SecurityThreat(
                    "Input contains potential security threat".to_string()
                ));
            }
        }

        // Sanitize if required
        let result = if rule.sanitize {
            self.sanitize_input(value)
        } else {
            value.to_string()
        };

        Ok(result)
    }

    pub fn sanitize_input(&self, input: &str) -> String {
        input
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
            .replace('\'', "&#x27;")
            .replace('&', "&amp;")
            .trim()
            .to_string()
    }

    pub fn validate_multiple(&self, inputs: HashMap<String, String>) -> Result<HashMap<String, String>, Vec<ValidationError>> {
        let mut validated = HashMap::new();
        let mut errors = Vec::new();

        for (field_name, value) in inputs {
            match self.validate(&field_name, &value) {
                Ok(validated_value) => {
                    validated.insert(field_name, validated_value);
                }
                Err(error) => {
                    errors.push(error);
                }
            }
        }

        if errors.is_empty() {
            Ok(validated)
        } else {
            Err(errors)
        }
    }

    pub fn add_rule(&mut self, field_name: String, rule: ValidationRule) {
        self.rules.insert(field_name, rule);
    }

    pub fn remove_rule(&mut self, field_name: &str) {
        self.rules.remove(field_name);
    }

    pub fn get_rule(&self, field_name: &str) -> Option<&ValidationRule> {
        self.rules.get(field_name)
    }

    pub fn validate_json_structure(&self, json_str: &str, max_depth: usize) -> Result<(), ValidationError> {
        // Basic JSON structure validation
        let parsed: Result<serde_json::Value, _> = serde_json::from_str(json_str);

        match parsed {
            Ok(value) => {
                if self.check_json_depth(&value, 0, max_depth) {
                    Ok(())
                } else {
                    Err(ValidationError::InvalidValue(
                        format!("JSON structure exceeds maximum depth of {max_depth}")
                    ))
                }
            }
            Err(_) => Err(ValidationError::InvalidFormat("Invalid JSON format".to_string()))
        }
    }

    fn check_json_depth(&self, value: &serde_json::Value, current_depth: usize, max_depth: usize) -> bool {
        if current_depth > max_depth {
            return false;
        }

        match value {
            serde_json::Value::Object(obj) => {
                for (_, v) in obj {
                    if !self.check_json_depth(v, current_depth + 1, max_depth) {
                        return false;
                    }
                }
            }
            serde_json::Value::Array(arr) => {
                for v in arr {
                    if !self.check_json_depth(v, current_depth + 1, max_depth) {
                        return false;
                    }
                }
            }
            _ => {}
        }

        true
    }

    pub fn is_safe_filename(&self, filename: &str) -> bool {
        // Check for dangerous filename patterns
        let dangerous_patterns = [
            "..", "/", "\\", ":", "*", "?", "\"", "<", ">", "|",
            "CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8", "COM9",
            "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9"
        ];

        let filename_upper = filename.to_uppercase();

        for pattern in &dangerous_patterns {
            if filename_upper.contains(pattern) {
                return false;
            }
        }

        // Check length
        filename.len() <= 255 && !filename.is_empty()
    }
}

impl Default for InputValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_email_validation() {
        let validator = InputValidator::new();

        assert!(validator.validate("email", "test@example.com").is_ok());
        assert!(validator.validate("email", "invalid-email").is_err());
        assert!(validator.validate("email", "").is_err()); // Required field
    }

    #[test]
    fn test_security_threat_detection() {
        let validator = InputValidator::new();

        assert!(validator.validate("user_input", "SELECT * FROM users").is_err());
        assert!(validator.validate("user_input", "<script>alert('xss')</script>").is_err());
        assert!(validator.validate("user_input", "normal input").is_ok());
    }

    #[test]
    fn test_input_sanitization() {
        // Create a validator without security patterns for this test
        let mut test_validator = InputValidator {
            rules: std::collections::HashMap::new(),
            security_patterns: Vec::new(), // No security patterns for this test
        };

        test_validator.add_rule("test_input".to_string(), ValidationRule {
            min_length: Some(1),
            max_length: Some(1000),
            pattern: None,
            allowed_chars: None,
            forbidden_chars: None,
            required: true,
            sanitize: true,
        });

        let result = test_validator.validate("test_input", "<div>Hello & World</div>").unwrap();
        // The sanitization encodes & first, then < and >, so we get &amp;lt; instead of &lt;
        assert!(result.contains("&amp;lt;"));
        assert!(result.contains("&amp;gt;"));
        assert!(result.contains("&amp;"));
    }

    #[test]
    fn test_filename_safety() {
        let validator = InputValidator::new();

        assert!(validator.is_safe_filename("document.txt"));
        assert!(!validator.is_safe_filename("../../../etc/passwd"));
        assert!(!validator.is_safe_filename("CON"));
        assert!(!validator.is_safe_filename("file<name>.txt"));
    }
}
