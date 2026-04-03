//! Path utilities

use std::path::{Path, PathBuf};

/// Get the home directory
pub fn home_dir() -> Option<PathBuf> {
    dirs::home_dir()
}

/// Get the config directory
pub fn config_dir() -> Option<PathBuf> {
    dirs::config_dir()
}

/// Get the data directory
pub fn data_dir() -> Option<PathBuf> {
    dirs::data_dir()
}

/// Get the cache directory
pub fn cache_dir() -> Option<PathBuf> {
    dirs::cache_dir()
}

/// Get the Claude Code config directory
pub fn claude_config_dir() -> PathBuf {
    config_dir()
        .map(|d| d.join("claude-code"))
        .unwrap_or_else(|| PathBuf::from(".claude-code"))
}

/// Get the Claude Code data directory
pub fn claude_data_dir() -> PathBuf {
    data_dir()
        .map(|d| d.join("claude-code"))
        .unwrap_or_else(|| PathBuf::from(".claude-code"))
}

/// Expand a path that starts with ~ to the home directory
pub fn expand_tilde<P: AsRef<Path>>(path: P) -> PathBuf {
    let path = path.as_ref();
    if path.starts_with("~") {
        if let Some(home) = home_dir() {
            return home.join(path.strip_prefix("~").unwrap_or(path));
        }
    }
    path.to_path_buf()
}

/// Normalize a path (resolve . and ..)
pub fn normalize_path<P: AsRef<Path>>(path: P) -> PathBuf {
    let path = expand_tilde(path);
    let mut result = PathBuf::new();
    
    for component in path.components() {
        match component {
            std::path::Component::ParentDir => {
                result.pop();
            }
            _ => {
                result.push(component);
            }
        }
    }
    
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expand_tilde() {
        let home = home_dir().unwrap();
        assert_eq!(expand_tilde("~/test"), home.join("test"));
        assert_eq!(expand_tilde("/absolute/path"), PathBuf::from("/absolute/path"));
    }
}
