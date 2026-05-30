use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::Arc;
use tokio::process::Command;
use tokio::sync::RwLock;

pub mod macros;

#[cfg(windows)]
use std::os::windows::process::CommandExt;

const CREATE_NO_WINDOW: u32 = 0x08000000;
// ---- memory tool stubs (dispatched to MemEx at runtime by tool_loop) ----

fn tool_memory_search(input: serde_json::Value) -> Result<serde_json::Value> {
    Ok(serde_json::json!({
        "stub": true,
        "message": "memory_search should be intercepted by ToolLoopExecutor",
        "query": input["query"],
        "results": []
    }))
}

fn tool_memory_ingest(input: serde_json::Value) -> Result<serde_json::Value> {
    Ok(serde_json::json!({
        "stub": true,
        "message": "memory_ingest should be intercepted by ToolLoopExecutor",
        "content": input["content"],
        "stored": false
    }))
}



#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
}

pub fn get_tool_definitions() -> Vec<ToolDefinition> {
    let mut defs = vec![
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
                    "command": { "type": "string", "description": "The shell command to execute" },
                    "timeout": { "type": "number", "description": "Timeout in seconds (default: 120)" },
                    "shell": { "type": "string", "description": "Shell to use: bash, zsh, fish, sh, cmd (default: bash)" }
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
        ToolDefinition {
            name: "WebFetch".to_string(),
            description: "Fetch content from a URL.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "url": { "type": "string", "description": "The URL to fetch" },
                    "headers": { "type": "object", "description": "Optional HTTP headers" }
                },
                "required": ["url"]
            }),
        },
        ToolDefinition {
            name: "WebSearch".to_string(),
            description: "Search the web for information.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "query": { "type": "string", "description": "The search query" }
                },
                "required": ["query"]
            }),
        },
        ToolDefinition {
            name: "MultiEdit".to_string(),
            description: "Make multiple string replacements in a file at once.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "file_path": { "type": "string", "description": "The path to the file" },
                    "edits": {
                        "type": "array",
                        "items": {
                            "type": "object",
                            "properties": {
                                "old_string": { "type": "string" },
                                "new_string": { "type": "string" }
                            }
                        }
                    }
                },
                "required": ["file_path", "edits"]
            }),
        },
        ToolDefinition {
            name: "AskUserQuestion".to_string(),
            description: "Ask the user a question with multiple options. Returns the user's selected options and any custom input.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "question": { "type": "string", "description": "The question to ask the user" },
                    "description": { "type": "string", "description": "Additional context or description for the question" },
                    "options": {
                        "type": "array",
                        "items": {
                            "type": "object",
                            "properties": {
                                "label": { "type": "string", "description": "The display label for this option" },
                                "description": { "type": "string", "description": "Additional description for this option" }
                            },
                            "required": ["label"]
                        },
                        "description": "The available options for the user to choose from"
                    },
                    "multiSelect": { "type": "boolean", "description": "Whether the user can select multiple options (default: false)" }
                },
                "required": ["question", "options"]
            }),
        },
        ToolDefinition {
            name: "git_status".to_string(),
            description: "Runs git status in the workspace directory to see changed, staged, and untracked files.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "The workspace directory path to run git status in" }
                },
                "required": ["path"]
            }),
        },
        ToolDefinition {
            name: "git_diff".to_string(),
            description: "Runs git diff to show changes in the working directory or staging area.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "The workspace directory path" },
                    "staged": { "type": "boolean", "description": "If true, show staged changes (git diff --staged)" },
                    "file": { "type": "string", "description": "Optional specific file path to diff" }
                },
                "required": ["path"]
            }),
        },
        ToolDefinition {
            name: "git_log".to_string(),
            description: "Runs git log to show commit history.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "The workspace directory path" },
                    "count": { "type": "number", "description": "Number of commits to show (default: 10)" },
                    "oneline": { "type": "boolean", "description": "Use oneline format (default: true)" }
                },
                "required": ["path"]
            }),
        },
        ToolDefinition {
            name: "git_commit".to_string(),
            description: "Stages all changes and commits them with the given message. Runs git add -A then git commit.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "The workspace directory path" },
                    "message": { "type": "string", "description": "The commit message" }
                },
                "required": ["path", "message"]
            }),
        },
        ToolDefinition {
            name: "git_add".to_string(),
            description: "Stages specific files for the next commit. Runs git add with the specified file paths.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "The workspace directory path" },
                    "files": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "Array of file paths to stage"
                    }
                },
                "required": ["path", "files"]
            }),
        },
        ToolDefinition {
            name: "computer_use".to_string(),
            description: "Control the computer: move mouse, click, type text, press keys, take screenshots. Use this to interact with the desktop GUI.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "action_type": {
                        "type": "string",
                        "enum": ["MouseMove", "MouseClick", "MouseDown", "MouseUp", "MouseScroll", "KeyPress", "KeyDown", "KeyUp", "TypeText", "Screenshot", "Wait"],
                        "description": "The type of computer action to perform"
                    },
                    "coordinate": {
                        "type": "object",
                        "properties": {
                            "x": { "type": "integer", "description": "X coordinate (pixels)" },
                            "y": { "type": "integer", "description": "Y coordinate (pixels)" }
                        },
                        "description": "Screen coordinate for mouse actions"
                    },
                    "button": {
                        "type": "string",
                        "enum": ["Left", "Right", "Middle", "Back", "Forward"],
                        "description": "Mouse button for click actions (default: Left)"
                    },
                    "key": {
                        "type": "string",
                        "description": "Key name for keyboard actions (e.g. 'Enter', 'Tab', 'Escape', 'a', 'F1')"
                    },
                    "text": {
                        "type": "string",
                        "description": "Text to type for TypeText action"
                    },
                    "scroll_y": {
                        "type": "integer",
                        "description": "Vertical scroll amount (positive = down, negative = up)"
                    },
                    "scroll_x": {
                        "type": "integer",
                        "description": "Horizontal scroll amount (positive = right, negative = left)"
                    },
                    "duration_ms": {
                        "type": "integer",
                        "description": "Duration in milliseconds for Wait action"
                    }
                },
                "required": ["action_type"]
            }),
        },
        ToolDefinition {
            name: "memory_search".to_string(),
            description: "Search your persistent memory for relevant information from previous conversations. Use this to recall important context, decisions, or facts from past interactions.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "query": { "type": "string", "description": "The search query to find relevant memories" },
                    "top_k": { "type": "number", "description": "Number of results to return (default: 5)" }
                },
                "required": ["query"]
            }),
        },
        ToolDefinition {
            name: "memory_ingest".to_string(),
            description: "Store important information in persistent memory for future retrieval. Use this to remember key decisions, architecture, configurations, or any information that should persist across conversations.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "content": { "type": "string", "description": "The information to store in memory" },
                    "importance": { "type": "number", "description": "Importance score 0.0-1.0 (default: auto-estimated)" }
                },
                "required": ["content"]
            }),
        },
    ];

    for macro_def in macros::get_macro_definitions() {
        defs.push(ToolDefinition {
            name: macro_def.name,
            description: macro_def.description,
            input_schema: macro_def.input_schema,
        });
    }

    defs
}

