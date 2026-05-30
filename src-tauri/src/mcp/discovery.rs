use crate::mcp::types::*;
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;
use tokio::fs;
use tracing::{info, warn};

pub struct McpDiscovery {
    config_dirs: Vec<PathBuf>,
    mdns_enabled: bool,
}

impl McpDiscovery {
    pub fn new() -> Self {
        Self {
            config_dirs: vec![
                dirs::home_dir().unwrap_or_default().join(".mcp"),
                std::env::current_dir().unwrap_or_default().join(".mcp.json"),
                PathBuf::from("/etc/mcp"),
            ],
            mdns_enabled: true,
        }
    }

    pub fn with_mdns(mut self, enabled: bool) -> Self {
        self.mdns_enabled = enabled;
        self
    }

    pub async fn discover_all(&self) -> Vec<McpServerInfo> {
        let mut servers = Vec::new();

        if let Ok(local) = self.discover_from_config().await {
            servers.extend(local);
        }

        if let Ok(path) = self.discover_from_path().await {
            servers.extend(path);
        }

        if let Ok(npm) = self.discover_from_npm().await {
            servers.extend(npm);
        }

        if self.mdns_enabled {
            if let Ok(mdns) = self.discover_from_mdns().await {
                servers.extend(mdns);
            }
        }

        servers.sort_by(|a, b| a.id.cmp(&b.id));
        servers.dedup_by(|a, b| a.id == b.id);

        servers
    }

    async fn discover_from_config(&self) -> Result<Vec<McpServerInfo>, String> {
        let mut servers = Vec::new();

        for dir in &self.config_dirs {
            let config_path = if dir.is_dir() {
                dir.join("servers.json")
            } else {
                dir.clone()
            };

            if !config_path.exists() {
                continue;
            }

            let content = fs::read_to_string(&config_path)
                .await
                .map_err(|e| format!("Failed to read config: {}", e))?;

            let config: McpServersConfig = serde_json::from_str(&content)
                .map_err(|e| format!("Failed to parse config: {}", e))?;

            for server_config in config.servers {
                servers.push(McpServerInfo {
                    id: format!("config_{}", server_config.name),
                    name: server_config.name,
                    version: "1.0.0".into(),
                    description: format!("From config: {}", config_path.display()),
                    transport: server_config.transport,
                    capabilities: ServerCapabilities::default(),
                    metadata: HashMap::from([
                        ("source".into(), "config".into()),
                        ("config_path".into(), config_path.display().to_string()),
                    ]),
                    health: McpHealth::Disconnected,
                });
            }

            info!("Discovered {} MCP servers from config: {}", servers.len(), config_path.display());
        }

        Ok(servers)
    }

    async fn discover_from_path(&self) -> Result<Vec<McpServerInfo>, String> {
        let mut servers = Vec::new();

        let path_env = std::env::var("PATH").unwrap_or_default();
        if path_env.is_empty() {
            return Ok(servers);
        }

        for dir in path_env.split(';').chain(path_env.split(':')) {
            let dir_path = PathBuf::from(dir);
            if !dir_path.is_dir() {
                continue;
            }

            let mut read_dir = match fs::read_dir(&dir_path).await {
                Ok(d) => d,
                Err(_) => continue,
            };

            while let Ok(Some(entry)) = read_dir.next_entry().await {
                let path = entry.path();
                let filename = path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("");

                if filename.starts_with("mcp-") {
                    let is_executable = {
                        #[cfg(unix)]
                        {
                            use std::os::unix::fs::PermissionsExt;
                            entry.metadata().await
                                .map(|m| m.permissions().mode() & 0o111 != 0)
                                .unwrap_or(false)
                        }
                        #[cfg(windows)]
                        {
                            path.extension()
                                .map(|e| e == "exe" || e == "bat" || e == "cmd")
                                .unwrap_or(false)
                        }
                    };

                    if is_executable {
                        let name = filename.strip_prefix("mcp-").unwrap_or(filename);
                        servers.push(McpServerInfo {
                            id: format!("path_{}", filename),
                            name: name.to_string(),
                            version: "1.0.0".into(),
                            description: format!("From PATH: {}", path.display()),
                            transport: McpTransport::Stdio {
                                command: path.display().to_string(),
                                args: vec![],
                                env: HashMap::new(),
                            },
                            capabilities: ServerCapabilities::default(),
                            metadata: HashMap::from([
                                ("source".into(), "path".into()),
                                ("executable".into(), path.display().to_string()),
                            ]),
                            health: McpHealth::Disconnected,
                        });
                    }
                }
            }
        }

        Ok(servers)
    }

