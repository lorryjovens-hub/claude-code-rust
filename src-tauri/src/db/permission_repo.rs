use anyhow::Result;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionApprovalRow {
    pub id: String,
    pub conversation_id: String,
    pub message_id: String,
    pub tool_name: String,
    pub action: String,
    pub risk_level: String,
    pub status: String,
    pub user_decision: Option<String>,
    pub decision_reason: Option<String>,
    pub created_at: String,
    pub decided_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlwaysAllowRuleRow {
    pub id: String,
    pub rule_pattern: String,
    pub rule_type: String,
    pub is_enabled: bool,
    pub created_at: String,
    pub updated_at: String,
}

fn row_to_permission_approval(row: &rusqlite::Row<'_>) -> rusqlite::Result<PermissionApprovalRow> {
    Ok(PermissionApprovalRow {
        id: row.get(0)?,
        conversation_id: row.get(1)?,
        message_id: row.get(2)?,
        tool_name: row.get(3)?,
        action: row.get(4)?,
        risk_level: row.get(5)?,
        status: row.get(6)?,
        user_decision: row.get(7)?,
        decision_reason: row.get(8)?,
        created_at: row.get(9)?,
        decided_at: row.get(10)?,
    })
}

fn row_to_always_allow_rule(row: &rusqlite::Row<'_>) -> rusqlite::Result<AlwaysAllowRuleRow> {
    Ok(AlwaysAllowRuleRow {
        id: row.get(0)?,
        rule_pattern: row.get(1)?,
        rule_type: row.get(2)?,
        is_enabled: row.get::<_, i64>(3)? != 0,
        created_at: row.get(4)?,
        updated_at: row.get(5)?,
    })
}

pub fn insert_permission_approval(
    conn: &Connection,
    id: &str,
    conversation_id: &str,
    message_id: &str,
    tool_name: &str,
    action: &str,
    risk_level: &str,
    status: &str,
    created_at: &str,
) -> Result<()> {
    let mut stmt = conn.prepare_cached(
        "INSERT INTO permission_approvals (id, conversation_id, message_id, tool_name, action, risk_level, status, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)"
    )?;
    stmt.execute(params![
        id,
        conversation_id,
        message_id,
        tool_name,
        action,
        risk_level,
        status,
        created_at,
    ])?;
    Ok(())
}

pub fn get_pending_approvals(
    conn: &Connection,
    conversation_id: &str,
) -> Result<Vec<PermissionApprovalRow>> {
    let mut stmt = conn.prepare_cached(
        "SELECT id, conversation_id, message_id, tool_name, action, risk_level, status, user_decision, decision_reason, created_at, decided_at FROM permission_approvals WHERE conversation_id = ?1 AND status = 'pending' ORDER BY created_at ASC"
    )?;
    let rows = stmt.query_map(params![conversation_id], |row| row_to_permission_approval(row))?;
    let mut result = Vec::new();
    for row in rows {
        result.push(row?);
    }
    Ok(result)
}

pub fn get_approval_by_id(
    conn: &Connection,
    id: &str,
) -> Result<Option<PermissionApprovalRow>> {
    let mut stmt = conn.prepare_cached(
        "SELECT id, conversation_id, message_id, tool_name, action, risk_level, status, user_decision, decision_reason, created_at, decided_at FROM permission_approvals WHERE id = ?1"
    )?;
    let mut rows = stmt.query(params![id])?;
    match rows.next()? {
        Some(row) => Ok(Some(row_to_permission_approval(row)?)),
        None => Ok(None),
    }
}

pub fn update_approval_status(
    conn: &Connection,
    id: &str,
    status: &str,
    user_decision: Option<&str>,
    decision_reason: Option<&str>,
    decided_at: &str,
) -> Result<()> {
    let mut stmt = conn.prepare_cached(
        "UPDATE permission_approvals SET status = ?1, user_decision = ?2, decision_reason = ?3, decided_at = ?4 WHERE id = ?5"
    )?;
    stmt.execute(params![status, user_decision, decision_reason, decided_at, id])?;
    Ok(())
}

pub fn insert_always_allow_rule(
    conn: &Connection,
    id: &str,
    rule_pattern: &str,
    rule_type: &str,
    created_at: &str,
    updated_at: &str,
) -> Result<()> {
    let mut stmt = conn.prepare_cached(
        "INSERT INTO always_allow_rules (id, rule_pattern, rule_type, is_enabled, created_at, updated_at) VALUES (?1, ?2, ?3, 1, ?4, ?5)"
    )?;
    stmt.execute(params![id, rule_pattern, rule_type, created_at, updated_at])?;
    Ok(())
}

pub fn get_always_allow_rules(conn: &Connection) -> Result<Vec<AlwaysAllowRuleRow>> {
    let mut stmt = conn.prepare_cached(
        "SELECT id, rule_pattern, rule_type, is_enabled, created_at, updated_at FROM always_allow_rules WHERE is_enabled = 1 ORDER BY created_at DESC"
    )?;
    let rows = stmt.query_map([], |row| row_to_always_allow_rule(row))?;
    let mut result = Vec::new();
    for row in rows {
        result.push(row?);
    }
    Ok(result)
}

pub fn delete_always_allow_rule(conn: &Connection, id: &str) -> Result<()> {
    let mut stmt = conn.prepare_cached("DELETE FROM always_allow_rules WHERE id = ?1")?;
    stmt.execute(params![id])?;
    Ok(())
}

pub fn check_always_allowed(conn: &Connection, tool_name: &str, action: &str) -> bool {
    let rules = get_always_allow_rules(conn).unwrap_or_default();
    for rule in &rules {
        if rule.rule_type == "tool" && rule.rule_pattern == tool_name {
            return true;
        }
        if rule.rule_type == "action" && rule.rule_pattern == action {
            return true;
        }
        if rule.rule_type == "combined" {
            let combined = format!("{}:{}", tool_name, action);
            if rule.rule_pattern == combined {
                return true;
            }
        }
    }
    false
}