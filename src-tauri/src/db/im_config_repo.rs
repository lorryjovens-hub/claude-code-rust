use anyhow::Result;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImConfigRow {
    pub id: String,
    pub platform: String,
    pub config_json: String,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
}

fn row_to_im_config(row: &rusqlite::Row<'_>) -> rusqlite::Result<ImConfigRow> {
    Ok(ImConfigRow {
        id: row.get(0)?,
        platform: row.get(1)?,
        config_json: row.get(2)?,
        status: row.get(3)?,
        created_at: row.get(4)?,
        updated_at: row.get(5)?,
    })
}

pub fn insert_im_config(
    conn: &Connection,
    id: &str,
    platform: &str,
    config_json: &str,
    status: &str,
    created_at: &str,
    updated_at: &str,
) -> Result<()> {
    conn.execute(
        "INSERT INTO im_configs (id, platform, config_json, status, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![id, platform, config_json, status, created_at, updated_at],
    )?;
    Ok(())
}

pub fn get_im_config_by_platform(
    conn: &Connection,
    platform: &str,
) -> Result<Option<ImConfigRow>> {
    let mut stmt = conn.prepare_cached(
        "SELECT id, platform, config_json, status, created_at, updated_at FROM im_configs WHERE platform = ?1",
    )?;
    let mut rows = stmt.query(params![platform])?;
    match rows.next()? {
        Some(row) => Ok(Some(row_to_im_config(row)?)),
        None => Ok(None),
    }
}

pub fn get_im_config_by_id(
    conn: &Connection,
    id: &str,
) -> Result<Option<ImConfigRow>> {
    let mut stmt = conn.prepare_cached(
        "SELECT id, platform, config_json, status, created_at, updated_at FROM im_configs WHERE id = ?1",
    )?;
    let mut rows = stmt.query(params![id])?;
    match rows.next()? {
        Some(row) => Ok(Some(row_to_im_config(row)?)),
        None => Ok(None),
    }
}

pub fn list_im_configs(conn: &Connection) -> Result<Vec<ImConfigRow>> {
    let mut stmt = conn.prepare_cached(
        "SELECT id, platform, config_json, status, created_at, updated_at FROM im_configs ORDER BY created_at ASC",
    )?;
    let rows = stmt.query_map([], |row| row_to_im_config(row))?;
    let mut result = Vec::new();
    for row in rows {
        result.push(row?);
    }
    Ok(result)
}

pub fn update_im_config(
    conn: &Connection,
    id: &str,
    config_json: &str,
    status: &str,
    updated_at: &str,
) -> Result<()> {
    conn.execute(
        "UPDATE im_configs SET config_json = ?1, status = ?2, updated_at = ?3 WHERE id = ?4",
        params![config_json, status, updated_at, id],
    )?;
    Ok(())
}

pub fn update_im_config_status(
    conn: &Connection,
    id: &str,
    status: &str,
    updated_at: &str,
) -> Result<()> {
    conn.execute(
        "UPDATE im_configs SET status = ?1, updated_at = ?2 WHERE id = ?3",
        params![status, updated_at, id],
    )?;
    Ok(())
}

pub fn delete_im_config(conn: &Connection, id: &str) -> Result<()> {
    conn.execute(
        "DELETE FROM im_configs WHERE id = ?1",
        params![id],
    )?;
    Ok(())
}
