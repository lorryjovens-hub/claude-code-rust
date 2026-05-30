use crate::cache::FileCache;
use anyhow::Result;
use regex::Regex;
use std::path::Path;
use std::sync::Arc;

pub struct PrefetchEngine {
    cache: Arc<FileCache>,
    workspace_cwd: String,
}

impl PrefetchEngine {
    pub fn new(cache: Arc<FileCache>, workspace_cwd: String) -> Self {
        Self { cache, workspace_cwd }
    }

    pub async fn prefetch_for_message(&self, user_message: &str) {
        let intents = self.detect_intents(user_message);
        for intent in intents {
            let cache = self.cache.clone();
            let cwd = self.workspace_cwd.clone();
            tokio::spawn(async move {
                let _ = Self::execute_prefetch(intent, &cache, &cwd).await;
            });
        }
    }

    fn detect_intents(&self, message: &str) -> Vec<PrefetchIntent> {
        let mut intents = Vec::new();

        if let Ok(re) = Regex::new(r#"(?:["'`])([\w./\\]+\.\w+)(?:["'`])|([\w./\\]+\.\w{1,10})"#) {
            for cap in re.captures_iter(message) {
                let path = cap.get(1)
                    .or_else(|| cap.get(2))
                    .map(|m| m.as_str().to_string());
                if let Some(path) = path {
                    if !path.starts_with("http") && path.len() > 3 {
                        intents.push(PrefetchIntent::ReadFile(path));
                    }
                }
            }
        }

        let project_signals = ["项目", "project", "代码库", "codebase", "仓库", "repo", "这个项目"];
        let msg_lower = message.to_lowercase();
        for signal in &project_signals {
            if msg_lower.contains(signal) {
                intents.push(PrefetchIntent::ProjectScan);
                break;
            }
        }

        let modify_signals = ["修改", "编辑", "更改", "变更", "modify", "edit", "change", "update", "fix", "修复", "重构", "refactor"];
        for signal in &modify_signals {
            if msg_lower.contains(signal) {
                intents.push(PrefetchIntent::GitStatus);
                break;
            }
        }

        intents
    }

    async fn execute_prefetch(intent: PrefetchIntent, cache: &Arc<FileCache>, cwd: &str) -> Result<()> {
        match intent {
            PrefetchIntent::ReadFile(path) => {
                let resolved = if Path::new(&path).is_absolute() {
                    path
                } else {
                    format!("{}/{}", cwd, path)
                };
                if Path::new(&resolved).exists() {
                    if let Ok(content) = std::fs::read_to_string(&resolved) {
                        if let Ok(metadata) = std::fs::metadata(&resolved) {
                            let mtime = metadata.modified()
                                .map(|t| t.duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs())
                                .unwrap_or(0);
                            cache.set_file(&resolved, content, mtime);
                        }
                    }
                }
                Ok(())
            }
            PrefetchIntent::ProjectScan => {
                let config_files = ["package.json", "Cargo.toml", "pyproject.toml", "go.mod", "pom.xml", "build.gradle"];
                for config in &config_files {
                    let path = format!("{}/{}", cwd, config);
                    if Path::new(&path).exists() {
                        if let Ok(content) = std::fs::read_to_string(&path) {
                            if let Ok(metadata) = std::fs::metadata(&path) {
                                let mtime = metadata.modified()
                                    .map(|t| t.duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs())
                                    .unwrap_or(0);
                                cache.set_file(&path, content, mtime);
                            }
                        }
                    }
                }
                Ok(())
            }
            PrefetchIntent::GitStatus => {
                let output = std::process::Command::new("git")
                    .args(["status", "--porcelain"])
                    .current_dir(cwd)
                    .output();
                if let Ok(out) = output {
                    if out.status.success() {
                        let stdout = String::from_utf8_lossy(&out.stdout).to_string();
                        cache.set_command("git status --porcelain", cwd, stdout);
                    }
                }
                Ok(())
            }
        }
    }
}

enum PrefetchIntent {
    ReadFile(String),
    ProjectScan,
    GitStatus,
}
