use anyhow::Result;
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::sse::{Event, KeepAlive, Sse},
    routing::{delete, get, post},
    Json, Router,
};
use futures::stream::{Stream, StreamExt};
use serde::{Deserialize, Serialize};
use std::convert::Infallible;
use std::sync::Arc;
use tokio::sync::Mutex;
use tower_http::cors::{Any, CorsLayer};

use crate::engine::EnginePool;
use crate::tools::{execute_tool, get_tool_definitions, ToolDefinition};

#[derive(Clone)]
pub struct BridgeServer {
    engine_pool: Arc<Mutex<EnginePool>>,
}

#[derive(Serialize, Deserialize, Clone)]
#[derive(Deserialize)]
pub struct ChatRequest {
    pub conversation_id: String,
    pub messages: Option<Vec<serde_json::Value>>,
    pub message: Option<String>,
    pub model: String,
    pub user_mode: Option<String>,
    pub env_token: Option<String>,
    pub env_base_url: Option<String>,
    pub research_mode: Option<bool>,
    pub attachments: Option<Vec<serde_json::Value>>,
}

impl ChatRequest {
    pub fn get_messages(&self) -> Vec<serde_json::Value> {
        if let Some(msgs) = &self.messages {
            return msgs.clone();
        }
        if let Some(msg) = &self.message {
            return vec![serde_json::json!({
                "role": "user",
                "content": msg
            })];
        }
        vec![]
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ToolRequest {
    pub name: String,
    pub input: serde_json::Value,
    pub cwd: Option<String>,
}

#[derive(Serialize)]
pub struct SystemStatus {
    pub platform: String,
    pub git_bash: GitBashStatus,
}

#[derive(Serialize)]
pub struct GitBashStatus {
    pub required: bool,
    pub found: bool,
    pub path: Option<String>,
}

#[derive(Deserialize)]
pub struct StreamQuery {
    pub conversation_id: String,
    pub model: String,
    pub user_mode: Option<String>,
    pub env_token: Option<String>,
    pub env_base_url: Option<String>,
    pub research_mode: Option<bool>,
    pub messages: Option<String>,
}

impl BridgeServer {
    pub fn new() -> Self {
        Self {
            engine_pool: Arc::new(Mutex::new(EnginePool::new())),
        }
    }

    pub async fn start(&self, port: u16) -> Result<()> {
        let engine_pool = self.engine_pool.clone();

        let cors = CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(Any)
            .allow_headers(Any);

        let app = Router::new()
            .route("/api/system-status", get(system_status))
            .route("/api/chat", post(chat_handler))
            .route("/api/chat/stream", get(chat_stream_handler))
            .route("/api/tools", post(tools_handler))
            .route("/api/tools/list", get(tools_list_handler))
            .route("/api/conversations", get(conversations_list))
            .route("/api/conversations", post(conversations_create))
            .route("/api/conversations/{id}", get(conversation_get))
            .route("/api/conversations/{id}", post(conversation_update))
            .route("/api/conversations/{id}", delete(conversation_delete))
            .route("/api/projects", get(projects_list))
            .route("/api/projects", post(projects_create))
            .route("/api/upload", post(upload_handler))
            .layer(cors)
            .with_state(engine_pool);

        let listener = tokio::net::TcpListener::bind(format!("127.0.0.1:{}", port)).await?;
        println!("[Bridge] Server running on http://127.0.0.1:{}", port);
        axum::serve(listener, app).await?;
        Ok(())
    }
}

async fn system_status() -> Json<SystemStatus> {
    let platform = std::env::consts::OS.to_string();
    let git_bash_path = which_git_bash();

    Json(SystemStatus {
        platform,
        git_bash: GitBashStatus {
            required: cfg!(target_os = "windows"),
            found: git_bash_path.is_some(),
            path: git_bash_path,
        },
    })
}

fn which_git_bash() -> Option<String> {
    let candidates = if cfg!(target_os = "windows") {
        vec![
            r"C:\Program Files\Git\bin\bash.exe",
            r"C:\Program Files (x86)\Git\bin\bash.exe",
        ]
    } else {
        vec!["/usr/bin/bash", "/bin/bash"]
    };

    for path in &candidates {
        if std::path::Path::new(path).exists() {
            return Some(path.to_string());
        }
    }

    if let Ok(output) = std::process::Command::new("which").arg("bash").output() {
        if output.status.success() {
            return Some(String::from_utf8_lossy(&output.stdout).trim().to_string());
        }
    }

    None
}

async fn chat_handler(
    State(pool): State<Arc<Mutex<EnginePool>>>,
    Json(req): Json<ChatRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let mut pool = pool.lock().await;
    match pool.send_message(&req.conversation_id, &req).await {
        Ok(response) => Ok(Json(response)),
        Err(e) => {
            eprintln!("[Chat] Error: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn chat_stream_handler(
    State(_pool): State<Arc<Mutex<EnginePool>>>,
    Query(query): Query<StreamQuery>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let (tx, mut rx) = tokio::sync::mpsc::channel::<Result<Event, Infallible>>(100);

    tokio::spawn(async move {
        let _ = tx.send(Ok(Event::default()
            .event("start")
            .data(serde_json::json!({"status": "connected", "conversation_id": &query.conversation_id}).to_string()))).await;

        let messages: Vec<serde_json::Value> = query
            .messages
            .and_then(|m| serde_json::from_str(&m).ok())
            .unwrap_or_default();

        let req = ChatRequest {
            conversation_id: query.conversation_id.clone(),
            messages,
            model: query.model.clone(),
            user_mode: query.user_mode.clone(),
            env_token: query.env_token.clone(),
            env_base_url: query.env_base_url.clone(),
            research_mode: query.research_mode,
            attachments: None,
        };

        match stream_api_response_realtime(&req, tx.clone()).await {
            Ok(()) => {
                let _ = tx.send(Ok(Event::default().event("done").data("{}"))).await;
            }
            Err(e) => {
                let _ = tx.send(Ok(Event::default()
                    .event("error")
                    .data(serde_json::json!({"error": e.to_string()}).to_string()))).await;
            }
        }
    });

    let stream = async_stream::stream! {
        while let Some(item) = rx.recv().await {
            yield item;
        }
    };

    Sse::new(stream).keep_alive(KeepAlive::default())
}

async fn stream_api_response_realtime(
    req: &ChatRequest,
    tx: tokio::sync::mpsc::Sender<Result<Event, Infallible>>,
) -> Result<()> {
    let env_token = req.env_token.clone().unwrap_or_default();
    let env_base_url = req.env_base_url.clone().unwrap_or_default();

    if env_token.is_empty() {
        anyhow::bail!("No API token provided");
    }

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(120))
        .build()?;
    let api_url = resolve_api_url(&req.user_mode, &env_base_url);

    let body = serde_json::json!({
        "model": req.model,
        "messages": req.get_messages(),
        "max_tokens": 8192,
        "stream": true,
    });

    let mut request_builder = client
        .post(&api_url)
        .header("Content-Type", "application/json")
        .json(&body);

    if api_url.contains("anthropic") {
        request_builder = request_builder
            .header("x-api-key", &env_token)
            .header("anthropic-version", "2023-06-01");
    } else {
        request_builder = request_builder.bearer_auth(&env_token);
    }

    let response = request_builder.send().await?;

    if !response.status().is_success() {
        let status = response.status();
        let text = response.text().await.unwrap_or_default();
        anyhow::bail!("API error {}: {}", status, text);
    }

    let mut stream = response.bytes_stream();
    let mut buffer = String::new();

    while let Some(chunk) = stream.next().await {
        match chunk {
            Ok(bytes) => {
                buffer.push_str(&String::from_utf8_lossy(&bytes));
                while let Some(pos) = buffer.find('\n') {
                    let line = buffer[..pos].to_string();
                    buffer = buffer[pos + 1..].to_string();

                    if line.starts_with("data: ") {
                        let data = &line[6..];
                        if data == "[DONE]" {
                            return Ok(());
                        }
                        let event = Event::default().event("message").data(data.to_string());
                        if tx.send(Ok(event)).await.is_err() {
                            return Ok(());
                        }
                    }
                }
            }
            Err(e) => {
                anyhow::bail!("Stream error: {}", e);
            }
        }
    }

    Ok(())
}

async fn tools_handler(
    Json(req): Json<ToolRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let cwd = req.cwd.unwrap_or_else(|| ".".to_string());
    match execute_tool(&req.name, req.input, &cwd) {
        Ok(result) => Ok(Json(result)),
        Err(e) => {
            eprintln!("[Tools] Error: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn tools_list_handler() -> Json<Vec<ToolDefinition>> {
    Json(get_tool_definitions())
}

async fn conversations_list() -> Json<serde_json::Value> {
    Json(serde_json::json!({ "conversations": [] }))
}

async fn conversations_create() -> Json<serde_json::Value> {
    Json(serde_json::json!({ "id": uuid::Uuid::new_v4().to_string() }))
}

async fn conversation_get() -> Json<serde_json::Value> {
    Json(serde_json::json!({}))
}

async fn conversation_update() -> Json<serde_json::Value> {
    Json(serde_json::json!({ "ok": true }))
}

async fn conversation_delete() -> Json<serde_json::Value> {
    Json(serde_json::json!({ "ok": true }))
}

async fn projects_list() -> Json<serde_json::Value> {
    Json(serde_json::json!({ "projects": [] }))
}

async fn projects_create() -> Json<serde_json::Value> {
    Json(serde_json::json!({ "id": uuid::Uuid::new_v4().to_string() }))
}

async fn upload_handler() -> Json<serde_json::Value> {
    Json(serde_json::json!({ "ok": true }))
}

fn resolve_api_url(user_mode: &Option<String>, env_base_url: &str) -> String {
    if !env_base_url.is_empty() {
        if env_base_url.contains("/v1/messages") || env_base_url.contains("/v1/chat/completions") {
            return env_base_url.to_string();
        }
        if env_base_url.contains("anthropic") || env_base_url.contains("claude") {
            return format!("{}/v1/messages", env_base_url.trim_end_matches('/'));
        }
        return format!("{}/v1/chat/completions", env_base_url.trim_end_matches('/'));
    }

    match user_mode.as_deref() {
        Some("clawparrot") => "http://127.0.0.1:30090/api/v1/messages".to_string(),
        _ => "https://api.anthropic.com/v1/messages".to_string(),
    }
}
