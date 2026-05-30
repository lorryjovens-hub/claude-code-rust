use anyhow::{anyhow, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

const COMPOSIO_API_BASE: &str = "https://backend.composio.dev";
const COMPOSIO_SERVER_NAME: &str = "composio";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComposioConnectorProfile {
    pub connector_id: String,
    pub toolkit_slug: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComposioConfig {
    pub api_key: String,
}

impl Default for ComposioConfig {
    fn default() -> Self {
        Self {
            api_key: String::new(),
        }
    }
}

fn composio_connector_profiles() -> HashMap<&'static str, ComposioConnectorProfile> {
    let mut map = HashMap::new();
    let profiles = [
        ("github", "github", "github"),
        ("google-drive", "google-drive", "googledrive"),
        ("gmail", "gmail", "gmail"),
        ("google-calendar", "google-calendar", "googlecalendar"),
        ("slack", "slack", "slack"),
        ("notion", "notion", "notion"),
        ("jira", "jira", "jira"),
        ("confluence", "confluence", "confluence"),
        ("linear", "linear", "linear"),
        ("airtable", "airtable", "airtable"),
        ("asana", "asana", "asana"),
        ("microsoft-teams", "microsoft-teams", "microsoft_teams"),
        ("onedrive", "onedrive", "one_drive"),
        ("figma", "figma", "figma"),
        ("zoom", "zoom", "zoom"),
        ("dropbox", "dropbox", "dropbox"),
        ("box", "box", "box"),
        ("salesforce", "salesforce", "salesforce"),
        ("hubspot", "hubspot", "hubspot"),
        ("intercom", "intercom", "intercom"),
        ("miro", "miro", "miro"),
        ("monday", "monday", "monday"),
        ("trello", "trello", "trello"),
        ("zendesk", "zendesk", "zendesk"),
        ("gitlab", "gitlab", "gitlab"),
        ("bitbucket", "bitbucket", "bitbucket"),
    ];

    for (id, connector_id, slug) in profiles {
        map.insert(id, ComposioConnectorProfile {
            connector_id: connector_id.to_string(),
            toolkit_slug: slug.to_string(),
        });
    }
    map
}

pub fn default_connector_ids() -> Vec<&'static str> {
    vec![
        "github", "google-drive", "gmail", "google-calendar",
        "slack", "notion", "jira", "confluence", "linear",
        "airtable", "asana", "microsoft-teams", "onedrive",
        "figma", "zoom", "dropbox", "box", "salesforce",
        "hubspot", "intercom", "miro", "monday", "trello",
        "zendesk", "gitlab", "bitbucket",
    ]
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComposioSession {
    pub session_id: Option<String>,
    pub mcp_url: Option<String>,
    pub updated_at: Option<String>,
    pub user_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComposioConnectorStatus {
    pub available: bool,
    pub connected: bool,
    pub connected_account_id: Option<String>,
    pub installed: bool,
    pub server_name: Option<String>,
    pub toolkit_slug: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComposioSessionStore(HashMap<String, ComposioSession>);

impl Default for ComposioSessionStore {
    fn default() -> Self {
        Self(HashMap::new())
    }
}

pub struct ComposioManager {
    client: Client,
    config_path: PathBuf,
    session_store_path: PathBuf,
    config: ComposioConfig,
    session_store: ComposioSessionStore,
}

impl ComposioManager {
    pub fn new(data_dir: PathBuf) -> Self {
        let config_path = data_dir.join("composio_config.json");
        let session_store_path = data_dir.join("composio_sessions.json");

        let mut manager = Self {
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .expect("Failed to create HTTP client"),
            config_path,
            session_store_path,
            config: ComposioConfig::default(),
            session_store: ComposioSessionStore::default(),
        };

        manager.load_config();
        manager.load_session_store();
        manager
    }

    fn load_config(&mut self) {
        if self.config_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&self.config_path) {
                if let Ok(config) = serde_json::from_str::<ComposioConfig>(&content) {
                    self.config = config;
                }
            }
        }
    }

    fn save_config(&self) -> Result<()> {
        if let Some(parent) = self.config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(&self.config)?;
        std::fs::write(&self.config_path, content)?;
        Ok(())
    }

    fn load_session_store(&mut self) {
        if self.session_store_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&self.session_store_path) {
                if let Ok(store) = serde_json::from_str::<ComposioSessionStore>(&content) {
                    self.session_store = store;
                }
            }
        }
    }

    fn save_session_store(&self) -> Result<()> {
        if let Some(parent) = self.session_store_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(&self.session_store)?;
        std::fs::write(&self.session_store_path, content)?;
        Ok(())
    }

    pub fn get_resolved_api_key(&self) -> String {
        if !self.config.api_key.is_empty() {
            return self.config.api_key.clone();
        }
        std::env::var("COMPOSIO_API_KEY")
            .or_else(|_| std::env::var("COMPOSIO_PROJECT_API_KEY"))
            .unwrap_or_default()
    }

    pub fn set_api_key(&mut self, api_key: String) -> Result<()> {
        self.config.api_key = api_key;
        self.save_config()
    }

    pub fn get_config(&self) -> &ComposioConfig {
        &self.config
    }

    pub fn get_session(&self, user_id: &str) -> Option<&ComposioSession> {
        self.session_store.0.get(user_id)
    }

    pub fn list_connector_ids(&self) -> Vec<String> {
        default_connector_ids().iter().map(|s| s.to_string()).collect()
    }

    pub fn get_connector_profile(&self, connector_id: &str) -> Option<ComposioConnectorProfile> {
        composio_connector_profiles().get(connector_id).cloned()
    }

    pub async fn create_session(&mut self, user_id: &str, toolkit_slugs: Vec<String>) -> Result<ComposioSession> {
        let api_key = self.get_resolved_api_key();
        if api_key.is_empty() {
            return Err(anyhow!("Missing Composio API key"));
        }

        let url = format!("{}/api/v3/tool_router/session", COMPOSIO_API_BASE);
        let mut body = serde_json::json!({
            "user_id": user_id,
        });

        if !toolkit_slugs.is_empty() {
            body["toolkits"] = serde_json::json!({ "enable": toolkit_slugs });
        }

        let response = self.client
            .post(&url)
            .header("Content-Type", "application/json")
            .header("x-api-key", &api_key)
            .json(&body)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(anyhow!("Composio session creation failed {}: {}", status, text));
        }

        let payload: serde_json::Value = response.json().await?;

        let session = ComposioSession {
            session_id: payload.get("session_id").and_then(|v| v.as_str()).map(String::from),
            mcp_url: payload.pointer("/mcp/url").and_then(|v| v.as_str()).map(String::from),
            updated_at: Some(chrono::Utc::now().to_rfc3339()),
            user_id: Some(user_id.to_string()),
        };

        self.session_store.0.insert(user_id.to_string(), session.clone());
        self.save_session_store()?;

        Ok(session)
    }

    pub async fn get_session_info(&self, session_id: &str) -> Result<ComposioSession> {
        let api_key = self.get_resolved_api_key();
        if api_key.is_empty() {
            return Err(anyhow!("Missing Composio API key"));
        }

        let url = format!("{}/api/v3/tool_router/session/{}", COMPOSIO_API_BASE, session_id);
        let response = self.client
            .get(&url)
            .header("x-api-key", &api_key)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(anyhow!("Composio get session failed {}: {}", status, text));
        }

        let payload: serde_json::Value = response.json().await?;

        Ok(ComposioSession {
            session_id: payload.get("session_id").and_then(|v| v.as_str()).map(String::from)
                .or(Some(session_id.to_string())),
            mcp_url: payload.pointer("/mcp/url").and_then(|v| v.as_str()).map(String::from),
            updated_at: Some(chrono::Utc::now().to_rfc3339()),
            user_id: None,
        })
    }

    pub async fn get_toolkits(&self, session_id: &str, toolkit_slugs: Option<Vec<String>>) -> Result<Vec<serde_json::Value>> {
        let api_key = self.get_resolved_api_key();
        if api_key.is_empty() {
            return Err(anyhow!("Missing Composio API key"));
        }

        let url = format!("{}/api/v3.1/tool_router/session/{}/toolkits", COMPOSIO_API_BASE, session_id);
        let mut request = self.client
            .get(&url)
            .header("x-api-key", &api_key);

        if let Some(slugs) = toolkit_slugs {
            request = request.query(&[("toolkits", slugs.join(","))]);
        }

        let response = request.send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(anyhow!("Composio get toolkits failed {}: {}", status, text));
        }

        let payload: serde_json::Value = response.json().await?;
        Ok(payload.get("items")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default())
    }

    pub async fn create_connection_link(&self, session_id: &str, toolkit_slug: &str, callback_url: Option<&str>) -> Result<serde_json::Value> {
        let api_key = self.get_resolved_api_key();
        if api_key.is_empty() {
            return Err(anyhow!("Missing Composio API key"));
        }

        let url = format!("{}/api/v3.1/tool_router/session/{}/link", COMPOSIO_API_BASE, session_id);
        let mut body = serde_json::json!({
            "toolkit": toolkit_slug,
        });

        if let Some(cb_url) = callback_url {
            body["callback_url"] = serde_json::json!(cb_url);
        }

        let response = self.client
            .post(&url)
            .header("Content-Type", "application/json")
            .header("x-api-key", &api_key)
            .json(&body)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(anyhow!("Composio create link failed {}: {}", status, text));
        }

        Ok(response.json().await?)
    }

    pub fn build_mcp_server_config(&self, session: &ComposioSession) -> Option<serde_json::Value> {
        let mcp_url = session.mcp_url.as_deref()?;
        let api_key = self.get_resolved_api_key();

        Some(serde_json::json!({
            "type": "http",
            "url": mcp_url,
            "headers": {
                "x-api-key": api_key,
            }
        }))
    }

    pub fn get_connector_statuses(&self, connector_ids: &[&str]) -> HashMap<String, ComposioConnectorStatus> {
        let profiles = composio_connector_profiles();
        let mut statuses = HashMap::new();

        for connector_id in connector_ids {
            let profile = profiles.get(connector_id);
            statuses.insert(connector_id.to_string(), ComposioConnectorStatus {
                available: profile.is_some(),
                connected: false,
                connected_account_id: None,
                installed: profile.is_some(),
                server_name: if profile.is_some() { Some(COMPOSIO_SERVER_NAME.to_string()) } else { None },
                toolkit_slug: profile.map(|p| p.toolkit_slug.clone()),
            });
        }

        statuses
    }
}

