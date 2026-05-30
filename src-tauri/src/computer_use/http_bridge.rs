use anyhow::{anyhow, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};

const DEFAULT_WORKER_URL: &str = "http://127.0.0.1:9527";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeActionRequest {
    pub action_type: String,
    pub coordinate: Option<[i32; 2]>,
    pub button: Option<String>,
    pub key: Option<String>,
    pub text: Option<String>,
    pub scroll_y: Option<i32>,
    pub scroll_x: Option<i32>,
    pub duration_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeActionResponse {
    pub success: bool,
    pub screenshot: Option<String>,
    pub error: Option<String>,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeOpenUrlRequest {
    pub url: String,
    pub wait_until: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeScreenshotResponse {
    pub success: bool,
    pub screenshot: Option<String>,
    pub error: Option<String>,
}

pub struct HttpBridgeClient {
    client: Client,
    base_url: String,
}

impl HttpBridgeClient {
    pub fn new(base_url: Option<&str>) -> Self {
        Self {
            client: Client::new(),
            base_url: base_url.unwrap_or(DEFAULT_WORKER_URL).to_string(),
        }
    }

    pub async fn ping(&self) -> Result<bool> {
        match self
            .client
            .get(&format!("{}/health", self.base_url))
            .timeout(std::time::Duration::from_secs(5))
            .send()
            .await
        {
            Ok(resp) => Ok(resp.status().is_success()),
            Err(_) => Ok(false),
        }
    }

    pub async fn execute_action(&self, action: BridgeActionRequest) -> Result<BridgeActionResponse> {
        let resp = self
            .client
            .post(&format!("{}/execute", self.base_url))
            .json(&action)
            .timeout(std::time::Duration::from_secs(30))
            .send()
            .await
            .map_err(|e| anyhow!("HTTP Bridge execute failed: {}", e))?;

        resp.json()
            .await
            .map_err(|e| anyhow!("Failed to parse HTTP Bridge response: {}", e))
    }

    pub async fn mouse_click(
        &self,
        x: i32,
        y: i32,
        button: Option<&str>,
    ) -> Result<BridgeActionResponse> {
        self.execute_action(BridgeActionRequest {
            action_type: "mouse_click".to_string(),
            coordinate: Some([x, y]),
            button: Some(button.unwrap_or("left").to_string()),
            key: None,
            text: None,
            scroll_y: None,
            scroll_x: None,
            duration_ms: None,
        })
        .await
    }

    pub async fn mouse_move(&self, x: i32, y: i32) -> Result<BridgeActionResponse> {
        self.execute_action(BridgeActionRequest {
            action_type: "mouse_move".to_string(),
            coordinate: Some([x, y]),
            button: None,
            key: None,
            text: None,
            scroll_y: None,
            scroll_x: None,
            duration_ms: None,
        })
        .await
    }

    pub async fn type_text(&self, text: &str) -> Result<BridgeActionResponse> {
        self.execute_action(BridgeActionRequest {
            action_type: "type_text".to_string(),
            coordinate: None,
            button: None,
            key: None,
            text: Some(text.to_string()),
            scroll_y: None,
            scroll_x: None,
            duration_ms: None,
        })
        .await
    }

    pub async fn key_press(&self, key: &str) -> Result<BridgeActionResponse> {
        self.execute_action(BridgeActionRequest {
            action_type: "key_press".to_string(),
            coordinate: None,
            button: None,
            key: Some(key.to_string()),
            text: None,
            scroll_y: None,
            scroll_x: None,
            duration_ms: None,
        })
        .await
    }

    pub async fn take_screenshot(&self) -> Result<BridgeScreenshotResponse> {
        let resp = self
            .client
            .get(&format!("{}/screenshot", self.base_url))
            .timeout(std::time::Duration::from_secs(15))
            .send()
            .await
            .map_err(|e| anyhow!("HTTP Bridge screenshot failed: {}", e))?;

        resp.json()
            .await
            .map_err(|e| anyhow!("Failed to parse screenshot response: {}", e))
    }

    pub async fn open_url(&self, url: &str) -> Result<BridgeActionResponse> {
        let resp = self
            .client
            .post(&format!("{}/open_url", self.base_url))
            .json(&BridgeOpenUrlRequest {
                url: url.to_string(),
                wait_until: Some("load".to_string()),
            })
            .timeout(std::time::Duration::from_secs(60))
            .send()
            .await
            .map_err(|e| anyhow!("HTTP Bridge open_url failed: {}", e))?;

        resp.json()
            .await
            .map_err(|e| anyhow!("Failed to parse open_url response: {}", e))
    }
}