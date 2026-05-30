use crate::execution::{
    ExecutionEvent, ParallelExecutionEngine, ToolCallRequest, ToolCallResult,
};
use crate::mcp::{McpRegistry, McpRegistryEvent, McpToolBridge};
use std::sync::Arc;
use tokio::sync::{broadcast, Mutex};
use tracing::{error, warn};

pub struct AppContext {
    pub mcp_registry: Arc<McpRegistry>,
    pub execution_engine: Arc<ParallelExecutionEngine>,
    bridge: Arc<Mutex<McpToolBridge>>,
    event_tx: broadcast::Sender<AppEvent>,
}

#[derive(Debug, Clone)]
pub enum AppEvent {
    McpEvent(McpRegistryEvent),
    ExecutionEvent(ExecutionEvent),
    ToolSyncComplete { count: usize },
}

impl AppContext {
    pub async fn initialize(max_mcp_connections: usize) -> Result<Self, String> {
        let (event_tx, _) = broadcast::channel(10000);

        let mcp_registry = Arc::new(McpRegistry::new(max_mcp_connections));

        let bridge = Arc::new(Mutex::new(
            McpToolBridge::new(mcp_registry.clone()),
        ));

        let execution_engine = Arc::new(
            crate::execution::create_execution_engine(4).await,
        );

        {
            let bridge_arc = bridge.clone();
            let event_tx_clone = event_tx.clone();
            let mcp_registry_clone = mcp_registry.clone();
            let execution_engine_clone = execution_engine.clone();
            tokio::spawn(async move {
                let mut mcp_rx = mcp_registry_clone.subscribe();
                let mut exec_rx = execution_engine_clone.subscribe();

                loop {
                    tokio::select! {
                        Ok(event) = mcp_rx.recv() => {
                            let _ = event_tx_clone.send(AppEvent::McpEvent(event));
                        }
                        Ok(event) = exec_rx.recv() => {
                            let _ = event_tx_clone.send(AppEvent::ExecutionEvent(event));
                        }
                        else => break,
                    }
                }
            });
        }

        let bridge_for_sync = bridge.clone();
        tokio::spawn(async move {
            let mut bridge = bridge_for_sync.lock().await;
            bridge.auto_sync_loop().await;
        });

        Ok(Self {
            mcp_registry,
            execution_engine,
            bridge,
            event_tx,
        })
    }

    pub async fn discover_and_connect_mcp(&self) -> Result<usize, String> {
        let servers = self.mcp_registry.discover().await;
        let count = servers.len();

        for server in &servers {
            if let Err(e) = self.mcp_registry.connect(&server.id).await {
                warn!(target: "app_context", "Failed to connect MCP server {}: {}", server.name, e);
            }
        }

        Ok(count)
    }

    pub async fn sync_mcp_tools(&self) -> Result<usize, String> {
        let mut bridge = self.bridge.lock().await;
        let count = bridge.sync_all().await?;

        let _ = self.event_tx.send(AppEvent::ToolSyncComplete { count });

        Ok(count)
    }

    pub async fn execute_tools(
        &self,
        tool_calls: Vec<ToolCallRequest>,
    ) -> Vec<ToolCallResult> {
        self.execution_engine
            .execute(tool_calls)
            .await
            .unwrap_or_else(|e| {
                error!(target: "app_context", "Tool execution failed: {}", e.message);
                vec![]
            })
    }

    pub fn subscribe(&self) -> broadcast::Receiver<AppEvent> {
        self.event_tx.subscribe()
    }

    pub fn execution_subscribe(&self) -> broadcast::Receiver<ExecutionEvent> {
        self.execution_engine.subscribe()
    }

    pub async fn get_stats(&self) -> AppStats {
        let engine_stats = self.execution_engine.get_statistics().await;
        let mcp_health = self.mcp_registry.health_check().await;

        AppStats {
            mcp_servers_total: mcp_health.total,
            mcp_servers_connected: mcp_health.connected,
            active_executions: engine_stats.active_executions,
            total_executions: engine_stats.total_executions,
            avg_duration_ms: engine_stats.average_duration.as_millis() as u64,
            cache_hit_count: engine_stats.cache_size,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct AppStats {
    pub mcp_servers_total: usize,
    pub mcp_servers_connected: usize,
    pub active_executions: usize,
    pub total_executions: u64,
    pub avg_duration_ms: u64,
    pub cache_hit_count: usize,
}