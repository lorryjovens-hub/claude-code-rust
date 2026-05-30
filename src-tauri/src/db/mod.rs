pub mod schema;
pub mod conversation_repo;
pub mod message_repo;
pub mod project_repo;
pub mod task_repo;
pub mod diff_repo;
pub mod migration;
pub mod permission_repo;
pub mod h5_repo;
pub mod im_config_repo;
pub mod im_permission_repo;

use anyhow::Result;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::Connection;
use std::path::PathBuf;

pub struct DbManager {
    pool: Pool<SqliteConnectionManager>,
}

impl DbManager {
    pub fn new(path: PathBuf) -> Result<Self> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let manager = SqliteConnectionManager::file(&path);
        let pool = Pool::builder()
            .max_size(10)
            .min_idle(Some(2))
            .connection_timeout(std::time::Duration::from_secs(5))
            .idle_timeout(Some(std::time::Duration::from_secs(30)))
            .build(manager)?;
        let conn = pool.get()?;
        conn.execute_batch(
            "PRAGMA journal_mode=WAL;
             PRAGMA busy_timeout=5000;
             PRAGMA synchronous=NORMAL;
             PRAGMA foreign_keys=ON;",
        )?;
        Ok(Self { pool })
    }

    pub fn init(&self) -> Result<()> {
        self.with_conn(|conn| {
            conn.execute_batch("BEGIN TRANSACTION")?;

            let result: Result<()> = (|| {
                conn.execute_batch(schema::SCHEMA_SQL)?;

                let has_col: i64 = conn
                    .query_row(
                        "SELECT COUNT(*) FROM pragma_table_info('im_configs') WHERE name='connection_type'",
                        [],
                        |row| row.get(0),
                    )
                    .unwrap_or(0);

                if has_col == 0 {
                    conn.execute(
                        "ALTER TABLE im_configs ADD COLUMN connection_type TEXT NOT NULL DEFAULT 'webhook'",
                        [],
                    )?;
                }

                conn.execute("CREATE INDEX IF NOT EXISTS idx_im_configs_platform ON im_configs(platform)", [])?;
                conn.execute("CREATE INDEX IF NOT EXISTS idx_im_configs_status ON im_configs(status)", [])?;
                conn.execute("CREATE INDEX IF NOT EXISTS idx_im_configs_connection_type ON im_configs(connection_type)", [])?;
                conn.execute("CREATE INDEX IF NOT EXISTS idx_im_connections_platform ON im_connections(platform)", [])?;
                conn.execute("CREATE INDEX IF NOT EXISTS idx_im_connections_status ON im_connections(status)", [])?;
                conn.execute("CREATE INDEX IF NOT EXISTS idx_im_connections_type ON im_connections(connection_type)", [])?;
                conn.execute("CREATE INDEX IF NOT EXISTS idx_im_message_stats_platform_date ON im_message_stats(platform, date)", [])?;
                conn.execute("CREATE INDEX IF NOT EXISTS idx_im_message_stats_date ON im_message_stats(date)", [])?;
                conn.execute("CREATE INDEX IF NOT EXISTS idx_im_user_permissions_platform_user ON im_user_permissions(platform, user_id)", [])?;
                conn.execute("CREATE INDEX IF NOT EXISTS idx_im_user_permissions_user_id ON im_user_permissions(user_id)", [])?;
                conn.execute("CREATE INDEX IF NOT EXISTS idx_im_user_permissions_paired_code ON im_user_permissions(paired_code)", [])?;
                conn.execute("CREATE INDEX IF NOT EXISTS idx_im_error_logs_platform ON im_error_logs(platform)", [])?;
                conn.execute("CREATE INDEX IF NOT EXISTS idx_im_error_logs_error_type ON im_error_logs(error_type)", [])?;
                conn.execute("CREATE INDEX IF NOT EXISTS idx_im_error_logs_created_at ON im_error_logs(created_at)", [])?;
                conn.execute("CREATE INDEX IF NOT EXISTS idx_im_sessions_platform ON im_sessions(platform)", [])?;
                conn.execute("CREATE INDEX IF NOT EXISTS idx_im_sessions_user ON im_sessions(user_id)", [])?;
                conn.execute("CREATE INDEX IF NOT EXISTS idx_im_sessions_chat ON im_sessions(chat_id)", [])?;

                let has_session_key: i64 = conn
                    .query_row(
                        "SELECT COUNT(*) FROM pragma_table_info('im_sessions') WHERE name='session_key'",
                        [],
                        |row| row.get(0),
                    )
                    .unwrap_or(0);
                if has_session_key == 0 {
                    conn.execute(
                        "ALTER TABLE im_sessions ADD COLUMN session_key TEXT NOT NULL DEFAULT ''",
                        [],
                    )?;
                }

                for col_name in &["thread_id", "message_count", "last_message_content", "metadata", "updated_at"] {
                    // Bug #22 fix: Validate column name to ensure no SQL injection
                    // (col_name comes from hardcoded array, but defensive validation is good practice)
                    debug_assert!(col_name.chars().all(|c| c.is_alphanumeric() || c == '_'),
                        "Invalid column name in migration: {}", col_name);
                    let has_col: i64 = conn
                        .query_row(
                            &format!("SELECT COUNT(*) FROM pragma_table_info('im_sessions') WHERE name='{}'", col_name),
                            [],
                            |row| row.get(0),
                        )
                        .unwrap_or(0);
                    if has_col == 0 {
                        conn.execute(
                            &format!("ALTER TABLE im_sessions ADD COLUMN {} TEXT", col_name),
                            [],
                        )?;
                    }
                }

                Ok(())
            })();

            match result {
                Ok(()) => {
                    conn.execute_batch("COMMIT")?;
                    Ok(()) as Result<()>
                }
                Err(e) => {
                    let _ = conn.execute_batch("ROLLBACK");
                    Err(e) as Result<()>
                }
            }
        })??;
        Ok(())
    }

    pub fn with_conn<F, R>(&self, f: F) -> Result<R>
    where
        F: FnOnce(&Connection) -> R,
    {
        let conn = self.pool.get()?;
        Ok(f(&conn))
    }
}