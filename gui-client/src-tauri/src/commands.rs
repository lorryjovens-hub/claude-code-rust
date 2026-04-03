use crate::models::*;
use tauri::State;
use tauri::Manager;
use uuid::Uuid;
use tokio::sync::mpsc;
use std::time::Duration;

pub type AppResult<T> = Result<ApiResponse<T>, String>;

// ============================================================
// Stream Types
// ============================================================

#[derive(Debug, Serialize, Deserialize)]
pub struct StreamChunk {
    pub id: String,
    pub content: String,
    pub is_thinking: bool,
    pub is_complete: bool,
    pub model: Option<String>,
}

// ============================================================
// Chat Commands
// ============================================================

#[tauri::command]
pub async fn send_chat_message(
    state: State<'_, AppState>,
    message: String,
    conversation_id: Option<String>,
    model: Option<String>,
) -> AppResult<Message> {
    let msg = Message {
        id: Uuid::new_v4().to_string(),
        role: "assistant".to_string(),
        content: format!("Echo: {}", message),
        timestamp: chrono::Utc::now().timestamp(),
        model: model.or(Some("claude-3-5-sonnet".to_string())),
        tokens: None,
        metadata: None,
    };

    if let Some(conv_id) = conversation_id {
        let mut conversations = state.conversations.write().await;
        if let Some(conv) = conversations.iter_mut().find(|c| c.id == conv_id) {
            conv.messages.push(msg.clone());
            conv.updated_at = chrono::Utc::now().timestamp();
        }
    }

    Ok(ApiResponse::success(msg))
}

