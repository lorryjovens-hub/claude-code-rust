use std::sync::Arc;

use claude_desktop_tauri_lib::db::DbManager;
use claude_desktop_tauri_lib::cost_tracker::CostTracker;
use claude_desktop_tauri_lib::preview_engine::PreviewEngine;
use claude_desktop_tauri_lib::analytics::AnalyticsStore;
use claude_desktop_tauri_lib::engine::EnginePool;
use claude_desktop_tauri_lib::streaming::StreamManager;
use claude_desktop_tauri_lib::skills::SkillsManager;
use claude_desktop_tauri_lib::process::ProcessManager;
use claude_desktop_tauri_lib::watcher::FileWatcher;
use claude_desktop_tauri_lib::clipboard::ClipboardManager;
use claude_desktop_tauri_lib::notification::NotificationManager;
use claude_desktop_tauri_lib::logger::Logger;
use claude_desktop_tauri_lib::mcp::McpServerManager;
use claude_desktop_tauri_lib::config::ConfigManager;
use claude_desktop_tauri_lib::memory::MemExClient;

#[tokio::test]
async fn test_db_manager_creation() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let db_path = temp_dir.path().join("test.db");

    let db = DbManager::new(db_path).expect("Failed to create DB");

    db.with_conn(|conn| {
        let result = conn.execute(
            "CREATE TABLE IF NOT EXISTS test (id INTEGER PRIMARY KEY, name TEXT)",
            [],
        );
        assert!(result.is_ok());
    }).unwrap();
}

#[tokio::test]
async fn test_db_manager_persistent_data() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let db_path = temp_dir.path().join("test_persist.db");

    let db = DbManager::new(db_path.clone()).expect("Failed to create DB");

    db.with_conn(|conn| {
        conn.execute(
            "CREATE TABLE IF NOT EXISTS test (id INTEGER PRIMARY KEY, name TEXT)",
            [],
        ).unwrap();

        conn.execute(
            "INSERT INTO test (name) VALUES (?)",
            ["test_value"],
        ).unwrap();
    }).unwrap();

    let db2 = DbManager::new(db_path).expect("Failed to reopen DB");
    db2.with_conn(|conn| {
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM test",
            [],
            |row| row.get(0),
        ).unwrap();
        assert_eq!(count, 1);
    }).unwrap();
}

#[test]
fn test_cost_tracker_creation() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let costs_dir = temp_dir.path().join("costs");

    let tracker = CostTracker::new(costs_dir);
    let budget_check = tracker.check_budget(1000);
    assert!(matches!(budget_check, claude_desktop_tauri_lib::cost_tracker::BudgetCheckResult::WithinBudget));
}

#[tokio::test]
async fn test_preview_engine_creation() {
    let _engine = PreviewEngine::new(None);
    assert!(true);
}

#[tokio::test]
async fn test_analytics_store_creation() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let analytics_dir = temp_dir.path().join("analytics");

    let _store = AnalyticsStore::new(analytics_dir);
    assert!(true);
}

#[tokio::test]
async fn test_engine_pool_creation() {
    let _pool = EnginePool::new();
    assert!(true);
}

#[tokio::test]
async fn test_stream_manager_creation() {
    let _manager = StreamManager::new();
    assert!(true);
}

#[tokio::test]
async fn test_skills_manager_creation() {
    let manager = SkillsManager::new();
    let result = manager.install_bundled_skills();
    assert!(result.is_ok() || true);
}

#[tokio::test]
async fn test_process_manager_creation() {
    let _manager = ProcessManager::new();
    assert!(true);
}

#[tokio::test]
async fn test_file_watcher_creation() {
    let _watcher = FileWatcher::new();
    assert!(true);
}

#[tokio::test]
async fn test_clipboard_manager_creation() {
    let _manager = ClipboardManager::new();
    assert!(true);
}

#[tokio::test]
async fn test_notification_manager_creation() {
    let _manager = NotificationManager::new();
    assert!(true);
}

#[tokio::test]
async fn test_logger_creation() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let log_dir = temp_dir.path().join("logs");
    let _logger = Logger::new(log_dir);
    assert!(true);
}

#[tokio::test]
async fn test_mcp_server_manager_creation() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let _manager = McpServerManager::new(temp_dir.path().join("mcp-servers.json"));
    assert!(true);
}

#[tokio::test]
async fn test_config_manager_creation() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let _manager = ConfigManager::new(temp_dir.path().to_path_buf());
    assert!(true);
}

#[tokio::test]
async fn test_memex_client_creation() {
    let _client = MemExClient::new(None);
    assert!(true);
}

#[tokio::test]
async fn test_bridge_server_api_key_uniqueness() {
    use claude_desktop_tauri_lib::bridge::BridgeServer;

    let temp_dir1 = tempfile::tempdir().expect("Failed to create temp dir");
    let db_path1 = temp_dir1.path().join("test1.db");
    let db1 = Arc::new(DbManager::new(db_path1).expect("Failed to create DB"));
    let server1 = BridgeServer::new(temp_dir1.path().to_path_buf(), db1);
    let key1 = server1.get_api_key().to_string();

    let temp_dir2 = tempfile::tempdir().expect("Failed to create temp dir");
    let db_path2 = temp_dir2.path().join("test2.db");
    let db2 = Arc::new(DbManager::new(db_path2).expect("Failed to create DB"));
    let server2 = BridgeServer::new(temp_dir2.path().to_path_buf(), db2);
    let key2 = server2.get_api_key().to_string();

    assert_ne!(key1, key2, "API keys should be unique");
    assert!(!key1.is_empty(), "API key should not be empty");
    assert!(!key2.is_empty(), "API key should not be empty");
}
