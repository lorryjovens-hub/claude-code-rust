use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadResult {
    #[serde(rename = "fileId")]
    pub file_id: String,
    #[serde(rename = "fileName")]
    pub file_name: String,
    #[serde(rename = "fileType")]
    pub file_type: String,
    #[serde(rename = "mimeType")]
    pub mime_type: String,
    pub size: u64,
    #[serde(rename = "lineCount")]
    pub line_count: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct FileMetadata {
    pub file_id: String,
    pub file_name: String,
    pub file_type: String,
    pub mime_type: String,
    pub size: u64,
    pub line_count: Option<usize>,
    pub created_at: String,
    pub conversation_id: Option<String>,
}

pub struct UploadManager {
    upload_dir: PathBuf,
    metadata: Arc<Mutex<HashMap<String, FileMetadata>>>,
}

impl UploadManager {
    pub fn new(upload_dir: PathBuf) -> Self {
        Self {
            upload_dir,
            metadata: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn init(&self) -> Result<()> {
        fs::create_dir_all(&self.upload_dir)?;
        Ok(())
    }

    pub fn get_upload_dir(&self) -> &Path {
        &self.upload_dir
    }

    fn detect_file_type(mime_type: &str, file_name: &str) -> String {
        if mime_type.starts_with("image/") {
            return "image".to_string();
        }

        let ext = Path::new(file_name)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        let text_extensions = [
            "txt", "md", "csv", "json", "xml", "yaml", "yml", "js", "jsx", "ts", "tsx",
            "py", "java", "cpp", "c", "h", "cs", "go", "rs", "rb", "php", "swift",
            "kt", "scala", "html", "css", "scss", "less", "sql", "sh", "bash",
            "vue", "svelte", "lua", "r", "m", "pl", "ex", "exs",
        ];

        if text_extensions.contains(&ext.as_str()) || mime_type.starts_with("text/") {
            return "text".to_string();
        }

        "document".to_string()
    }

    fn count_lines(file_path: &Path) -> Option<usize> {
        if let Ok(content) = fs::read_to_string(file_path) {
            Some(content.lines().count())
        } else {
            None
        }
    }

    pub async fn save_file(
        &self,
        file_name: &str,
        mime_type: &str,
        data: &[u8],
        conversation_id: Option<&str>,
    ) -> Result<UploadResult> {
        let file_id = Uuid::new_v4().to_string();
        let file_path = self.upload_dir.join(&file_id);

        fs::write(&file_path, data)?;

        let file_type = Self::detect_file_type(mime_type, file_name);
        let size = data.len() as u64;
        let line_count = if file_type == "text" {
            Self::count_lines(&file_path)
        } else {
            None
        };

        let metadata = FileMetadata {
            file_id: file_id.clone(),
            file_name: file_name.to_string(),
            file_type: file_type.clone(),
            mime_type: mime_type.to_string(),
            size,
            line_count,
            created_at: chrono::Utc::now().to_rfc3339(),
            conversation_id: conversation_id.map(String::from),
        };

        let mut meta_map = self.metadata.lock().await;
        meta_map.insert(file_id.clone(), metadata);

        Ok(UploadResult {
            file_id,
            file_name: file_name.to_string(),
            file_type,
            mime_type: mime_type.to_string(),
            size,
            line_count,
        })
    }

    pub async fn get_file(&self, file_id: &str) -> Result<Vec<u8>> {
        let file_path = self.upload_dir.join(file_id);
        if !file_path.exists() {
            return Err(anyhow!("File not found: {}", file_id));
        }
        Ok(fs::read(&file_path)?)
    }

    pub async fn get_metadata(&self, file_id: &str) -> Result<FileMetadata> {
        let meta_map = self.metadata.lock().await;
        meta_map
            .get(file_id)
            .cloned()
            .ok_or_else(|| anyhow!("Metadata not found: {}", file_id))
    }

    pub fn get_file_path(&self, file_id: &str) -> String {
        self.upload_dir.join(file_id).to_string_lossy().to_string()
    }

    pub async fn delete_file(&self, file_id: &str) -> Result<()> {
        let file_path = self.upload_dir.join(file_id);
        if file_path.exists() {
            fs::remove_file(&file_path)?;
        }

        let mut meta_map = self.metadata.lock().await;
        meta_map.remove(file_id);

        Ok(())
    }

    pub async fn list_files(&self) -> Vec<FileMetadata> {
        let meta_map = self.metadata.lock().await;
        meta_map.values().cloned().collect()
    }
}