    async fn discover_from_npm(&self) -> Result<Vec<McpServerInfo>, String> {
        let output = tokio::process::Command::new("npm")
            .args(["list", "-g", "--json", "--depth=0"])
            .output()
            .await
            .map_err(|e| format!("npm list failed: {}", e))?;

        if !output.status.success() {
            return Ok(vec![]);
        }

        let npm_output: serde_json::Value = serde_json::from_slice(&output.stdout)
            .map_err(|e| format!("npm parse failed: {}", e))?;

        let mut servers = Vec::new();

        if let Some(dependencies) = npm_output.get("dependencies").and_then(|d| d.as_object()) {
            for (pkg_name, pkg_info) in dependencies {
                if pkg_name.starts_with("@modelcontextprotocol/")
                    || pkg_name.contains("mcp-server")
                    || pkg_name.contains("mcp-client")
                {
                    let server_name = pkg_name.split('/').last().unwrap_or(pkg_name);
                    let server_path = pkg_info.get("resolved")
                        .and_then(|r| r.as_str())
                        .unwrap_or("");

                    servers.push(McpServerInfo {
                        id: format!("npm_{}", pkg_name),
                        name: server_name.to_string(),
                        version: pkg_info.get("version")
                            .and_then(|v| v.as_str())
                            .unwrap_or("unknown")
                            .to_string(),
                        description: format!("npm global package: {}", pkg_name),
                        transport: McpTransport::Stdio {
                            command: format!("npx {}", pkg_name),
                            args: vec![],
                            env: HashMap::new(),
                        },
                        capabilities: ServerCapabilities::default(),
                        metadata: HashMap::from([
                            ("source".into(), "npm".into()),
                            ("package".into(), pkg_name.to_string()),
                            ("path".into(), server_path.to_string()),
                        ]),
                        health: McpHealth::Disconnected,
                    });
                }
            }
        }

        Ok(servers)
    }

    async fn discover_from_mdns(&self) -> Result<Vec<McpServerInfo>, String> {
        let mut servers = Vec::new();

        let mdns = mdns_sd::ServiceDaemon::new()
            .map_err(|e| format!("mDNS init failed: {}", e))?;

        let receiver = mdns.browse("_mcp._tcp.local.")
            .map_err(|e| format!("mDNS browse failed: {}", e))?;

        let timeout = tokio::time::sleep(Duration::from_secs(3));
        tokio::pin!(timeout);

        loop {
            tokio::select! {
                event = receiver.recv_async() => {
                    match event {
                        Ok(mdns_sd::ServiceEvent::ServiceResolved(info)) => {
                            let properties: HashMap<String, String> = info.get_properties()
                                .iter()
                                .map(|p| (p.key().to_string(), p.val_str().to_string()))
                                .collect();

                            let transport_type = properties.get("transport")
                                .map(|s: &String| s.as_str())
                                .unwrap_or("stdio");

                            let transport = match transport_type {
                                "sse" => McpTransport::Sse {
                                    url: format!("http://{}:{}/sse", info.get_hostname(), info.get_port()),
                                    headers: HashMap::new(),
                                },
                                "ws" | "websocket" => McpTransport::WebSocket {
                                    url: format!("ws://{}:{}", info.get_hostname(), info.get_port()),
                                    headers: HashMap::new(),
                                },
                                _ => McpTransport::Stdio {
                                    command: properties.get("command")
                                        .cloned()
                                        .unwrap_or_else(|| "mcp-server".into()),
                                    args: properties.get("args")
                                        .map(|a: &String| a.split(' ').map(String::from).collect())
                                        .unwrap_or_default(),
                                    env: HashMap::new(),
                                },
                            };

                            servers.push(McpServerInfo {
                                id: format!("mdns_{}", info.get_fullname()),
                                name: info.get_fullname().strip_suffix(&format!(".{}", info.get_type()))
                                    .unwrap_or_else(|| info.get_fullname())
                                    .to_string(),
                                version: properties.get("version")
                                    .cloned()
                                    .unwrap_or_else(|| "1.0.0".into()),
                                description: format!("LAN MCP server: {}:{}",
                                    info.get_hostname(), info.get_port()),
                                transport,
                                capabilities: ServerCapabilities::default(),
                                metadata: properties,
                                health: McpHealth::Disconnected,
                            });
                        }
                        Ok(mdns_sd::ServiceEvent::SearchStopped(_)) => break,
                        Err(e) => warn!("mDNS receive error: {}", e),
                        _ => {}
                    }
                }
                _ = &mut timeout => break,
            }
        }

        let _ = mdns.stop_browse("_mcp._tcp.local.");

        Ok(servers)
    }
}

impl Default for McpDiscovery {
    fn default() -> Self {
        Self::new()
    }
}