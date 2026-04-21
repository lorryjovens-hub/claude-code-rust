//! JavaScript Bridge - Utilities for JS interop

use wasm_bindgen::prelude::*;
use js_sys::{Promise, Function, Object, Reflect};

/// Bridge for calling JavaScript functions from Rust
pub struct JsBridge;

impl JsBridge {
    /// Call a JavaScript function with arguments
    pub fn call_function(name: &str, args: &[JsValue]) -> Result<JsValue, JsValue> {
        let window = web_sys::window().ok_or("No window object")?;
        let func = Reflect::get(&window, &JsValue::from_str(name))?;
        
        if func.is_undefined() || func.is_null() {
            return Err(JsValue::from_str(&format!("Function {} not found", name)));
        }

        let func: Function = func.dyn_into()?;
        func.apply(&JsValue::NULL, &js_sys::Array::from(args))
    }

    /// Call a method on a JavaScript object
    pub fn call_method(
        obj: &JsValue,
        method: &str,
        args: &[JsValue],
    ) -> Result<JsValue, JsValue> {
        let method = Reflect::get(obj, &JsValue::from_str(method))?;
        
        if method.is_undefined() || method.is_null() {
            return Err(JsValue::from_str(&format!("Method {} not found", method)));
        }

        let func: Function = method.dyn_into()?;
        func.apply(obj, &js_sys::Array::from(args))
    }

    /// Get a property from a JavaScript object
    pub fn get_property(obj: &JsValue, prop: &str) -> Result<JsValue, JsValue> {
        Reflect::get(obj, &JsValue::from_str(prop))
    }

    /// Set a property on a JavaScript object
    pub fn set_property(obj: &JsValue, prop: &str, value: &JsValue) -> Result<(), JsValue> {
        Reflect::set(obj, &JsValue::from_str(prop), value)?;
        Ok(())
    }

    /// Create a new JavaScript object
    pub fn create_object() -> Object {
        Object::new()
    }

    /// Convert a Rust value to a JavaScript Promise
    pub fn to_promise<F, T>(f: F) -> Promise
    where
        F: std::future::Future<Output = Result<T, JsValue>> + 'static,
        T: Into<JsValue>,
    {
        let future = async move {
            match f.await {
                Ok(value) => Ok(value.into()),
                Err(e) => Err(e),
            }
        };

        wasm_bindgen_futures::future_to_promise(future)
    }

    /// Log a message to the browser console
    pub fn console_log(message: &str) {
        web_sys::console::log_1(&JsValue::from_str(message));
    }

    /// Log an error to the browser console
    pub fn console_error(message: &str) {
        web_sys::console::error_1(&JsValue::from_str(message));
    }

    /// Show a browser alert
    pub fn alert(message: &str) {
        if let Some(window) = web_sys::window() {
            window.alert_with_message(message).ok();
        }
    }

    /// Show a browser confirm dialog
    pub fn confirm(message: &str) -> bool {
        web_sys::window()
            .and_then(|w| w.confirm_with_message(message).ok())
            .unwrap_or(false)
    }

    /// Show a browser prompt dialog
    pub fn prompt(message: &str, default: Option<&str>) -> Option<String> {
        web_sys::window()
            .and_then(|w| {
                w.prompt_with_message_and_default(message, default.unwrap_or(""))
                    .ok()
                    .flatten()
            })
    }

    /// Get the current URL
    pub fn current_url() -> Option<String> {
        web_sys::window()
            .and_then(|w| w.location().href().ok())
    }

    /// Navigate to a URL
    pub fn navigate(url: &str) -> Result<(), JsValue> {
        if let Some(window) = web_sys::window() {
            window.location().set_href(url)?;
        }
        Ok(())
    }

    /// Reload the page
    pub fn reload() {
        if let Some(window) = web_sys::window() {
            window.location().reload().ok();
        }
    }

    /// Open a new window/tab
    pub fn open(url: &str, target: &str) -> Option<web_sys::Window> {
        web_sys::window()
            .and_then(|w| w.open_with_url_and_target(url, target).ok().flatten())
    }

    /// Get the user agent string
    pub fn user_agent() -> Option<String> {
        web_sys::window()
            .map(|w| w.navigator())
            .map(|n| n.user_agent().ok())
            .flatten()
    }

    /// Check if the page is visible
    pub fn is_visible() -> bool {
        if let Some(document) = web_sys::window().and_then(|w| w.document()) {
            document.visibility_state() == web_sys::VisibilityState::Visible
        } else {
            false
        }
    }

    /// Set a timeout (in milliseconds)
    pub fn set_timeout<F>(callback: F, delay_ms: i32) -> i32
    where
        F: FnOnce() + 'static,
    {
        let closure = Closure::once_into_js(callback);
        web_sys::window()
            .map(|w| w.set_timeout_with_callback_and_timeout_and_arguments_0(
                closure.as_ref().unchecked_ref(),
                delay_ms,
            ))
            .unwrap_or(Ok(-1))
            .unwrap_or(-1)
    }

    /// Clear a timeout
    pub fn clear_timeout(handle: i32) {
        if let Some(window) = web_sys::window() {
            window.clear_timeout_with_handle(handle);
        }
    }