/// Unified tool dispatch macro — eliminates duplication between
/// execute_tool (sync) and execute_tool_async.  Only the three divergent
/// tools (Bash, WebFetch, WebSearch) are parameterised; the other 19+
/// arms are shared.
macro_rules! tool_dispatch {
    ($name:ident, $input:ident, $cwd:ident,
     Bash => $bash:expr,
     WebFetch => $webfetch:expr,
     WebSearch => $websearch:expr $(,)?) => {
        match $name {
            "Read" => tool_read($input, $cwd),
            "Write" => tool_write($input, $cwd),
            "Edit" => tool_edit($input, $cwd),
            "Bash" => $bash,
            "Glob" => tool_glob($input, $cwd),
            "Grep" => tool_grep($input, $cwd),
            "ListDir" => tool_list_dir($input, $cwd),
            "WebFetch" => $webfetch,
            "WebSearch" => $websearch,
            "MultiEdit" => tool_multi_edit($input, $cwd),
            "AskUserQuestion" => tool_ask_user_question($input),
            "git_status" => tool_git_status($input),
            "git_diff" => tool_git_diff($input),
            "git_log" => tool_git_log($input),
            "git_commit" => tool_git_commit($input),
            "git_add" => tool_git_add($input),
            "computer_use" => tool_computer_use($input),
            "memory_search" => tool_memory_search($input),
            "memory_ingest" => tool_memory_ingest($input),
            "smart_edit" => macros::execute_smart_edit($input, $cwd),
            "smart_grep" => macros::execute_smart_grep($input, $cwd),
            "smart_project_scan" => macros::execute_smart_project_scan($input, $cwd),
            _ => Ok(serde_json::json!({ "error": format!("Unknown tool: {}", $name) })),
        }
    };
}

pub fn execute_tool(name: &str, input: serde_json::Value, cwd: &str) -> Result<serde_json::Value> {
    tool_dispatch!(name, input, cwd,
        Bash => tool_bash(input, cwd),
        WebFetch => tool_web_fetch_blocking(input),
        WebSearch => tool_web_search_blocking(input),
    )
}

pub async fn execute_tool_async(name: &str, input: serde_json::Value, cwd: &str) -> Result<serde_json::Value> {
    tool_dispatch!(name, input, cwd,
        Bash => tool_bash_async(input, cwd).await,
        WebFetch => tool_web_fetch_async(input).await,
        WebSearch => tool_web_search_async(input).await,
    )
}

/// UTF-8 safe truncation: finds the last valid char boundary at or before `max_bytes`.
fn truncate_utf8(s: &str, max_bytes: usize) -> usize {
    if s.len() <= max_bytes {
        return s.len();
    }
    let mut end = max_bytes;
    while end > 0 && !s.is_char_boundary(end) {
        end -= 1;
    }
    end
}

fn max_output_bytes() -> usize { crate::config::OrchestrationConfig::max_output_bytes() }
fn default_bash_timeout_secs() -> u64 { crate::config::OrchestrationConfig::default_bash_timeout_secs() }

pub fn is_dangerous_command(command: &str) -> bool {
    let dangerous_patterns = [
        // ── Linux destructive ──
        "rm -rf /",
        "rm -rf /*",
        "rm -rf ~",
        "rm -rf .",
        "dd if=",
        "mkfs",
        ":(){ :|:& };:",
        "> /dev/sda",
        "> /dev/hda",
        "> /dev/nvme",
        "chmod 777 /",
        "chmod -R 777",
        "chown -R",
        // ── Linux priv esc / backdoor ──
        "curl | bash",
        "curl | sh",
        "wget | bash",
        "wget | sh",
        "curl|bash",
        "curl|sh",
        "wget|bash",
        "wget|sh",
        "sudo su",
        "sudo -i",
        "passwd",
        // ── Windows destructive ──
        "format c:",
        "format d:",
        "del /s /q c:",
        "del /s /q d:",
        "rd /s /q c:",
        "rd /s /q d:",
        "remove-item -recurse -force c:",
        "remove-item -recurse -force /",
        "diskpart",
        "bcdedit",
        // ── Windows priv esc / backdoor ──
        "net user administrator",
        "net localgroup administrators",
        "add-mppreference -exclusionpath",
        "set-mppreference -disable",
        "reg delete hklm\\software",
        "reg delete hkcu\\software",
        "sc stop",
        "sc delete",
        // ── Cross-platform dangerous ──
        "eval(",
        "fork bomb",
        "shutdown",
        "restart -f",
        "docker rm -f",
        "docker system prune",
        // ── Data exfiltration ──
        "nc -e",
        "/dev/tcp/",
        "invoke-webrequest -uri",
        "iwr -uri",
        "curl -F",
        "curl --data",
    ];

    let lower = command.to_lowercase();
    let stripped = lower
        .replace("  ", " ")
        .replace("\\ ", " ")
        .replace("\"", "")
        .replace("'", "");

    dangerous_patterns.iter().any(|pat| stripped.contains(pat))
}

