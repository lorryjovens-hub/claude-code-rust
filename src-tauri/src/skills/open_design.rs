use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenDesignMetadata {
    pub mode: Option<String>,
    pub platform: Option<String>,
    pub scenario: Option<String>,
    pub preview: Option<PreviewConfig>,
    pub design_system: Option<DesignSystemConfig>,
    pub inputs: Option<Vec<InputField>>,
    pub outputs: Option<OutputConfig>,
    pub capabilities_required: Option<Vec<String>>,
    pub category: Option<String>,
    pub upstream: Option<String>,
    pub triggers: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreviewConfig {
    pub r#type: String,
    pub entry: Option<String>,
    pub reload: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DesignSystemConfig {
    pub requires: Option<bool>,
    pub generates: Option<bool>,
    pub sections: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputField {
    pub name: String,
    pub r#type: String,
    pub required: Option<bool>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputConfig {
    pub primary: Option<String>,
    pub secondary: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenDesignSkill {
    pub skill_id: String,
    pub name: String,
    pub description: String,
    pub metadata: OpenDesignMetadata,
    pub source_dir: PathBuf,
}

impl OpenDesignMetadata {
    pub fn is_design_system_mode(&self) -> bool {
        self.mode.as_deref() == Some("design-system")
    }

    pub fn has_preview(&self) -> bool {
        self.preview.is_some()
    }

    pub fn get_preview_entry(&self) -> Option<String> {
        self.preview.as_ref().and_then(|p| p.entry.clone())
    }

    pub fn get_reload_strategy(&self) -> String {
        self.preview.as_ref()
            .and_then(|p| p.reload.clone())
            .unwrap_or_else(|| "debounce-100".to_string())
    }
}

impl OpenDesignSkill {
    pub fn preview_exists(&self) -> bool {
        if let Some(entry) = self.metadata.get_preview_entry() {
            let preview_path = self.source_dir.join(&entry);
            preview_path.exists()
        } else {
            false
        }
    }

    pub fn get_preview_path(&self) -> Option<PathBuf> {
        self.metadata.get_preview_entry()
            .map(|entry| self.source_dir.join(entry))
    }

    pub fn get_output_files(&self) -> Vec<String> {
        let mut files = Vec::new();
        if let Some(outputs) = &self.metadata.outputs {
            if let Some(primary) = &outputs.primary {
                files.push(primary.clone());
            }
            if let Some(secondary) = &outputs.secondary {
                files.push(secondary.clone());
            }
        }
        files
    }

    pub fn requires_capability(&self, capability: &str) -> bool {
        self.metadata.capabilities_required
            .as_ref()
            .map(|caps| caps.contains(&capability.to_string()))
            .unwrap_or(false)
    }
}