    /// Set an interval (in milliseconds)
    pub fn set_interval<F>(callback: F, interval_ms: i32) -> i32
    where
        F: FnMut() + 'static,
    {
        let closure = Closure::wrap(Box::new(callback) as Box<dyn FnMut()>);
        let handle = web_sys::window()
            .map(|w| w.set_interval_with_callback_and_timeout_and_arguments_0(
                closure.as_ref().unchecked_ref(),
                interval_ms,
            ))
            .unwrap_or(Ok(-1))
            .unwrap_or(-1);
        
        closure.forget();
        handle
    }

    /// Clear an interval
    pub fn clear_interval(handle: i32) {
        if let Some(window) = web_sys::window() {
            window.clear_interval_with_handle(handle);
        }
    }

    /// Request animation frame
    pub fn request_animation_frame<F>(callback: F) -> i32
    where
        F: FnMut(f64) + 'static,
    {
        let closure = Closure::wrap(Box::new(callback) as Box<dyn FnMut(f64)>);
        let handle = web_sys::window()
            .and_then(|w| w.request_animation_frame(closure.as_ref().unchecked_ref()).ok())
            .unwrap_or(0);
        
        closure.forget();
        handle
    }

    /// Cancel animation frame
    pub fn cancel_animation_frame(handle: i32) {
        if let Some(window) = web_sys::window() {
            window.cancel_animation_frame(handle);
        }
    }

    /// Copy text to clipboard
    pub async fn copy_to_clipboard(text: &str) -> Result<(), JsValue> {
        let window = web_sys::window().ok_or("No window")?;
        let navigator = window.navigator();
        let clipboard = navigator.clipboard().ok_or("Clipboard not available")?;
        
        let promise = clipboard.write_text(text);
        wasm_bindgen_futures::JsFuture::from(promise).await?;
        
        Ok(())
    }

    /// Read text from clipboard
    pub async fn read_from_clipboard() -> Result<String, JsValue> {
        let window = web_sys::window().ok_or("No window")?;
        let navigator = window.navigator();
        let clipboard = navigator.clipboard().ok_or("Clipboard not available")?;
        
        let promise = clipboard.read_text();
        let result = wasm_bindgen_futures::JsFuture::from(promise).await?;
        
        result.as_string().ok_or_else(|| JsValue::from_str("Failed to read clipboard"))
    }

    /// Get the current language
    pub fn language() -> Option<String> {
        web_sys::window()
            .map(|w| w.navigator())
            .map(|n| n.language())
    }

    /// Get the preferred languages
    pub fn languages() -> Vec<String> {
        web_sys::window()
            .map(|w| w.navigator())
            .map(|n| {
                let langs = n.languages();
                let mut result = Vec::new();
                for i in 0..langs.length() {
                    if let Some(lang) = langs.get(i).as_string() {
                        result.push(lang);
                    }
                }
                result
            })
            .unwrap_or_default()
    }
}

/// Utility trait for converting between Rust and JavaScript types
pub trait JsConvert {
    fn to_js_value(&self) -> JsValue;
    fn from_js_value(value: &JsValue) -> Option<Self> where Self: Sized;
}

impl JsConvert for String {
    fn to_js_value(&self) -> JsValue {
        JsValue::from_str(self)
    }

    fn from_js_value(value: &JsValue) -> Option<Self> {
        value.as_string()
    }
}

impl JsConvert for bool {
    fn to_js_value(&self) -> JsValue {
        JsValue::from_bool(*self)
    }

    fn from_js_value(value: &JsValue) -> Option<Self> {
        value.as_bool()
    }
}

impl JsConvert for f64 {
    fn to_js_value(&self) -> JsValue {
        JsValue::from_f64(*self)
    }

    fn from_js_value(value: &JsValue) -> Option<Self> {
        value.as_f64()
    }
}

impl JsConvert for i32 {
    fn to_js_value(&self) -> JsValue {
        JsValue::from_f64(*self as f64)
    }

    fn from_js_value(value: &JsValue) -> Option<Self> {
        value.as_f64().map(|f| f as i32)
    }
}

/// Event listener helper
pub struct EventListener {
    target: web_sys::EventTarget,
    event_type: String,
    closure: Closure<dyn FnMut(web_sys::Event)>,
}

impl EventListener {
    /// Add an event listener
    pub fn new<F>(
        target: &web_sys::EventTarget,
        event_type: &str,
        callback: F,
    ) -> Result<Self, JsValue>
    where
        F: FnMut(web_sys::Event) + 'static,
    {
        let closure = Closure::wrap(Box::new(callback) as Box<dyn FnMut(_)>);
        
        target.add_event_listener_with_callback(
            event_type,
            closure.as_ref().unchecked_ref(),
        )?;

        Ok(Self {
            target: target.clone(),
            event_type: event_type.to_string(),
            closure,
        })
    }

    /// Remove the event listener
    pub fn remove(&self) -> Result<(), JsValue> {
        self.target.remove_event_listener_with_callback(
            &self.event_type,
            self.closure.as_ref().unchecked_ref(),
        )
    }
}

impl Drop for EventListener {
    fn drop(&mut self) {
        let _ = self.remove();
    }
}
