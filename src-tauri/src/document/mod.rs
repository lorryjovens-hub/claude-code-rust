use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlideInfo {
    pub title: String,
    pub content: String,
    pub notes: Option<String>,
    pub layout: Option<String>,
    pub left_content: Option<String>,
    pub right_content: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SheetInfo {
    pub name: String,
    pub headers: Vec<String>,
    pub rows: Vec<Vec<serde_json::Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PdfSection {
    #[serde(rename = "type")]
    pub section_type: String,
    pub content: Option<serde_json::Value>,
    pub level: Option<u32>,
    pub headers: Option<Vec<String>>,
    pub rows: Option<Vec<Vec<serde_json::Value>>>,
    pub ordered: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentInfo {
    pub id: String,
    pub title: String,
    pub filename: String,
    pub url: String,
    pub content: Option<String>,
    pub format: Option<String>,
    pub slides: Option<Vec<SlideInfo>>,
    pub sheets: Option<Vec<SheetInfo>>,
    pub sections: Option<Vec<PdfSection>>,
    pub color_scheme: Option<String>,
    pub created_at: String,
    pub conversation_id: Option<String>,
}

pub struct DocumentManager {
    documents: Arc<RwLock<HashMap<String, DocumentInfo>>>,
    storage_path: PathBuf,
}

impl DocumentManager {
    pub fn new(storage_path: PathBuf) -> Self {
        std::fs::create_dir_all(&storage_path).ok();
        Self {
            documents: Arc::new(RwLock::new(HashMap::new())),
            storage_path,
        }
    }

    pub async fn create_document(&self, doc: DocumentInfo) -> String {
        let id = doc.id.clone();
        self.save_document(&id, &doc).await;
        self.documents.write().await.insert(id.clone(), doc);
        id
    }

    pub async fn get_document(&self, id: &str) -> Option<DocumentInfo> {
        if let Some(doc) = self.documents.read().await.get(id) {
            return Some(doc.clone());
        }
        
        let path = self.storage_path.join(format!("{}.json", id));
        if path.exists() {
            if let Ok(content) = std::fs::read_to_string(&path) {
                if let Ok(doc) = serde_json::from_str::<DocumentInfo>(&content) {
                    self.documents.write().await.insert(id.to_string(), doc.clone());
                    return Some(doc);
                }
            }
        }
        None
    }

    pub async fn list_documents(&self) -> Vec<DocumentInfo> {
        let docs = self.documents.read().await;
        let mut result: Vec<DocumentInfo> = docs.values().cloned().collect();
        result.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        result
    }

    pub async fn delete_document(&self, id: &str) -> bool {
        self.documents.write().await.remove(id);
        let path = self.storage_path.join(format!("{}.json", id));
        if path.exists() {
            std::fs::remove_file(&path).ok();
            return true;
        }
        false
    }

    pub async fn update_document(&self, id: &str, updates: DocumentUpdate) -> Option<DocumentInfo> {
        let mut docs = self.documents.write().await;
        if let Some(doc) = docs.get_mut(id) {
            if let Some(title) = updates.title {
                doc.title = title;
            }
            if let Some(content) = updates.content {
                doc.content = Some(content);
            }
            if let Some(format) = updates.format {
                doc.format = Some(format);
            }
            if let Some(slides) = updates.slides {
                doc.slides = Some(slides);
            }
            if let Some(sheets) = updates.sheets {
                doc.sheets = Some(sheets);
            }
            if let Some(sections) = updates.sections {
                doc.sections = Some(sections);
            }
            if let Some(color_scheme) = updates.color_scheme {
                doc.color_scheme = Some(color_scheme);
            }
            self.save_document(id, doc).await;
            return Some(doc.clone());
        }
        None
    }

    async fn save_document(&self, id: &str, doc: &DocumentInfo) {
        let path = self.storage_path.join(format!("{}.json", id));
        if let Ok(json) = serde_json::to_string_pretty(doc) {
            std::fs::write(&path, json).ok();
        }
    }
}

#[derive(Deserialize)]
pub struct DocumentUpdate {
    pub title: Option<String>,
    pub content: Option<String>,
    pub format: Option<String>,
    pub slides: Option<Vec<SlideInfo>>,
    pub sheets: Option<Vec<SheetInfo>>,
    pub sections: Option<Vec<PdfSection>>,
    pub color_scheme: Option<String>,
}
