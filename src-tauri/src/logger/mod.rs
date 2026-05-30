use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: String,
    pub message: String,
    pub source: Option<String>,
    pub details: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogFilter {
    pub level: Option<String>,
    pub source: Option<String>,
    pub from_time: Option<String>,
    pub to_time: Option<String>,
    pub search: Option<String>,
}

pub struct Logger {
    log_dir: PathBuf,
    max_file_size: u64,
    max_files: usize,
}

impl Logger {
    pub fn new(log_dir: PathBuf) -> Self {
        Self {
            log_dir,
            max_file_size: 10 * 1024 * 1024,
            max_files: 5,
        }
    }

    pub fn init(&self) -> Result<()> {
        std::fs::create_dir_all(&self.log_dir)?;
        Ok(())
    }

    fn get_log_file_path(&self) -> PathBuf {
        let today = chrono::Local::now().format("%Y-%m-%d");
        self.log_dir.join(format!("app_{}.log", today))
    }

    fn rotate_if_needed(&self, path: &PathBuf) -> Result<()> {
        if let Ok(metadata) = std::fs::metadata(path) {
            if metadata.len() > self.max_file_size {
                let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
                let rotated = self.log_dir.join(format!("app_{}.log.{}", 
                    chrono::Local::now().format("%Y-%m-%d"), timestamp));
                std::fs::rename(path, rotated)?;

                let entries: Vec<_> = std::fs::read_dir(&self.log_dir)?
                    .filter_map(|e| e.ok())
                    .filter(|e| e.path().to_string_lossy().contains(".log."))
                    .collect();
                
                if entries.len() > self.max_files {
                    let mut paths: Vec<_> = entries.iter().map(|e| e.path()).collect();
                    paths.sort();
                    for path in paths.into_iter().take(entries.len() - self.max_files) {
                        let _ = std::fs::remove_file(path);
                    }
                }
            }
        }
        Ok(())
    }

    pub fn log(&self, level: &str, message: &str, source: Option<&str>, details: Option<serde_json::Value>) -> Result<()> {
        self.init()?;
        
        let path = self.get_log_file_path();
        self.rotate_if_needed(&path)?;

        let entry = LogEntry {
            timestamp: chrono::Local::now().to_rfc3339(),
            level: level.to_uppercase(),
            message: message.to_string(),
            source: source.map(|s| s.to_string()),
            details,
        };

        let line = serde_json::to_string(&entry)?;
        std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)?
            .write_all(format!("{}\n", line).as_bytes())?;

        Ok(())
    }

    pub fn info(&self, message: &str) -> Result<()> {
        self.log("INFO", message, None, None)
    }

    pub fn warn(&self, message: &str) -> Result<()> {
        self.log("WARN", message, None, None)
    }

    pub fn error(&self, message: &str) -> Result<()> {
        self.log("ERROR", message, None, None)
    }

    pub fn debug(&self, message: &str) -> Result<()> {
        self.log("DEBUG", message, None, None)
    }

    pub fn read_logs(&self, filter: Option<LogFilter>, limit: usize) -> Result<Vec<LogEntry>> {
        let path = self.get_log_file_path();
        if !path.exists() {
            return Ok(vec![]);
        }

        let content = std::fs::read_to_string(&path)?;
        let mut entries: Vec<LogEntry> = content
            .lines()
            .filter_map(|line| serde_json::from_str(line).ok())
            .collect();

        if let Some(filter) = filter {
            if let Some(level) = filter.level {
                entries.retain(|e| e.level.to_uppercase() == level.to_uppercase());
            }
            if let Some(source) = filter.source {
                entries.retain(|e| e.source.as_ref().map(|s| s.contains(&source)).unwrap_or(false));
            }
            if let Some(search) = filter.search {
                entries.retain(|e| e.message.contains(&search) || 
                    e.source.as_ref().map(|s| s.contains(&search)).unwrap_or(false));
            }
        }

        entries.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        entries.truncate(limit);

        Ok(entries)
    }

    pub fn clear_old_logs(&self, days: u32) -> Result<()> {
        let cutoff = chrono::Local::now() - chrono::Duration::days(days as i64);
        let cutoff_system_time = std::time::SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(cutoff.timestamp() as u64);
        
        let entries: Vec<_> = std::fs::read_dir(&self.log_dir)?
            .filter_map(|e| e.ok())
            .filter(|e| {
                let path = e.path();
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    if name.contains(".log.") {
                        return true;
                    }
                }
                false
            })
            .collect();

        for entry in entries {
            if let Ok(metadata) = entry.metadata() {
                if let Ok(modified) = metadata.modified() {
                    if modified < cutoff_system_time {
                        let _ = std::fs::remove_file(entry.path());
                    }
                }
            }
        }

        Ok(())
    }
}

use std::io::Write;