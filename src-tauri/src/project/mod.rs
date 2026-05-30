use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub instructions: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub is_archived: bool,
    pub file_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectFile {
    pub id: String,
    pub project_id: String,
    pub file_name: String,
    pub file_path: String,
    pub file_size: u64,
    pub mime_type: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectMetadata {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub instructions: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub is_archived: bool,
    pub files: Vec<ProjectFile>,
}

pub struct ProjectManager {
    projects_dir: PathBuf,
    projects: Arc<Mutex<HashMap<String, ProjectMetadata>>>,
}

impl ProjectManager {
    pub fn new(projects_dir: PathBuf) -> Self {
        Self {
            projects_dir,
            projects: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn init(&self) -> Result<()> {
        fs::create_dir_all(&self.projects_dir)?;
        Ok(())
    }

    pub fn get_projects_dir(&self) -> &Path {
        &self.projects_dir
    }

    pub async fn create_project(&self, name: &str, description: Option<&str>) -> Result<Project> {
        let id = Uuid::new_v4().to_string();
        let project_dir = self.projects_dir.join(&id);
        fs::create_dir_all(&project_dir)?;

        let now = chrono::Utc::now().to_rfc3339();
        let project = Project {
            id: id.clone(),
            name: name.to_string(),
            description: description.map(String::from),
            instructions: None,
            created_at: now.clone(),
            updated_at: now,
            is_archived: false,
            file_count: 0,
        };

        let metadata = ProjectMetadata {
            id: id.clone(),
            name: name.to_string(),
            description: description.map(String::from),
            instructions: None,
            created_at: project.created_at.clone(),
            updated_at: project.updated_at.clone(),
            is_archived: false,
            files: Vec::new(),
        };

        let mut projects_map = self.projects.lock().await;
        projects_map.insert(id.clone(), metadata);

        Ok(project)
    }

    pub async fn get_project(&self, id: &str) -> Result<Project> {
        let projects_map = self.projects.lock().await;
        let metadata = projects_map
            .get(id)
            .ok_or_else(|| anyhow!("Project not found: {}", id))?;

        Ok(Project {
            id: metadata.id.clone(),
            name: metadata.name.clone(),
            description: metadata.description.clone(),
            instructions: metadata.instructions.clone(),
            created_at: metadata.created_at.clone(),
            updated_at: metadata.updated_at.clone(),
            is_archived: metadata.is_archived,
            file_count: metadata.files.len(),
        })
    }

    pub async fn list_projects(&self) -> Vec<Project> {
        let projects_map = self.projects.lock().await;
        projects_map
            .values()
            .map(|m| Project {
                id: m.id.clone(),
                name: m.name.clone(),
                description: m.description.clone(),
                instructions: m.instructions.clone(),
                created_at: m.created_at.clone(),
                updated_at: m.updated_at.clone(),
                is_archived: m.is_archived,
                file_count: m.files.len(),
            })
            .collect()
    }

    pub async fn update_project(
        &self,
        id: &str,
        name: Option<&str>,
        description: Option<&str>,
        instructions: Option<&str>,
        is_archived: Option<bool>,
    ) -> Result<Project> {
        let mut projects_map = self.projects.lock().await;
        let metadata = projects_map
            .get_mut(id)
            .ok_or_else(|| anyhow!("Project not found: {}", id))?;

        if let Some(n) = name {
            metadata.name = n.to_string();
        }
        if let Some(d) = description {
            metadata.description = Some(d.to_string());
        }
        if let Some(i) = instructions {
            metadata.instructions = Some(i.to_string());
        }
        if let Some(a) = is_archived {
            metadata.is_archived = a;
        }
        metadata.updated_at = chrono::Utc::now().to_rfc3339();

        Ok(Project {
            id: metadata.id.clone(),
            name: metadata.name.clone(),
            description: metadata.description.clone(),
            instructions: metadata.instructions.clone(),
            created_at: metadata.created_at.clone(),
            updated_at: metadata.updated_at.clone(),
            is_archived: metadata.is_archived,
            file_count: metadata.files.len(),
        })
    }

    pub async fn delete_project(&self, id: &str) -> Result<()> {
        let project_dir = self.projects_dir.join(id);
        if project_dir.exists() {
            fs::remove_dir_all(&project_dir)?;
        }

        let mut projects_map = self.projects.lock().await;
        projects_map.remove(id);

        Ok(())
    }

    pub async fn upload_file(
        &self,
        project_id: &str,
        file_name: &str,
        mime_type: &str,
        data: &[u8],
    ) -> Result<ProjectFile> {
        let mut projects_map = self.projects.lock().await;
        let metadata = projects_map
            .get_mut(project_id)
            .ok_or_else(|| anyhow!("Project not found: {}", project_id))?;

        let file_id = Uuid::new_v4().to_string();
        let project_dir = self.projects_dir.join(project_id);
        let file_path = project_dir.join(&file_id);

        fs::write(&file_path, data)?;

        let project_file = ProjectFile {
            id: file_id.clone(),
            project_id: project_id.to_string(),
            file_name: file_name.to_string(),
            file_path: file_path.to_string_lossy().to_string(),
            file_size: data.len() as u64,
            mime_type: mime_type.to_string(),
            created_at: chrono::Utc::now().to_rfc3339(),
        };

        metadata.files.push(project_file.clone());
        metadata.updated_at = chrono::Utc::now().to_rfc3339();

        Ok(project_file)
    }

    pub async fn get_project_files(&self, project_id: &str) -> Result<Vec<ProjectFile>> {
        let projects_map = self.projects.lock().await;
        let metadata = projects_map
            .get(project_id)
            .ok_or_else(|| anyhow!("Project not found: {}", project_id))?;

        Ok(metadata.files.clone())
    }

    pub async fn delete_project_file(
        &self,
        project_id: &str,
        file_id: &str,
    ) -> Result<()> {
        let mut projects_map = self.projects.lock().await;
        let metadata = projects_map
            .get_mut(project_id)
            .ok_or_else(|| anyhow!("Project not found: {}", project_id))?;

        let file_index = metadata.files.iter().position(|f| f.id == file_id);
        if let Some(index) = file_index {
            let file = metadata.files.remove(index);
            let file_path = Path::new(&file.file_path);
            if file_path.exists() {
                fs::remove_file(file_path)?;
            }
            metadata.updated_at = chrono::Utc::now().to_rfc3339();
        }

        Ok(())
    }

    pub async fn get_file_content(&self, project_id: &str, file_id: &str) -> Result<Vec<u8>> {
        let projects_map = self.projects.lock().await;
        let metadata = projects_map
            .get(project_id)
            .ok_or_else(|| anyhow!("Project not found: {}", project_id))?;

        let file = metadata
            .files
            .iter()
            .find(|f| f.id == file_id)
            .ok_or_else(|| anyhow!("File not found: {}", file_id))?;

        let file_path = Path::new(&file.file_path);
        if !file_path.exists() {
            return Err(anyhow!("File not found on disk: {}", file_id));
        }

        Ok(fs::read(file_path)?)
    }
}
