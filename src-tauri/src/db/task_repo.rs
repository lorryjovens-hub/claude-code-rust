use anyhow::Result;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduledTaskRow {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub cron_expression: String,
    pub task_type: String,
    pub task_config: String,
    pub conversation_id: Option<String>,
    pub is_enabled: bool,
    pub last_run_at: Option<String>,
    pub last_run_status: Option<String>,
    pub last_run_output: Option<String>,
    pub next_run_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

fn row_to_task(row: &rusqlite::Row<'_>) -> rusqlite::Result<ScheduledTaskRow> {
    Ok(ScheduledTaskRow {
        id: row.get(0)?,
        name: row.get(1)?,
        description: row.get(2)?,
        cron_expression: row.get(3)?,
        task_type: row.get(4)?,
        task_config: row.get(5)?,
        conversation_id: row.get(6)?,
        is_enabled: row.get::<_, i64>(7)? != 0,
        last_run_at: row.get(8)?,
        last_run_status: row.get(9)?,
        last_run_output: row.get(10)?,
        next_run_at: row.get(11)?,
        created_at: row.get(12)?,
        updated_at: row.get(13)?,
    })
}

pub fn insert_scheduled_task(
    conn: &Connection,
    id: &str,
    name: &str,
    description: Option<&str>,
    cron_expression: &str,
    task_type: &str,
    task_config: &str,
    conversation_id: Option<&str>,
    is_enabled: bool,
    next_run_at: Option<&str>,
    created_at: &str,
    updated_at: &str,
) -> Result<()> {
    let mut stmt = conn.prepare_cached(
        "INSERT INTO scheduled_tasks (id, name, description, cron_expression, task_type, task_config, conversation_id, is_enabled, last_run_at, last_run_status, last_run_output, next_run_at, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, NULL, NULL, NULL, ?9, ?10, ?11)"
    )?;
    stmt.execute(params![
        id,
        name,
        description,
        cron_expression,
        task_type,
        task_config,
        conversation_id,
        is_enabled as i64,
        next_run_at,
        created_at,
        updated_at,
    ])?;
    Ok(())
}

pub fn list_scheduled_tasks(conn: &Connection) -> Result<Vec<ScheduledTaskRow>> {
    let mut stmt = conn.prepare_cached(
        "SELECT id, name, description, cron_expression, task_type, task_config, conversation_id, is_enabled, last_run_at, last_run_status, last_run_output, next_run_at, created_at, updated_at FROM scheduled_tasks ORDER BY created_at DESC"
    )?;
    let rows = stmt.query_map([], |row| row_to_task(row))?;
    let mut result = Vec::new();
    for row in rows {
        result.push(row?);
    }
    Ok(result)
}

pub fn get_scheduled_task(conn: &Connection, id: &str) -> Result<Option<ScheduledTaskRow>> {
    let mut stmt = conn.prepare_cached(
        "SELECT id, name, description, cron_expression, task_type, task_config, conversation_id, is_enabled, last_run_at, last_run_status, last_run_output, next_run_at, created_at, updated_at FROM scheduled_tasks WHERE id = ?1"
    )?;
    let mut rows = stmt.query(params![id])?;
    match rows.next()? {
        Some(row) => Ok(Some(row_to_task(row)?)),
        None => Ok(None),
    }
}

pub fn update_scheduled_task(
    conn: &Connection,
    id: &str,
    name: Option<&str>,
    description: Option<&str>,
    cron_expression: Option<&str>,
    task_type: Option<&str>,
    task_config: Option<&str>,
    conversation_id: Option<&str>,
    is_enabled: Option<bool>,
    next_run_at: Option<&str>,
    updated_at: &str,
) -> Result<()> {
    let mut sql_parts: Vec<&str> = Vec::new();
    let mut param_values: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

    if let Some(v) = name {
        sql_parts.push("name = ?");
        param_values.push(Box::new(v.to_string()));
    }
    if let Some(v) = description {
        sql_parts.push("description = ?");
        param_values.push(Box::new(v.to_string()));
    }
    if let Some(v) = cron_expression {
        sql_parts.push("cron_expression = ?");
        param_values.push(Box::new(v.to_string()));
    }
    if let Some(v) = task_type {
        sql_parts.push("task_type = ?");
        param_values.push(Box::new(v.to_string()));
    }
    if let Some(v) = task_config {
        sql_parts.push("task_config = ?");
        param_values.push(Box::new(v.to_string()));
    }
    if let Some(v) = conversation_id {
        sql_parts.push("conversation_id = ?");
        param_values.push(Box::new(v.to_string()));
    }
    if let Some(v) = is_enabled {
        sql_parts.push("is_enabled = ?");
        param_values.push(Box::new(v as i64));
    }
    if let Some(v) = next_run_at {
        sql_parts.push("next_run_at = ?");
        param_values.push(Box::new(v.to_string()));
    }

    sql_parts.push("updated_at = ?");
    param_values.push(Box::new(updated_at.to_string()));

    if sql_parts.is_empty() || sql_parts.len() == 1 {
        return Ok(());
    }

    let sql = format!(
        "UPDATE scheduled_tasks SET {} WHERE id = ?",
        sql_parts.join(", ")
    );
    param_values.push(Box::new(id.to_string()));

    let mut stmt = conn.prepare_cached(&sql)?;
    stmt.execute(rusqlite::params_from_iter(param_values))?;
    Ok(())
}

pub fn delete_scheduled_task(conn: &Connection, id: &str) -> Result<()> {
    let mut stmt = conn.prepare_cached("DELETE FROM scheduled_tasks WHERE id = ?1")?;
    stmt.execute(params![id])?;
    Ok(())
}

pub fn get_due_tasks(conn: &Connection, now: &str) -> Result<Vec<ScheduledTaskRow>> {
    let mut stmt = conn.prepare_cached(
        "SELECT id, name, description, cron_expression, task_type, task_config, conversation_id, is_enabled, last_run_at, last_run_status, last_run_output, next_run_at, created_at, updated_at FROM scheduled_tasks WHERE is_enabled = 1 AND next_run_at IS NOT NULL AND next_run_at <= ?1 ORDER BY next_run_at ASC"
    )?;
    let rows = stmt.query_map(params![now], |row| row_to_task(row))?;
    let mut result = Vec::new();
    for row in rows {
        result.push(row?);
    }
    Ok(result)
}

pub fn update_task_run_result(
    conn: &Connection,
    id: &str,
    last_run_at: &str,
    last_run_status: &str,
    last_run_output: Option<&str>,
    next_run_at: Option<&str>,
) -> Result<()> {
    let mut stmt = conn.prepare_cached(
        "UPDATE scheduled_tasks SET last_run_at = ?1, last_run_status = ?2, last_run_output = ?3, next_run_at = ?4, updated_at = ?5 WHERE id = ?6"
    )?;
    let now = chrono::Utc::now().to_rfc3339();
    stmt.execute(params![
        last_run_at,
        last_run_status,
        last_run_output.unwrap_or(""),
        next_run_at,
        now,
        id,
    ])?;
    Ok(())
}

pub fn get_enabled_tasks(conn: &Connection) -> Result<Vec<ScheduledTaskRow>> {
    let mut stmt = conn.prepare_cached(
        "SELECT id, name, description, cron_expression, task_type, task_config, conversation_id, is_enabled, last_run_at, last_run_status, last_run_output, next_run_at, created_at, updated_at FROM scheduled_tasks WHERE is_enabled = 1 ORDER BY next_run_at ASC"
    )?;
    let rows = stmt.query_map([], |row| row_to_task(row))?;
    let mut result = Vec::new();
    for row in rows {
        result.push(row?);
    }
    Ok(result)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskRunRow {
    pub id: String,
    pub task_id: String,
    pub started_at: String,
    pub finished_at: Option<String>,
    pub status: String,
    pub output: Option<String>,
    pub error_message: Option<String>,
    pub duration_ms: Option<i64>,
}

fn row_to_task_run(row: &rusqlite::Row<'_>) -> rusqlite::Result<TaskRunRow> {
    Ok(TaskRunRow {
        id: row.get(0)?,
        task_id: row.get(1)?,
        started_at: row.get(2)?,
        finished_at: row.get(3)?,
        status: row.get(4)?,
        output: row.get(5)?,
        error_message: row.get(6)?,
        duration_ms: row.get(7)?,
    })
}

pub fn insert_task_run(
    conn: &Connection,
    id: &str,
    task_id: &str,
    started_at: &str,
) -> Result<()> {
    let mut stmt = conn.prepare_cached(
        "INSERT INTO task_runs (id, task_id, started_at, finished_at, status, output, error_message, duration_ms) VALUES (?1, ?2, ?3, NULL, 'running', NULL, NULL, NULL)"
    )?;
    stmt.execute(params![id, task_id, started_at])?;
    Ok(())
}

pub fn update_task_run_status(
    conn: &Connection,
    id: &str,
    status: &str,
    output: Option<&str>,
    error_message: Option<&str>,
    finished_at: &str,
    duration_ms: i64,
) -> Result<()> {
    let mut stmt = conn.prepare_cached(
        "UPDATE task_runs SET status = ?1, output = ?2, error_message = ?3, finished_at = ?4, duration_ms = ?5 WHERE id = ?6"
    )?;
    stmt.execute(params![status, output, error_message, finished_at, duration_ms, id])?;
    Ok(())
}

pub fn get_task_runs(conn: &Connection, task_id: &str, limit: usize) -> Result<Vec<TaskRunRow>> {
    let mut stmt = conn.prepare_cached(
        "SELECT id, task_id, started_at, finished_at, status, output, error_message, duration_ms FROM task_runs WHERE task_id = ?1 ORDER BY started_at DESC LIMIT ?2"
    )?;
    let rows = stmt.query_map(params![task_id, limit as i64], |row| row_to_task_run(row))?;
    let mut result = Vec::new();
    for row in rows {
        result.push(row?);
    }
    Ok(result)
}

pub fn list_task_runs(conn: &Connection, limit: usize) -> Result<Vec<TaskRunRow>> {
    let mut stmt = conn.prepare_cached(
        "SELECT id, task_id, started_at, finished_at, status, output, error_message, duration_ms FROM task_runs ORDER BY started_at DESC LIMIT ?1"
    )?;
    let rows = stmt.query_map(params![limit as i64], |row| row_to_task_run(row))?;
    let mut result = Vec::new();
    for row in rows {
        result.push(row?);
    }
    Ok(result)
}