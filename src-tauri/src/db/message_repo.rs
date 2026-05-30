use anyhow::Result;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageRow {
    pub id: String,
    pub conversation_id: String,
    pub role: String,
    pub content: String,
    pub thinking: Option<String>,
    pub created_at: String,
    pub is_compact_boundary: bool,
    pub sort_order: i64,
}

fn row_to_message(row: &rusqlite::Row<'_>) -> rusqlite::Result<MessageRow> {
    Ok(MessageRow {
        id: row.get(0)?,
        conversation_id: row.get(1)?,
        role: row.get(2)?,
        content: row.get(3)?,
        thinking: row.get(4)?,
        created_at: row.get(5)?,
        is_compact_boundary: row.get::<_, i64>(6)? != 0,
        sort_order: row.get(7)?,
    })
}

pub fn insert_message(
    conn: &Connection,
    id: &str,
    conversation_id: &str,
    role: &str,
    content: &str,
    thinking: Option<&str>,
    created_at: &str,
    is_compact_boundary: bool,
    sort_order: i64,
) -> Result<()> {
    let mut stmt = conn.prepare_cached(
        "INSERT INTO messages (id, conversation_id, role, content, thinking, created_at, is_compact_boundary, sort_order) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)"
    )?;
    stmt.execute(params![
        id,
        conversation_id,
        role,
        content,
        thinking,
        created_at,
        is_compact_boundary as i64,
        sort_order,
    ])?;
    Ok(())
}

pub fn get_messages_by_conversation(conn: &Connection, conversation_id: &str) -> Result<Vec<MessageRow>> {
    let mut stmt = conn.prepare_cached(
        "SELECT id, conversation_id, role, content, thinking, created_at, is_compact_boundary, sort_order FROM messages WHERE conversation_id = ?1 ORDER BY sort_order ASC"
    )?;
    let rows = stmt.query_map(params![conversation_id], |row| row_to_message(row))?;
    let mut result = Vec::new();
    for row in rows {
        result.push(row?);
    }
    Ok(result)
}

pub fn get_message(conn: &Connection, id: &str) -> Result<Option<MessageRow>> {
    let mut stmt = conn.prepare_cached(
        "SELECT id, conversation_id, role, content, thinking, created_at, is_compact_boundary, sort_order FROM messages WHERE id = ?1"
    )?;
    let mut rows = stmt.query(params![id])?;
    match rows.next()? {
        Some(row) => Ok(Some(row_to_message(row)?)),
        None => Ok(None),
    }
}

pub fn delete_message(conn: &Connection, id: &str) -> Result<()> {
    let mut stmt = conn.prepare_cached("DELETE FROM messages WHERE id = ?1")?;
    stmt.execute(params![id])?;
    Ok(())
}

pub fn delete_messages_from(conn: &Connection, conversation_id: &str, sort_order: i64) -> Result<()> {
    let mut stmt = conn.prepare_cached(
        "DELETE FROM messages WHERE conversation_id = ?1 AND sort_order >= ?2"
    )?;
    stmt.execute(params![conversation_id, sort_order])?;
    Ok(())
}

pub fn delete_messages_tail(conn: &Connection, conversation_id: &str, count: i64) -> Result<()> {
    let mut stmt = conn.prepare_cached(
        "DELETE FROM messages WHERE conversation_id = ?1 AND sort_order >= (SELECT MIN(sort_order) FROM (SELECT sort_order FROM messages WHERE conversation_id = ?1 ORDER BY sort_order DESC LIMIT ?2))"
    )?;
    stmt.execute(params![conversation_id, count])?;
    Ok(())
}

pub fn update_message_content(conn: &Connection, id: &str, content: &str) -> Result<()> {
    let mut stmt = conn.prepare_cached(
        "UPDATE messages SET content = ?1 WHERE id = ?2"
    )?;
    stmt.execute(params![content, id])?;
    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallRow {
    pub id: String,
    pub message_id: String,
    pub name: String,
    pub input: Option<String>,
    pub output: Option<String>,
    pub is_error: bool,
    pub sort_order: i64,
}

pub fn get_tool_calls_for_message(conn: &Connection, message_id: &str) -> Result<Vec<ToolCallRow>> {
    let mut stmt = conn.prepare_cached(
        "SELECT id, message_id, name, input, output, is_error, sort_order FROM tool_calls WHERE message_id = ?1 ORDER BY sort_order ASC"
    )?;
    let rows = stmt.query_map(params![message_id], |row| {
        Ok(ToolCallRow {
            id: row.get(0)?,
            message_id: row.get(1)?,
            name: row.get(2)?,
            input: row.get(3)?,
            output: row.get(4)?,
            is_error: row.get::<_, i64>(5)? != 0,
            sort_order: row.get(6)?,
        })
    })?;
    let mut result = Vec::new();
    for row in rows {
        result.push(row?);
    }
    Ok(result)
}

pub fn insert_tool_call(
    conn: &Connection,
    id: &str,
    message_id: &str,
    name: &str,
    input: Option<&str>,
    output: Option<&str>,
    is_error: bool,
    sort_order: i64,
) -> Result<()> {
    let mut stmt = conn.prepare_cached(
        "INSERT OR REPLACE INTO tool_calls (id, message_id, name, input, output, is_error, sort_order) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)"
    )?;
    stmt.execute(params![
        id,
        message_id,
        name,
        input,
        output,
        is_error as i64,
        sort_order,
    ])?;
    Ok(())
}
