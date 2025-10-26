use image::{ImageBuffer, Rgba};
use log::{error, info};
use rdev::{simulate, EventType};
pub use rdev::{Button, Key};
use screenshots::Screen;
use std::time::Duration;
use thiserror::Error;
// use tokio::time::sleep; // Reserved for future use

#[derive(Error, Debug)]
pub enum RPAError {
    #[error("Mouse operation failed: {0}")]
    Mouse(String),
    #[error("Keyboard operation failed: {0}")]
    Keyboard(String),
    #[error("Screen capture failed: {0}")]
    ScreenCapture(String),
    #[error("Simulation error: {0}")]
    Simulation(String),
}

pub struct MouseController;

impl Default for MouseController {
    fn default() -> Self {
        Self::new()
    }
}

impl MouseController {
    pub fn new() -> Self {
        Self {}
    }

    pub fn move_to(&self, x: i32, y: i32) {
        info!("Moving mouse to ({x}, {y})");
        simulate(&EventType::MouseMove {
            x: x as f64,
            y: y as f64,
        })
        .unwrap();
    }

    pub fn click(&self, button: Button) {
        info!("Clicking mouse button: {button:?}");
        simulate(&EventType::ButtonPress(button)).unwrap();
        simulate(&EventType::ButtonRelease(button)).unwrap();
    }

    pub fn scroll(&self, delta_x: i32, delta_y: i32) {
        info!("Scrolling mouse by ({delta_x}, {delta_y})");
        simulate(&EventType::Wheel { delta_x: delta_x.into(), delta_y: delta_y.into() }).unwrap();
    }
}

pub struct KeyboardController;

impl Default for KeyboardController {
    fn default() -> Self {
        Self::new()
    }
}

impl KeyboardController {
    pub fn new() -> Self {
        Self {}
    }

    pub fn type_text(&self, text: &str) {
        info!("Typing text: {text}");
        for char_code in text.chars() {
            let key = match char_code {
                'a' => Key::KeyA,
                'b' => Key::KeyB,
                'c' => Key::KeyC,
                'd' => Key::KeyD,
                'e' => Key::KeyE,
                'f' => Key::KeyF,
                'g' => Key::KeyG,
                'h' => Key::KeyH,
                'i' => Key::KeyI,
                'j' => Key::KeyJ,
                'k' => Key::KeyK,
                'l' => Key::KeyL,
                'm' => Key::KeyM,
                'n' => Key::KeyN,
                'o' => Key::KeyO,
                'p' => Key::KeyP,
                'q' => Key::KeyQ,
                'r' => Key::KeyR,
                's' => Key::KeyS,
                't' => Key::KeyT,
                'u' => Key::KeyU,
                'v' => Key::KeyV,
                'w' => Key::KeyW,
                'x' => Key::KeyX,
                'y' => Key::KeyY,
                'z' => Key::KeyZ,
                'A' => Key::KeyA,
                'B' => Key::KeyB,
                'C' => Key::KeyC,
                'D' => Key::KeyD,
                'E' => Key::KeyE,
                'F' => Key::KeyF,
                'G' => Key::KeyG,
                'H' => Key::KeyH,
                'I' => Key::KeyI,
                'J' => Key::KeyJ,
                'K' => Key::KeyK,
                'L' => Key::KeyL,
                'M' => Key::KeyM,
                'N' => Key::KeyN,
                'O' => Key::KeyO,
                'P' => Key::KeyP,
                'Q' => Key::KeyQ,
                'R' => Key::KeyR,
                'S' => Key::KeyS,
                'T' => Key::KeyT,
                'U' => Key::KeyU,
                'V' => Key::KeyV,
                'W' => Key::KeyW,
                'X' => Key::KeyX,
                'Y' => Key::KeyY,
                'Z' => Key::KeyZ,
                '0' => Key::Num0,
                '1' => Key::Num1,
                '2' => Key::Num2,
                '3' => Key::Num3,
                '4' => Key::Num4,
                '5' => Key::Num5,
                '6' => Key::Num6,
                '7' => Key::Num7,
                '8' => Key::Num8,
                '9' => Key::Num9,
                ' ' => Key::Space,
                '.' => Key::Dot,
                ',' => Key::Comma,
                '!' => Key::Num1, // Fallback for special characters
                '?' => Key::Slash, // Fallback for special characters
                _ => {
                    error!("Unsupported character for typing: {char_code}");
                    continue;
                }
            };
            simulate(&EventType::KeyPress(key)).unwrap();
            simulate(&EventType::KeyRelease(key)).unwrap();
            std::thread::sleep(Duration::from_millis(10)); // Small delay for realism
        }
    }

    pub fn press_key(&self, key: Key) {
        info!("Pressing key: {key:?}");
        simulate(&EventType::KeyPress(key)).unwrap();
    }

    pub fn release_key(&self, key: Key) {
        info!("Releasing key: {key:?}");
        simulate(&EventType::KeyRelease(key)).unwrap();
    }
}

pub struct ScreenCapture;

impl Default for ScreenCapture {
    fn default() -> Self {
        Self::new()
    }
}

impl ScreenCapture {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn capture_screen(&self) -> Result<ImageBuffer<Rgba<u8>, Vec<u8>>, String> {
        info!("Capturing screen...");
        let screens = Screen::all().map_err(|e| e.to_string())?;
        if let Some(screen) = screens.first() {
            let image = screen.capture().map_err(|e| e.to_string())?;
            // Convert screenshots::Image to image::ImageBuffer
            let rgba_image = ImageBuffer::from_raw(image.width(), image.height(), image.rgba().to_vec())
                .ok_or("Failed to convert image format")?;
            Ok(rgba_image)
        } else {
            Err("No screens found.".to_string())
        }
    }

    pub async fn capture_area(
        &self,
        x: u32,
        y: u32,
        width: u32,
        height: u32,
    ) -> Result<ImageBuffer<Rgba<u8>, Vec<u8>>, String> {
        info!(
            "Capturing screen area: x={x}, y={y}, width={width}, height={height}"
        );
        let screens = Screen::all().map_err(|e| e.to_string())?;
        if let Some(screen) = screens.first() {
            let image = screen
                .capture_area(x.try_into().unwrap_or(0), y.try_into().unwrap_or(0), width, height)
                .map_err(|e| e.to_string())?;
            // Convert screenshots::Image to image::ImageBuffer
            let rgba_image = ImageBuffer::from_raw(image.width(), image.height(), image.rgba().to_vec())
                .ok_or("Failed to convert image format")?;
            Ok(rgba_image)
        } else {
            Err("No screens found.".to_string())
        }
    }

    // Placeholder for image analysis (e.g., template matching, OCR)
    pub fn analyze_image(&self, image: &ImageBuffer<Rgba<u8>, Vec<u8>>) {
        info!(
            "Analyzing image of size {}x{}",
            image.width(),
            image.height()
        );
        // This is where image processing and analysis logic would go.
        // For example, using image processing libraries to find UI elements.
    }
}
