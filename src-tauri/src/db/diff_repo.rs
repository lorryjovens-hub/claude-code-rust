use anyhow::Result;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeDiffRow {
    pub id: String,
    pub conversation_id: String,
    pub message_id: String,
    pub file_path: String,
    pub original_content: Option<String>,
    pub modified_content: Option<String>,
    pub diff_text: Option<String>,
    pub status: String,
    pub applied_at: Option<String>,
    pub created_at: String,
}

fn row_to_code_diff(row: &rusqlite::Row<'_>) -> rusqlite::Result<CodeDiffRow> {
    Ok(CodeDiffRow {
        id: row.get(0)?,
        conversation_id: row.get(1)?,
        message_id: row.get(2)?,
        file_path: row.get(3)?,
        original_content: row.get(4)?,
        modified_content: row.get(5)?,
        diff_text: row.get(6)?,
        status: row.get(7)?,
        applied_at: row.get(8)?,
        created_at: row.get(9)?,
    })
}

pub fn insert_code_diff(
    conn: &Connection,
    id: &str,
    conversation_id: &str,
    message_id: &str,
    file_path: &str,
    original_content: Option<&str>,
    modified_content: Option<&str>,
    diff_text: Option<&str>,
    status: &str,
    created_at: &str,
) -> Result<()> {
    let mut stmt = conn.prepare_cached(
        "INSERT INTO code_diffs (id, conversation_id, message_id, file_path, original_content, modified_content, diff_text, status, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)"
    )?;
    stmt.execute(params![
        id,
        conversation_id,
        message_id,
        file_path,
        original_content,
        modified_content,
        diff_text,
        status,
        created_at,
    ])?;
    Ok(())
}

pub fn get_diffs_by_conversation(conn: &Connection, conversation_id: &str) -> Result<Vec<CodeDiffRow>> {
    let mut stmt = conn.prepare_cached(
        "SELECT id, conversation_id, message_id, file_path, original_content, modified_content, diff_text, status, applied_at, created_at FROM code_diffs WHERE conversation_id = ?1 ORDER BY created_at DESC"
    )?;
    let rows = stmt.query_map(params![conversation_id], |row| row_to_code_diff(row))?;
    let mut result = Vec::new();
    for row in rows {
        result.push(row?);
    }
    Ok(result)
}

pub fn get_diffs_by_message(conn: &Connection, message_id: &str) -> Result<Vec<CodeDiffRow>> {
    let mut stmt = conn.prepare_cached(
        "SELECT id, conversation_id, message_id, file_path, original_content, modified_content, diff_text, status, applied_at, created_at FROM code_diffs WHERE message_id = ?1 ORDER BY created_at ASC"
    )?;
    let rows = stmt.query_map(params![message_id], |row| row_to_code_diff(row))?;
    let mut result = Vec::new();
    for row in rows {
        result.push(row?);
    }
    Ok(result)
}

pub fn update_diff_status(conn: &Connection, id: &str, status: &str, applied_at: Option<&str>) -> Result<()> {
    let mut stmt = conn.prepare_cached(
        "UPDATE code_diffs SET status = ?1, applied_at = ?2 WHERE id = ?3"
    )?;
    stmt.execute(params![status, applied_at, id])?;
    Ok(())
}