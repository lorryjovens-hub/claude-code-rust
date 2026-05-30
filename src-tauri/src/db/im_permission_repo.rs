use anyhow::Result;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImUserPermissionRow {
    pub id: String,
    pub platform: String,
    pub user_id: String,
    pub permission_mode: String,
    pub is_allowed: bool,
    pub paired_code: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

fn row_to_im_user_permission(row: &rusqlite::Row<'_>) -> rusqlite::Result<ImUserPermissionRow> {
    Ok(ImUserPermissionRow {
        id: row.get(0)?,
        platform: row.get(1)?,
        user_id: row.get(2)?,
        permission_mode: row.get(3)?,
        is_allowed: row.get::<_, i64>(4)? != 0,
        paired_code: row.get(5)?,
        created_at: row.get(6)?,
        updated_at: row.get(7)?,
    })
}

pub fn insert_im_user_permission(
    conn: &Connection,
    id: &str,
    platform: &str,
    user_id: &str,
    permission_mode: &str,
    is_allowed: bool,
    paired_code: Option<&str>,
    created_at: &str,
    updated_at: &str,
) -> Result<()> {
    let mut stmt = conn.prepare_cached(
        "INSERT INTO im_user_permissions (id, platform, user_id, permission_mode, is_allowed, paired_code, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
    )?;
    stmt.execute(params![
        id,
        platform,
        user_id,
        permission_mode,
        if is_allowed { 1 } else { 0 },
        paired_code,
        created_at,
        updated_at,
    ])?;
    Ok(())
}

pub fn update_im_user_permission(
    conn: &Connection,
    id: &str,
    permission_mode: &str,
    is_allowed: bool,
    paired_code: Option<&str>,
    updated_at: &str,
) -> Result<()> {
    let mut stmt = conn.prepare_cached(
        "UPDATE im_user_permissions SET permission_mode = ?1, is_allowed = ?2, paired_code = ?3, updated_at = ?4 WHERE id = ?5",
    )?;
    stmt.execute(params![
        permission_mode,
        if is_allowed { 1 } else { 0 },
        paired_code,
        updated_at,
        id,
    ])?;
    Ok(())
}

pub fn update_im_user_permission_by_platform_user(
    conn: &Connection,
    platform: &str,
    user_id: &str,
    permission_mode: &str,
    is_allowed: bool,
    paired_code: Option<&str>,
    updated_at: &str,
) -> Result<()> {
    let mut stmt = conn.prepare_cached(
        "UPDATE im_user_permissions SET permission_mode = ?1, is_allowed = ?2, paired_code = ?3, updated_at = ?4 WHERE platform = ?5 AND user_id = ?6",
    )?;
    stmt.execute(params![
        permission_mode,
        if is_allowed { 1 } else { 0 },
        paired_code,
        updated_at,
        platform,
        user_id,
    ])?;
    Ok(())
}

pub fn delete_im_user_permission(conn: &Connection, id: &str) -> Result<()> {
    let mut stmt = conn.prepare_cached("DELETE FROM im_user_permissions WHERE id = ?1")?;
    stmt.execute(params![id])?;
    Ok(())
}

pub fn delete_im_user_permission_by_platform_user(
    conn: &Connection,
    platform: &str,
    user_id: &str,
) -> Result<()> {
    let mut stmt = conn.prepare_cached(
        "DELETE FROM im_user_permissions WHERE platform = ?1 AND user_id = ?2",
    )?;
    stmt.execute(params![platform, user_id])?;
    Ok(())
}

pub fn get_im_user_permission_by_platform_user(
    conn: &Connection,
    platform: &str,
    user_id: &str,
) -> Result<Option<ImUserPermissionRow>> {
    let mut stmt = conn.prepare_cached(
        "SELECT id, platform, user_id, permission_mode, is_allowed, paired_code, created_at, updated_at FROM im_user_permissions WHERE platform = ?1 AND user_id = ?2",
    )?;
    let mut rows = stmt.query(params![platform, user_id])?;
    match rows.next()? {
        Some(row) => Ok(Some(row_to_im_user_permission(row)?)),
        None => Ok(None),
    }
}

pub fn get_im_user_permission_by_id(
    conn: &Connection,
    id: &str,
) -> Result<Option<ImUserPermissionRow>> {
    let mut stmt = conn.prepare_cached(
        "SELECT id, platform, user_id, permission_mode, is_allowed, paired_code, created_at, updated_at FROM im_user_permissions WHERE id = ?1",
    )?;
    let mut rows = stmt.query(params![id])?;
    match rows.next()? {
        Some(row) => Ok(Some(row_to_im_user_permission(row)?)),
        None => Ok(None),
    }
}

pub fn list_im_user_permissions_by_platform(
    conn: &Connection,
    platform: &str,
) -> Result<Vec<ImUserPermissionRow>> {
    let mut stmt = conn.prepare_cached(
        "SELECT id, platform, user_id, permission_mode, is_allowed, paired_code, created_at, updated_at FROM im_user_permissions WHERE platform = ?1 ORDER BY updated_at DESC",
    )?;
    let rows = stmt.query_map(params![platform], |row| row_to_im_user_permission(row))?;
    let mut result = Vec::new();
    for row in rows {
        result.push(row?);
    }
    Ok(result)
}

pub fn list_all_im_user_permissions(conn: &Connection) -> Result<Vec<ImUserPermissionRow>> {
    let mut stmt = conn.prepare_cached(
        "SELECT id, platform, user_id, permission_mode, is_allowed, paired_code, created_at, updated_at FROM im_user_permissions ORDER BY updated_at DESC",
    )?;
    let rows = stmt.query_map([], |row| row_to_im_user_permission(row))?;
    let mut result = Vec::new();
    for row in rows {
        result.push(row?);
    }
    Ok(result)
}

pub fn list_pending_im_user_permissions(
    conn: &Connection,
    platform: &str,
) -> Result<Vec<ImUserPermissionRow>> {
    let mut stmt = conn.prepare_cached(
        "SELECT id, platform, user_id, permission_mode, is_allowed, paired_code, created_at, updated_at FROM im_user_permissions WHERE platform = ?1 AND permission_mode = 'pairing' AND is_allowed = 0 ORDER BY created_at ASC",
    )?;
    let rows = stmt.query_map(params![platform], |row| row_to_im_user_permission(row))?;
    let mut result = Vec::new();
    for row in rows {
        result.push(row?);
    }
    Ok(result)
}

pub fn get_im_user_permission_by_paired_code(
    conn: &Connection,
    platform: &str,
    paired_code: &str,
) -> Result<Option<ImUserPermissionRow>> {
    let mut stmt = conn.prepare_cached(
        "SELECT id, platform, user_id, permission_mode, is_allowed, paired_code, created_at, updated_at FROM im_user_permissions WHERE platform = ?1 AND paired_code = ?2",
    )?;
    let mut rows = stmt.query(params![platform, paired_code])?;
    match rows.next()? {
        Some(row) => Ok(Some(row_to_im_user_permission(row)?)),
        None => Ok(None),
    }
}
