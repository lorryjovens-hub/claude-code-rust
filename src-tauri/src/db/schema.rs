pub const SCHEMA_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS conversations (
    id TEXT PRIMARY KEY,
    title TEXT,
    model TEXT,
    provider TEXT,
    workspace_path TEXT,
    project_id TEXT,
    research_mode INTEGER DEFAULT 0,
    pinned INTEGER DEFAULT 0,
    archived INTEGER DEFAULT 0,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    message_count INTEGER DEFAULT 0
);

CREATE TABLE IF NOT EXISTS messages (
    id TEXT PRIMARY KEY,
    conversation_id TEXT NOT NULL,
    role TEXT NOT NULL,
    content TEXT NOT NULL,
    thinking TEXT,
    created_at TEXT NOT NULL,
    is_compact_boundary INTEGER DEFAULT 0,
    sort_order INTEGER NOT NULL,
    FOREIGN KEY (conversation_id) REFERENCES conversations(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS tool_calls (
    id TEXT PRIMARY KEY,
    message_id TEXT NOT NULL,
    name TEXT NOT NULL,
    input TEXT,
    output TEXT,
    is_error INTEGER DEFAULT 0,
    sort_order INTEGER NOT NULL,
    FOREIGN KEY (message_id) REFERENCES messages(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS attachments (
    id TEXT PRIMARY KEY,
    message_id TEXT NOT NULL,
    file_name TEXT,
    file_type TEXT,
    mime_type TEXT,
    file_size INTEGER,
    source TEXT,
    FOREIGN KEY (message_id) REFERENCES messages(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS projects (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT,
    instructions TEXT,
    workspace_path TEXT,
    is_archived INTEGER DEFAULT 0,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS project_files (
    id TEXT PRIMARY KEY,
    project_id TEXT NOT NULL,
    file_name TEXT,
    file_path TEXT,
    file_size INTEGER,
    mime_type TEXT,
    FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS permission_approvals (
    id TEXT PRIMARY KEY,
    conversation_id TEXT NOT NULL,
    message_id TEXT NOT NULL,
    tool_name TEXT NOT NULL,
    action TEXT NOT NULL,
    risk_level TEXT NOT NULL DEFAULT 'medium',
    status TEXT NOT NULL DEFAULT 'pending',
    user_decision TEXT,
    decision_reason TEXT,
    created_at TEXT NOT NULL,
    decided_at TEXT,
    FOREIGN KEY (conversation_id) REFERENCES conversations(id) ON DELETE CASCADE,
    FOREIGN KEY (message_id) REFERENCES messages(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS always_allow_rules (
    id TEXT PRIMARY KEY,
    rule_pattern TEXT NOT NULL,
    rule_type TEXT NOT NULL,
    is_enabled INTEGER DEFAULT 1,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS scheduled_tasks (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT,
    cron_expression TEXT NOT NULL,
    task_type TEXT NOT NULL,
    task_config TEXT NOT NULL,
    conversation_id TEXT,
    is_enabled INTEGER DEFAULT 1,
    last_run_at TEXT,
    last_run_status TEXT,
    last_run_output TEXT,
    next_run_at TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    FOREIGN KEY (conversation_id) REFERENCES conversations(id) ON DELETE SET NULL
);

CREATE TABLE IF NOT EXISTS h5_access_tokens (
    id TEXT PRIMARY KEY,
    token TEXT NOT NULL UNIQUE,
    conversation_id TEXT NOT NULL,
    expires_at TEXT NOT NULL,
    is_revoked INTEGER DEFAULT 0,
    created_at TEXT NOT NULL,
    used_at TEXT,
    FOREIGN KEY (conversation_id) REFERENCES conversations(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS code_diffs (
    id TEXT PRIMARY KEY,
    conversation_id TEXT NOT NULL,
    message_id TEXT NOT NULL,
    file_path TEXT NOT NULL,
    original_content TEXT,
    modified_content TEXT,
    diff_text TEXT,
    status TEXT NOT NULL DEFAULT 'pending',
    applied_at TEXT,
    created_at TEXT NOT NULL,
    FOREIGN KEY (conversation_id) REFERENCES conversations(id) ON DELETE CASCADE,
    FOREIGN KEY (message_id) REFERENCES messages(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS task_runs (
    id TEXT PRIMARY KEY,
    task_id TEXT NOT NULL,
    started_at TEXT NOT NULL,
    finished_at TEXT,
    status TEXT NOT NULL DEFAULT 'running',
    output TEXT,
    error_message TEXT,
    duration_ms INTEGER,
    FOREIGN KEY (task_id) REFERENCES scheduled_tasks(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_messages_conversation_id ON messages(conversation_id);
CREATE INDEX IF NOT EXISTS idx_messages_created_at ON messages(created_at);
CREATE INDEX IF NOT EXISTS idx_conversations_updated_at ON conversations(updated_at);
CREATE INDEX IF NOT EXISTS idx_conversations_model ON conversations(model);
CREATE INDEX IF NOT EXISTS idx_tool_calls_message_id ON tool_calls(message_id);
CREATE INDEX IF NOT EXISTS idx_attachments_message_id ON attachments(message_id);
CREATE INDEX IF NOT EXISTS idx_project_files_project_id ON project_files(project_id);
CREATE INDEX IF NOT EXISTS idx_permission_approvals_conversation ON permission_approvals(conversation_id);
CREATE INDEX IF NOT EXISTS idx_permission_approvals_status ON permission_approvals(status);
CREATE INDEX IF NOT EXISTS idx_scheduled_tasks_enabled ON scheduled_tasks(is_enabled);
CREATE INDEX IF NOT EXISTS idx_scheduled_tasks_next_run ON scheduled_tasks(next_run_at);
CREATE INDEX IF NOT EXISTS idx_h5_tokens_conversation ON h5_access_tokens(conversation_id);
CREATE INDEX IF NOT EXISTS idx_h5_tokens_expires ON h5_access_tokens(expires_at);
CREATE INDEX IF NOT EXISTS idx_code_diffs_conversation ON code_diffs(conversation_id);
CREATE INDEX IF NOT EXISTS idx_code_diffs_status ON code_diffs(status);
CREATE TABLE IF NOT EXISTS im_configs (
    id TEXT PRIMARY KEY,
    platform TEXT NOT NULL UNIQUE,
    config_json TEXT NOT NULL,
    connection_type TEXT NOT NULL DEFAULT 'webhook' CHECK(connection_type IN ('webhook', 'websocket', 'polling')),
    status TEXT NOT NULL DEFAULT 'disconnected',
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS im_connections (
    id TEXT PRIMARY KEY,
    platform TEXT NOT NULL,
    connection_type TEXT NOT NULL DEFAULT 'websocket' CHECK(connection_type IN ('websocket', 'polling')),
    app_id TEXT,
    app_secret TEXT,
    bot_token TEXT,
    status TEXT NOT NULL DEFAULT 'disconnected',
    ws_url TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS im_message_stats (
    id TEXT PRIMARY KEY,
    platform TEXT NOT NULL,
    date TEXT NOT NULL,
    message_count INTEGER DEFAULT 0,
    user_count INTEGER DEFAULT 0,
    avg_response_time REAL DEFAULT 0.0,
    created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS im_user_permissions (
    id TEXT PRIMARY KEY,
    platform TEXT NOT NULL,
    user_id TEXT NOT NULL,
    permission_mode TEXT NOT NULL DEFAULT 'allow' CHECK(permission_mode IN ('allow', 'deny')),
    is_allowed INTEGER DEFAULT 1,
    paired_code TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS im_error_logs (
    id TEXT PRIMARY KEY,
    platform TEXT NOT NULL,
    error_type TEXT NOT NULL,
    error_message TEXT,
    stack_trace TEXT,
    created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS im_sessions (
    session_key TEXT PRIMARY KEY,
    platform TEXT NOT NULL,
    user_id TEXT NOT NULL,
    chat_id TEXT,
    thread_id TEXT,
    conversation_context TEXT,
    last_message_at TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT,
    message_count INTEGER DEFAULT 0,
    metadata TEXT
);

CREATE INDEX IF NOT EXISTS idx_task_runs_task_id ON task_runs(task_id);
CREATE INDEX IF NOT EXISTS idx_task_runs_started_at ON task_runs(started_at);

CREATE TABLE IF NOT EXISTS feishu_chat_mappings (
    chat_id TEXT PRIMARY KEY,
    conversation_id TEXT NOT NULL,
    title TEXT NOT NULL DEFAULT '',
    created_at TEXT NOT NULL,
    last_active_at TEXT NOT NULL,
    message_count INTEGER DEFAULT 0,
    FOREIGN KEY (conversation_id) REFERENCES conversations(id) ON DELETE CASCADE
);
"#;
