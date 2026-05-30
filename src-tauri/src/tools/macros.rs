use anyhow::Result;
use serde_json::{json, Value};
use std::fs;
use std::path::Path;

pub struct MacroToolDefinition {
    pub name: String,
    pub description: String,
    pub input_schema: Value,
}

pub fn get_macro_definitions() -> Vec<MacroToolDefinition> {
    vec![
        MacroToolDefinition {
            name: "smart_edit".to_string(),
            description: "Read a file, search for matching lines, and apply edits in a single operation. More efficient than separate Read+Grep+Edit calls.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "file_path": { "type": "string", "description": "Path to the file" },
                    "search_pattern": { "type": "string", "description": "Regex pattern to search for" },
                    "replacement": { "type": "string", "description": "Replacement text for matched lines" },
                    "replace_all": { "type": "boolean", "description": "Replace all occurrences (default: false)" }
                },
                "required": ["file_path", "search_pattern", "replacement"]
            }),
        },
        MacroToolDefinition {
            name: "smart_grep".to_string(),
            description: "Search for a pattern across files and return matching lines with file paths. Combines Glob+Grep in one call.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "pattern": { "type": "string", "description": "Regex pattern to search for" },
                    "path": { "type": "string", "description": "Directory to search in" },
                    "include": { "type": "string", "description": "Glob pattern to filter files (e.g. '*.rs', '*.ts')" }
                },
                "required": ["pattern"]
            }),
        },
        MacroToolDefinition {
            name: "smart_project_scan".to_string(),
            description: "Scan a project directory structure, read key config files, and check git status in one operation.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Project root directory" }
                },
                "required": ["path"]
            }),
        },
    ]
}

pub fn execute_smart_edit(input: Value, cwd: &str) -> Result<Value> {
    let file_path = input["file_path"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("file_path required"))?;
    let search_pattern = input["search_pattern"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("search_pattern required"))?;
    let replacement = input["replacement"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("replacement required"))?;
    let replace_all = input["replace_all"].as_bool().unwrap_or(false);

    let path = super::resolve_path(file_path, cwd)?;
    let content = fs::read_to_string(&path)?;
    let re = regex::Regex::new(search_pattern)?;

    let match_count = if replace_all {
        re.find_iter(&content).count()
    } else {
        usize::from(re.is_match(&content))
    };

    if match_count == 0 {
        return Ok(json!({"success": false, "error": "Pattern not found", "matches": 0}));
    }

    let new_content = if replace_all {
        re.replace_all(&content, replacement).to_string()
    } else {
        re.replace(&content, replacement).to_string()
    };

    fs::write(&path, &new_content)?;
    Ok(json!({"success": true, "matches": match_count, "file": path}))
}

pub fn execute_smart_grep(input: Value, cwd: &str) -> Result<Value> {
    let pattern = input["pattern"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("pattern required"))?;
    let search_path = input["path"].as_str().unwrap_or(cwd);
    let include = input["include"].as_str();

    let re = regex::Regex::new(pattern)?;
    let glob_pattern = include.and_then(|inc| glob::Pattern::new(inc).ok());

    let mut results: Vec<Value> = Vec::new();

    for entry in walkdir::WalkDir::new(search_path)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }

        if let Some(ref gp) = glob_pattern {
            let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if !gp.matches(file_name) {
                continue;
            }
        }

        if let Ok(content) = fs::read_to_string(path) {
            for (line_num, line) in content.lines().enumerate() {
                if re.is_match(line) {
                    results.push(json!({
                        "file": path.to_string_lossy(),
                        "line": line_num + 1,
                        "content": line
                    }));
                }
            }
        }
    }

    Ok(json!({
        "matches": results,
        "count": results.len()
    }))
}

pub fn execute_smart_project_scan(input: Value, cwd: &str) -> Result<Value> {
    let project_path = input["path"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("path required"))?;
    let resolved_path = super::resolve_path(project_path, cwd)?;

    let mut structure: Vec<Value> = Vec::new();

    for entry in walkdir::WalkDir::new(&resolved_path)
        .max_depth(3)
        .into_iter()
        .filter_entry(|e| {
            if e.depth() == 0 {
                return true;
            }
            let name = e.file_name().to_string_lossy();
            !name.starts_with('.')
                && name != "node_modules"
                && name != "target"
                && name != "dist"
                && name != "build"
        })
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        let depth = entry.depth();
        let is_dir = entry.file_type().is_dir();

        structure.push(json!({
            "path": path.to_string_lossy(),
            "depth": depth,
            "is_dir": is_dir
        }));
    }

    let config_files = ["package.json", "Cargo.toml", "pyproject.toml", "go.mod"];
    let mut configs = serde_json::Map::new();

    for config_name in &config_files {
        let config_path = Path::new(&resolved_path).join(config_name);
        if config_path.exists() {
            if let Ok(content) = fs::read_to_string(&config_path) {
                configs.insert(config_name.to_string(), json!(content));
            }
        }
    }

    let git_status = match super::run_git_command(&["status", "--porcelain"], &resolved_path) {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let files: Vec<Value> = stdout
                .lines()
                .filter(|l| !l.is_empty())
                .map(|line| {
                    let status = &line[..2.min(line.len())];
                    let file_path = if line.len() > 3 { &line[3..] } else { "" };
                    json!({
                        "status": status.trim(),
                        "file": file_path
                    })
                })
                .collect();
            json!({"files": files, "count": files.len()})
        }
        Err(e) => json!({"error": e.to_string()}),
    };

    Ok(json!({
        "structure": structure,
        "configs": Value::Object(configs),
        "git_status": git_status
    }))
}