pub struct McpManagedConnector {
    profiles: HashMap<&'static str, McpManagedProfile>,
}

#[derive(Debug, Clone)]
struct McpManagedProfile {
    connector_ids: Vec<String>,
    server_name: String,
    server_url: String,
}

impl McpManagedConnector {
    pub fn new() -> Self {
        let mut profiles = HashMap::new();

        profiles.insert("figma", McpManagedProfile {
            connector_ids: vec!["figma".to_string()],
            server_name: "figma".to_string(),
            server_url: "https://mcp.figma.com/mcp".to_string(),
        });

        profiles.insert("notion", McpManagedProfile {
            connector_ids: vec!["notion".to_string()],
            server_name: "notion".to_string(),
            server_url: "https://mcp.notion.com/mcp".to_string(),
        });

        profiles.insert("linear", McpManagedProfile {
            connector_ids: vec!["linear".to_string()],
            server_name: "linear".to_string(),
            server_url: "https://mcp.linear.app/mcp".to_string(),
        });

        profiles.insert("atlassian", McpManagedProfile {
            connector_ids: vec!["jira".to_string(), "confluence".to_string()],
            server_name: "atlassian".to_string(),
            server_url: "https://mcp.atlassian.com/v1/mcp".to_string(),
        });

        Self { profiles }
    }

    pub fn get_profile_for_connector(&self, connector_id: &str) -> Option<&McpManagedProfile> {
        self.profiles.values().find(|p| p.connector_ids.iter().any(|c| c == connector_id))
    }

    pub fn list_profiles(&self) -> &HashMap<&'static str, McpManagedProfile> {
        &self.profiles
    }

    pub fn build_server_config(&self, profile_key: &str) -> Option<serde_json::Value> {
        self.profiles.get(profile_key).map(|profile| {
            serde_json::json!({
                "type": "http",
                "url": profile.server_url,
            })
        })
    }
}