fn limit_output(output: &str) -> String {
    if output.len() > max_output_bytes() {
        format!(
            "{}\n... (output truncated, exceeded {} bytes)",
            &output[..truncate_utf8(output, max_output_bytes())],
            max_output_bytes()
        )
    } else {
        output.to_string()
    }
}

fn is_path_allowed(resolved_path: &Path, cwd: &str) -> bool {
    let canonical = match resolved_path.canonicalize() {
        Ok(p) => p,
        Err(_) => {
            let mut current = resolved_path.to_path_buf();
            let mut suffix = PathBuf::new();
            loop {
                if let Ok(p) = current.canonicalize() { break p.join(suffix); }
                match current.file_name() {
                    Some(name) => { suffix = PathBuf::from(name).join(suffix); current = match current.parent() { Some(p) => p.to_path_buf(), None => return false }; }
                    None => return false,
                }
            }
        }
    };
    if let Ok(cwd_canonical) = Path::new(cwd).canonicalize() {
        if canonical.starts_with(&cwd_canonical) { return true; }
    }
    if let Some(appdata) = dirs::data_dir() {
        if canonical.starts_with(&appdata) { return true; }
    }
    if let Some(local_appdata) = dirs::data_local_dir() {
        if canonical.starts_with(&local_appdata) { return true; }
    }
    false
}

pub fn resolve_path(file_path: &str, cwd: &str) -> Result<String> {
    let resolved = if Path::new(file_path).is_absolute() { PathBuf::from(file_path) } else { Path::new(cwd).join(file_path) };
    if !is_path_allowed(&resolved, cwd) {
        return Err(anyhow!("Path is outside allowed directories: {}", resolved.display()));
    }
    Ok(resolved.to_string_lossy().to_string())
}

fn tool_read(input: serde_json::Value, cwd: &str) -> Result<serde_json::Value> {
    let file_path = input["file_path"].as_str().ok_or_else(|| anyhow!("file_path required"))?;
    let path = resolve_path(file_path, cwd)?;

    if !Path::new(&path).exists() {
        return Ok(serde_json::json!({ "content": format!("File not found: {}", path), "is_error": true }));
    }

    let content = fs::read_to_string(&path)?;
    let offset = input["offset"].as_u64().unwrap_or(1) as usize;
    let limit = input["limit"].as_u64().unwrap_or(2000) as usize;

    let lines: Vec<&str> = content.lines().collect();
    let total_lines = lines.len();
    let selected: Vec<String> = lines
        .iter()
        .skip(offset.saturating_sub(1))
        .take(limit)
        .enumerate()
        .map(|(i, line)| format!("{:>6}\t{}", i + offset, line))
        .collect();

    Ok(serde_json::json!({
        "content": selected.join("\n"),
        "lines": total_lines,
        "truncated": total_lines > limit
    }))
}

fn tool_write(input: serde_json::Value, cwd: &str) -> Result<serde_json::Value> {
    let file_path = input["file_path"].as_str().ok_or_else(|| anyhow!("file_path required"))?;
    let content = input["content"].as_str().ok_or_else(|| anyhow!("content required"))?;
    let path = resolve_path(file_path, cwd)?;

    if let Some(parent) = Path::new(&path).parent() {
        fs::create_dir_all(parent)?;
    }

    fs::write(&path, content)?;

    Ok(serde_json::json!({
        "success": true,
        "content": format!("Successfully wrote to {}", path),
        "bytes_written": content.len()
    }))
}

fn tool_edit(input: serde_json::Value, cwd: &str) -> Result<serde_json::Value> {
    let file_path = input["file_path"].as_str().ok_or_else(|| anyhow!("file_path required"))?;
    let old_string = input["old_string"].as_str().ok_or_else(|| anyhow!("old_string required"))?;
    let new_string = input["new_string"].as_str().ok_or_else(|| anyhow!("new_string required"))?;
    let replace_all = input["replace_all"].as_bool().unwrap_or(false);
    let path = resolve_path(file_path, cwd)?;

    let content = fs::read_to_string(&path)?;

    let (new_content, replacements) = if replace_all {
        let count = content.matches(old_string).count();
        (content.replace(old_string, new_string), count)
    } else {
        let count = content.matches(old_string).count();
        if count == 0 {
            return Ok(serde_json::json!({
                "success": false,
                "error": "old_string not found in file"
            }));
        }
        if count > 1 {
            return Ok(serde_json::json!({
                "success": false,
                "error": format!("old_string found {} times, use replace_all=true", count)
            }));
        }
        (content.replacen(old_string, new_string, 1), 1)
    };

    fs::write(&path, new_content)?;

    Ok(serde_json::json!({
        "success": true,
        "content": format!("Successfully replaced {} occurrence(s) in {}", replacements, path),
        "replacements": replacements
    }))
}

fn tool_multi_edit(input: serde_json::Value, cwd: &str) -> Result<serde_json::Value> {
    let file_path = input["file_path"].as_str().ok_or_else(|| anyhow!("file_path required"))?;
    let edits = input["edits"].as_array().ok_or_else(|| anyhow!("edits array required"))?;
    let path = resolve_path(file_path, cwd)?;

    let content = fs::read_to_string(&path)?;
    let mut new_content = content;
    let mut total_replacements = 0;
    let mut failed_edits: Vec<String> = Vec::new();

    for edit in edits {
        let old_string = edit.get("old_string").and_then(|s| s.as_str());
        let new_string = edit.get("new_string").and_then(|s| s.as_str());

        if let (Some(old), Some(new)) = (old_string, new_string) {
            if new_content.contains(old) {
                new_content = new_content.replace(old, new);
                total_replacements += 1;
            } else {
                failed_edits.push(old.to_string());
            }
        }
    }

    if !failed_edits.is_empty() {
        return Ok(serde_json::json!({
            "success": false,
            "error": format!("Some edits failed: {:?}", failed_edits),
            "replacements": total_replacements
        }));
    }

    fs::write(&path, new_content)?;

    Ok(serde_json::json!({
        "success": true,
        "content": format!("Successfully applied {} edits to {}", total_replacements, path),
        "replacements": total_replacements
    }))
}

