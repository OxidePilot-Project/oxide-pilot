use serde_json::{Value, json};
use std::collections::HashMap;
use log::{info, error};
use async_trait::async_trait;
use tokio::fs;

// Define a trait for executable functions
#[async_trait]
pub trait ExecutableFunction: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn parameters(&self) -> Value;
    async fn execute(&self, args: Value) -> Result<Value, String>;
}

// Example function: get_current_time
pub struct GetCurrentTimeFunction;

#[async_trait]
impl ExecutableFunction for GetCurrentTimeFunction {
    fn name(&self) -> &str {
        "get_current_time"
    }

    fn description(&self) -> &str {
        "Returns the current date and time."
    }

    fn parameters(&self) -> Value {
        json!({})
    }

    async fn execute(&self, _args: Value) -> Result<Value, String> {
        info!("Executing get_current_time function.");
        Ok(json!({ "current_time": chrono::Utc::now().to_string() }))
    }
}

// New function: read_file
pub struct ReadFileFunction;

#[async_trait]
impl ExecutableFunction for ReadFileFunction {
    fn name(&self) -> &str {
        "read_file"
    }

    fn description(&self) -> &str {
        "Reads the content of a specified file."
    }

    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "The path to the file to read."
                }
            },
            "required": ["path"]
        })
    }

    async fn execute(&self, args: Value) -> Result<Value, String> {
        let path = args["path"].as_str().ok_or("Missing 'path' argument for read_file function.".to_string())?;
        info!("Executing read_file function for path: {}", path);
        match fs::read_to_string(path).await {
            Ok(content) => Ok(json!({ "content": content })),
            Err(e) => Err(format!("Failed to read file '{}': {}", path, e)),
        }
    }
}

// Function Registry
pub struct FunctionRegistry {
    functions: HashMap<String, Box<dyn ExecutableFunction>>,
}

impl FunctionRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            functions: HashMap::new(),
        };
        registry.register_function(Box::new(GetCurrentTimeFunction));
        registry.register_function(Box::new(ReadFileFunction)); // Register the new function
        registry
    }

    pub fn register_function(&mut self, function: Box<dyn ExecutableFunction>) {
        info!("Registering function: {}", function.name());
        self.functions.insert(function.name().to_string(), function);
    }

    pub fn get_function(&self, name: &str) -> Option<&Box<dyn ExecutableFunction>> {
        self.functions.get(name)
    }

    pub async fn execute_function(&self, name: &str, args: Value) -> Result<Value, String> {
        if let Some(function) = self.get_function(name) {
            function.execute(args).await
        } else {
            Err(format!("Function not found: {}", name))
        }
    }

    pub fn get_all_function_schemas(&self) -> Vec<Value> {
        self.functions.values().map(|f| {
            json!({ "name": f.name(), "description": f.description(), "parameters": f.parameters() })
        }).collect()
    }
}