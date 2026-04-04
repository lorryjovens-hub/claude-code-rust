//! 测试工具链
//!
//! 测试现有工具链的功能：Read/Edit/Write/Glob/Grep/Bash等。

use claude_code_rs::tools;
use claude_code_rs::tools::Tool;
use claude_code_rs::tools::types::ToolUseContext;
use std::sync::Arc;
use tempfile::TempDir;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("=== 测试 Claude Code Rust 工具链 ===\n");

    // 初始化工具管理器
    let tool_manager = tools::init().await?;
    let registry = tool_manager.registry();

    println!("已加载 {} 个工具", registry.len().await);
    println!("");

    // 创建临时目录用于测试
    let temp_dir = TempDir::new()?;
    let temp_path = temp_dir.path();
    println!("测试目录: {}", temp_path.display());

    // 测试 1: 文件写入工具
    println!("\n1. 测试文件写入工具 (Write)...");
    let test_file = temp_path.join("test.txt");
    let write_tool = registry.get("Write").await.expect("Write tool not found");

    let write_input = serde_json::json!({
        "file_path": test_file.to_str().unwrap(),
        "content": "Hello, Claude Code!\nThis is a test file."
    });

    let write_context = ToolUseContext {
        cwd: temp_path.to_path_buf(),
        user_id: "test".to_string(),
        session_id: "test-session".to_string(),
    };

    let write_result = write_tool.execute(write_input.clone(), write_context.clone()).await?;
    println!("写入结果: {:?}", write_result.success);
    assert!(write_result.success);

    // 测试 2: 文件读取工具
    println!("\n2. 测试文件读取工具 (Read)...");
    let read_tool = registry.get("Read").await.expect("Read tool not found");

    let read_input = serde_json::json!({
        "file_path": test_file.to_str().unwrap()
    });

    let read_result = read_tool.execute(read_input, write_context.clone()).await?;
    println!("读取结果成功: {}", read_result.success);
    assert!(read_result.success);

    if let Some(content) = read_result.data.get("content") {
        println!("文件内容: {}", content);
        assert!(content.as_str().unwrap().contains("Hello, Claude Code!"));
    }

    // 测试 3: 文件编辑工具
    println!("\n3. 测试文件编辑工具 (Edit)...");
    let edit_tool = registry.get("Edit").await.expect("Edit tool not found");

    let edit_input = serde_json::json!({
        "file_path": test_file.to_str().unwrap(),
        "old_str": "Hello, Claude Code!",
        "new_str": "Hello, Rust Claude Code!"
    });

    let edit_result = edit_tool.execute(edit_input, write_context.clone()).await?;
    println!("编辑结果成功: {}", edit_result.success);
    assert!(edit_result.success);

    // 再次读取验证编辑
    let read_again_input = serde_json::json!({
        "file_path": test_file.to_str().unwrap()
    });
    let read_again_result = read_tool.execute(read_again_input, write_context.clone()).await?;
    if let Some(content) = read_again_result.data.get("content") {
        println!("编辑后内容: {}", content);
        assert!(content.as_str().unwrap().contains("Hello, Rust Claude Code!"));
    }

    // 测试 4: Glob 工具
    println!("\n4. 测试文件搜索工具 (Glob)...");
    let glob_tool = registry.get("Glob").await.expect("Glob tool not found");

    let glob_input = serde_json::json!({
        "pattern": "*.txt"
    });

    let glob_result = glob_tool.execute(glob_input, write_context.clone()).await?;
    println!("Glob 结果成功: {}", glob_result.success);
    assert!(glob_result.success);

    if let Some(files) = glob_result.data.get("files") {
        println!("找到文件: {}", files);
    }

    // 测试 5: Grep 工具
    println!("\n5. 测试内容搜索工具 (Grep)...");
    let grep_tool = registry.get("Grep").await.expect("Grep tool not found");

    // 创建另一个测试文件
    let test_file2 = temp_path.join("test2.rs");
    std::fs::write(&test_file2, "fn main() {\n    println!(\"Hello, world!\");\n}\n")?;

    let grep_input = serde_json::json!({
        "pattern": "println",
        "path": temp_path.to_str().unwrap()
    });

    let grep_result = grep_tool.execute(grep_input, write_context.clone()).await?;
    println!("Grep 结果成功: {}", grep_result.success);
    assert!(grep_result.success);

    if let Some(matches) = grep_result.data.get("matches") {
        println!("Grep 匹配: {}", matches);
    }

    // 测试 6: Bash 工具 (简单命令)
    println!("\n6. 测试 Bash 工具...");
    let bash_tool = registry.get("Bash").await.expect("Bash tool not found");

    let bash_input = serde_json::json!({
        "command": "echo 'Bash tool test'"
    });

    let bash_result = bash_tool.execute(bash_input, write_context.clone()).await?;
    println!("Bash 结果成功: {}", bash_result.success);
    assert!(bash_result.success);

    if let Some(output) = bash_result.data.get("output") {
        println!("Bash 输出: {}", output);
        assert!(output.as_str().unwrap().contains("Bash tool test"));
    }

    // 列出所有工具
    println!("\n=== 所有可用工具 ===");
    let tool_names = registry.tool_names().await;
    for name in tool_names {
        if let Some(tool) = registry.get(&name).await {
            let metadata = tool.metadata();
            println!("- {}: {}", metadata.name, metadata.description);
        }
    }

    println!("\n=== 所有测试通过 ===");
    Ok(())
}