fn tool_ask_user_question(input: serde_json::Value) -> Result<serde_json::Value> {
    let question = input["question"].as_str().ok_or_else(|| anyhow!("question required"))?;
    let options = input["options"].as_array().ok_or_else(|| anyhow!("options array required"))?;

    if options.is_empty() {
        return Ok(serde_json::json!({
            "content": "Question must have at least one option",
            "is_error": true
        }));
    }

    let description = input["description"].as_str().unwrap_or("");
    let multi_select = input["multiSelect"].as_bool().unwrap_or(false);

    let options_list: Vec<serde_json::Value> = options.iter().map(|opt| {
        serde_json::json!({
            "label": opt["label"].as_str().unwrap_or(""),
            "description": opt["description"].as_str().unwrap_or("")
        })
    }).collect();

    Ok(serde_json::json!({
        "type": "ask_user_question",
        "question": question,
        "description": description,
        "options": options_list,
        "multiSelect": multi_select,
        "content": format!("Waiting for user response to: {}", question),
        "requires_user_input": true
    }))
}

fn tool_computer_use(input: serde_json::Value) -> Result<serde_json::Value> {
    use crate::computer_use::{
        ComputerAction, ComputerActionType, ComputerUseConfig, ComputerUseManager, MouseButton,
        ScreenCoordinate,
    };

    let action_type_str = input["action_type"]
        .as_str()
        .ok_or_else(|| anyhow!("action_type required"))?;

    let action_type = match action_type_str {
        "MouseMove" => ComputerActionType::MouseMove,
        "MouseClick" => ComputerActionType::MouseClick,
        "MouseDown" => ComputerActionType::MouseDown,
        "MouseUp" => ComputerActionType::MouseUp,
        "MouseScroll" => ComputerActionType::MouseScroll,
        "KeyPress" => ComputerActionType::KeyPress,
        "KeyDown" => ComputerActionType::KeyDown,
        "KeyUp" => ComputerActionType::KeyUp,
        "TypeText" => ComputerActionType::TypeText,
        "Screenshot" => ComputerActionType::Screenshot,
        "Wait" => ComputerActionType::Wait,
        _ => return Ok(serde_json::json!({ "error": format!("Unknown action_type: {}", action_type_str), "is_error": true })),
    };

    let coordinate = input
        .get("coordinate")
        .and_then(|c| {
            Some(ScreenCoordinate {
                x: c.get("x")?.as_i64()? as i32,
                y: c.get("y")?.as_i64()? as i32,
            })
        });

    let button = input["button"]
        .as_str()
        .and_then(|b| match b {
            "Left" => Some(MouseButton::Left),
            "Right" => Some(MouseButton::Right),
            "Middle" => Some(MouseButton::Middle),
            "Back" => Some(MouseButton::Back),
            "Forward" => Some(MouseButton::Forward),
            _ => None,
        });

    let key = input["key"].as_str().map(|s| s.to_string());
    let text = input["text"].as_str().map(|s| s.to_string());
    let scroll_y = input["scroll_y"].as_i64().map(|v| v as i32);
    let scroll_x = input["scroll_x"].as_i64().map(|v| v as i32);
    let duration_ms = input["duration_ms"].as_u64();

    let action = ComputerAction {
        action_type,
        coordinate,
        button,
        key,
        text,
        scroll_y,
        scroll_x,
        duration_ms,
    };

    let manager = ComputerUseManager::new(ComputerUseConfig::default());

    let rt = tokio::runtime::Handle::try_current();
    let result: crate::computer_use::ComputerActionResult = match rt {
        Ok(handle) => handle.block_on(manager.execute_action(action))?,
        Err(_) => {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_time()
                .build()
                .map_err(|e| anyhow!("Failed to create fallback runtime: {}", e))?;
            rt.block_on(manager.execute_action(action))?
        }
    };

    Ok(serde_json::json!({
        "success": result.success,
        "action": action_type_str,
        "screenshot": result.screenshot,
        "error": result.error,
        "duration_ms": result.duration_ms,
    }))
}

async fn tool_computer_use_async(input: serde_json::Value) -> Result<serde_json::Value> {
    use crate::computer_use::{
        ComputerAction, ComputerActionType, ComputerUseConfig, ComputerUseManager, MouseButton,
        ScreenCoordinate,
    };

    let action_type_str = input["action_type"]
        .as_str()
        .ok_or_else(|| anyhow!("action_type required"))?;

    let action_type = match action_type_str {
        "MouseMove" => ComputerActionType::MouseMove,
        "MouseClick" => ComputerActionType::MouseClick,
        "MouseDown" => ComputerActionType::MouseDown,
        "MouseUp" => ComputerActionType::MouseUp,
        "MouseScroll" => ComputerActionType::MouseScroll,
        "KeyPress" => ComputerActionType::KeyPress,
        "KeyDown" => ComputerActionType::KeyDown,
        "KeyUp" => ComputerActionType::KeyUp,
        "TypeText" => ComputerActionType::TypeText,
        "Screenshot" => ComputerActionType::Screenshot,
        "Wait" => ComputerActionType::Wait,
        _ => return Ok(serde_json::json!({ "error": format!("Unknown action_type: {}", action_type_str), "is_error": true })),
    };

    let coordinate = input
        .get("coordinate")
        .and_then(|c| {
            Some(ScreenCoordinate {
                x: c.get("x")?.as_i64()? as i32,
                y: c.get("y")?.as_i64()? as i32,
            })
        });

    let button = input["button"]
        .as_str()
        .and_then(|b| match b {
            "Left" => Some(MouseButton::Left),
            "Right" => Some(MouseButton::Right),
            "Middle" => Some(MouseButton::Middle),
            "Back" => Some(MouseButton::Back),
            "Forward" => Some(MouseButton::Forward),
            _ => None,
        });

    let key = input["key"].as_str().map(|s| s.to_string());
    let text = input["text"].as_str().map(|s| s.to_string());
    let scroll_y = input["scroll_y"].as_i64().map(|v| v as i32);
    let scroll_x = input["scroll_x"].as_i64().map(|v| v as i32);
    let duration_ms = input["duration_ms"].as_u64();

    let action = ComputerAction {
        action_type,
        coordinate,
        button,
        key,
        text,
        scroll_y,
        scroll_x,
        duration_ms,
    };

    let manager = ComputerUseManager::new(ComputerUseConfig::default());
    let result: crate::computer_use::ComputerActionResult = manager.execute_action(action).await?;

    Ok(serde_json::json!({
        "success": result.success,
        "action": action_type_str,
        "screenshot": result.screenshot,
        "error": result.error,
        "duration_ms": result.duration_ms,
    }))
}

