//! Note Edit Tool
//!
//! Create, edit, and manage notes with Markdown support.
//! Features include:
//! - create: Create a new note
//! - edit: Edit an existing note
//! - delete: Delete a note
//! - list: List all notes
//! - search: Search notes by content or tags
//! - get: Get note details and content

use super::{Tool, ToolOutput, ToolError};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Note {
    pub id: String,
    pub title: String,
    pub content: String,
    pub format: NoteFormat,
    pub tags: Vec<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub metadata: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum NoteFormat {
    Markdown,
    PlainText,
    RichText,
}

pub struct NoteEditTool {
    notes: Arc<RwLock<HashMap<String, Note>>>,
}

impl Default for NoteEditTool {
    fn default() -> Self {
        Self::new()
    }
}

impl NoteEditTool {
    pub fn new() -> Self {
        Self {
            notes: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    fn generate_id(&self) -> String {
        uuid::Uuid::new_v4().to_string()
    }

    async fn create_note(&self, input: &serde_json::Value) -> Result<Note, ToolError> {
        let title = input["title"].as_str()
            .ok_or_else(|| ToolError {
                message: "title is required".to_string(),
                code: Some("missing_title".to_string()),
            })?;

        let content = input["content"].as_str()
            .ok_or_else(|| ToolError {
                message: "content is required".to_string(),
                code: Some("missing_content".to_string()),
            })?;

        let format = input["format"].as_str()
            .unwrap_or("markdown")
            .parse::<NoteFormat>()
            .map_err(|_| ToolError {
                message: "Invalid format. Must be markdown, plaintext, or richtext".to_string(),
                code: Some("invalid_format".to_string()),
            })?;

        let tags: Vec<String> = input["tags"]
            .as_array()
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
            .unwrap_or_default();

        let metadata: HashMap<String, serde_json::Value> = input["metadata"]
            .as_object()
            .map(|obj| {
                obj.iter()
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .collect()
            })
            .unwrap_or_default();

        let now = chrono::Utc::now();
        let note = Note {
            id: self.generate_id(),
            title: title.to_string(),
            content: content.to_string(),
            format,
            tags,
            created_at: now,
            updated_at: now,
            metadata,
        };

        Ok(note)
    }

    async fn update_note(&self, note_id: &str, input: &serde_json::Value) -> Result<Note, ToolError> {
        let mut notes = self.notes.write().await;
        let note = notes.get_mut(note_id)
            .ok_or_else(|| ToolError {
                message: format!("Note not found: {}", note_id),
                code: Some("note_not_found".to_string()),
            })?;

        if let Some(title) = input["title"].as_str() {
            note.title = title.to_string();
        }

        if let Some(content) = input["content"].as_str() {
            note.content = content.to_string();
        }

        if let Some(format_str) = input["format"].as_str() {
            note.format = format_str.parse::<NoteFormat>()
                .map_err(|_| ToolError {
                    message: "Invalid format. Must be markdown, plaintext, or richtext".to_string(),
                    code: Some("invalid_format".to_string()),
                })?;
        }

        if let Some(tags_array) = input["tags"].as_array() {
            note.tags = tags_array.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect();
        }

        if let Some(metadata_obj) = input["metadata"].as_object() {
            for (key, value) in metadata_obj {
                note.metadata.insert(key.clone(), value.clone());
            }
        }

        note.updated_at = chrono::Utc::now();

        Ok(note.clone())
    }

    async fn search_notes(&self, query: &str, tags: &[String]) -> Result<Vec<Note>, ToolError> {
        let notes = self.notes.read().await;

        let query_lower = query.to_lowercase();
        let results: Vec<Note> = notes.values()
            .filter(|note| {
                // Search in title and content
                let matches_content = note.title.to_lowercase().contains(&query_lower) ||
                                     note.content.to_lowercase().contains(&query_lower);

                // Filter by tags if provided
                let matches_tags = if tags.is_empty() {
                    true
                } else {
                    tags.iter().all(|tag| note.tags.contains(tag))
                };

                matches_content && matches_tags
            })
            .cloned()
            .collect();

        Ok(results)
    }
}

#[async_trait]
impl Tool for NoteEditTool {
    fn name(&self) -> &str {
        "note_edit"
    }

    fn description(&self) -> &str {
        "Create, edit, and manage notes with Markdown support"
    }

    fn input_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "operation": {
                    "type": "string",
                    "description": "Note operation: create, edit, delete, list, search, get",
                    "enum": ["create", "edit", "delete", "list", "search", "get"]
                },
                "note_id": {
                    "type": "string",
                    "description": "Note ID (required for edit, delete, get)"
                },
                "title": {
                    "type": "string",
                    "description": "Note title (required for create)"
                },
                "content": {
                    "type": "string",
                    "description": "Note content (required for create)"
                },
                "format": {
                    "type": "string",
                    "description": "Note format: markdown, plaintext, richtext",
                    "enum": ["markdown", "plaintext", "richtext"]
                },
                "tags": {
                    "type": "array",
                    "items": {
                        "type": "string"
                    },
                    "description": "Tags for the note"
                },
                "metadata": {
                    "type": "object",
                    "description": "Additional metadata for the note"
                },
                "search_query": {
                    "type": "string",
                    "description": "Search query (for search operation)"
                },
                "search_tags": {
                    "type": "array",
                    "items": {
                        "type": "string"
                    },
                    "description": "Tags to filter by (for search operation)"
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

        match operation {
            "create" => self.handle_create(input).await,
            "edit" => self.handle_edit(input).await,
            "delete" => self.handle_delete(input).await,
            "list" => self.handle_list(input).await,
            "search" => self.handle_search(input).await,
            "get" => self.handle_get(input).await,
            _ => Err(ToolError {
                message: format!("Unknown note operation: {}", operation),
                code: Some("invalid_operation".to_string()),
            }),
        }
    }
}

impl NoteEditTool {
    async fn handle_create(&self, input: serde_json::Value) -> Result<ToolOutput, ToolError> {
        let note = self.create_note(&input).await?;
        let note_id = note.id.clone();

        let mut notes = self.notes.write().await;
        notes.insert(note_id.clone(), note);

        Ok(ToolOutput {
            output_type: "json".to_string(),
            content: serde_json::json!({
                "success": true,
                "message": "Note created successfully",
                "note_id": note_id
            }).to_string(),
            metadata: std::collections::HashMap::new(),
        })
    }

    async fn handle_edit(&self, input: serde_json::Value) -> Result<ToolOutput, ToolError> {
        let note_id = input["note_id"].as_str()
            .ok_or_else(|| ToolError {
                message: "note_id is required for edit".to_string(),
                code: Some("missing_note_id".to_string()),
            })?;

        let note = self.update_note(note_id, &input).await?;

        Ok(ToolOutput {
            output_type: "json".to_string(),
            content: serde_json::json!({
                "success": true,
                "message": "Note updated successfully",
                "note": note
            }).to_string(),
            metadata: std::collections::HashMap::new(),
        })
    }

    async fn handle_delete(&self, input: serde_json::Value) -> Result<ToolOutput, ToolError> {
        let note_id = input["note_id"].as_str()
            .ok_or_else(|| ToolError {
                message: "note_id is required for delete".to_string(),
                code: Some("missing_note_id".to_string()),
            })?;

        let mut notes = self.notes.write().await;
        let removed = notes.remove(note_id);

        if removed.is_some() {
            Ok(ToolOutput {
                output_type: "json".to_string(),
                content: serde_json::json!({
                    "success": true,
                    "message": "Note deleted successfully"
                }).to_string(),
                metadata: std::collections::HashMap::new(),
            })
        } else {
            Err(ToolError {
                message: format!("Note not found: {}", note_id),
                code: Some("note_not_found".to_string()),
            })
        }
    }

    async fn handle_list(&self, _input: serde_json::Value) -> Result<ToolOutput, ToolError> {
        let notes = self.notes.read().await;
        let note_list: Vec<&Note> = notes.values().collect();

        Ok(ToolOutput {
            output_type: "json".to_string(),
            content: serde_json::json!({
                "success": true,
                "notes": note_list,
                "count": note_list.len()
            }).to_string(),
            metadata: std::collections::HashMap::new(),
        })
    }

    async fn handle_search(&self, input: serde_json::Value) -> Result<ToolOutput, ToolError> {
        let query = input["search_query"].as_str().unwrap_or("");
        let tags: Vec<String> = input["search_tags"]
            .as_array()
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
            .unwrap_or_default();

        let results = self.search_notes(query, &tags).await?;

        Ok(ToolOutput {
            output_type: "json".to_string(),
            content: serde_json::json!({
                "success": true,
                "results": results,
                "count": results.len()
            }).to_string(),
            metadata: std::collections::HashMap::new(),
        })
    }

    async fn handle_get(&self, input: serde_json::Value) -> Result<ToolOutput, ToolError> {
        let note_id = input["note_id"].as_str()
            .ok_or_else(|| ToolError {
                message: "note_id is required for get".to_string(),
                code: Some("missing_note_id".to_string()),
            })?;

        let notes = self.notes.read().await;
        let note = notes.get(note_id)
            .ok_or_else(|| ToolError {
                message: format!("Note not found: {}", note_id),
                code: Some("note_not_found".to_string()),
            })?;

        Ok(ToolOutput {
            output_type: "json".to_string(),
            content: serde_json::json!({
                "success": true,
                "note": note
            }).to_string(),
            metadata: std::collections::HashMap::new(),
        })
    }
}

impl std::str::FromStr for NoteFormat {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "markdown" => Ok(NoteFormat::Markdown),
            "plaintext" => Ok(NoteFormat::PlainText),
            "richtext" => Ok(NoteFormat::RichText),
            _ => Err(()),
        }
    }
}