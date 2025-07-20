/*!
Controlador RPA - Automatización robótica de procesos
*/

use oxide_core::{
    events::{SystemEvent, UserEventData, EventType},
    types::{AgentId, Priority, OxideResult},
    utils::is_safe_process,
};
use rdev::{simulate, EventType as RdevEventType, Key, Button};
use screenshots::Screen;
use std::time::Duration;
use tokio::sync::mpsc;
use tracing::{info, warn, error, debug};

/// Controlador de automatización RPA
pub struct RpaController {
    agent_id: AgentId,
    enabled: bool,
    safety_checks: bool,
    event_sender: Option<mpsc::UnboundedSender<SystemEvent>>,
}

impl RpaController {
    /// Crear nuevo controlador RPA
    pub fn new(agent_id: AgentId, enabled: bool) -> Self {
        Self {
            agent_id,
            enabled,
            safety_checks: true, // Siempre habilitado por seguridad
            event_sender: None,
        }
    }

    /// Configurar canal de eventos
    pub fn set_event_sender(&mut self, sender: mpsc::UnboundedSender<SystemEvent>) {
        self.event_sender = Some(sender);
    }

    /// Habilitar/deshabilitar RPA
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        if enabled {
            info!("🤖 Control RPA habilitado");
        } else {
            info!("🚫 Control RPA deshabilitado");
        }
    }

    /// Tomar captura de pantalla
    pub async fn take_screenshot(&self) -> OxideResult<Vec<u8>> {
        if !self.enabled {
            return Err(anyhow::anyhow!("RPA está deshabilitado"));
        }

        info!("📸 Tomando captura de pantalla");

        let screens = Screen::all()?;
        if let Some(screen) = screens.first() {
            let image = screen.capture()?;
            
            // Convertir imagen a bytes (PNG)
            let mut buffer = Vec::new();
            image.save_png(&mut buffer)?;
            
            info!("✅ Captura de pantalla tomada: {} bytes", buffer.len());
            
            // Enviar evento
            self.send_rpa_event("screenshot_taken", serde_json::json!({
                "size_bytes": buffer.len(),
                "width": image.width(),
                "height": image.height()
            })).await;
            
            Ok(buffer)
        } else {
            Err(anyhow::anyhow!("No se encontraron pantallas"))
        }
    }

    /// Mover el mouse a una posición
    pub async fn move_mouse(&self, x: f64, y: f64) -> OxideResult<()> {
        if !self.enabled {
            return Err(anyhow::anyhow!("RPA está deshabilitado"));
        }

        if !self.is_safe_mouse_position(x, y) {
            return Err(anyhow::anyhow!("Posición del mouse no es segura"));
        }

        debug!("🖱️ Moviendo mouse a ({}, {})", x, y);

        simulate(&RdevEventType::MouseMove { x, y })?;
        
        self.send_rpa_event("mouse_moved", serde_json::json!({
            "x": x,
            "y": y
        })).await;

        Ok(())
    }

    /// Hacer clic del mouse
    pub async fn click_mouse(&self, button: MouseButton, x: f64, y: f64) -> OxideResult<()> {
        if !self.enabled {
            return Err(anyhow::anyhow!("RPA está deshabilitado"));
        }

        if !self.is_safe_mouse_position(x, y) {
            return Err(anyhow::anyhow!("Posición del mouse no es segura"));
        }

        info!("🖱️ Haciendo clic {:?} en ({}, {})", button, x, y);

        // Mover mouse a la posición
        self.move_mouse(x, y).await?;
        
        // Pequeña pausa
        tokio::time::sleep(Duration::from_millis(100)).await;

        let rdev_button = match button {
            MouseButton::Left => Button::Left,
            MouseButton::Right => Button::Right,
            MouseButton::Middle => Button::Middle,
        };

        // Presionar botón
        simulate(&RdevEventType::ButtonPress(rdev_button))?;
        tokio::time::sleep(Duration::from_millis(50)).await;
        
        // Soltar botón
        simulate(&RdevEventType::ButtonRelease(rdev_button))?;

        self.send_rpa_event("mouse_clicked", serde_json::json!({
            "button": format!("{:?}", button),
            "x": x,
            "y": y
        })).await;

        Ok(())
    }

    /// Escribir texto
    pub async fn type_text(&self, text: &str) -> OxideResult<()> {
        if !self.enabled {
            return Err(anyhow::anyhow!("RPA está deshabilitado"));
        }

        if !self.is_safe_text(text) {
            return Err(anyhow::anyhow!("Texto contiene caracteres no seguros"));
        }

        info!("⌨️ Escribiendo texto: '{}'", text);

        for char in text.chars() {
            if let Some(key) = self.char_to_key(char) {
                simulate(&RdevEventType::KeyPress(key))?;
                tokio::time::sleep(Duration::from_millis(50)).await;
                simulate(&RdevEventType::KeyRelease(key))?;
                tokio::time::sleep(Duration::from_millis(50)).await;
            }
        }

        self.send_rpa_event("text_typed", serde_json::json!({
            "text": text,
            "length": text.len()
        })).await;

        Ok(())
    }

    /// Presionar tecla especial
    pub async fn press_key(&self, key: SpecialKey) -> OxideResult<()> {
        if !self.enabled {
            return Err(anyhow::anyhow!("RPA está deshabilitado"));
        }

        info!("⌨️ Presionando tecla: {:?}", key);

        let rdev_key = match key {
            SpecialKey::Enter => Key::Return,
            SpecialKey::Escape => Key::Escape,
            SpecialKey::Tab => Key::Tab,
            SpecialKey::Space => Key::Space,
            SpecialKey::Backspace => Key::BackSpace,
            SpecialKey::Delete => Key::Delete,
            SpecialKey::ArrowUp => Key::UpArrow,
            SpecialKey::ArrowDown => Key::DownArrow,
            SpecialKey::ArrowLeft => Key::LeftArrow,
            SpecialKey::ArrowRight => Key::RightArrow,
            SpecialKey::Home => Key::Home,
            SpecialKey::End => Key::End,
            SpecialKey::PageUp => Key::PageUp,
            SpecialKey::PageDown => Key::PageDown,
            SpecialKey::F1 => Key::F1,
            SpecialKey::F2 => Key::F2,
            SpecialKey::F3 => Key::F3,
            SpecialKey::F4 => Key::F4,
            SpecialKey::F5 => Key::F5,
            SpecialKey::F12 => Key::F12,
            SpecialKey::CtrlC => {
                // Combinación Ctrl+C
                simulate(&RdevEventType::KeyPress(Key::ControlLeft))?;
                simulate(&RdevEventType::KeyPress(Key::KeyC))?;
                tokio::time::sleep(Duration::from_millis(50)).await;
                simulate(&RdevEventType::KeyRelease(Key::KeyC))?;
                simulate(&RdevEventType::KeyRelease(Key::ControlLeft))?;
                
                self.send_rpa_event("key_pressed", serde_json::json!({
                    "key": "Ctrl+C"
                })).await;
                
                return Ok(());
            }
            SpecialKey::CtrlV => {
                // Combinación Ctrl+V
                simulate(&RdevEventType::KeyPress(Key::ControlLeft))?;
                simulate(&RdevEventType::KeyPress(Key::KeyV))?;
                tokio::time::sleep(Duration::from_millis(50)).await;
                simulate(&RdevEventType::KeyRelease(Key::KeyV))?;
                simulate(&RdevEventType::KeyRelease(Key::ControlLeft))?;
                
                self.send_rpa_event("key_pressed", serde_json::json!({
                    "key": "Ctrl+V"
                })).await;
                
                return Ok(());
            }
        };

        simulate(&RdevEventType::KeyPress(rdev_key))?;
        tokio::time::sleep(Duration::from_millis(50)).await;
        simulate(&RdevEventType::KeyRelease(rdev_key))?;

        self.send_rpa_event("key_pressed", serde_json::json!({
            "key": format!("{:?}", key)
        })).await;

        Ok(())
    }

    /// Verificar si una posición del mouse es segura
    fn is_safe_mouse_position(&self, x: f64, y: f64) -> bool {
        if !self.safety_checks {
            return true;
        }

        // Verificar que las coordenadas estén dentro de rangos razonables
        x >= 0.0 && y >= 0.0 && x <= 3840.0 && y <= 2160.0 // 4K máximo
    }

    /// Verificar si el texto es seguro para escribir
    fn is_safe_text(&self, text: &str) -> bool {
        if !self.safety_checks {
            return true;
        }

        // No permitir comandos peligrosos
        let dangerous_patterns = vec![
            "rm -rf", "del /f", "format", "shutdown", "reboot",
            "taskkill", "net user", "reg delete", "powershell",
        ];

        let text_lower = text.to_lowercase();
        !dangerous_patterns.iter().any(|&pattern| text_lower.contains(pattern))
    }

    /// Convertir carácter a tecla
    fn char_to_key(&self, c: char) -> Option<Key> {
        match c {
            'a'..='z' => Some(Key::KeyA), // Simplificado - en realidad necesitaríamos mapeo completo
            'A'..='Z' => Some(Key::KeyA), // Simplificado
            '0'..='9' => Some(Key::Num0), // Simplificado
            ' ' => Some(Key::Space),
            '\n' => Some(Key::Return),
            '\t' => Some(Key::Tab),
            _ => None, // Caracteres no soportados
        }
    }

    /// Enviar evento RPA
    async fn send_rpa_event(&self, action: &str, parameters: serde_json::Value) {
        if let Some(sender) = &self.event_sender {
            let event = SystemEvent::new(
                self.agent_id.clone(),
                Priority::Normal,
                EventType::User(UserEventData::RpaRequest {
                    action: action.to_string(),
                    parameters,
                }),
            );
            
            if let Err(e) = sender.send(event) {
                error!("Error enviando evento RPA: {}", e);
            }
        }
    }

    /// Obtener información de pantallas disponibles
    pub fn get_screen_info(&self) -> OxideResult<Vec<ScreenInfo>> {
        let screens = Screen::all()?;
        let mut screen_info = Vec::new();

        for (index, screen) in screens.iter().enumerate() {
            screen_info.push(ScreenInfo {
                index,
                width: screen.display_info.width,
                height: screen.display_info.height,
                x: screen.display_info.x,
                y: screen.display_info.y,
                is_primary: screen.display_info.is_primary,
            });
        }

        Ok(screen_info)
    }
}

/// Botones del mouse
#[derive(Debug, Clone, Copy)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
}

/// Teclas especiales
#[derive(Debug, Clone, Copy)]
pub enum SpecialKey {
    Enter,
    Escape,
    Tab,
    Space,
    Backspace,
    Delete,
    ArrowUp,
    ArrowDown,
    ArrowLeft,
    ArrowRight,
    Home,
    End,
    PageUp,
    PageDown,
    F1,
    F2,
    F3,
    F4,
    F5,
    F12,
    CtrlC,
    CtrlV,
}

/// Información de pantalla
#[derive(Debug, Clone)]
pub struct ScreenInfo {
    pub index: usize,
    pub width: u32,
    pub height: u32,
    pub x: i32,
    pub y: i32,
    pub is_primary: bool,
}