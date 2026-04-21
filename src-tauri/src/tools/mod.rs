use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
}

pub fn get_tool_definitions() -> Vec<ToolDefinition> {
    vec![
        ToolDefinition {
            name: "Read".to_string(),
            description: "Read a file from the local filesystem. Returns content with line numbers.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "file_path": { "type": "string", "description": "The absolute or relative path to the file to read" },
                    "offset": { "type": "number", "description": "Line number to start reading from (1-based)" },
                    "limit": { "type": "number", "description": "Max number of lines to read" }
                },
                "required": ["file_path"]
            }),
        },
        ToolDefinition {
            name: "Write".to_string(),
            description: "Write content to a file. Creates the file and parent directories if needed.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "file_path": { "type": "string", "description": "The path to the file" },
                    "content": { "type": "string", "description": "The full content to write" }
                },
                "required": ["file_path", "content"]
            }),
        },
        ToolDefinition {
            name: "Edit".to_string(),
            description: "Make an exact string replacement in a file.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "file_path": { "type": "string", "description": "The path to the file" },
                    "old_string": { "type": "string", "description": "The exact text to find" },
                    "new_string": { "type": "string", "description": "The replacement text" },
                    "replace_all": { "type": "boolean", "description": "If true, replace ALL occurrences" }
                },
                "required": ["file_path", "old_string", "new_string"]
            }),
        },
        ToolDefinition {
            name: "Bash".to_string(),
            description: "Execute a shell command and return stdout/stderr.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "command": { "type": "string", "description": "The shell command to execute" }
                },
                "required": ["command"]
            }),
        },
        ToolDefinition {
            name: "Glob".to_string(),
            description: "Find files matching a glob pattern.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "pattern": { "type": "string", "description": "Glob pattern" },
                    "path": { "type": "string", "description": "Base directory to search in" }
                },
                "required": ["pattern"]
            }),
        },
        ToolDefinition {
            name: "Grep".to_string(),
            description: "Search file contents using regex.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "pattern": { "type": "string", "description": "Regex pattern to search for" },
                    "path": { "type": "string", "description": "File or directory to search in" },
                    "include": { "type": "string", "description": "Glob to filter files" }
                },
                "required": ["pattern"]
            }),
        },
        ToolDefinition {
            name: "ListDir".to_string(),
            description: "List the contents of a directory.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Directory path to list" }
                },
                "required": ["path"]
            }),
        },
    ]
}

pub fn execute_tool(name: &str, input: serde_json::Value, cwd: &str) -> Result<serde_json::Value> {
    match name {
        "Read" => tool_read(input, cwd),
        "Write" => tool_write(input, cwd),
        "Edit" => tool_edit(input, cwd),
        "Bash" => tool_bash(input, cwd),
        "Glob" => tool_glob(input, cwd),
        "Grep" => tool_grep(input, cwd),
        "ListDir" => tool_list_dir(input, cwd),
        _ => Ok(serde_json::json!({ "error": format!("Unknown tool: {}", name) })),
    }
}

fn resolve_path(file_path: &str, cwd: &str) -> String {
    if Path::new(file_path).is_absolute() {
        file_path.to_string()
    } else {
        Path::new(cwd).join(file_path).to_string_lossy().to_string()
    }
}

fn tool_read(input: serde_json::Value, cwd: &str) -> Result<serde_json::Value> {
    let file_path = input["file_path"].as_str().ok_or_else(|| anyhow::anyhow!("file_path required"))?;
    let path = resolve_path(file_path, cwd);

    if !Path::new(&path).exists() {
        return Ok(serde_json::json!({ "content": format!("File not found: {}", path), "is_error": true }));
    }

    let content = fs::read_to_string(&path)?;
    let offset = input["offset"].as_u64().unwrap_or(1) as usize;
    let limit = input["limit"].as_u64().unwrap_or(2000) as usize;

    let lines: Vec<&str> = content.lines().collect();
    let selected: Vec<String> = lines
        .iter()
        .skip(offset.saturating_sub(1))
        .take(limit)
        .enumerate()
        .map(|(i, line)| format!("{:>6}\t{}", i + offset, line))
        .collect();

    Ok(serde_json::json!({
        "content": selected.join("\n"),
        "lines": lines.len()
    }))
}

fn tool_write(input: serde_json::Value, cwd: &str) -> Result<serde_json::Value> {
    let file_path = input["file_path"].as_str().ok_or_else(|| anyhow::anyhow!("file_path required"))?;
    let content = input["content"].as_str().ok_or_else(|| anyhow::anyhow!("content required"))?;
    let path = resolve_path(file_path, cwd);

    if let Some(parent) = Path::new(&path).parent() {
        fs::create_dir_all(parent)?;
    }

    fs::write(&path, content)?;

    Ok(serde_json::json!({
        "content": format!("Successfully wrote to {}", path),
        "bytes_written": content.len()
    }))
}

