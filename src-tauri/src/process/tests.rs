use super::*;

#[test]
fn allowed_commands_includes_git() {
    assert!(ALLOWED_COMMANDS.contains(&"git"));
}

#[test]
fn allowed_commands_includes_cargo() {
    assert!(ALLOWED_COMMANDS.contains(&"cargo"));
}

#[test]
fn allowed_commands_includes_node() {
    assert!(ALLOWED_COMMANDS.contains(&"node"));
}

#[test]
fn allowed_commands_includes_npm() {
    assert!(ALLOWED_COMMANDS.contains(&"npm"));
}

#[test]
fn allowed_commands_includes_python() {
    assert!(ALLOWED_COMMANDS.contains(&"python"));
}

#[test]
fn allowed_commands_includes_go() {
    assert!(ALLOWED_COMMANDS.contains(&"go"));
}

#[test]
fn allowed_commands_includes_docker() {
    assert!(ALLOWED_COMMANDS.contains(&"docker"));
}

#[test]
fn allowed_commands_includes_bash_and_sh() {
    assert!(ALLOWED_COMMANDS.contains(&"bash"));
    assert!(ALLOWED_COMMANDS.contains(&"sh"));
}

#[test]
fn dangerous_cmd_blocked_by_allowlist() {
    // These are truly dangerous commands that should NEVER be in the allowlist
    assert!(!ALLOWED_COMMANDS.contains(&"dd"));
    assert!(!ALLOWED_COMMANDS.contains(&"mkfs"));
    assert!(!ALLOWED_COMMANDS.contains(&"sudo"));
    assert!(!ALLOWED_COMMANDS.contains(&"chmod"));
    assert!(!ALLOWED_COMMANDS.contains(&"chown"));
    assert!(!ALLOWED_COMMANDS.contains(&"format"));
    assert!(!ALLOWED_COMMANDS.contains(&"shutdown"));
    assert!(!ALLOWED_COMMANDS.contains(&"reboot"));
}

#[test]
fn process_manager_spawn_blocks_non_allowlisted() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let mgr = ProcessManager::new();
        let result = mgr.spawn("sudo rm -rf /", None, None).await;
        assert!(result.is_err(), "Expected error for non-allowlisted 'sudo', got: {:?}", result);
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("not in the allowed") || err.contains("allowed commands"),
            "Error message should mention allowlist: {}",
            err
        );
    });
}

#[test]
fn process_manager_new_defaults_empty() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let mgr = ProcessManager::new();
        assert!(mgr.list_processes().await.is_empty());
    });
}