async fn tool_bash_async(input: serde_json::Value, cwd: &str) -> Result<serde_json::Value> {
    let command = input["command"].as_str().ok_or_else(|| anyhow!("command required"))?;
    let timeout_secs = input["timeout"].as_u64().unwrap_or(default_bash_timeout_secs());
    let shell_type = input["shell"].as_str().unwrap_or("bash");

    if is_dangerous_command(command) {
        return Ok(serde_json::json!({
            "error": "Command blocked by security filter: dangerous command detected",
            "is_error": true,
            "blocked": true
        }));
    }

    let (shell, flag) = match shell_type.to_lowercase().as_str() {
        "zsh" => {
            if let Some(zsh_path) = find_shell("zsh") {
                (zsh_path, "-c".to_string())
            } else {
                return Ok(serde_json::json!({
                    "error": "zsh shell not found",
                    "is_error": true
                }));
            }
        }
        "fish" => {
            if let Some(fish_path) = find_shell("fish") {
                (fish_path, "-c".to_string())
            } else {
                return Ok(serde_json::json!({
                    "error": "fish shell not found",
                    "is_error": true
                }));
            }
        }
        "sh" => {
            if let Some(sh_path) = find_shell("sh") {
                (sh_path, "-c".to_string())
            } else {
                return Ok(serde_json::json!({
                    "error": "sh shell not found",
                    "is_error": true
                }));
            }
        }
        "cmd" => {
            if cfg!(target_os = "windows") {
                ("cmd".to_string(), "/C".to_string())
            } else {
                return Ok(serde_json::json!({
                    "error": "cmd shell only available on Windows",
                    "is_error": true
                }));
            }
        }
        "powershell" | "pwsh" => {
            if let Some(pwsh_path) = find_shell("powershell").or_else(|| find_shell("pwsh")) {
                (pwsh_path, "-Command".to_string())
            } else {
                return Ok(serde_json::json!({
                    "error": "PowerShell not found",
                    "is_error": true
                }));
            }
        }
        _ => { // bash (default)
            if let Some(bash_path) = find_shell("bash") {
                (bash_path, "-c".to_string())
            } else if cfg!(target_os = "windows") {
                ("cmd".to_string(), "/C".to_string())
            } else {
                ("sh".to_string(), "-c".to_string())
            }
        }
    };

    let mut cmd = Command::new(&shell);

    #[cfg(windows)]
    cmd.creation_flags(CREATE_NO_WINDOW);

    let safe_cwd = if cfg!(target_os = "windows") {
        if std::path::Path::new(cwd).exists() {
            cwd.to_string()
        } else {
            std::env::var("USERPROFILE").unwrap_or_else(|_| "C:\\".to_string())
        }
    } else {
        cwd.to_string()
    };

    cmd.arg(&flag)
        .arg(command)
        .current_dir(&safe_cwd)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let output = tokio::time::timeout(
        std::time::Duration::from_secs(timeout_secs),
        cmd.output()
    ).await;

    match output {
        Ok(Ok(output)) => {
            let stdout = limit_output(&String::from_utf8_lossy(&output.stdout));
            let stderr = limit_output(&String::from_utf8_lossy(&output.stderr));
            let exit_code = output.status.code().unwrap_or(-1);

            Ok(serde_json::json!({
                "stdout": stdout,
                "stderr": stderr,
                "exit_code": exit_code,
                "success": output.status.success(),
                "shell": shell_type
            }))
        }
        Ok(Err(e)) => Ok(serde_json::json!({
            "error": format!("Command failed: {}", e),
            "is_error": true,
            "shell": shell_type
        })),
        Err(_) => Ok(serde_json::json!({
            "error": format!("Command timed out after {} seconds", timeout_secs),
            "is_error": true,
            "timed_out": true,
            "shell": shell_type
        })),
    }
}

// Sync wrapper: delegates to tool_bash_async via block_on.
// Single source of truth — avoids duplicating shell/danger/error logic.
fn tool_bash(input: serde_json::Value, cwd: &str) -> Result<serde_json::Value> {
    let handle = match tokio::runtime::Handle::try_current() {
        Ok(h) => h,
        Err(_) => {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_time()
                .build()
                .map_err(|e| anyhow!("Cannot create runtime for sync bash: {}", e))?;
            rt.handle().clone()
        }
    };
    handle.block_on(tool_bash_async(input, cwd))
}

pub fn find_git_bash() -> Option<String> {
    find_shell("bash")
}

