use anyhow::Result;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectRow {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub instructions: Option<String>,
    pub workspace_path: Option<String>,
    pub is_archived: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectFileRow {
    pub id: String,
    pub project_id: String,
    pub file_name: Option<String>,
    pub file_path: Option<String>,
    pub file_size: Option<i64>,
    pub mime_type: Option<String>,
}

fn row_to_project(row: &rusqlite::Row<'_>) -> rusqlite::Result<ProjectRow> {
    Ok(ProjectRow {
        id: row.get(0)?,
        name: row.get(1)?,
        description: row.get(2)?,
        instructions: row.get(3)?,
        workspace_path: row.get(4)?,
        is_archived: row.get::<_, i64>(5)? != 0,
        created_at: row.get(6)?,
        updated_at: row.get(7)?,
    })
}

fn row_to_project_file(row: &rusqlite::Row<'_>) -> rusqlite::Result<ProjectFileRow> {
    Ok(ProjectFileRow {
        id: row.get(0)?,
        project_id: row.get(1)?,
        file_name: row.get(2)?,
        file_path: row.get(3)?,
        file_size: row.get(4)?,
        mime_type: row.get(5)?,
    })
}

pub fn insert_project(
    conn: &Connection,
    id: &str,
    name: &str,
    description: Option<&str>,
    instructions: Option<&str>,
    workspace_path: Option<&str>,
    is_archived: bool,
    created_at: &str,
    updated_at: &str,
) -> Result<()> {
    let mut stmt = conn.prepare_cached(
        "INSERT INTO projects (id, name, description, instructions, workspace_path, is_archived, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)"
    )?;
    stmt.execute(params![
        id,
        name,
        description,
        instructions,
        workspace_path,
        is_archived as i64,
        created_at,
        updated_at,
    ])?;
    Ok(())
}

pub fn get_project(conn: &Connection, id: &str) -> Result<Option<ProjectRow>> {
    let mut stmt = conn.prepare_cached(
        "SELECT id, name, description, instructions, workspace_path, is_archived, created_at, updated_at FROM projects WHERE id = ?1"
    )?;
    let mut rows = stmt.query(params![id])?;
    match rows.next()? {
        Some(row) => Ok(Some(row_to_project(row)?)),
        None => Ok(None),
    }
}

pub fn list_projects(conn: &Connection) -> Result<Vec<ProjectRow>> {
    let mut stmt = conn.prepare_cached(
        "SELECT id, name, description, instructions, workspace_path, is_archived, created_at, updated_at FROM projects ORDER BY updated_at DESC"
    )?;
    let rows = stmt.query_map([], |row| row_to_project(row))?;
    let mut result = Vec::new();
    for row in rows {
        result.push(row?);
    }
    Ok(result)
}

pub fn update_project(
    conn: &Connection,
    id: &str,
    name: Option<&str>,
    description: Option<&str>,
    instructions: Option<&str>,
    workspace_path: Option<&str>,
    is_archived: Option<bool>,
) -> Result<()> {
    let current = get_project(conn, id)?
        .ok_or_else(|| anyhow::anyhow!("Project not found: {}", id))?;

    let name = name.unwrap_or(&current.name);
    let description = description.or(current.description.as_deref());
    let instructions = instructions.or(current.instructions.as_deref());
    let workspace_path = workspace_path.or(current.workspace_path.as_deref());
    let is_archived = is_archived.unwrap_or(current.is_archived);

    let mut stmt = conn.prepare_cached(
        "UPDATE projects SET name = ?1, description = ?2, instructions = ?3, workspace_path = ?4, is_archived = ?5, updated_at = ?6 WHERE id = ?7"
    )?;
    let now = chrono::Utc::now().to_rfc3339();
    stmt.execute(params![
        name,
        description,
        instructions,
        workspace_path,
        is_archived as i64,
        now,
        id,
    ])?;
    Ok(())
}

pub fn delete_project(conn: &Connection, id: &str) -> Result<()> {
    let mut stmt = conn.prepare_cached("DELETE FROM projects WHERE id = ?1")?;
    stmt.execute(params![id])?;
    Ok(())
}

pub fn insert_project_file(
    conn: &Connection,
    id: &str,
    project_id: &str,
    file_name: Option<&str>,
    file_path: Option<&str>,
    file_size: Option<i64>,
    mime_type: Option<&str>,
) -> Result<()> {
    let mut stmt = conn.prepare_cached(
        "INSERT INTO project_files (id, project_id, file_name, file_path, file_size, mime_type) VALUES (?1, ?2, ?3, ?4, ?5, ?6)"
    )?;
    stmt.execute(params![id, project_id, file_name, file_path, file_size, mime_type])?;
    Ok(())
}

pub fn list_project_files(conn: &Connection, project_id: &str) -> Result<Vec<ProjectFileRow>> {
    let mut stmt = conn.prepare_cached(
        "SELECT id, project_id, file_name, file_path, file_size, mime_type FROM project_files WHERE project_id = ?1"
    )?;
    let rows = stmt.query_map(params![project_id], |row| row_to_project_file(row))?;
    let mut result = Vec::new();
    for row in rows {
        result.push(row?);
    }
    Ok(result)
}

pub fn delete_project_file(conn: &Connection, id: &str) -> Result<()> {
    let mut stmt = conn.prepare_cached("DELETE FROM project_files WHERE id = ?1")?;
    stmt.execute(params![id])?;
    Ok(())
}
