use anyhow::{anyhow, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};

const DEFAULT_BROWSERLESS_URL: &str = "http://127.0.0.1:3000";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserlessAction {
    pub action: String,
    pub selector: Option<String>,
    pub value: Option<String>,
    pub coordinate: Option<[i32; 2]>,
    pub button: Option<String>,
    pub key: Option<String>,
    pub wait_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserlessScript {
    pub url: String,
    pub wait_until: Option<String>,
    pub actions: Vec<BrowserlessAction>,
    pub viewport: Option<BrowserlessViewport>,
    pub timeout: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserlessViewport {
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserlessResponse {
    pub success: bool,
    pub screenshot: Option<String>,
    pub html: Option<String>,
    pub result: Option<serde_json::Value>,
    pub error: Option<String>,
}

pub struct BrowserlessClient {
    client: Client,
    base_url: String,
}

impl BrowserlessClient {
    pub fn new(base_url: Option<&str>) -> Self {
        Self {
            client: Client::new(),
            base_url: base_url.unwrap_or(DEFAULT_BROWSERLESS_URL).to_string(),
        }
    }

    pub async fn ping(&self) -> Result<bool> {
        match self
            .client
            .get(&format!("{}/", self.base_url))
            .timeout(std::time::Duration::from_secs(5))
            .send()
            .await
        {
            Ok(resp) => Ok(resp.status().is_success()),
            Err(_) => Ok(false),
        }
    }

    pub async fn execute_puppeteer(
        &self,
        url: &str,
        actions: Vec<BrowserlessAction>,
        viewport: Option<BrowserlessViewport>,
    ) -> Result<BrowserlessResponse> {
        let script = BrowserlessScript {
            url: url.to_string(),
            wait_until: Some("networkidle0".to_string()),
            actions,
            viewport: viewport.or_else(|| {
                Some(BrowserlessViewport {
                    width: 1920,
                    height: 1080,
                })
            }),
            timeout: Some(60000),
        };

        let resp = self
            .client
            .post(&format!("{}/puppeteer", self.base_url))
            .json(&script)
            .timeout(std::time::Duration::from_secs(90))
            .send()
            .await
            .map_err(|e| anyhow!("Browserless puppeteer request failed: {}", e))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(anyhow!(
                "Browserless returned {}: {}",
                status,
                body
            ));
        }

        resp.json()
            .await
            .map_err(|e| anyhow!("Failed to parse Browserless response: {}", e))
    }

    pub async fn execute_playwright(
        &self,
        code: &str,
    ) -> Result<BrowserlessResponse> {
        let resp = self
            .client
            .post(&format!("{}/playwright", self.base_url))
            .json(&serde_json::json!({ "code": code }))
            .timeout(std::time::Duration::from_secs(90))
            .send()
            .await
            .map_err(|e| anyhow!("Browserless playwright request failed: {}", e))?;

        resp.json()
            .await
            .map_err(|e| anyhow!("Failed to parse Browserless response: {}", e))
    }

    pub async fn take_screenshot(
        &self,
        url: &str,
    ) -> Result<Option<String>> {
        let script = BrowserlessScript {
            url: url.to_string(),
            wait_until: Some("networkidle0".to_string()),
            actions: vec![BrowserlessAction {
                action: "screenshot".to_string(),
                selector: None,
                value: None,
                coordinate: None,
                button: None,
                key: None,
                wait_ms: None,
            }],
            viewport: Some(BrowserlessViewport {
                width: 1920,
                height: 1080,
            }),
            timeout: Some(30000),
        };

        let resp = self
            .client
            .post(&format!("{}/screenshot", self.base_url))
            .json(&script)
            .timeout(std::time::Duration::from_secs(45))
            .send()
            .await
            .map_err(|e| anyhow!("Browserless screenshot failed: {}", e))?;

        let body = resp.bytes().await?;
        use base64::{engine::general_purpose::STANDARD, Engine};
        Ok(Some(STANDARD.encode(&body)))
    }

    pub async fn navigate(&self, url: &str) -> Result<BrowserlessResponse> {
        self.execute_puppeteer(
            url,
            vec![BrowserlessAction {
                action: "navigate".to_string(),
                selector: None,
                value: Some(url.to_string()),
                coordinate: None,
                button: None,
                key: None,
                wait_ms: Some(5000),
            }],
            None,
        )
        .await
    }

    pub async fn click(
        &self,
        selector: &str,
    ) -> Result<BrowserlessResponse> {
        self.execute_puppeteer(
            "about:blank",
            vec![BrowserlessAction {
                action: "click".to_string(),
                selector: Some(selector.to_string()),
                value: None,
                coordinate: None,
                button: None,
                key: None,
                wait_ms: None,
            }],
            None,
        )
        .await
    }

    pub async fn type_into(
        &self,
        selector: &str,
        text: &str,
    ) -> Result<BrowserlessResponse> {
        self.execute_puppeteer(
            "about:blank",
            vec![BrowserlessAction {
                action: "type".to_string(),
                selector: Some(selector.to_string()),
                value: Some(text.to_string()),
                coordinate: None,
                button: None,
                key: None,
                wait_ms: None,
            }],
            None,
        )
        .await
    }
}