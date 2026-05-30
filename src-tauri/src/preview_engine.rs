use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::{RwLock, Notify};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreviewContent {
    pub id: String,
    pub content: String,
    pub content_type: String,
    pub last_updated: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreviewConfig {
    pub reload_strategy: String,
    pub debounce_ms: u64,
}

impl Default for PreviewConfig {
    fn default() -> Self {
        PreviewConfig {
            reload_strategy: "debounce-100".to_string(),
            debounce_ms: 100,
        }
    }
}

pub struct PreviewEngine {
    contents: RwLock<HashMap<String, PreviewContent>>,
    config: PreviewConfig,
    notify: Arc<Notify>,
}

impl PreviewEngine {
    pub fn new(config: Option<PreviewConfig>) -> Self {
        PreviewEngine {
            contents: RwLock::new(HashMap::new()),
            config: config.unwrap_or_default(),
            notify: Arc::new(Notify::new()),
        }
    }

    pub async fn set_content(&self, id: &str, content: &str, content_type: &str) -> Result<()> {
        let mut contents = self.contents.write().await;
        contents.insert(
            id.to_string(),
            PreviewContent {
                id: id.to_string(),
                content: content.to_string(),
                content_type: content_type.to_string(),
                last_updated: Self::current_timestamp(),
            },
        );
        self.notify.notify_waiters();
        Ok(())
    }

    pub async fn get_content(&self, id: &str) -> Result<Option<PreviewContent>> {
        let contents = self.contents.read().await;
        Ok(contents.get(id).cloned())
    }

    pub async fn remove_content(&self, id: &str) -> Result<bool> {
        let mut contents = self.contents.write().await;
        Ok(contents.remove(id).is_some())
    }

    pub async fn load_from_file(&self, id: &str, file_path: &PathBuf) -> Result<()> {
        if !file_path.exists() {
            return Err(anyhow!("File not found: {}", file_path.display()));
        }

        let content = std::fs::read_to_string(file_path)?;
        let content_type = self.infer_content_type(file_path);
        
        self.set_content(id, &content, &content_type).await
    }

    pub async fn list_contents(&self) -> Vec<PreviewContent> {
        let contents = self.contents.read().await;
        contents.values().cloned().collect()
    }

    pub fn infer_content_type(&self, file_path: &PathBuf) -> String {
        match file_path.extension().and_then(|ext| ext.to_str()) {
            Some("html") => "text/html".to_string(),
            Some("css") => "text/css".to_string(),
            Some("js") => "application/javascript".to_string(),
            Some("json") => "application/json".to_string(),
            Some("md") => "text/markdown".to_string(),
            _ => "text/plain".to_string(),
        }
    }

    pub fn get_config(&self) -> PreviewConfig {
        self.config.clone()
    }

    pub fn set_config(&mut self, config: PreviewConfig) {
        self.config = config;
    }

    pub async fn wait_for_update(&self) {
        self.notify.notified().await;
    }

    fn current_timestamp() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64
    }
}

impl Default for PreviewEngine {
    fn default() -> Self {
        Self::new(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[tokio::test]
    async fn test_set_and_get_content() {
        let engine = PreviewEngine::new(None);
        
        engine.set_content("test", "<html>Hello</html>", "text/html").await.unwrap();
        let content = engine.get_content("test").await.unwrap();
        
        assert!(content.is_some());
        assert_eq!(content.unwrap().content, "<html>Hello</html>");
    }

    #[tokio::test]
    async fn test_load_from_file() {
        let engine = PreviewEngine::new(None);
        let mut temp_file = NamedTempFile::new().unwrap();
        std::fs::write(temp_file.path(), "<html>Test</html>").unwrap();
        
        engine.load_from_file("file_test", &temp_file.path().to_path_buf()).await.unwrap();
        let content = engine.get_content("file_test").await.unwrap();
        
        assert!(content.is_some());
        assert_eq!(content.unwrap().content, "<html>Test</html>");
    }

    #[tokio::test]
    async fn test_remove_content() {
        let engine = PreviewEngine::new(None);
        
        engine.set_content("test", "content", "text/plain").await.unwrap();
        assert!(engine.remove_content("test").await.unwrap());
        assert!(engine.get_content("test").await.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_infer_content_type() {
        let engine = PreviewEngine::new(None);
        
        assert_eq!(engine.infer_content_type(&PathBuf::from("test.html")), "text/html");
        assert_eq!(engine.infer_content_type(&PathBuf::from("style.css")), "text/css");
        assert_eq!(engine.infer_content_type(&PathBuf::from("app.js")), "application/javascript");
        assert_eq!(engine.infer_content_type(&PathBuf::from("data.json")), "application/json");
        assert_eq!(engine.infer_content_type(&PathBuf::from("readme.md")), "text/markdown");
        assert_eq!(engine.infer_content_type(&PathBuf::from("unknown.xyz")), "text/plain");
    }
}
