use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::process::Command;
use tokio::process::Command as AsyncCommand;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitStatus {
    pub branch: String,
    pub staged: Vec<String>,
    pub modified: Vec<String>,
    pub untracked: Vec<String>,
    pub ahead: i32,
    pub behind: i32,
    pub is_dirty: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitCommit {
    pub hash: String,
    pub short_hash: String,
    pub message: String,
    pub author: String,
    pub date: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitBranch {
    pub name: String,
    pub is_current: bool,
    pub is_remote: bool,
    pub upstream: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitDiff {
    pub file: String,
    pub hunks: Vec<DiffHunk>,
    pub is_binary: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffHunk {
    pub old_start: u32,
    pub old_lines: u32,
    pub new_start: u32,
    pub new_lines: u32,
    pub lines: Vec<DiffLine>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffLine {
    pub line_type: String,
    pub content: String,
    pub old_line_no: Option<u32>,
    pub new_line_no: Option<u32>,
}

pub struct GitIntegration {
    working_dir: std::path::PathBuf,
}

impl GitIntegration {
    pub fn new(working_dir: PathBuf) -> Self {
        Self { working_dir }
    }

    pub fn with_cwd(cwd: Option<String>) -> Self {
        let working_dir = cwd
            .map(|p| Path::new(&p).to_path_buf())
            .unwrap_or_else(|| std::env::current_dir().unwrap_or_default());
        Self { working_dir }
    }

    fn run_git(&self, args: &[&str]) -> Result<String> {
        let output = Command::new("git")
            .args(args)
            .current_dir(&self.working_dir)
            .output()?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(anyhow!("Git error: {}", stderr))
        }
    }

    #[allow(dead_code)]
    async fn run_git_async(&self, args: &[&str]) -> Result<String> {
        let mut cmd = AsyncCommand::new("git");
        cmd.args(args).current_dir(&self.working_dir);

        let output = cmd.output().await?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(anyhow!("Git error: {}", stderr))
        }
    }

    pub fn is_repo(&self) -> bool {
        Command::new("git")
            .args(["rev-parse", "--git-dir"])
            .current_dir(&self.working_dir)
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    pub fn get_status(&self) -> Result<GitStatus> {
        let output = self.run_git(&["status", "--porcelain=v1"])?;

        let mut staged = Vec::new();
        let mut modified = Vec::new();
        let mut untracked = Vec::new();

        for line in output.lines() {
            if line.len() < 3 {
                continue;
            }
            let index = &line[..1];
            let worktree = &line[1..2];
            let path = line[3..].to_string();

            if index != " " && index != "?" {
                staged.push(path.clone());
            }
            if worktree == "M" || worktree == "D" {
                modified.push(path.clone());
            }
            if index == "?" && worktree == "?" {
                untracked.push(path);
            }
        }

        let branch = self.get_current_branch()?;
        let (ahead, behind) = self.get_ahead_behind()?;
        let is_dirty = !staged.is_empty() || !modified.is_empty() || !untracked.is_empty();

        Ok(GitStatus {
            branch,
            staged,
            modified,
            untracked,
            ahead,
            behind,
            is_dirty,
        })
    }

    pub fn get_current_branch(&self) -> Result<String> {
        let output = self.run_git(&["rev-parse", "--abbrev-ref", "HEAD"])?;
        Ok(output.trim().to_string())
    }

    fn get_ahead_behind(&self) -> Result<(i32, i32)> {
        let output = self.run_git(&["rev-list", "--left-right", "--count", "@{upstream}...HEAD"])?;

        let parts: Vec<&str> = output.trim().split_whitespace().collect();
        if parts.len() == 2 {
            let ahead = parts[0].parse().unwrap_or(0);
            let behind = parts[1].parse().unwrap_or(0);
            return Ok((ahead, behind));
        }

        Ok((0, 0))
    }

    pub fn get_branches(&self, include_remote: bool) -> Result<Vec<GitBranch>> {
        let mut args = vec!["branch"];
        if include_remote {
            args.push("-a");
        }

        let output = self.run_git(&args)?;
        let mut branches = Vec::new();

        for line in output.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            let is_current = line.starts_with('*');
            let name = line.trim_start_matches(['*', ' ']).to_string();
            let is_remote = name.contains('/') && !name.starts_with("remotes/");

            branches.push(GitBranch {
                name,
                is_current,
                is_remote,
                upstream: None,
            });
        }

        Ok(branches)
    }

    pub fn create_branch(&self, name: &str, checkout: bool) -> Result<()> {
        self.run_git(&["branch", name])?;

        if checkout {
            self.run_git(&["checkout", name])?;
        }

        Ok(())
    }

    pub fn checkout_branch(&self, name: &str) -> Result<()> {
        self.run_git(&["checkout", name])?;
        Ok(())
    }

    pub fn get_commits(&self, limit: Option<usize>, skip: Option<usize>) -> Result<Vec<GitCommit>> {
        let limit = limit.unwrap_or(50);
        let skip = skip.unwrap_or(0);

        let output = self.run_git(&[
            "log",
            &format!("--max-count={}", limit),
            &format!("--skip={}", skip),
            "--pretty=format:%H|%h|%s|%an|%ai",
        ])?;

        let mut commits = Vec::new();

        for line in output.lines() {
            let parts: Vec<&str> = line.split('|').collect();
            if parts.len() >= 5 {
                commits.push(GitCommit {
                    hash: parts[0].to_string(),
                    short_hash: parts[1].to_string(),
                    message: parts[2].to_string(),
                    author: parts[3].to_string(),
                    date: parts[4].to_string(),
                });
            }
        }

        Ok(commits)
    }

    pub fn get_commit_diff(&self, commit_hash: &str) -> Result<Vec<GitDiff>> {
        let output = self.run_git(&["show", commit_hash, "--format=", "-p"])?;
        self.parse_diff_output(&output)
    }

    pub fn get_file_diff(&self, file: Option<&str>) -> Result<Vec<GitDiff>> {
        let args: Vec<&str> = if let Some(f) = file {
            vec!["diff", "--", f]
        } else {
            vec!["diff"]
        };

        let output = self.run_git(&args)?;
        self.parse_diff_output(&output)
    }

    fn parse_diff_output(&self, output: &str) -> Result<Vec<GitDiff>> {
        let mut diffs = Vec::new();
        let mut current_file: Option<String> = None;
        let mut current_hunk: Option<DiffHunk> = None;
        let mut current_lines: Vec<DiffLine> = Vec::new();

        for line in output.lines() {
            if line.starts_with("diff --git") {
                if let Some(file) = current_file.take() {
                    if let Some(_hunk) = current_hunk.take() {
                        current_lines.push(DiffLine {
                            line_type: "header".to_string(),
                            content: String::new(),
                            old_line_no: None,
                            new_line_no: None,
                        });
                    }
                    diffs.push(GitDiff {
                        file,
                        hunks: vec![current_hunk.take()].into_iter().filter_map(|h| h).collect(),
                        is_binary: false,
                    });
                }

                if let Some(name) = line.split(" b/").nth(1) {
                    current_file = Some(name.to_string());
                }
            } else if line.starts_with("@@") {
                if let Some(hunk) = current_hunk.take() {
                    current_lines.push(DiffLine {
                        line_type: "hunk_header".to_string(),
                        content: format!("{:?}", hunk),
                        old_line_no: None,
                        new_line_no: None,
                    });
                }

                if let Some(parsed) = self.parse_hunk_header(line) {
                    current_hunk = Some(DiffHunk {
                        old_start: parsed.0,
                        old_lines: parsed.1,
                        new_start: parsed.2,
                        new_lines: parsed.3,
                        lines: Vec::new(),
                    });
                }
            } else if line.starts_with('+') && !line.starts_with("+++") {
                current_hunk.as_mut().map(|h| {
                    h.lines.push(DiffLine {
                        line_type: "addition".to_string(),
                        content: line[1..].to_string(),
                        old_line_no: None,
                        new_line_no: None,
                    })
                });
            } else if line.starts_with('-') && !line.starts_with("---") {
                current_hunk.as_mut().map(|h| {
                    h.lines.push(DiffLine {
                        line_type: "deletion".to_string(),
                        content: line[1..].to_string(),
                        old_line_no: None,
                        new_line_no: None,
                    })
                });
            } else if !line.starts_with('\\') && !line.is_empty() {
                current_hunk.as_mut().map(|h| {
                    h.lines.push(DiffLine {
                        line_type: "context".to_string(),
                        content: line.to_string(),
                        old_line_no: None,
                        new_line_no: None,
                    })
                });
            }
        }

        if let Some(file) = current_file.take() {
            diffs.push(GitDiff {
                file,
                hunks: vec![current_hunk.take()].into_iter().filter_map(|h| h).collect(),
                is_binary: false,
            });
        }

        Ok(diffs)
    }

    fn parse_hunk_header(&self, line: &str) -> Option<(u32, u32, u32, u32)> {
        let re = regex::Regex::new(r"@@ -(\d+)(?:,(\d+))? \+(\d+)(?:,(\d+))? @@").ok()?;
        let caps = re.captures(line)?;

        Some((
            caps.get(1)?.as_str().parse().ok()?,
            caps.get(2).and_then(|m| m.as_str().parse().ok()).unwrap_or(1),
            caps.get(3)?.as_str().parse().ok()?,
            caps.get(4).and_then(|m| m.as_str().parse().ok()).unwrap_or(1),
        ))
    }

    pub fn stage(&self, files: &[&str]) -> Result<()> {
        let mut args = vec!["add"];
        args.extend(files);

        self.run_git(&args)?;
        Ok(())
    }

    pub fn unstage(&self, files: &[&str]) -> Result<()> {
        let mut args = vec!["reset", "HEAD", "--"];
        args.extend(files);

        self.run_git(&args)?;
        Ok(())
    }

    pub fn commit(&self, message: &str) -> Result<String> {
        let output = self.run_git(&["commit", "-m", message])?;
        Ok(output)
    }

    pub fn amend(&self, message: Option<&str>) -> Result<()> {
        let mut args = vec!["commit", "--amend"];
        if let Some(msg) = message {
            args.push("-m");
            args.push(msg);
        }

        self.run_git(&args)?;
        Ok(())
    }

    pub fn push(&self, remote: Option<&str>, branch: Option<&str>, force: bool) -> Result<String> {
        let mut args = vec!["push"];

        if force {
            args.push("--force");
        }

        if let Some(r) = remote {
            args.push(r);
        }

        if let Some(b) = branch {
            args.push(b);
        }

        let output = self.run_git(&args)?;
        Ok(output)
    }

    pub fn pull(&self, remote: Option<&str>, branch: Option<&str>) -> Result<String> {
        let mut args = vec!["pull"];

        if let Some(r) = remote {
            args.push(r);
        }

        if let Some(b) = branch {
            args.push(b);
        }

        let output = self.run_git(&args)?;
        Ok(output)
    }

    pub fn fetch(&self, remote: Option<&str>) -> Result<String> {
        let mut args = vec!["fetch"];

        if let Some(r) = remote {
            args.push(r);
        }

        let output = self.run_git(&args)?;
        Ok(output)
    }

    pub fn merge(&self, branch: &str) -> Result<String> {
        let output = self.run_git(&["merge", branch])?;
        Ok(output)
    }

    pub fn rebase(&self, branch: Option<&str>) -> Result<String> {
        let mut args = vec!["rebase"];

        if let Some(b) = branch {
            args.push(b);
        }

        let output = self.run_git(&args)?;
        Ok(output)
    }

    pub fn get_file_history(&self, file: &str, limit: Option<usize>) -> Result<Vec<GitCommit>> {
        let limit = limit.unwrap_or(20);

        let output = self.run_git(&[
            "log",
            "--follow",
            &format!("--max-count={}", limit),
            "--pretty=format:%H|%h|%s|%an|%ai",
            "--",
            file,
        ])?;

        let mut commits = Vec::new();

        for line in output.lines() {
            let parts: Vec<&str> = line.split('|').collect();
            if parts.len() >= 5 {
                commits.push(GitCommit {
                    hash: parts[0].to_string(),
                    short_hash: parts[1].to_string(),
                    message: parts[2].to_string(),
                    author: parts[3].to_string(),
                    date: parts[4].to_string(),
                });
            }
        }

        Ok(commits)
    }

    pub fn blame(&self, file: &str) -> Result<String> {
        let output = self.run_git(&["blame", file])?;
        Ok(output)
    }

    pub fn stash(&self, message: Option<&str>) -> Result<String> {
        let mut args = vec!["stash"];

        if let Some(msg) = message {
            args.push("-m");
            args.push(msg);
        }

        let output = self.run_git(&args)?;
        Ok(output)
    }

    pub fn stash_pop(&self) -> Result<String> {
        let output = self.run_git(&["stash", "pop"])?;
        Ok(output)
    }

    pub fn stash_list(&self) -> Result<Vec<(String, String)>> {
        let output = self.run_git(&["stash", "list"])?;
        let mut stashes = Vec::new();

        for line in output.lines() {
            if let Some(stash) = line.strip_prefix("stash@{") {
                if let Some(end) = stash.find("}: ") {
                    let idx = stash[..end].to_string();
                    let rest = stash[end + 2..].to_string();
                    stashes.push((idx, rest));
                }
            }
        }

        Ok(stashes)
    }

    pub fn get_remote_url(&self, remote: Option<&str>) -> Result<String> {
        let name = remote.unwrap_or("origin");
        let output = self.run_git(&["remote", "get-url", name])?;
        Ok(output.trim().to_string())
    }

    pub fn list_remotes(&self) -> Result<Vec<String>> {
        let output = self.run_git(&["remote", "-v"])?;
        let mut remotes: Vec<String> = Vec::new();

        for line in output.lines() {
            if let Some(name) = line.split_whitespace().next() {
                if !remotes.contains(&name.to_string()) {
                    remotes.push(name.to_string());
                }
            }
        }

        Ok(remotes)
    }
}