#[tauri::command]
pub async fn stream_chat_message(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    message: String,
    conversation_id: Option<String>,
    model: Option<String>,
) -> AppResult<()> {
    let stream_id = Uuid::new_v4().to_string();
    let model_name = model.or(Some("claude-3-5-sonnet".to_string())).unwrap();
    let response = format!("Echo: {}", message);
    
    // Simulate streaming
    let mut chunks = response.chars().collect::<Vec<char>>();
    let mut current_content = String::new();
    
    // Stream the response
    for (i, chunk) in chunks.into_iter().enumerate() {
        current_content.push(chunk);
        
        // Send chunk to frontend
        let stream_chunk = StreamChunk {
            id: stream_id.clone(),
            content: current_content.clone(),
            is_thinking: i == 0,
            is_complete: i == response.len() - 1,
            model: Some(model_name.clone()),
        };
        
        app.emit_all("stream_chunk", stream_chunk).unwrap();
        
        // Simulate network delay
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    
    // Save to conversation
    if let Some(conv_id) = conversation_id {
        let msg = Message {
            id: Uuid::new_v4().to_string(),
            role: "assistant".to_string(),
            content: response,
            timestamp: chrono::Utc::now().timestamp(),
            model: Some(model_name),
            tokens: None,
            metadata: None,
        };
        
        let mut conversations = state.conversations.write().await;
        if let Some(conv) = conversations.iter_mut().find(|c| c.id == conv_id) {
            conv.messages.push(msg);
            conv.updated_at = chrono::Utc::now().timestamp();
        }
    }
    
    Ok(ApiResponse::success(()))
}

#[tauri::command]
pub async fn get_conversations(
    state: State<'_, AppState>,
) -> AppResult<Vec<Conversation>> {
    let conversations = state.conversations.read().await;
    Ok(ApiResponse::success(conversations.clone()))
}

#[tauri::command]
pub async fn get_conversation(
    state: State<'_, AppState>,
    id: String,
) -> AppResult<Option<Conversation>> {
    let conversations = state.conversations.read().await;
    let conversation = conversations.iter().find(|c| c.id == id).cloned();
    Ok(ApiResponse::success(conversation))
}

#[tauri::command]
pub async fn create_conversation(
    state: State<'_, AppState>,
    title: Option<String>,
    model: Option<String>,
) -> AppResult<Conversation> {
    let now = chrono::Utc::now().timestamp();
    let conversation = Conversation {
        id: Uuid::new_v4().to_string(),
        title: title.unwrap_or_else(|| "New Conversation".to_string()),
        messages: Vec::new(),
        model: model.unwrap_or_else(|| "claude-3-5-sonnet".to_string()),
        created_at: now,
        updated_at: now,
        system_prompt: None,
    };

    let mut conversations = state.conversations.write().await;
    conversations.insert(0, conversation.clone());

    Ok(ApiResponse::success(conversation))
}

#[tauri::command]
pub async fn delete_conversation(
    state: State<'_, AppState>,
    id: String,
) -> AppResult<()> {
    let mut conversations = state.conversations.write().await;
    conversations.retain(|c| c.id != id);
    Ok(ApiResponse::success(()))
}

#[tauri::command]
pub async fn clear_conversation(
    state: State<'_, AppState>,
    id: String,
) -> AppResult<()> {
    let mut conversations = state.conversations.write().await;
    if let Some(conv) = conversations.iter_mut().find(|c| c.id == id) {
        conv.messages.clear();
        conv.updated_at = chrono::Utc::now().timestamp();
    }
    Ok(ApiResponse::success(()))
}

// ============================================================
// Task Commands
// ============================================================

#[tauri::command]
pub async fn get_tasks(
    state: State<'_, AppState>,
) -> AppResult<Vec<Task>> {
    let tasks = state.tasks.read().await;
    Ok(ApiResponse::success(tasks.clone()))
}

#[tauri::command]
pub async fn create_task(
    state: State<'_, AppState>,
    title: String,
    description: String,
    priority: String,
) -> AppResult<Task> {
    let now = chrono::Utc::now().timestamp();
    let task = Task {
        id: Uuid::new_v4().to_string(),
        title,
        description,
        status: "pending".to_string(),
        priority,
        created_at: now,
        updated_at: now,
        completed_at: None,
        progress: 0,
        subtasks: Vec::new(),
    };

    let mut tasks = state.tasks.write().await;
    tasks.insert(0, task.clone());

    Ok(ApiResponse::success(task))
}

#[tauri::command]
pub async fn update_task(
    state: State<'_, AppState>,
    id: String,
    title: Option<String>,
    description: Option<String>,
    status: Option<String>,
    priority: Option<String>,
    progress: Option<u8>,
    subtasks: Option<Vec<SubTask>>,
) -> AppResult<Task> {
    let mut tasks = state.tasks.write().await;
    
    if let Some(task) = tasks.iter_mut().find(|t| t.id == id) {
        if let Some(t) = title {
            task.title = t;
        }
        if let Some(d) = description {
            task.description = d;
        }
        if let Some(s) = status {
            task.status = s;
            if task.status == "completed" {
                task.completed_at = Some(chrono::Utc::now().timestamp());
                task.progress = 100;
            }
        }
        if let Some(p) = priority {
            task.priority = p;
        }
        if let Some(p) = progress {
            task.progress = p;
        }
        if let Some(s) = subtasks {
            task.subtasks = s;
        }
        task.updated_at = chrono::Utc::now().timestamp();
        
        return Ok(ApiResponse::success(task.clone()));
    }

    Err(format!("Task not found: {}", id))
}

#[tauri::command]
pub async fn delete_task(
    state: State<'_, AppState>,
    id: String,
) -> AppResult<()> {
    let mut tasks = state.tasks.write().await;
    tasks.retain(|t| t.id != id);
    Ok(ApiResponse::success(()))
}

#[tauri::command]
pub async fn generate_subtasks(
    _state: State<'_, AppState>,
    _task_id: String,
    description: String,
) -> AppResult<Vec<String>> {
    let mut subtasks = Vec::new();
    
    if description.contains("实现") || description.contains("开发") {
        subtasks.push("分析需求".to_string());
        subtasks.push("设计方案".to_string());
        subtasks.push("编写代码".to_string());
        subtasks.push("测试验证".to_string());
    } else if description.contains("修复") || description.contains("bug") {
        subtasks.push("复现问题".to_string());
        subtasks.push("定位原因".to_string());
        subtasks.push("编写修复".to_string());
        subtasks.push("验证修复".to_string());
    } else if description.contains("优化") {
        subtasks.push("分析现状".to_string());
        subtasks.push("制定方案".to_string());
        subtasks.push("实施优化".to_string());
        subtasks.push("效果验证".to_string());
    } else {
        subtasks.push("分析任务".to_string());
        subtasks.push("制定计划".to_string());
        subtasks.push("执行任务".to_string());
        subtasks.push("验收结果".to_string());
    }

    Ok(ApiResponse::success(subtasks))
}

// ============================================================
// Model Commands
// ============================================================

#[tauri::command]
pub async fn get_model_providers(
    state: State<'_, AppState>,
) -> AppResult<Vec<ModelProvider>> {
    let providers = state.providers.read().await;
    Ok(ApiResponse::success(providers.clone()))
}

#[tauri::command]
pub async fn get_models(
    state: State<'_, AppState>,
    provider_id: Option<String>,
) -> AppResult<Vec<Model>> {
    let providers = state.providers.read().await;
    let models: Vec<Model> = providers
        .iter()
        .filter(|p| provider_id.as_ref().map_or(true, |id| &p.id == id))
        .flat_map(|p| p.models.clone())
        .collect();
    Ok(ApiResponse::success(models))
}

#[tauri::command]
pub async fn set_default_model(
    state: State<'_, AppState>,
    model_id: String,
) -> AppResult<()> {
    let mut settings = state.settings.write().await;
    settings.default_model = model_id;
    Ok(ApiResponse::success(()))
}

#[tauri::command]
pub async fn test_model(
    _state: State<'_, AppState>,
    model_id: String,
) -> AppResult<serde_json::Value> {
    Ok(ApiResponse::success(serde_json::json!({
        "model_id": model_id,
        "latency": 150,
        "success": true
    })))
}

#[tauri::command]
pub async fn update_provider_config(
    state: State<'_, AppState>,
    provider_id: String,
    api_key: Option<String>,
    base_url: Option<String>,
) -> AppResult<()> {
    let mut providers = state.providers.write().await;
    if let Some(provider) = providers.iter_mut().find(|p| p.id == provider_id) {
        if let Some(key) = api_key {
            provider.api_key = Some(key);
        }
        if let Some(url) = base_url {
            provider.base_url = Some(url);
        }
        provider.is_enabled = provider.api_key.is_some();
    }
    Ok(ApiResponse::success(()))
}

// ============================================================
// Settings Commands
// ============================================================

#[tauri::command]
pub async fn get_settings(
    state: State<'_, AppState>,
) -> AppResult<AppSettings> {
    let settings = state.settings.read().await;
    Ok(ApiResponse::success(settings.clone()))
}

#[tauri::command]
pub async fn update_settings(
    state: State<'_, AppState>,
    theme: Option<String>,
    language: Option<String>,
    default_model: Option<String>,
    auto_save: Option<bool>,
    stream_response: Option<bool>,
    show_thinking: Option<bool>,
    max_context_messages: Option<u32>,
) -> AppResult<AppSettings> {
    let mut settings = state.settings.write().await;
    
    if let Some(t) = theme {
        settings.theme = t;
    }
    if let Some(l) = language {
        settings.language = l;
    }
    if let Some(m) = default_model {
        settings.default_model = m;
    }
    if let Some(a) = auto_save {
        settings.auto_save = a;
    }
    if let Some(s) = stream_response {
        settings.stream_response = s;
    }
    if let Some(s) = show_thinking {
        settings.show_thinking = s;
    }
    if let Some(m) = max_context_messages {
        settings.max_context_messages = m;
    }

    Ok(ApiResponse::success(settings.clone()))
}

#[tauri::command]
pub async fn reset_settings(
    state: State<'_, AppState>,
) -> AppResult<AppSettings> {
    let mut settings = state.settings.write().await;
    *settings = AppSettings::default();
    Ok(ApiResponse::success(settings.clone()))
}

// ============================================================
// System Commands
// ============================================================

#[tauri::command]
pub async fn get_health() -> AppResult<serde_json::Value> {
    Ok(ApiResponse::success(serde_json::json!({
        "status": "healthy",
        "version": "1.0.0"
    })))
}

#[tauri::command]
pub async fn open_external(url: String) -> AppResult<()> {
    let _ = open::that(&url);
    Ok(ApiResponse::success(()))
}
