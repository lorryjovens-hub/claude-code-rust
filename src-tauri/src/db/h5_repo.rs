use anyhow::Result;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct H5TokenRow {
    pub id: String,
    pub token: String,
    pub conversation_id: String,
    pub expires_at: String,
    pub is_revoked: bool,
    pub created_at: String,
    pub used_at: Option<String>,
}

fn row_to_h5_token(row: &rusqlite::Row<'_>) -> rusqlite::Result<H5TokenRow> {
    Ok(H5TokenRow {
        id: row.get(0)?,
        token: row.get(1)?,
        conversation_id: row.get(2)?,
        expires_at: row.get(3)?,
        is_revoked: row.get::<_, i64>(4)? != 0,
        created_at: row.get(5)?,
        used_at: row.get(6)?,
    })
}

pub fn insert_h5_token(
    conn: &Connection,
    id: &str,
    token: &str,
    conversation_id: &str,
    expires_at: &str,
    created_at: &str,
) -> Result<()> {
    let mut stmt = conn.prepare_cached(
        "INSERT INTO h5_access_tokens (id, token, conversation_id, expires_at, is_revoked, created_at) VALUES (?1, ?2, ?3, ?4, 0, ?5)"
    )?;
    stmt.execute(params![id, token, conversation_id, expires_at, created_at])?;
    tracing::info!(module = "H5Repo", "Token inserted: id={}, conversation_id={}", id, conversation_id);
    Ok(())
}

pub fn get_h5_token(conn: &Connection, token: &str) -> Result<Option<H5TokenRow>> {
    let mut stmt = conn.prepare_cached(
        "SELECT id, token, conversation_id, expires_at, is_revoked, created_at, used_at FROM h5_access_tokens WHERE token = ?1"
    )?;
    let mut rows = stmt.query(params![token])?;
    match rows.next()? {
        Some(row) => Ok(Some(row_to_h5_token(row)?)),
        None => Ok(None),
    }
}

pub fn get_h5_token_by_id(conn: &Connection, id: &str) -> Result<Option<H5TokenRow>> {
    let mut stmt = conn.prepare_cached(
        "SELECT id, token, conversation_id, expires_at, is_revoked, created_at, used_at FROM h5_access_tokens WHERE id = ?1"
    )?;
    let mut rows = stmt.query(params![id])?;
    match rows.next()? {
        Some(row) => Ok(Some(row_to_h5_token(row)?)),
        None => Ok(None),
    }
}

pub fn revoke_h5_token(conn: &Connection, id: &str) -> Result<()> {
    let mut stmt = conn.prepare_cached(
        "UPDATE h5_access_tokens SET is_revoked = 1 WHERE id = ?1"
    )?;
    stmt.execute(params![id])?;
    tracing::info!(module = "H5Repo", "Token revoked: id={}", id);
    Ok(())
}

pub fn list_h5_tokens_by_conversation(conn: &Connection, conversation_id: &str) -> Result<Vec<H5TokenRow>> {
    let mut stmt = conn.prepare_cached(
        "SELECT id, token, conversation_id, expires_at, is_revoked, created_at, used_at FROM h5_access_tokens WHERE conversation_id = ?1 ORDER BY created_at DESC"
    )?;
    let rows = stmt.query_map(params![conversation_id], |row| row_to_h5_token(row))?;
    let mut result = Vec::new();
    for row in rows {
        result.push(row?);
    }
    Ok(result)
}

pub fn cleanup_expired_tokens(conn: &Connection) -> Result<()> {
    let now = chrono::Utc::now().to_rfc3339();
    let count = conn.execute(
        "DELETE FROM h5_access_tokens WHERE expires_at < ?1",
        params![now],
    )?;
    tracing::info!(module = "H5Repo", "Cleaned up {} expired tokens", count);
    Ok(())
}

pub fn update_h5_token_used_at(conn: &Connection, id: &str) -> Result<()> {
    let now = chrono::Utc::now().to_rfc3339();
    let mut stmt = conn.prepare_cached(
        "UPDATE h5_access_tokens SET used_at = ?1 WHERE id = ?2"
    )?;
    stmt.execute(params![now, id])?;
    Ok(())
}

pub fn validate_h5_token(conn: &Connection, token: &str) -> Result<Option<H5TokenRow>> {
    let token_row = get_h5_token(conn, token)?;
    match token_row {
        Some(row) => {
            if row.is_revoked {
                tracing::warn!(module = "H5Repo", "Token is revoked: id={}", row.id);
                return Ok(None);
            }
            let expires_at = chrono::DateTime::parse_from_rfc3339(&row.expires_at)
                .unwrap_or_else(|_| chrono::DateTime::UNIX_EPOCH.into())
                .with_timezone(&chrono::Utc);
            if chrono::Utc::now() > expires_at {
                tracing::warn!(module = "H5Repo", "Token expired: id={}", row.id);
                return Ok(None);
            }
            let _ = update_h5_token_used_at(conn, &row.id);
            tracing::info!(module = "H5Repo", "Token validated: id={}", row.id);
            Ok(Some(row))
        }
        None => Ok(None),
    }
}