fn tool_edit(input: serde_json::Value, cwd: &str) -> Result<serde_json::Value> {
    let file_path = input["file_path"].as_str().ok_or_else(|| anyhow::anyhow!("file_path required"))?;
    let old_string = input["old_string"].as_str().ok_or_else(|| anyhow::anyhow!("old_string required"))?;
    let new_string = input["new_string"].as_str().ok_or_else(|| anyhow::anyhow!("new_string required"))?;
    let replace_all = input["replace_all"].as_bool().unwrap_or(false);
    let path = resolve_path(file_path, cwd);

    let content = fs::read_to_string(&path)?;

    let (new_content, replacements) = if replace_all {
        let count = content.matches(old_string).count();
        (content.replace(old_string, new_string), count)
    } else {
        let count = content.matches(old_string).count();
        if count == 0 {
            return Ok(serde_json::json!({ "content": "old_string not found in file", "is_error": true }));
        }
        if count > 1 {
            return Ok(serde_json::json!({ "content": format!("old_string found {} times, use replace_all=true", count), "is_error": true }));
        }
        (content.replacen(old_string, new_string, 1), 1)
    };

    fs::write(&path, new_content)?;

    Ok(serde_json::json!({
        "content": format!("Successfully replaced {} occurrence(s) in {}", replacements, path)
    }))
}

fn tool_bash(input: serde_json::Value, _cwd: &str) -> Result<serde_json::Value> {
    let command = input["command"].as_str().ok_or_else(|| anyhow::anyhow!("command required"))?;

    let shell = if cfg!(target_os = "windows") { "cmd" } else { "sh" };
    let flag = if cfg!(target_os = "windows") { "/C" } else { "-c" };

    let output = std::process::Command::new(shell)
        .arg(flag)
        .arg(command)
        .output();

    match output {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            let exit_code = output.status.code().unwrap_or(-1);

            Ok(serde_json::json!({
                "stdout": stdout,
                "stderr": stderr,
                "exit_code": exit_code
            }))
        }
        Err(e) => Ok(serde_json::json!({
            "content": format!("Command failed: {}", e),
            "is_error": true
        })),
    }
}

fn tool_glob(input: serde_json::Value, cwd: &str) -> Result<serde_json::Value> {
    let pattern = input["pattern"].as_str().ok_or_else(|| anyhow::anyhow!("pattern required"))?;
    let base_path = input["path"].as_str().unwrap_or(cwd);

    let mut matches: Vec<String> = Vec::new();
    for entry in walkdir::WalkDir::new(base_path).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if let Some(path_str) = path.to_str() {
            if glob::Pattern::new(pattern).map(|p| p.matches(path_str)).unwrap_or(false) {
                matches.push(path_str.to_string());
            }
        }
    }

    Ok(serde_json::json!({
        "files": matches,
        "count": matches.len()
    }))
}

fn tool_grep(input: serde_json::Value, cwd: &str) -> Result<serde_json::Value> {
    let pattern = input["pattern"].as_str().ok_or_else(|| anyhow::anyhow!("pattern required"))?;
    let search_path = input["path"].as_str().unwrap_or(cwd);

    let re = regex::Regex::new(pattern)?;
    let mut results: Vec<serde_json::Value> = Vec::new();

    for entry in walkdir::WalkDir::new(search_path).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }

        if let Ok(content) = fs::read_to_string(path) {
            for (line_num, line) in content.lines().enumerate() {
                if re.is_match(line) {
                    results.push(serde_json::json!({
                        "file": path.to_string_lossy(),
                        "line": line_num + 1,
                        "content": line
                    }));
                }
            }
        }
    }

    Ok(serde_json::json!({
        "matches": results,
        "count": results.len()
    }))
}

fn tool_list_dir(input: serde_json::Value, _cwd: &str) -> Result<serde_json::Value> {
    let dir_path = input["path"].as_str().ok_or_else(|| anyhow::anyhow!("path required"))?;

    let mut entries: Vec<serde_json::Value> = Vec::new();
    for entry in fs::read_dir(dir_path)? {
        let entry = entry?;
        let metadata = entry.metadata()?;
        entries.push(serde_json::json!({
            "name": entry.file_name().to_string_lossy(),
            "is_dir": metadata.is_dir(),
            "size": metadata.len()
        }));
    }

    Ok(serde_json::json!({ "entries": entries }))
}
