use async_trait::async_trait;
use base64;
use log::{error, info};
use oxide_rpa::rpa::{KeyboardController, MouseController, ScreenCapture};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::process::Command;
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
        let path = args["path"]
            .as_str()
            .ok_or("Missing 'path' argument for read_file function.".to_string())?;
        info!("Executing read_file function for path: {}", path);
        match fs::read_to_string(path).await {
            Ok(content) => Ok(json!({ "content": content })),
            Err(e) => Err(format!("Failed to read file '{}': {}", path, e)),
        }
    }
}

// Function: take_screenshot
pub struct TakeScreenshotFunction {
    screen_capture: ScreenCapture,
}

impl TakeScreenshotFunction {
    pub fn new() -> Self {
        Self {
            screen_capture: ScreenCapture::new(),
        }
    }
}

#[async_trait]
impl ExecutableFunction for TakeScreenshotFunction {
    fn name(&self) -> &str {
        "take_screenshot"
    }

    fn description(&self) -> &str {
        "Takes a screenshot of the current screen and saves it to a file."
    }

    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "filename": {
                    "type": "string",
                    "description": "The filename to save the screenshot (optional, defaults to timestamp)"
                }
            }
        })
    }

    async fn execute(&self, args: Value) -> Result<Value, String> {
        let filename = args["filename"].as_str().unwrap_or(&format!(
            "screenshot_{}.png",
            chrono::Utc::now().timestamp()
        ));

        info!("Taking screenshot and saving to: {}", filename);

        match self.screen_capture.capture_screen().await {
            Ok(image) => match image.save(filename) {
                Ok(_) => Ok(json!({
                    "success": true,
                    "filename": filename,
                    "message": "Screenshot saved successfully"
                })),
                Err(e) => Err(format!("Failed to save screenshot: {}", e)),
            },
            Err(e) => Err(format!("Failed to capture screen: {}", e)),
        }
    }
}

// Function: click_mouse
pub struct ClickMouseFunction {
    mouse_controller: MouseController,
}

impl ClickMouseFunction {
    pub fn new() -> Self {
        Self {
            mouse_controller: MouseController::new(),
        }
    }
}

#[async_trait]
impl ExecutableFunction for ClickMouseFunction {
    fn name(&self) -> &str {
        "click_mouse"
    }

    fn description(&self) -> &str {
        "Clicks the mouse at specified coordinates."
    }

    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "x": {
                    "type": "number",
                    "description": "X coordinate to click"
                },
                "y": {
                    "type": "number",
                    "description": "Y coordinate to click"
                },
                "button": {
                    "type": "string",
                    "description": "Mouse button to click (left, right, middle)",
                    "enum": ["left", "right", "middle"]
                }
            },
            "required": ["x", "y"]
        })
    }

    async fn execute(&self, args: Value) -> Result<Value, String> {
        let x = args["x"]
            .as_f64()
            .ok_or("Missing or invalid 'x' coordinate")? as i32;
        let y = args["y"]
            .as_f64()
            .ok_or("Missing or invalid 'y' coordinate")? as i32;
        let button = args["button"].as_str().unwrap_or("left");

        info!("Clicking mouse at ({}, {}) with {} button", x, y, button);

        match self.mouse_controller.click(x, y, button).await {
            Ok(_) => Ok(json!({
                "success": true,
                "message": format!("Clicked {} button at ({}, {})", button, x, y)
            })),
            Err(e) => Err(format!("Failed to click mouse: {}", e)),
        }
    }
}

// Function: type_text
pub struct TypeTextFunction {
    keyboard_controller: KeyboardController,
}

impl TypeTextFunction {
    pub fn new() -> Self {
        Self {
            keyboard_controller: KeyboardController::new(),
        }
    }
}

#[async_trait]
impl ExecutableFunction for TypeTextFunction {
    fn name(&self) -> &str {
        "type_text"
    }

    fn description(&self) -> &str {
        "Types the specified text using keyboard simulation."
    }

    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "text": {
                    "type": "string",
                    "description": "The text to type"
                }
            },
            "required": ["text"]
        })
    }

    async fn execute(&self, args: Value) -> Result<Value, String> {
        let text = args["text"].as_str().ok_or("Missing 'text' argument")?;

        info!("Typing text: {}", text);

        match self.keyboard_controller.type_text(text).await {
            Ok(_) => Ok(json!({
                "success": true,
                "message": format!("Successfully typed: {}", text)
            })),
            Err(e) => Err(format!("Failed to type text: {}", e)),
        }
    }
}

// Function: analyze_screen
pub struct AnalyzeScreenFunction {
    screen_capture: ScreenCapture,
}

impl AnalyzeScreenFunction {
    pub fn new() -> Self {
        Self {
            screen_capture: ScreenCapture::new(),
        }
    }
}

#[async_trait]
impl ExecutableFunction for AnalyzeScreenFunction {
    fn name(&self) -> &str {
        "analyze_screen"
    }

    fn description(&self) -> &str {
        "Takes a screenshot and returns it for AI analysis. The AI can see what's on the screen."
    }

    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "question": {
                    "type": "string",
                    "description": "Specific question about what to look for on the screen (optional)"
                }
            }
        })
    }

    async fn execute(&self, args: Value) -> Result<Value, String> {
        let question = args["question"]
            .as_str()
            .unwrap_or("Analyze what's visible on the screen");

        info!("Analyzing screen with question: {}", question);

        match self.screen_capture.capture_screen().await {
            Ok(image) => {
                // Convert image to bytes for AI analysis
                let mut buffer = Vec::new();
                match image.write_to(
                    &mut std::io::Cursor::new(&mut buffer),
                    image::ImageOutputFormat::Png,
                ) {
                    Ok(_) => Ok(json!({
                        "success": true,
                        "message": "Screenshot captured for analysis",
                        "question": question,
                        "image_data": base64::encode(&buffer),
                        "mime_type": "image/png"
                    })),
                    Err(e) => Err(format!("Failed to encode image: {}", e)),
                }
            }
            Err(e) => Err(format!("Failed to capture screen: {}", e)),
        }
    }
}

// Function: execute_command
pub struct ExecuteCommandFunction;

#[async_trait]
impl ExecutableFunction for ExecuteCommandFunction {
    fn name(&self) -> &str {
        "execute_command"
    }

    fn description(&self) -> &str {
        "Executes a system command and returns the output."
    }

    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "command": {
                    "type": "string",
                    "description": "The command to execute"
                },
                "args": {
                    "type": "array",
                    "items": {"type": "string"},
                    "description": "Command arguments (optional)"
                }
            },
            "required": ["command"]
        })
    }

    async fn execute(&self, args: Value) -> Result<Value, String> {
        let command = args["command"]
            .as_str()
            .ok_or("Missing 'command' argument")?;
        let cmd_args: Vec<String> = args["args"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();

        info!("Executing command: {} with args: {:?}", command, cmd_args);

        let output = Command::new(command)
            .args(&cmd_args)
            .output()
            .map_err(|e| format!("Failed to execute command: {}", e))?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        Ok(json!({
            "success": output.status.success(),
            "exit_code": output.status.code(),
            "stdout": stdout,
            "stderr": stderr
        }))
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
        registry.register_function(Box::new(ReadFileFunction));
        registry.register_function(Box::new(TakeScreenshotFunction::new()));
        registry.register_function(Box::new(ClickMouseFunction::new()));
        registry.register_function(Box::new(TypeTextFunction::new()));
        registry.register_function(Box::new(AnalyzeScreenFunction::new()));
        registry.register_function(Box::new(ExecuteCommandFunction));
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
