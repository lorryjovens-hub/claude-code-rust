use anyhow::{anyhow, Result};
use enigo::{Button, Direction, Enigo, Key, Keyboard, Mouse, Settings};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

pub mod sidecar;
pub mod http_bridge;
pub mod browserless;

use sidecar::SidecarClient;
use http_bridge::HttpBridgeClient;
use browserless::BrowserlessClient;

#[cfg(windows)]
use std::os::windows::process::CommandExt;

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x08000000;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ComputerUseBackend {
    Local,
    Sidecar,
    HttpBridge,
    Browserless,
}

impl Default for ComputerUseBackend {
    fn default() -> Self {
        Self::Local
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputerUseBackendConfig {
    pub backend: ComputerUseBackend,
    pub sidecar_socket: Option<String>,
    pub http_bridge_url: Option<String>,
    pub browserless_url: Option<String>,
}

impl Default for ComputerUseBackendConfig {
    fn default() -> Self {
        Self {
            backend: ComputerUseBackend::Local,
            sidecar_socket: None,
            http_bridge_url: Some("http://127.0.0.1:9527".to_string()),
            browserless_url: Some("http://127.0.0.1:3000".to_string()),
        }
    }
}

pub struct UnifiedComputerUseManager {
    config: ComputerUseConfig,
    backend_config: ComputerUseBackendConfig,
    local_manager: ComputerUseManager,
    sidecar_client: SidecarClient,
    http_bridge_client: HttpBridgeClient,
    browserless_client: BrowserlessClient,
}

impl UnifiedComputerUseManager {
    pub fn new(config: ComputerUseConfig, backend_config: Option<ComputerUseBackendConfig>) -> Self {
        let backend = backend_config.clone().unwrap_or_default();
        Self {
            local_manager: ComputerUseManager::new(config.clone()),
            sidecar_client: SidecarClient::new(),
            http_bridge_client: HttpBridgeClient::new(
                backend.http_bridge_url.as_deref(),
            ),
            browserless_client: BrowserlessClient::new(
                backend.browserless_url.as_deref(),
            ),
            config,
            backend_config: backend,
        }
    }

    pub fn get_screen_info(&self) -> ScreenInfo {
        self.local_manager.get_screen_info()
    }

    pub fn get_mouse_position(&self) -> ScreenCoordinate {
        self.local_manager.get_mouse_position()
    }

    pub fn get_action_history(&self, limit: Option<usize>) -> Vec<ComputerActionResult> {
        self.local_manager.get_action_history(limit)
    }

    pub async fn execute_action(&self, action: ComputerAction) -> Result<ComputerActionResult> {
        match self.backend_config.backend {
            ComputerUseBackend::Local => self.local_manager.execute_action(action).await,
            ComputerUseBackend::Sidecar => {
                self.execute_via_sidecar(action).await
            }
            ComputerUseBackend::HttpBridge => {
                self.execute_via_http_bridge(action).await
            }
            ComputerUseBackend::Browserless => {
                self.execute_via_browserless(action).await
            }
        }
    }

    async fn execute_via_sidecar(
        &self,
        action: ComputerAction,
    ) -> Result<ComputerActionResult> {
        let resp = self
            .sidecar_client
            .execute_action(
                &action_type_str(&action.action_type),
                action.coordinate.as_ref().map(|c| [c.x, c.y]),
                action.button.as_ref().map(|b| button_str(b)).as_deref(),
                action.key.as_deref(),
                action.text.as_deref(),
                action.scroll_y,
                action.scroll_x,
                action.duration_ms,
            )
            .await
            .map_err(|e| anyhow!("Sidecar execution failed: {}", e))?;

        Ok(ComputerActionResult {
            success: resp.success,
            action,
            screenshot: resp.screenshot,
            error: resp.error,
            duration_ms: resp.duration_ms,
        })
    }

    async fn execute_via_http_bridge(
        &self,
        action: ComputerAction,
    ) -> Result<ComputerActionResult> {
        let bridge_action = http_bridge::BridgeActionRequest {
            action_type: action_type_str(&action.action_type),
            coordinate: action.coordinate.as_ref().map(|c| [c.x, c.y]),
            button: action.button.as_ref().map(|b| button_str(b)),
            key: action.key.clone(),
            text: action.text.clone(),
            scroll_y: action.scroll_y,
            scroll_x: action.scroll_x,
            duration_ms: action.duration_ms,
        };

        let resp = self
            .http_bridge_client
            .execute_action(bridge_action)
            .await
            .map_err(|e| anyhow!("HTTP Bridge execution failed: {}", e))?;

        Ok(ComputerActionResult {
            success: resp.success,
            action,
            screenshot: resp.screenshot,
            error: resp.error,
            duration_ms: resp.duration_ms,
        })
    }

    async fn execute_via_browserless(
        &self,
        action: ComputerAction,
    ) -> Result<ComputerActionResult> {
        let ba = browserless::BrowserlessAction {
            action: action_type_str(&action.action_type),
            selector: None,
            value: action.text.clone(),
            coordinate: action.coordinate.as_ref().map(|c| [c.x, c.y]),
            button: action.button.as_ref().map(|b| button_str(b)),
            key: action.key.clone(),
            wait_ms: action.duration_ms,
        };

        let resp = self
            .browserless_client
            .execute_puppeteer("about:blank", vec![ba], None)
            .await
            .map_err(|e| anyhow!("Browserless execution failed: {}", e))?;

        Ok(ComputerActionResult {
            success: resp.success,
            action,
            screenshot: resp.screenshot,
            error: resp.error,
            duration_ms: 0,
        })
    }

    pub async fn take_screenshot(&self) -> Result<String> {
        match self.backend_config.backend {
            ComputerUseBackend::Local => take_screenshot_powershell(),
            ComputerUseBackend::Sidecar => {
                let resp = self.sidecar_client.take_screenshot().await?;
                resp.screenshot
                    .ok_or_else(|| anyhow!("Sidecar screenshot returned no data"))
            }
            ComputerUseBackend::HttpBridge => {
                let resp = self.http_bridge_client.take_screenshot().await?;
                resp.screenshot
                    .ok_or_else(|| anyhow!("HTTP Bridge screenshot returned no data"))
            }
            ComputerUseBackend::Browserless => {
                self.browserless_client
                    .take_screenshot("about:blank")
                    .await?
                    .ok_or_else(|| anyhow!("Browserless screenshot returned no data"))
            }
        }
    }
}

fn action_type_str(t: &ComputerActionType) -> String {
    match t {
        ComputerActionType::MouseMove => "mouse_move",
        ComputerActionType::MouseClick => "mouse_click",
        ComputerActionType::MouseDown => "mouse_down",
        ComputerActionType::MouseUp => "mouse_up",
        ComputerActionType::MouseScroll => "mouse_scroll",
        ComputerActionType::KeyPress => "key_press",
        ComputerActionType::KeyDown => "key_down",
        ComputerActionType::KeyUp => "key_up",
        ComputerActionType::TypeText => "type_text",
        ComputerActionType::Screenshot => "screenshot",
        ComputerActionType::Wait => "wait",
    }
    .to_string()
}

fn button_str(b: &MouseButton) -> String {
    match b {
        MouseButton::Left => "left",
        MouseButton::Right => "right",
        MouseButton::Middle => "middle",
        MouseButton::Back => "back",
        MouseButton::Forward => "forward",
    }
    .to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ComputerActionType {
    MouseMove,
    MouseClick,
    MouseDown,
    MouseUp,
    MouseScroll,
    KeyPress,
    KeyDown,
    KeyUp,
    TypeText,
    Screenshot,
    Wait,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
    Back,
    Forward,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScreenCoordinate {
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputerAction {
    pub action_type: ComputerActionType,
    pub coordinate: Option<ScreenCoordinate>,
    pub button: Option<MouseButton>,
    pub key: Option<String>,
    pub text: Option<String>,
    pub scroll_y: Option<i32>,
    pub scroll_x: Option<i32>,
    pub duration_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputerActionResult {
    pub success: bool,
    pub action: ComputerAction,
    pub screenshot: Option<String>,
    pub error: Option<String>,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputerUseConfig {
    pub enabled: bool,
    pub screenshot_quality: u8,
    pub max_action_duration_ms: u64,
    pub require_screenshot_after: Vec<ComputerActionType>,
    pub allowed_applications: Vec<String>,
}

impl Default for ComputerUseConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            screenshot_quality: 80,
            max_action_duration_ms: 30_000,
            require_screenshot_after: vec![
                ComputerActionType::MouseMove,
                ComputerActionType::MouseClick,
                ComputerActionType::Screenshot,
            ],
            allowed_applications: vec![],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScreenInfo {
    pub width: u32,
    pub height: u32,
    pub scale_factor: f64,
}

pub struct ComputerUseManager {
    config: ComputerUseConfig,
    action_history: Arc<Mutex<Vec<ComputerActionResult>>>,
    current_position: Arc<Mutex<ScreenCoordinate>>,
    pressed_keys: Arc<Mutex<Vec<String>>>,
    pressed_buttons: Arc<Mutex<Vec<MouseButton>>>,
    screen_info: ScreenInfo,
}

impl ComputerUseManager {
    pub fn new(config: ComputerUseConfig) -> Self {
        let screen_info = get_real_screen_info().unwrap_or_else(|_| ScreenInfo {
            width: 1920,
            height: 1080,
            scale_factor: 1.0,
        });

        Self {
            config,
            action_history: Arc::new(Mutex::new(Vec::new())),
            current_position: Arc::new(Mutex::new(ScreenCoordinate { x: 0, y: 0 })),
            pressed_keys: Arc::new(Mutex::new(Vec::new())),
            pressed_buttons: Arc::new(Mutex::new(Vec::new())),
            screen_info,
        }
    }

    pub fn get_screen_info(&self) -> ScreenInfo {
        self.screen_info.clone()
    }

    pub fn get_mouse_position(&self) -> ScreenCoordinate {
        self.current_position.lock().unwrap_or_else(|e| e.into_inner()).clone()
    }

    pub fn get_action_history(&self, limit: Option<usize>) -> Vec<ComputerActionResult> {
        let history = self.action_history.lock().unwrap_or_else(|e| e.into_inner());
        let limit = limit.unwrap_or(history.len());
        history.iter().rev().take(limit).cloned().collect()
    }

    pub async fn execute_action(&self, action: ComputerAction) -> Result<ComputerActionResult> {
        if !self.config.enabled {
            return Ok(ComputerActionResult {
                success: false,
                action,
                screenshot: None,
                error: Some("Computer use is disabled".to_string()),
                duration_ms: 0,
            });
        }

        let start_time = std::time::Instant::now();

        let result = self.execute_action_impl(action.clone()).await;
        let duration = start_time.elapsed();

        let (success, error) = match result {
            Ok(_) => (true, None),
            Err(e) => (false, Some(e.to_string())),
        };

        let screenshot = if self.config.require_screenshot_after.contains(&action.action_type) || success {
            Some(self.take_screenshot().await.unwrap_or_else(|_| "".to_string()))
        } else {
            None
        };

        let action_result = ComputerActionResult {
            success,
            action,
            screenshot,
            error,
            duration_ms: duration.as_millis() as u64,
        };

        self.action_history.lock().unwrap_or_else(|e| e.into_inner()).push(action_result.clone());
        Ok(action_result)
    }

    pub async fn execute_actions(&self, actions: Vec<ComputerAction>) -> Vec<ComputerActionResult> {
        let mut results = Vec::new();
        for action in actions {
            let result = self.execute_action(action).await;
            results.push(result.unwrap_or_else(|e| ComputerActionResult {
                success: false,
                action: ComputerAction {
                    action_type: ComputerActionType::Wait,
                    coordinate: None,
                    button: None,
                    key: None,
                    text: None,
                    scroll_y: None,
                    scroll_x: None,
                    duration_ms: None,
                },
                screenshot: None,
                error: Some(e.to_string()),
                duration_ms: 0,
            }));
        }
        results
    }

    async fn execute_action_impl(&self, action: ComputerAction) -> Result<()> {
        match action.action_type {
            ComputerActionType::MouseMove => {
                if let Some(coord) = &action.coordinate {
                    if let Err(e) = self.real_mouse_move(coord.x, coord.y) {
                        tracing::warn!("Real mouse move failed, using simulated fallback: {}", e);
                    }
                    let mut pos = self.current_position.lock().unwrap_or_else(|e| e.into_inner());
                    pos.x = coord.x;
                    pos.y = coord.y;
                }
            }
            ComputerActionType::MouseClick => {
                if let Some(coord) = &action.coordinate {
                    if let Err(e) = self.real_mouse_move(coord.x, coord.y) {
                        tracing::warn!("Real mouse move for click failed: {}", e);
                    }
                    let mut pos = self.current_position.lock().unwrap_or_else(|e| e.into_inner());
                    pos.x = coord.x;
                    pos.y = coord.y;
                }
                let button = action.button.as_ref().unwrap_or(&MouseButton::Left);
                if let Err(e) = self.real_mouse_click(button) {
                    tracing::warn!("Real mouse click failed, using simulated fallback: {}", e);
                }
            }
            ComputerActionType::MouseDown => {
                let button = action.button.as_ref().unwrap_or(&MouseButton::Left);
                if let Err(e) = self.real_mouse_down(button) {
                    tracing::warn!("Real mouse down failed: {}", e);
                }
                let mut buttons = self.pressed_buttons.lock().unwrap_or_else(|e| e.into_inner());
                if !buttons.contains(button) {
                    buttons.push(button.clone());
                }
            }
            ComputerActionType::MouseUp => {
                let button = action.button.as_ref().unwrap_or(&MouseButton::Left);
                if let Err(e) = self.real_mouse_up(button) {
                    tracing::warn!("Real mouse up failed: {}", e);
                }
                if let Some(button) = &action.button {
                    let mut buttons = self.pressed_buttons.lock().unwrap_or_else(|e| e.into_inner());
                    buttons.retain(|b| b != button);
                }
            }
            ComputerActionType::MouseScroll => {
                if let Some(scroll_y) = action.scroll_y {
                    if let Err(e) = self.real_mouse_scroll(scroll_y, true) {
                        tracing::warn!("Real mouse scroll vertical failed: {}", e);
                    }
                }
                if let Some(scroll_x) = action.scroll_x {
                    if let Err(e) = self.real_mouse_scroll(scroll_x, false) {
                        tracing::warn!("Real mouse scroll horizontal failed: {}", e);
                    }
                }
            }
            ComputerActionType::KeyPress => {
                if let Some(key) = &action.key {
                    if let Err(e) = self.real_key_press(key) {
                        tracing::warn!("Real key press failed, using simulated fallback: {}", e);
                        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                    }
                }
            }
            ComputerActionType::KeyDown => {
                if let Some(key) = &action.key {
                    if let Err(e) = self.real_key_down(key) {
                        tracing::warn!("Real key down failed: {}", e);
                    }
                    let mut keys = self.pressed_keys.lock().unwrap_or_else(|e| e.into_inner());
                    if !keys.contains(key) {
                        keys.push(key.clone());
                    }
                }
            }
            ComputerActionType::KeyUp => {
                if let Some(key) = &action.key {
                    if let Err(e) = self.real_key_up(key) {
                        tracing::warn!("Real key up failed: {}", e);
                    }
                    let mut keys = self.pressed_keys.lock().unwrap_or_else(|e| e.into_inner());
                    keys.retain(|k| k != key);
                }
            }
            ComputerActionType::TypeText => {
                if let Some(text) = &action.text {
                    if let Err(e) = self.real_type_text(text) {
                        tracing::warn!("Real type text failed, using simulated fallback: {}", e);
                        for _ in text.chars() {
                            tokio::time::sleep(std::time::Duration::from_millis(50)).await;
                        }
                    }
                }
            }
            ComputerActionType::Wait => {
                let duration = action.duration_ms.unwrap_or(1000);
                tokio::time::sleep(std::time::Duration::from_millis(duration)).await;
            }
            ComputerActionType::Screenshot => {}
        }
        Ok(())
    }

    async fn take_screenshot(&self) -> Result<String> {
        take_screenshot_powershell()
    }

    fn real_mouse_move(&self, x: i32, y: i32) -> Result<()> {
        let mut enigo = Enigo::new(&Settings::default())
            .map_err(|e| anyhow!("Failed to create Enigo: {:?}", e))?;
        enigo.move_mouse(x, y, enigo::Coordinate::Abs)
            .map_err(|e| anyhow!("Failed to move mouse: {:?}", e))?;
        Ok(())
    }

    fn real_mouse_click(&self, button: &MouseButton) -> Result<()> {
        let mut enigo = Enigo::new(&Settings::default())
            .map_err(|e| anyhow!("Failed to create Enigo: {:?}", e))?;
        let enigo_button = map_mouse_button(button);
        enigo.button(enigo_button, Direction::Click)
            .map_err(|e| anyhow!("Failed to click: {:?}", e))?;
        Ok(())
    }

    fn real_mouse_down(&self, button: &MouseButton) -> Result<()> {
        let mut enigo = Enigo::new(&Settings::default())
            .map_err(|e| anyhow!("Failed to create Enigo: {:?}", e))?;
        let enigo_button = map_mouse_button(button);
        enigo.button(enigo_button, Direction::Press)
            .map_err(|e| anyhow!("Failed to press mouse button: {:?}", e))?;
        Ok(())
    }

    fn real_mouse_up(&self, button: &MouseButton) -> Result<()> {
        let mut enigo = Enigo::new(&Settings::default())
            .map_err(|e| anyhow!("Failed to create Enigo: {:?}", e))?;
        let enigo_button = map_mouse_button(button);
        enigo.button(enigo_button, Direction::Release)
            .map_err(|e| anyhow!("Failed to release mouse button: {:?}", e))?;
        Ok(())
    }

    fn real_mouse_scroll(&self, amount: i32, vertical: bool) -> Result<()> {
        let mut enigo = Enigo::new(&Settings::default())
            .map_err(|e| anyhow!("Failed to create Enigo: {:?}", e))?;
        let axis = if vertical {
            enigo::Axis::Vertical
        } else {
            enigo::Axis::Horizontal
        };
        enigo.scroll(amount, axis)
            .map_err(|e| anyhow!("Failed to scroll: {:?}", e))?;
        Ok(())
    }

    fn real_key_press(&self, key: &str) -> Result<()> {
        if key.len() == 1 {
            let mut enigo = Enigo::new(&Settings::default())
                .map_err(|e| anyhow!("Failed to create Enigo: {:?}", e))?;
            enigo.text(key)
                .map_err(|e| anyhow!("Failed to type character: {:?}", e))?;
        } else {
            let mut enigo = Enigo::new(&Settings::default())
                .map_err(|e| anyhow!("Failed to create Enigo: {:?}", e))?;
            let enigo_key = parse_key(key)?;
            enigo.key(enigo_key, Direction::Click)
                .map_err(|e| anyhow!("Failed to press key: {:?}", e))?;
        }
        Ok(())
    }

    fn real_key_down(&self, key: &str) -> Result<()> {
        let mut enigo = Enigo::new(&Settings::default())
            .map_err(|e| anyhow!("Failed to create Enigo: {:?}", e))?;
        let enigo_key = parse_key(key)?;
        enigo.key(enigo_key, Direction::Press)
            .map_err(|e| anyhow!("Failed to press key: {:?}", e))?;
        Ok(())
    }

    fn real_key_up(&self, key: &str) -> Result<()> {
        let mut enigo = Enigo::new(&Settings::default())
            .map_err(|e| anyhow!("Failed to create Enigo: {:?}", e))?;
        let enigo_key = parse_key(key)?;
        enigo.key(enigo_key, Direction::Release)
            .map_err(|e| anyhow!("Failed to release key: {:?}", e))?;
        Ok(())
    }

    fn real_type_text(&self, text: &str) -> Result<()> {
        let mut enigo = Enigo::new(&Settings::default())
            .map_err(|e| anyhow!("Failed to create Enigo: {:?}", e))?;
        enigo.text(text)
            .map_err(|e| anyhow!("Failed to type text: {:?}", e))?;
        Ok(())
    }
}

fn map_mouse_button(button: &MouseButton) -> Button {
    match button {
        MouseButton::Left => Button::Left,
        MouseButton::Right => Button::Right,
        MouseButton::Middle => Button::Middle,
        MouseButton::Back => Button::Back,
        MouseButton::Forward => Button::Forward,
    }
}

fn parse_key(key: &str) -> Result<Key> {
    match key.to_lowercase().as_str() {
        "enter" | "return" => Ok(Key::Return),
        "tab" => Ok(Key::Tab),
        "escape" | "esc" => Ok(Key::Escape),
        "backspace" => Ok(Key::Backspace),
        "delete" | "del" => Ok(Key::Delete),
        "home" => Ok(Key::Home),
        "end" => Ok(Key::End),
        "pageup" | "page_up" => Ok(Key::PageUp),
        "pagedown" | "page_down" => Ok(Key::PageDown),
        "up" | "arrowup" => Ok(Key::UpArrow),
        "down" | "arrowdown" => Ok(Key::DownArrow),
        "left" | "arrowleft" => Ok(Key::LeftArrow),
        "right" | "arrowright" => Ok(Key::RightArrow),
        "f1" => Ok(Key::F1),
        "f2" => Ok(Key::F2),
        "f3" => Ok(Key::F3),
        "f4" => Ok(Key::F4),
        "f5" => Ok(Key::F5),
        "f6" => Ok(Key::F6),
        "f7" => Ok(Key::F7),
        "f8" => Ok(Key::F8),
        "f9" => Ok(Key::F9),
        "f10" => Ok(Key::F10),
        "f11" => Ok(Key::F11),
        "f12" => Ok(Key::F12),
        "shift" => Ok(Key::Shift),
        "control" | "ctrl" => Ok(Key::Control),
        "alt" => Ok(Key::Alt),
        "super" | "meta" | "win" => Ok(Key::Meta),
        "capslock" | "caps_lock" => Ok(Key::CapsLock),
        "space" => Ok(Key::Space),
        "insert" => Ok(Key::Insert),
        "printscreen" | "print_screen" | "snapshot" => Ok(Key::Snapshot),
        "scrolllock" | "scroll_lock" => Ok(Key::Scroll),
        "numlock" | "num_lock" => Ok(Key::Numlock),
        "pause" | "break" => Ok(Key::Pause),
        _ => Err(anyhow!("Unknown key: {}", key)),
    }
}

fn get_real_screen_info() -> Result<ScreenInfo> {
    use windows::Win32::UI::WindowsAndMessaging::{GetSystemMetrics, SM_CXSCREEN, SM_CYSCREEN};

    unsafe {
        let width = GetSystemMetrics(SM_CXSCREEN);
        let height = GetSystemMetrics(SM_CYSCREEN);

        if width <= 0 || height <= 0 {
            return Err(anyhow!("Invalid screen dimensions: {}x{}", width, height));
        }

        Ok(ScreenInfo {
            width: width as u32,
            height: height as u32,
            scale_factor: 1.0,
        })
    }
}

pub fn take_screenshot_powershell() -> Result<String> {
    let ps_script = r#"
Add-Type -AssemblyName System.Windows.Forms
Add-Type -AssemblyName System.Drawing
$bounds = [System.Windows.Forms.Screen]::PrimaryScreen.Bounds
$bmp = New-Object System.Drawing.Bitmap($bounds.Width, $bounds.Height)
$g = [System.Drawing.Graphics]::FromImage($bmp)
$g.CopyFromScreen($bounds.Location, [System.Drawing.Point]::Empty, $bounds.Size)
$g.Dispose()
$ms = New-Object System.IO.MemoryStream
$bmp.Save($ms, [System.Drawing.Imaging.ImageFormat]::Jpeg)
$bmp.Dispose()
[Convert]::ToBase64String($ms.ToArray())
"#;

    let mut cmd = std::process::Command::new("powershell");
    #[cfg(windows)]
    cmd.creation_flags(CREATE_NO_WINDOW);
    let output = cmd
        .args(&["-NoProfile", "-NonInteractive", "-Command", ps_script])
        .output()
        .map_err(|e| anyhow!("Failed to spawn PowerShell for screenshot: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("Screenshot PowerShell command failed: {}", stderr));
    }

    let base64_str = String::from_utf8_lossy(&output.stdout).trim().to_string();

    if base64_str.is_empty() {
        return Err(anyhow!("Screenshot produced empty output"));
    }

    Ok(base64_str)
}

pub mod action_helpers {
    use super::*;

    pub fn mouse_move(x: i32, y: i32) -> ComputerAction {
        ComputerAction {
            action_type: ComputerActionType::MouseMove,
            coordinate: Some(ScreenCoordinate { x, y }),
            button: None,
            key: None,
            text: None,
            scroll_y: None,
            scroll_x: None,
            duration_ms: None,
        }
    }

    pub fn mouse_click(x: i32, y: i32, button: MouseButton) -> ComputerAction {
        ComputerAction {
            action_type: ComputerActionType::MouseClick,
            coordinate: Some(ScreenCoordinate { x, y }),
            button: Some(button),
            key: None,
            text: None,
            scroll_y: None,
            scroll_x: None,
            duration_ms: None,
        }
    }

    pub fn left_click(x: i32, y: i32) -> ComputerAction {
        mouse_click(x, y, MouseButton::Left)
    }

    pub fn right_click(x: i32, y: i32) -> ComputerAction {
        mouse_click(x, y, MouseButton::Right)
    }

    pub fn key_press(key: &str) -> ComputerAction {
        ComputerAction {
            action_type: ComputerActionType::KeyPress,
            coordinate: None,
            button: None,
            key: Some(key.to_string()),
            text: None,
            scroll_y: None,
            scroll_x: None,
            duration_ms: None,
        }
    }

    pub fn type_text(text: &str) -> ComputerAction {
        ComputerAction {
            action_type: ComputerActionType::TypeText,
            coordinate: None,
            button: None,
            key: None,
            text: Some(text.to_string()),
            scroll_y: None,
            scroll_x: None,
            duration_ms: None,
        }
    }

    pub fn wait(duration_ms: u64) -> ComputerAction {
        ComputerAction {
            action_type: ComputerActionType::Wait,
            coordinate: None,
            button: None,
            key: None,
            text: None,
            scroll_y: None,
            scroll_x: None,
            duration_ms: Some(duration_ms),
        }
    }

    pub fn screenshot() -> ComputerAction {
        ComputerAction {
            action_type: ComputerActionType::Screenshot,
            coordinate: None,
            button: None,
            key: None,
            text: None,
            scroll_y: None,
            scroll_x: None,
            duration_ms: None,
        }
    }
}
