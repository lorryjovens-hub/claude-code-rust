//! Git Operations Tool
//!
//! Provides Git version control operations including:
//! - git status: Show working tree status
//! - git add: Add files to staging area
//! - git commit: Create a new commit
//! - git push: Push to remote repository
//! - git pull: Pull from remote repository
//! - git log: Show commit history
//! - git diff: Show changes
//! - git branch: Manage branches
//! - git checkout: Switch branches or restore files

use super::{Tool, ToolOutput, ToolError};
use async_trait::async_trait;
use serde_json;
use std::path::Path;

pub struct GitOperationsTool;

impl Default for GitOperationsTool {
    fn default() -> Self {
        Self::new()
    }
}

impl GitOperationsTool {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Tool for GitOperationsTool {
    fn name(&self) -> &str {
        "git_operations"
    }

    fn description(&self) -> &str {
        "Execute Git version control operations"
    }

    fn input_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "operation": {
                    "type": "string",
                    "description": "Git operation to perform: status, add, commit, push, pull, log, diff, branch, checkout",
                    "enum": ["status", "add", "commit", "push", "pull", "log", "diff", "branch", "checkout"]
                },
                "path": {
                    "type": "string",
                    "description": "Path to the git repository (optional, defaults to current directory)"
                },
                "args": {
                    "type": "array",
                    "items": {
                        "type": "string"
                    },
                    "description": "Additional arguments for the git command"
                },
                "message": {
                    "type": "string",
                    "description": "Commit message (required for commit operation)"
                },
                "files": {
                    "type": "array",
                    "items": {
                        "type": "string"
                    },
                    "description": "Files to add (for add operation)"
                },
                "branch": {
                    "type": "string",
                    "description": "Branch name (for branch, checkout, push, pull operations)"
                },
                "remote": {
                    "type": "string",
                    "description": "Remote name (defaults to 'origin')"
                }
            },
            "required": ["operation"]
        })
    }

    async fn execute(&self, input: serde_json::Value) -> Result<ToolOutput, ToolError> {
        let operation = input["operation"].as_str()
            .ok_or_else(|| ToolError {
                message: "operation is required".to_string(),
                code: Some("missing_parameter".to_string()),
            })?;

        let path = input["path"].as_str().unwrap_or(".");
        let args: Vec<String> = input["args"]
            .as_array()
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
            .unwrap_or_default();

        // Change to the git repository directory
        let repo_path = Path::new(path);

        match operation {
            "status" => self.git_status(repo_path, &args).await,
            "add" => self.git_add(repo_path, &input, &args).await,
            "commit" => self.git_commit(repo_path, &input, &args).await,
            "push" => self.git_push(repo_path, &input, &args).await,
            "pull" => self.git_pull(repo_path, &input, &args).await,
            "log" => self.git_log(repo_path, &args).await,
            "diff" => self.git_diff(repo_path, &args).await,
            "branch" => self.git_branch(repo_path, &input, &args).await,
            "checkout" => self.git_checkout(repo_path, &input, &args).await,
            _ => Err(ToolError {
                message: format!("Unknown git operation: {}", operation),
                code: Some("invalid_operation".to_string()),
            }),
        }
    }
}

impl GitOperationsTool {
    async fn execute_git_command(&self, repo_path: &Path, args: &[String]) -> Result<ToolOutput, ToolError> {
        let output = tokio::process::Command::new("git")
            .current_dir(repo_path)
            .args(args)
            .output()
            .await
            .map_err(|e| ToolError {
                message: format!("Failed to execute git command: {}", e),
                code: Some("git_error".to_string()),
            })?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        let content = if output.status.success() {
            if stdout.is_empty() && !stderr.is_empty() {
                stderr
            } else {
                stdout
            }
        } else {
            format!("Git command failed with status {}\n{}\n{}",
                   output.status, stdout, stderr)
        };

        Ok(ToolOutput {
            output_type: "text".to_string(),
            content,
            metadata: std::collections::HashMap::new(),
        })
    }

