use serde::{Deserialize, Serialize};
use std::sync::Arc;
use crate::db::DbManager;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FeishuChatMapping {
    pub chat_id: String,
    pub conversation_id: String,
    pub title: String,
    pub created_at: String,
    pub last_active_at: String,
    pub message_count: u64,
}

fn with_db<F, R>(db: &DbManager, f: F) -> Result<R, String>
where
    F: FnOnce(&rusqlite::Connection) -> R,
{
    db.with_conn(f).map_err(|e| format!("DB error: {}", e))
}

/// Internal version: takes &DbManager directly (used by the message processor)
pub async fn feishu_get_or_create_conversation_inner(
    db: &DbManager,
    chat_id: String,
    title: Option<String>,
) -> Result<FeishuChatMapping, String> {
    let existing = with_db(db, |conn| -> Option<FeishuChatMapping> {
        let mut stmt = conn.prepare(
            "SELECT chat_id, conversation_id, title, created_at, last_active_at, message_count FROM feishu_chat_mappings WHERE chat_id = ?1"
        ).ok()?;
        stmt.query_row([&chat_id], |row| {
            Ok(FeishuChatMapping {
                chat_id: row.get(0).unwrap_or_default(),
                conversation_id: row.get(1).unwrap_or_default(),
                title: row.get(2).unwrap_or_default(),
                created_at: row.get(3).unwrap_or_default(),
                last_active_at: row.get(4).unwrap_or_default(),
                message_count: row.get(5).unwrap_or(0),
            })
        }).ok()
    })?;

    if let Some(mapping) = existing {
        return Ok(mapping);
    }

    let conv_id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();
    let effective_title = title.unwrap_or_else(|| format!("飞书-{}", &chat_id.chars().take(8).collect::<String>()));

    with_db(db, |conn| {
        conn.execute(
            "INSERT INTO conversations (id, title, created_at, updated_at, message_count) VALUES (?1, ?2, ?3, ?4, 0)",
            rusqlite::params![conv_id, effective_title, now, now],
        ).ok();
    })?;

    with_db(db, |conn| {
        conn.execute(
            "INSERT INTO feishu_chat_mappings (chat_id, conversation_id, title, created_at, last_active_at, message_count) VALUES (?1, ?2, ?3, ?4, ?4, 0)",
            rusqlite::params![chat_id, conv_id, effective_title, now],
        ).ok();
    })?;

    Ok(FeishuChatMapping {
        chat_id,
        conversation_id: conv_id,
        title: effective_title,
        created_at: now.clone(),
        last_active_at: now,
        message_count: 0,
    })
}

#[tauri::command]
pub async fn feishu_get_or_create_conversation(
    db: tauri::State<'_, Arc<DbManager>>,
    chat_id: String,
    title: Option<String>,
) -> Result<FeishuChatMapping, String> {
    feishu_get_or_create_conversation_inner(&db, chat_id, title).await
}

#[tauri::command]
pub async fn feishu_list_conversations(
    db: tauri::State<'_, Arc<DbManager>>,
) -> Result<Vec<FeishuChatMapping>, String> {
    with_db(&db, |conn| {
        let mut stmt = conn.prepare(
            "SELECT chat_id, conversation_id, title, created_at, last_active_at, message_count FROM feishu_chat_mappings ORDER BY last_active_at DESC"
        ).ok();

        let mut result = Vec::new();
        if let Some(ref mut s) = stmt {
            if let Ok(rows) = s.query_map([], |row| {
                Ok(FeishuChatMapping {
                    chat_id: row.get(0).unwrap_or_default(),
                    conversation_id: row.get(1).unwrap_or_default(),
                    title: row.get(2).unwrap_or_default(),
                    created_at: row.get(3).unwrap_or_default(),
                    last_active_at: row.get(4).unwrap_or_default(),
                    message_count: row.get(5).unwrap_or(0),
                })
            }) {
                for row in rows.flatten() {
                    result.push(row);
                }
            }
        }
        result
    })
}

#[tauri::command]
pub async fn feishu_delete_conversation(
    db: tauri::State<'_, Arc<DbManager>>,
    chat_id: String,
) -> Result<(), String> {
    with_db(&db, |conn| {
        conn.execute("DELETE FROM feishu_chat_mappings WHERE chat_id = ?1", rusqlite::params![chat_id]).ok();
    })
}
