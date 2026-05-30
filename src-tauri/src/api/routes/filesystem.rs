//! Filesystem endpoints with path traversal protection.

use crate::api::error::ApiError;
use crate::api::state::AppState;
use crate::fs::FileOperations;
use axum::{Router, routing::get, routing::post, extract::Query, Json};
use serde::Deserialize;
use serde_json::json;

#[derive(Deserialize)]
struct FsPathQuery {
    path: Option<String>,
}

#[derive(Deserialize)]
struct FsWriteRequest {
    path: String,
    content: String,
}

#[derive(Deserialize)]
struct FsCreateRequest {
    path: String,
    #[serde(default)]
    is_dir: bool,
}

/// Build a file tree with cycle detection and depth limits.
fn build_tree(dir_path: &str, max_depth: u32, visited: &mut Vec<String>) -> Result<serde_json::Value, ApiError> {
    if max_depth == 0 {
        return Ok(json!([{"name": "... (max depth)", "path": "", "is_dir": false, "size": 0}]));
    }

    let canonical = std::path::Path::new(dir_path)
        .canonicalize()
        .map_err(|_| ApiError::bad_request("invalid path"))?;
    let canonical_str = canonical.to_string_lossy().to_string();

    if visited.contains(&canonical_str) {
        return Ok(json!([{"name": "... (symlink cycle)", "path": "", "is_dir": false, "size": 0}]));
    }
    visited.push(canonical_str);

    let entries = FileOperations::list_directory(dir_path, false)
        .map_err(|_| ApiError::internal("failed to list directory"))?;

    let mut children = Vec::new();
    for entry in entries {
        let mut node = json!({
            "name": entry.name,
            "path": entry.path,
            "is_dir": entry.is_dir,
            "size": entry.size,
        });

        if entry.is_dir {
            node["children"] = build_tree(&entry.path, max_depth - 1, visited).unwrap_or(json!([]));
        }

        children.push(node);
    }

    Ok(json!(children))
}

async fn fs_tree(Query(query): Query<FsPathQuery>) -> Result<Json<serde_json::Value>, ApiError> {
    let path = query.path.unwrap_or_else(|| {
        dirs::home_dir()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| ".".to_string())
    });

    let tree = build_tree(&path, 10, &mut Vec::new())?;

    Ok(Json(json!({"success": true, "path": path, "tree": tree})))
}

async fn fs_read(Query(query): Query<FsPathQuery>) -> Result<Json<serde_json::Value>, ApiError> {
    let path = query.path.ok_or(ApiError::bad_request("path is required"))?;
    let content = FileOperations::read_file(&path, None, None)?;
    Ok(Json(json!({"success": true, "path": path, "content": content})))
}

async fn fs_write(Json(req): Json<FsWriteRequest>) -> Result<Json<serde_json::Value>, ApiError> {
    FileOperations::write_file(&req.path, &req.content)?;
    Ok(Json(json!({"success": true, "path": req.path})))
}

async fn fs_create(Json(req): Json<FsCreateRequest>) -> Result<Json<serde_json::Value>, ApiError> {
    if req.is_dir {
        FileOperations::create_directory(&req.path)?;
    } else {
        FileOperations::write_file(&req.path, "")?;
    }
    Ok(Json(json!({"success": true, "path": req.path})))
}

async fn fs_delete(Json(req): Json<FsCreateRequest>) -> Result<Json<serde_json::Value>, ApiError> {
    FileOperations::delete_file(&req.path)?;
    Ok(Json(json!({"success": true, "path": req.path})))
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/api/fs/tree", get(fs_tree))
        .route("/api/fs/read", get(fs_read))
        .route("/api/fs/write", post(fs_write))
        .route("/api/fs/create", post(fs_create))
        .route("/api/fs/delete", post(fs_delete))
}