pub fn find_shell(shell_name: &str) -> Option<String> {
    let candidates = match (shell_name, cfg!(target_os = "windows")) {
        ("bash", true) => vec![
            r"C:\Program Files\Git\bin\bash.exe".to_string(),
            r"C:\Program Files (x86)\Git\bin\bash.exe".to_string(),
            r"C:\Git\bin\bash.exe".to_string(),
        ],
        ("bash", false) => vec![
            "/usr/local/bin/bash".to_string(),
            "/usr/bin/bash".to_string(),
            "/bin/bash".to_string(),
        ],
        ("zsh", true) => vec![
            r"C:\Program Files\Git\usr\bin\zsh.exe".to_string(),
            r"C:\msys64\usr\bin\zsh.exe".to_string(),
        ],
        ("zsh", false) => vec![
            "/usr/local/bin/zsh".to_string(),
            "/usr/bin/zsh".to_string(),
            "/bin/zsh".to_string(),
        ],
        ("fish", true) => vec![
            r"C:\Program Files\fish\bin\fish.exe".to_string(),
            r"C:\msys64\usr\bin\fish.exe".to_string(),
        ],
        ("fish", false) => vec![
            "/usr/local/bin/fish".to_string(),
            "/usr/bin/fish".to_string(),
            "/bin/fish".to_string(),
        ],
        ("powershell", true) => vec![
            r"C:\Windows\System32\WindowsPowerShell\v1.0\powershell.exe".to_string(),
            r"C:\Program Files\PowerShell\7\pwsh.exe".to_string(),
        ],
        ("pwsh", true) => vec![
            r"C:\Program Files\PowerShell\7\pwsh.exe".to_string(),
            r"C:\Windows\System32\WindowsPowerShell\v1.0\powershell.exe".to_string(),
        ],
        ("sh", false) => vec![
            "/usr/local/bin/sh".to_string(),
            "/usr/bin/sh".to_string(),
            "/bin/sh".to_string(),
        ],
        _ => vec![],
    };

    for path in &candidates {
        if std::path::Path::new(path).exists() {
            return Some(path.clone());
        }
    }

    if !cfg!(target_os = "windows") {
        if let Ok(output) = std::process::Command::new("which").arg(shell_name).output() {
            let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !path.is_empty() && std::path::Path::new(&path).exists() {
                return Some(path);
            }
        }
    }

    None
}

fn tool_glob(input: serde_json::Value, cwd: &str) -> Result<serde_json::Value> {
    let pattern = input["pattern"].as_str().ok_or_else(|| anyhow!("pattern required"))?;
    let base_path = input["path"].as_str().unwrap_or(cwd);

    let mut matches: Vec<String> = Vec::new();
    for entry in walkdir::WalkDir::new(base_path).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if let Some(path_str) = path.to_str() {
            if let Ok(glob_pattern) = glob::Pattern::new(pattern) {
                if glob_pattern.matches(path_str) {
                    matches.push(path_str.to_string());
                }
            }
        }
    }

    Ok(serde_json::json!({
        "files": matches,
        "count": matches.len()
    }))
}

