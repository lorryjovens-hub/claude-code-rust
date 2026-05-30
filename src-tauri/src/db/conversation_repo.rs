use anyhow::Result;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationRow {
    pub id: String,
    pub title: Option<String>,
    pub model: Option<String>,
    pub provider: Option<String>,
    pub workspace_path: Option<String>,
    pub project_id: Option<String>,
    pub research_mode: bool,
    pub pinned: bool,
    pub archived: bool,
    pub created_at: String,
    pub updated_at: String,
    pub message_count: i64,
}

fn row_to_conversation(row: &rusqlite::Row<'_>) -> rusqlite::Result<ConversationRow> {
    Ok(ConversationRow {
        id: row.get(0)?,
        title: row.get(1)?,
        model: row.get(2)?,
        provider: row.get(3)?,
        workspace_path: row.get(4)?,
        project_id: row.get(5)?,
        research_mode: row.get::<_, i64>(6)? != 0,
        pinned: row.get::<_, i64>(7)? != 0,
        archived: row.get::<_, i64>(8)? != 0,
        created_at: row.get(9)?,
        updated_at: row.get(10)?,
        message_count: row.get(11)?,
    })
}

pub fn insert_conversation(
    conn: &Connection,
    id: &str,
    title: Option<&str>,
    model: Option<&str>,
    provider: Option<&str>,
    workspace_path: Option<&str>,
    project_id: Option<&str>,
    research_mode: bool,
    pinned: bool,
    archived: bool,
    created_at: &str,
    updated_at: &str,
    message_count: i64,
) -> Result<()> {
    let mut stmt = conn.prepare_cached(
        "INSERT INTO conversations (id, title, model, provider, workspace_path, project_id, research_mode, pinned, archived, created_at, updated_at, message_count) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)"
    )?;
    stmt.execute(params![
        id,
        title,
        model,
        provider,
        workspace_path,
        project_id,
        research_mode as i64,
        pinned as i64,
        archived as i64,
        created_at,
        updated_at,
        message_count,
    ])?;
    Ok(())
}

pub fn get_conversation(conn: &Connection, id: &str) -> Result<Option<ConversationRow>> {
    let mut stmt = conn.prepare_cached(
        "SELECT id, title, model, provider, workspace_path, project_id, research_mode, pinned, archived, created_at, updated_at, message_count FROM conversations WHERE id = ?1"
    )?;
    let mut rows = stmt.query(params![id])?;
    match rows.next()? {
        Some(row) => Ok(Some(row_to_conversation(row)?)),
        None => Ok(None),
    }
}

pub fn list_conversations(conn: &Connection) -> Result<Vec<ConversationRow>> {
    let mut stmt = conn.prepare_cached(
        "SELECT id, title, model, provider, workspace_path, project_id, research_mode, pinned, archived, created_at, updated_at, message_count FROM conversations ORDER BY updated_at DESC"
    )?;
    let rows = stmt.query_map([], |row| row_to_conversation(row))?;
    let mut result = Vec::new();
    for row in rows {
        result.push(row?);
    }
    Ok(result)
}

pub fn update_conversation_title(conn: &Connection, id: &str, title: &str) -> Result<()> {
    let mut stmt = conn.prepare_cached(
        "UPDATE conversations SET title = ?1, updated_at = ?2 WHERE id = ?3"
    )?;
    let now = chrono::Utc::now().to_rfc3339();
    stmt.execute(params![title, now, id])?;
    Ok(())
}

pub fn update_title_if_empty(conn: &Connection, id: &str, title: &str) -> Result<bool> {
    let now = chrono::Utc::now().to_rfc3339();
    let mut stmt = conn.prepare_cached(
        "UPDATE conversations SET title = ?1, updated_at = ?2 WHERE id = ?3 AND (title IS NULL OR title = '')"
    )?;
    let rows = stmt.execute(params![title, now, id])?;
    Ok(rows > 0)
}