    async fn git_status(&self, repo_path: &Path, args: &[String]) -> Result<ToolOutput, ToolError> {
        let mut cmd_args = vec!["status".to_string()];
        cmd_args.extend_from_slice(args);
        self.execute_git_command(repo_path, &cmd_args).await
    }

    async fn git_add(&self, repo_path: &Path, input: &serde_json::Value, args: &[String]) -> Result<ToolOutput, ToolError> {
        let files: Vec<String> = input["files"]
            .as_array()
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
            .unwrap_or_else(|| vec![".".to_string()]); // Default to add all

        let mut cmd_args = vec!["add".to_string()];
        cmd_args.extend_from_slice(args);
        cmd_args.extend(files);

        self.execute_git_command(repo_path, &cmd_args).await
    }

    async fn git_commit(&self, repo_path: &Path, input: &serde_json::Value, args: &[String]) -> Result<ToolOutput, ToolError> {
        let message = input["message"].as_str()
            .ok_or_else(|| ToolError {
                message: "Commit message is required".to_string(),
                code: Some("missing_message".to_string()),
            })?;

        let mut cmd_args = vec!["commit".to_string(), "-m".to_string(), message.to_string()];
        cmd_args.extend_from_slice(args);

        self.execute_git_command(repo_path, &cmd_args).await
    }

    async fn git_push(&self, repo_path: &Path, input: &serde_json::Value, args: &[String]) -> Result<ToolOutput, ToolError> {
        let remote = input["remote"].as_str().unwrap_or("origin");
        let branch = input["branch"].as_str().unwrap_or("");

        let mut cmd_args = vec!["push".to_string()];
        cmd_args.extend_from_slice(args);

        if !branch.is_empty() {
            cmd_args.push(format!("{}:{}", remote, branch));
        } else {
            cmd_args.push(remote.to_string());
        }

        self.execute_git_command(repo_path, &cmd_args).await
    }

    async fn git_pull(&self, repo_path: &Path, input: &serde_json::Value, args: &[String]) -> Result<ToolOutput, ToolError> {
        let remote = input["remote"].as_str().unwrap_or("origin");
        let branch = input["branch"].as_str().unwrap_or("");

        let mut cmd_args = vec!["pull".to_string()];
        cmd_args.extend_from_slice(args);

        if !branch.is_empty() {
            cmd_args.push(remote.to_string());
            cmd_args.push(branch.to_string());
        }

        self.execute_git_command(repo_path, &cmd_args).await
    }

    async fn git_log(&self, repo_path: &Path, args: &[String]) -> Result<ToolOutput, ToolError> {
        let mut cmd_args = vec!["log".to_string()];
        cmd_args.extend_from_slice(args);

        // Add pretty format if no custom format specified
        if !args.iter().any(|arg| arg.starts_with("--pretty=") || arg == "--oneline") {
            cmd_args.push("--oneline".to_string());
        }

        self.execute_git_command(repo_path, &cmd_args).await
    }

    async fn git_diff(&self, repo_path: &Path, args: &[String]) -> Result<ToolOutput, ToolError> {
        let mut cmd_args = vec!["diff".to_string()];
        cmd_args.extend_from_slice(args);
        self.execute_git_command(repo_path, &cmd_args).await
    }

    async fn git_branch(&self, repo_path: &Path, input: &serde_json::Value, args: &[String]) -> Result<ToolOutput, ToolError> {
        let branch = input["branch"].as_str();

        let mut cmd_args = vec!["branch".to_string()];
        cmd_args.extend_from_slice(args);

        if let Some(branch_name) = branch {
            cmd_args.push(branch_name.to_string());
        }

        self.execute_git_command(repo_path, &cmd_args).await
    }

    async fn git_checkout(&self, repo_path: &Path, input: &serde_json::Value, args: &[String]) -> Result<ToolOutput, ToolError> {
        let branch = input["branch"].as_str()
            .ok_or_else(|| ToolError {
                message: "Branch name is required for checkout".to_string(),
                code: Some("missing_branch".to_string()),
            })?;

        let mut cmd_args = vec!["checkout".to_string(), branch.to_string()];
        cmd_args.extend_from_slice(args);

        self.execute_git_command(repo_path, &cmd_args).await
    }
}