fn tool_grep(input: serde_json::Value, cwd: &str) -> Result<serde_json::Value> {
    let pattern = input["pattern"].as_str().ok_or_else(|| anyhow!("pattern required"))?;
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
    let dir_path = input["path"].as_str().ok_or_else(|| anyhow!("path required"))?;

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

fn tool_web_fetch_blocking(input: serde_json::Value) -> Result<serde_json::Value> {
    let url = input["url"].as_str().ok_or_else(|| anyhow!("url required"))?;

    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()?;

    let mut request = client.get(url);
    if let Some(headers) = input.get("headers").and_then(|h| h.as_object()) {
        for (key, value) in headers {
            if let Some(value_str) = value.as_str() {
                request = request.header(key.as_str(), value_str);
            }
        }
    }

    let response = request.send()?;

    if !response.status().is_success() {
        return Ok(serde_json::json!({
            "error": format!("HTTP error: {}", response.status()),
            "status": response.status().as_u16()
        }));
    }

    let content_type = response.headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("text/plain")
        .to_string();

    let body = response.text()?;

    Ok(serde_json::json!({
        "content": body,
        "content_type": content_type,
        "url": url
    }))
}

async fn tool_web_fetch_async(input: serde_json::Value) -> Result<serde_json::Value> {
    let url = input["url"].as_str().ok_or_else(|| anyhow!("url required"))?;

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()?;

    let mut request = client.get(url);
    if let Some(headers) = input.get("headers").and_then(|h| h.as_object()) {
        for (key, value) in headers {
            if let Some(value_str) = value.as_str() {
                request = request.header(key.as_str(), value_str);
            }
        }
    }

    let response = request.send().await?;

    if !response.status().is_success() {
        return Ok(serde_json::json!({
            "error": format!("HTTP error: {}", response.status()),
            "status": response.status().as_u16()
        }));
    }

    let content_type = response.headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("text/plain")
        .to_string();

    let body = response.text().await?;

    Ok(serde_json::json!({
        "content": body,
        "content_type": content_type,
        "url": url
    }))
}

fn tool_web_search_blocking(input: serde_json::Value) -> Result<serde_json::Value> {
    let query = input["query"].as_str().ok_or_else(|| anyhow!("query required"))?;

    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()?;

    let searx_url = format!(
        "https://searx.be/search?q={}&format=json",
        urlencoding::encode(query)
    );

    if let Ok(response) = client.get(&searx_url).send() {
        if response.status().is_success() {
            if let Ok(data) = response.json::<serde_json::Value>() {
                if let Some(results) = data.get("results").and_then(|r| r.as_array()) {
                    let search_results: Vec<serde_json::Value> = results
                        .iter()
                        .filter(|r| r.get("url").and_then(|u| u.as_str()).map(|u| !u.is_empty()).unwrap_or(false))
                        .take(10)
                        .map(|r| serde_json::json!({
                            "title": r.get("title").and_then(|t| t.as_str()).unwrap_or(""),
                            "url": r.get("url").and_then(|u| u.as_str()).unwrap_or(""),
                            "snippet": r.get("content").and_then(|c| c.as_str()).unwrap_or("")
                        }))
                        .collect();

                    if !search_results.is_empty() {
                        return Ok(serde_json::json!({
                            "results": search_results,
                            "query": query,
                            "source": "searx"
                        }));
                    }
                }
            }
        }
    }

    let ddg_url = format!(
        "https://api.duckduckgo.com/?q={}&format=json&no_html=1&skip_disambig=1",
        urlencoding::encode(query)
    );

    let response = match client.get(&ddg_url).send() {
        Ok(r) => r,
        Err(e) => {
            return Ok(serde_json::json!({
                "error": format!("Search failed: {}", e),
                "results": [],
                "query": query
            }));
        }
    };

    if !response.status().is_success() {
        return Ok(serde_json::json!({
            "error": format!("Search failed: {}", response.status()),
            "results": [],
            "query": query
        }));
    }

    let data: serde_json::Value = match response.json() {
        Ok(d) => d,
        Err(_) => {
            return Ok(serde_json::json!({
                "error": "Failed to parse search response",
                "results": [],
                "query": query
            }));
        }
    };

    let mut results: Vec<serde_json::Value> = Vec::new();

    if let Some(topics) = data.get("RelatedTopics").and_then(|t| t.as_array()) {
        for topic in topics {
            if let Some(text) = topic.get("Text").and_then(|t| t.as_str()) {
                let url = topic.get("FirstURL").and_then(|u| u.as_str()).unwrap_or("");
                results.push(serde_json::json!({
                    "title": text,
                    "url": url,
                    "snippet": ""
                }));
            }
            if let Some(nested) = topic.get("Topics").and_then(|t| t.as_array()) {
                for sub in nested {
                    if let Some(text) = sub.get("Text").and_then(|t| t.as_str()) {
                        let url = sub.get("FirstURL").and_then(|u| u.as_str()).unwrap_or("");
                        results.push(serde_json::json!({
                            "title": text,
                            "url": url,
                            "snippet": ""
                        }));
                    }
                }
            }
            if results.len() >= 10 { break; }
        }
    }

    if let Some(abstract_text) = data.get("AbstractText").and_then(|t| t.as_str()) {
        if !abstract_text.is_empty() {
            let abstract_url = data.get("AbstractURL").and_then(|u| u.as_str()).unwrap_or("");
            results.insert(0, serde_json::json!({
                "title": data.get("Heading").and_then(|h| h.as_str()).unwrap_or(query),
                "url": abstract_url,
                "snippet": abstract_text
            }));
        }
    }

    Ok(serde_json::json!({
        "results": results,
        "query": query,
        "source": "duckduckgo"
    }))
}

pub fn run_git_command(args: &[&str], cwd: &str) -> Result<std::process::Output> {
    let mut cmd = std::process::Command::new("git");
    #[cfg(windows)]
    cmd.creation_flags(CREATE_NO_WINDOW);
    cmd.args(args)
        .current_dir(cwd)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .map_err(|e| anyhow!("Failed to execute git: {}", e))
}

fn tool_git_status(input: serde_json::Value) -> Result<serde_json::Value> {
    let path = input["path"].as_str().ok_or_else(|| anyhow!("path required"))?;

    let output = run_git_command(&["status", "--porcelain"], path)?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    if !output.status.success() && !stderr.is_empty() {
        return Ok(serde_json::json!({
            "error": stderr.trim(),
            "is_error": true
        }));
    }

    let files: Vec<serde_json::Value> = stdout.lines().filter(|l| !l.is_empty()).map(|line| {
        let status = &line[..2];
        let file_path = &line[3..];
        serde_json::json!({
            "status": status.trim(),
            "file": file_path
        })
    }).collect();

    Ok(serde_json::json!({
        "files": files,
        "count": files.len(),
        "raw": stdout
    }))
}

fn tool_git_diff(input: serde_json::Value) -> Result<serde_json::Value> {
    let path = input["path"].as_str().ok_or_else(|| anyhow!("path required"))?;
    let staged = input["staged"].as_bool().unwrap_or(false);
    let file = input["file"].as_str();

    let mut args: Vec<&str> = vec!["diff"];
    if staged {
        args.push("--staged");
    }
    if let Some(f) = file {
        args.push("--");
        args.push(f);
    }

    let output = run_git_command(&args, path)?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    if !output.status.success() && !stderr.is_empty() {
        return Ok(serde_json::json!({
            "error": stderr.trim(),
            "is_error": true
        }));
    }

    Ok(serde_json::json!({
        "diff": stdout,
        "staged": staged,
        "file": file
    }))
}

fn tool_git_log(input: serde_json::Value) -> Result<serde_json::Value> {
    let path = input["path"].as_str().ok_or_else(|| anyhow!("path required"))?;
    let count = input["count"].as_u64().unwrap_or(10);
    let oneline = input["oneline"].as_bool().unwrap_or(true);

    let mut args: Vec<String> = vec!["log".to_string()];
    if oneline {
        args.push("--oneline".to_string());
    }
    args.push(format!("-n{}", count));
    let arg_refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();

    let output = run_git_command(&arg_refs, path)?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    if !output.status.success() && !stderr.is_empty() {
        return Ok(serde_json::json!({
            "error": stderr.trim(),
            "is_error": true
        }));
    }

    let commits: Vec<serde_json::Value> = stdout.lines().filter(|l| !l.is_empty()).map(|line| {
        if oneline {
            let parts: Vec<&str> = line.splitn(2, ' ').collect();
            serde_json::json!({
                "hash": parts.first().unwrap_or(&""),
                "message": parts.get(1).unwrap_or(&"")
            })
        } else {
            serde_json::json!({ "raw": line })
        }
    }).collect();

    Ok(serde_json::json!({
        "commits": commits,
        "count": commits.len(),
        "raw": stdout
    }))
}

fn tool_git_commit(input: serde_json::Value) -> Result<serde_json::Value> {
    let path = input["path"].as_str().ok_or_else(|| anyhow!("path required"))?;
    let message = input["message"].as_str().ok_or_else(|| anyhow!("message required"))?;

    let add_output = run_git_command(&["add", "-A"], path)?;
    if !add_output.status.success() {
        let stderr = String::from_utf8_lossy(&add_output.stderr).to_string();
        if !stderr.is_empty() {
            return Ok(serde_json::json!({
                "error": format!("git add failed: {}", stderr.trim()),
                "is_error": true
            }));
        }
    }

    let commit_output = run_git_command(&["commit", "-m", message], path)?;

    let stdout = String::from_utf8_lossy(&commit_output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&commit_output.stderr).to_string();

    if !commit_output.status.success() {
        return Ok(serde_json::json!({
            "error": format!("git commit failed: {}", if stderr.is_empty() { &stdout } else { &stderr }.trim()),
            "is_error": true
        }));
    }

    Ok(serde_json::json!({
        "success": true,
        "output": stdout,
        "message": message
    }))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ToolExecutionStatus {
    Running,
    Completed,
    Canceled,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolExecutionInfo {
    pub task_id: String,
    pub tool_name: String,
    pub status: ToolExecutionStatus,
    pub result: Option<serde_json::Value>,
    pub error: Option<String>,
}

pub struct ToolExecutionManager {
    executions: Arc<RwLock<HashMap<String, tokio::sync::oneshot::Sender<()>>>>,
}

impl ToolExecutionManager {
    pub fn new() -> Self {
        Self {
            executions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn execute_with_cancel(
        &self,
        task_id: String,
        tool_name: &str,
        input: serde_json::Value,
        cwd: &str,
    ) -> Result<serde_json::Value> {
        let (tx, rx) = tokio::sync::oneshot::channel();
        self.executions.write().await.insert(task_id.clone(), tx);

        let result = tokio::select! {
            _ = rx => {
                Ok(serde_json::json!({
                    "error": "Tool execution canceled",
                    "is_error": true,
                    "canceled": true
                }))
            }
            res = execute_tool_async(tool_name, input, cwd) => res
        };

        self.executions.write().await.remove(&task_id);
        result
    }

    pub async fn cancel_execution(&self, task_id: &str) -> bool {
        if let Some(tx) = self.executions.write().await.remove(task_id) {
            let _ = tx.send(());
            true
        } else {
            false
        }
    }

    pub async fn is_running(&self, task_id: &str) -> bool {
        self.executions.read().await.contains_key(task_id)
    }

    pub async fn get_running_tasks(&self) -> Vec<String> {
        self.executions.read().await.keys().cloned().collect()
    }
}

fn tool_git_add(input: serde_json::Value) -> Result<serde_json::Value> {
    let path = input["path"].as_str().ok_or_else(|| anyhow!("path required"))?;
    let files = input["files"].as_array().ok_or_else(|| anyhow!("files array required"))?;

    let file_strs: Vec<String> = files.iter()
        .filter_map(|f| f.as_str().map(String::from))
        .collect();

    if file_strs.is_empty() {
        return Ok(serde_json::json!({
            "error": "No files specified",
            "is_error": true
        }));
    }

    let mut args: Vec<String> = vec!["add".to_string()];
    args.extend(file_strs.clone());
    let arg_refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();

    let output = run_git_command(&arg_refs, path)?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    if !output.status.success() && !stderr.is_empty() {
        return Ok(serde_json::json!({
            "error": stderr.trim(),
            "is_error": true
        }));
    }

    Ok(serde_json::json!({
        "success": true,
        "files": file_strs,
        "output": stdout
    }))
}

async fn tool_web_search_async(input: serde_json::Value) -> Result<serde_json::Value> {
    let query = input["query"].as_str().ok_or_else(|| anyhow!("query required"))?;

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()?;

    let searx_url = format!(
        "https://searx.be/search?q={}&format=json",
        urlencoding::encode(query)
    );

    if let Ok(response) = client.get(&searx_url).send().await {
        if response.status().is_success() {
            if let Ok(data) = response.json::<serde_json::Value>().await {
                if let Some(results) = data.get("results").and_then(|r| r.as_array()) {
                    let search_results: Vec<serde_json::Value> = results
                        .iter()
                        .filter(|r| r.get("url").and_then(|u| u.as_str()).map(|u| !u.is_empty()).unwrap_or(false))
                        .take(10)
                        .map(|r| serde_json::json!({
                            "title": r.get("title").and_then(|t| t.as_str()).unwrap_or(""),
                            "url": r.get("url").and_then(|u| u.as_str()).unwrap_or(""),
                            "snippet": r.get("content").and_then(|c| c.as_str()).unwrap_or("")
                        }))
                        .collect();

                    if !search_results.is_empty() {
                        return Ok(serde_json::json!({
                            "results": search_results,
                            "query": query,
                            "source": "searx"
                        }));
                    }
                }
            }
        }
    }

    let ddg_url = format!(
        "https://api.duckduckgo.com/?q={}&format=json&no_html=1&skip_disambig=1",
        urlencoding::encode(query)
    );

    let response = match client.get(&ddg_url).send().await {
        Ok(r) => r,
        Err(e) => {
            return Ok(serde_json::json!({
                "error": format!("Search failed: {}", e),
                "results": [],
                "query": query
            }));
        }
    };

    if !response.status().is_success() {
        return Ok(serde_json::json!({
            "error": format!("Search failed: {}", response.status()),
            "results": [],
            "query": query
        }));
    }

    let data: serde_json::Value = match response.json().await {
        Ok(d) => d,
        Err(_) => {
            return Ok(serde_json::json!({
                "error": "Failed to parse search response",
                "results": [],
                "query": query
            }));
        }
    };

    let mut results: Vec<serde_json::Value> = Vec::new();

    if let Some(topics) = data.get("RelatedTopics").and_then(|t| t.as_array()) {
        for topic in topics {
            if let Some(text) = topic.get("Text").and_then(|t| t.as_str()) {
                let url = topic.get("FirstURL").and_then(|u| u.as_str()).unwrap_or("");
                results.push(serde_json::json!({
                    "title": text,
                    "url": url,
                    "snippet": ""
                }));
            }
            if let Some(nested) = topic.get("Topics").and_then(|t| t.as_array()) {
                for sub in nested {
                    if let Some(text) = sub.get("Text").and_then(|t| t.as_str()) {
                        let url = sub.get("FirstURL").and_then(|u| u.as_str()).unwrap_or("");
                        results.push(serde_json::json!({
                            "title": text,
                            "url": url,
                            "snippet": ""
                        }));
                    }
                }
            }
            if results.len() >= 10 { break; }
        }
    }

    if let Some(abstract_text) = data.get("AbstractText").and_then(|t| t.as_str()) {
        if !abstract_text.is_empty() {
            let abstract_url = data.get("AbstractURL").and_then(|u| u.as_str()).unwrap_or("");
            results.insert(0, serde_json::json!({
                "title": data.get("Heading").and_then(|h| h.as_str()).unwrap_or(query),
                "url": abstract_url,
                "snippet": abstract_text
            }));
        }
    }

    Ok(serde_json::json!({
        "results": results,
        "query": query,
        "source": "duckduckgo"
    }))
}