pub fn update_conversation_model(conn: &Connection, id: &str, model: &str) -> Result<()> {
    let mut stmt = conn.prepare_cached(
        "UPDATE conversations SET model = ?1, updated_at = ?2 WHERE id = ?3"
    )?;
    let now = chrono::Utc::now().to_rfc3339();
    stmt.execute(params![model, now, id])?;
    Ok(())
}

pub fn update_conversation_timestamp(conn: &Connection, id: &str, updated_at: &str) -> Result<()> {
    let mut stmt = conn.prepare_cached(
        "UPDATE conversations SET updated_at = ?1 WHERE id = ?2"
    )?;
    stmt.execute(params![updated_at, id])?;
    Ok(())
}

pub fn increment_message_count(conn: &Connection, id: &str) -> Result<()> {
    let mut stmt = conn.prepare_cached(
        "UPDATE conversations SET message_count = message_count + 1, updated_at = ?1 WHERE id = ?2"
    )?;
    let now = chrono::Utc::now().to_rfc3339();
    stmt.execute(params![now, id])?;
    Ok(())
}

pub fn delete_conversation(conn: &Connection, id: &str) -> Result<()> {
    let mut stmt = conn.prepare_cached("DELETE FROM conversations WHERE id = ?1")?;
    stmt.execute(params![id])?;
    Ok(())
}

pub fn pin_conversation(conn: &Connection, id: &str, pinned: bool) -> Result<()> {
    let mut stmt = conn.prepare_cached(
        "UPDATE conversations SET pinned = ?1, updated_at = ?2 WHERE id = ?3"
    )?;
    let now = chrono::Utc::now().to_rfc3339();
    stmt.execute(params![pinned as i64, now, id])?;
    Ok(())
}

pub fn archive_conversation(conn: &Connection, id: &str, archived: bool) -> Result<()> {
    let mut stmt = conn.prepare_cached(
        "UPDATE conversations SET archived = ?1, updated_at = ?2 WHERE id = ?3"
    )?;
    let now = chrono::Utc::now().to_rfc3339();
    stmt.execute(params![archived as i64, now, id])?;
    Ok(())
}

pub fn update_conversation(
    conn: &Connection,
    id: &str,
    title: Option<&str>,
    model: Option<&str>,
    provider: Option<&str>,
    workspace_path: Option<&str>,
    project_id: Option<&str>,
    research_mode: Option<bool>,
    pinned: Option<bool>,
    archived: Option<bool>,
    updated_at: Option<&str>,
    message_count: Option<i64>,
) -> Result<()> {
    let mut sql_parts = Vec::new();
    let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

    if let Some(t) = title {
        sql_parts.push("title = ?");
        params.push(Box::new(t));
    }
    if let Some(m) = model {
        sql_parts.push("model = ?");
        params.push(Box::new(m));
    }
    if let Some(p) = provider {
        sql_parts.push("provider = ?");
        params.push(Box::new(p));
    }
    if let Some(wp) = workspace_path {
        sql_parts.push("workspace_path = ?");
        params.push(Box::new(wp));
    }
    if let Some(pi) = project_id {
        sql_parts.push("project_id = ?");
        params.push(Box::new(pi));
    }
    if let Some(rm) = research_mode {
        sql_parts.push("research_mode = ?");
        params.push(Box::new(rm as i64));
    }
    if let Some(pn) = pinned {
        sql_parts.push("pinned = ?");
        params.push(Box::new(pn as i64));
    }
    if let Some(a) = archived {
        sql_parts.push("archived = ?");
        params.push(Box::new(a as i64));
    }
    if let Some(ua) = updated_at {
        sql_parts.push("updated_at = ?");
        params.push(Box::new(ua));
    }
    if let Some(mc) = message_count {
        sql_parts.push("message_count = ?");
        params.push(Box::new(mc));
    }

    if sql_parts.is_empty() {
        return Ok(());
    }

    let sql = format!(
        "UPDATE conversations SET {} WHERE id = ?",
        sql_parts.join(", ")
    );
    
    let mut stmt = conn.prepare_cached(&sql)?;
    params.push(Box::new(id));
    stmt.execute(rusqlite::params_from_iter(params))?;
    Ok(())
}
