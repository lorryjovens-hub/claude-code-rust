use anyhow::Result;
use rusqlite::Connection;
use std::fs;
use std::path::Path;

use super::conversation_repo;
use super::message_repo;

pub fn migrate_conversation_store(store_path: &Path, conn: &Connection) -> Result<()> {
    if !store_path.exists() {
        return Ok(());
    }

    let tx = conn.unchecked_transaction()?;

    let entries = fs::read_dir(store_path)?;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }

        let content = fs::read_to_string(&path)?;
        let data: serde_json::Value = serde_json::from_str(&content)?;

        let conv_id = data
            .get("id")
            .and_then(|v| v.as_str())
            .unwrap_or_else(|| {
                path.file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("")
            })
            .to_string();

        if conversation_repo::get_conversation(&tx, &conv_id)?.is_some() {
            continue;
        }

        let title = data.get("title").and_then(|v| v.as_str());
        let model = data.get("model").and_then(|v| v.as_str());
        let created_at = data
            .get("created_at")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let updated_at = data
            .get("saved_at")
            .and_then(|v| v.as_str())
            .unwrap_or(created_at);
        let research_mode = data
            .get("research_mode")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let messages = data
            .get("messages")
            .and_then(|m| m.as_array())
            .cloned()
            .unwrap_or_default();

        conversation_repo::insert_conversation(
            &tx,
            &conv_id,
            title,
            model,
            None,
            None,
            None,
            research_mode,
            false,
            false,
            created_at,
            updated_at,
            messages.len() as i64,
        )?;

        for (idx, msg) in messages.iter().enumerate() {
            let fallback_id = uuid::Uuid::new_v4().to_string();
            let msg_id = msg
                .get("id")
                .and_then(|v| v.as_str())
                .unwrap_or(&fallback_id)
                .to_string();
            let role = msg
                .get("role")
                .and_then(|v| v.as_str())
                .unwrap_or("user");
            let content = match msg.get("content") {
                Some(v) if v.is_string() => v.as_str().unwrap_or("").to_string(),
                Some(v) => serde_json::to_string(v).unwrap_or_default(),
                None => String::new(),
            };
            let thinking = msg.get("thinking").and_then(|v| v.as_str());
            let msg_created_at = msg
                .get("created_at")
                .and_then(|v| v.as_str())
                .unwrap_or(updated_at);
            let is_compact_boundary = msg
                .get("is_compact_boundary")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);

            message_repo::insert_message(
                &tx,
                &msg_id,
                &conv_id,
                role,
                &content,
                thinking,
                msg_created_at,
                is_compact_boundary,
                idx as i64,
            )?;
        }
    }

    tx.commit()?;
    Ok(())
}

pub fn migrate_session_manager(db_path: &Path, conn: &Connection) -> Result<()> {
    if !db_path.exists() {
        return Ok(());
    }

    let tx = conn.unchecked_transaction()?;

    let content = fs::read_to_string(db_path)?;
    let data: serde_json::Value = serde_json::from_str(&content)?;

    let conversations = data
        .get("conversations")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();

    for conv in conversations {
        let conv_id = conv
            .get("id")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        if conv_id.is_empty() {
            continue;
        }

        if conversation_repo::get_conversation(&tx, &conv_id)?.is_some() {
            continue;
        }

        let title = conv.get("title").and_then(|v| v.as_str());
        let model = conv.get("model").and_then(|v| v.as_str());
        let workspace_path = conv.get("workspace_path").and_then(|v| v.as_str());
        let project_id = conv.get("project_id").and_then(|v| v.as_str());
        let research_mode = conv
            .get("research_mode")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let created_at = conv
            .get("created_at")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let updated_at = conv
            .get("updated_at")
            .and_then(|v| v.as_str())
            .unwrap_or(created_at);

        conversation_repo::insert_conversation(
            &tx,
            &conv_id,
            title,
            model,
            None,
            workspace_path,
            project_id,
            research_mode,
            false,
            false,
            created_at,
            updated_at,
            0,
        )?;
    }

    let messages = data
        .get("messages")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();

    for (idx, msg) in messages.iter().enumerate() {
        let fallback_id = uuid::Uuid::new_v4().to_string();
        let msg_id = msg
            .get("id")
            .and_then(|v| v.as_str())
            .unwrap_or(&fallback_id)
            .to_string();
        let conversation_id = msg
            .get("conversation_id")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        if conversation_id.is_empty() {
            continue;
        }

        if message_repo::get_message(&tx, &msg_id)?.is_some() {
            continue;
        }

        let role = msg
            .get("role")
            .and_then(|v| v.as_str())
            .unwrap_or("user");
        let content = match msg.get("content") {
            Some(v) if v.is_string() => v.as_str().unwrap_or("").to_string(),
            Some(v) => serde_json::to_string(v).unwrap_or_default(),
            None => String::new(),
        };
        let thinking = msg.get("thinking").and_then(|v| v.as_str());
        let msg_created_at = msg
            .get("created_at")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let is_compact_boundary = msg
            .get("is_compact_boundary")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        message_repo::insert_message(
            &tx,
            &msg_id,
            conversation_id,
            role,
            &content,
            thinking,
            msg_created_at,
            is_compact_boundary,
            idx as i64,
        )?;
    }

    let projects = data
        .get("projects")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();

    for proj in projects {
        let proj_id = proj
            .get("id")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        if proj_id.is_empty() {
            continue;
        }

        if super::project_repo::get_project(&tx, &proj_id)?.is_some() {
            continue;
        }

        let name = proj
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("Unnamed");
        let description = proj.get("description").and_then(|v| v.as_str());
        let instructions = proj.get("instructions").and_then(|v| v.as_str());
        let workspace_path = proj.get("workspace_path").and_then(|v| v.as_str());
        let is_archived = proj
            .get("is_archived")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let created_at = proj
            .get("created_at")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let updated_at = proj
            .get("updated_at")
            .and_then(|v| v.as_str())
            .unwrap_or(created_at);

        super::project_repo::insert_project(
            &tx,
            &proj_id,
            name,
            description,
            instructions,
            workspace_path,
            is_archived,
            created_at,
            updated_at,
        )?;
    }

    let project_files = data
        .get("project_files")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();

    for pf in project_files {
        let pf_id = pf
            .get("id")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        if pf_id.is_empty() {
            continue;
        }

        let project_id = pf
            .get("project_id")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let file_name = pf.get("file_name").and_then(|v| v.as_str());
        let file_path = pf.get("file_path").and_then(|v| v.as_str());
        let file_size = pf
            .get("file_size")
            .and_then(|v| v.as_u64())
            .map(|v| v as i64);
        let mime_type = pf.get("mime_type").and_then(|v| v.as_str());

        super::project_repo::insert_project_file(
            &tx,
            &pf_id,
            project_id,
            file_name,
            file_path,
            file_size,
            mime_type,
        )?;
    }

    tx.commit()?;
    Ok(())
}

pub fn check_and_migrate(data_dir: &Path, conn: &Connection) -> Result<()> {
    let migrated_marker = data_dir.join(".migrated");
    if migrated_marker.exists() {
        return Ok(());
    }

    let store_path = data_dir.join("conversations");
    if store_path.exists() {
        migrate_conversation_store(&store_path, conn)?;
    }

    let session_db_path = data_dir.join("claude-desktop.json");
    if session_db_path.exists() {
        migrate_session_manager(&session_db_path, conn)?;
    }

    fs::write(&migrated_marker, "")?;

    Ok(())
}
