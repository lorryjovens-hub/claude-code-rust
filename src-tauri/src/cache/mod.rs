use std::collections::HashMap;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::RwLock;
use std::time::Instant;

#[derive(Debug, Clone)]
pub struct CacheEntry {
    pub content: String,
    pub mtime: u64,
    pub hash: String,
    pub cached_at: Instant,
    pub etag: Option<String>,
    pub last_modified: Option<String>,
}

const COMMAND_CACHE_TTL_SECS: u64 = 30;

const IDEMPOTENT_COMMANDS: &[&str] = &[
    "ls", "dir", "cat", "type", "pwd", "cd", "echo", "find", "tree", "stat", "wc", "head", "tail", "grep",
];

fn compute_hash(content: &str) -> String {
    let mut hasher = DefaultHasher::new();
    content.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

fn normalize_path(path: &str) -> String {
    PathBuf::from(path)
        .canonicalize()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| path.to_string())
}

pub struct FileCache {
    file_cache: Arc<RwLock<HashMap<String, CacheEntry>>>,
    url_cache: Arc<RwLock<HashMap<String, CacheEntry>>>,
    command_cache: Arc<RwLock<HashMap<String, CacheEntry>>>,
    dir_cache: Arc<RwLock<HashMap<String, CacheEntry>>>,
}

impl FileCache {
    pub fn new() -> Self {
        Self {
            file_cache: Arc::new(RwLock::new(HashMap::new())),
            url_cache: Arc::new(RwLock::new(HashMap::new())),
            command_cache: Arc::new(RwLock::new(HashMap::new())),
            dir_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn get_file(&self, path: &str, current_mtime: u64) -> Option<String> {
        let key = normalize_path(path);
        let cache = self.file_cache.read().ok()?;
        let entry = cache.get(&key)?;
        if entry.mtime == current_mtime {
            Some(entry.content.clone())
        } else {
            None
        }
    }

    pub fn set_file(&self, path: &str, content: String, mtime: u64) {
        let key = normalize_path(path);
        let entry = CacheEntry {
            content: content.clone(),
            mtime,
            hash: compute_hash(&content),
            cached_at: Instant::now(),
            etag: None,
            last_modified: None,
        };
        if let Ok(mut cache) = self.file_cache.write() {
            cache.insert(key, entry);
        }
    }

    pub fn invalidate_file(&self, path: &str) {
        let key = normalize_path(path);
        if let Ok(mut cache) = self.file_cache.write() {
            cache.remove(&key);
        }
    }

    pub fn invalidate_dir(&self, dir: &str) {
        let dir_key = normalize_path(dir);
        if let Ok(mut cache) = self.file_cache.write() {
            cache.retain(|k, _| !k.starts_with(&dir_key));
        }
        if let Ok(mut cache) = self.dir_cache.write() {
            cache.remove(&dir_key);
        }
    }

    pub fn get_url(&self, url: &str) -> Option<(String, Option<String>, Option<String>)> {
        let cache = self.url_cache.read().ok()?;
        let entry = cache.get(url)?;
        Some((entry.content.clone(), entry.etag.clone(), entry.last_modified.clone()))
    }

    pub fn set_url(&self, url: &str, content: String, etag: Option<String>, last_modified: Option<String>) {
        let entry = CacheEntry {
            content: content.clone(),
            mtime: 0,
            hash: compute_hash(&content),
            cached_at: Instant::now(),
            etag,
            last_modified,
        };
        if let Ok(mut cache) = self.url_cache.write() {
            cache.insert(url.to_string(), entry);
        }
    }

    pub fn get_command(&self, command: &str, cwd: &str) -> Option<String> {
        let key = format!("{}:{}", cwd, command);
        let cache = self.command_cache.read().ok()?;
        let entry = cache.get(&key)?;
        if entry.cached_at.elapsed().as_secs() < COMMAND_CACHE_TTL_SECS {
            Some(entry.content.clone())
        } else {
            None
        }
    }

    pub fn set_command(&self, command: &str, cwd: &str, output: String) {
        let key = format!("{}:{}", cwd, command);
        let entry = CacheEntry {
            content: output.clone(),
            mtime: 0,
            hash: compute_hash(&output),
            cached_at: Instant::now(),
            etag: None,
            last_modified: None,
        };
        if let Ok(mut cache) = self.command_cache.write() {
            cache.insert(key, entry);
        }
    }

    pub fn on_file_write(&self, path: &str) {
        self.invalidate_file(path);
        if let Some(parent) = Path::new(path).parent() {
            self.invalidate_dir(&parent.to_string_lossy());
        }
    }

    pub fn is_idempotent_command(command: &str) -> bool {
        let trimmed = command.trim();
        IDEMPOTENT_COMMANDS.iter().any(|cmd| {
            trimmed.starts_with(cmd) && (trimmed.len() == cmd.len() || trimmed.as_bytes()[cmd.len()] == b' ')
        })
    }

    pub fn get_dir(&self, key: &str) -> Option<String> {
        let norm_key = normalize_path(key);
        let cache = self.dir_cache.read().ok()?;
        let entry = cache.get(&norm_key)?;
        Some(entry.content.clone())
    }

    pub fn set_dir(&self, key: &str, content: String) {
        let norm_key = normalize_path(key);
        let entry = CacheEntry {
            content,
            mtime: 0,
            hash: String::new(),
            cached_at: Instant::now(),
            etag: None,
            last_modified: None,
        };
        if let Ok(mut cache) = self.dir_cache.write() {
            cache.insert(norm_key, entry);
        }
    }
}
