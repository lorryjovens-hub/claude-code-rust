//! Project Initialization

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectConfig {
    pub default_template: String,
    pub templates_dir: PathBuf,
    pub enable_git: bool,
    pub enable_vscode: bool,
    pub author_name: Option<String>,
    pub author_email: Option<String>,
}

impl Default for ProjectConfig {
    fn default() -> Self {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        Self {
            default_template: "basic".to_string(),
            templates_dir: home.join(".claude-code").join("templates"),
            enable_git: true,
            enable_vscode: true,
            author_name: None,
            author_email: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectTemplate {
    pub name: String,
    pub description: String,
    pub language: String,
    pub files: Vec<TemplateFile>,
    pub commands: Vec<String>,
    pub variables: HashMap<String, String>,
}

impl ProjectTemplate {
    pub fn new(name: &str, language: &str) -> Self {
        Self {
            name: name.to_string(),
            description: String::new(),
            language: language.to_string(),
            files: Vec::new(),
            commands: Vec::new(),
            variables: HashMap::new(),
        }
    }
    
    pub fn with_description(mut self, description: &str) -> Self {
        self.description = description.to_string();
        self
    }
    
    pub fn with_file(mut self, path: &str, content: &str) -> Self {
        self.files.push(TemplateFile {
            path: path.to_string(),
            content: content.to_string(),
            executable: false,
        });
        self
    }
    
    pub fn with_command(mut self, command: &str) -> Self {
        self.commands.push(command.to_string());
        self
    }
    
    pub fn with_variable(mut self, key: &str, default: &str) -> Self {
        self.variables.insert(key.to_string(), default.to_string());
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateFile {
    pub path: String,
    pub content: String,
    pub executable: bool,
}

pub struct ProjectInitializer {
    config: ProjectConfig,
    templates: HashMap<String, ProjectTemplate>,
}

impl ProjectInitializer {
    pub fn new(config: ProjectConfig) -> Self {
        let mut initializer = Self {
            config,
            templates: HashMap::new(),
        };
        
        initializer.register_builtin_templates();
        initializer
    }
    
    fn register_builtin_templates(&mut self) {
        self.templates.insert("rust".to_string(), 
            ProjectTemplate::new("rust", "Rust")
                .with_description("Rust project with Cargo")
                .with_file("Cargo.toml", r#"[package]
name = "{{project_name}}"
version = "0.1.0"
edition = "2021"

[dependencies]
"#)
                .with_file("src/main.rs", r#"fn main() {
    println!("Hello, world!");
}
"#)
                .with_file(".gitignore", "/target\n")
                .with_command("cargo init")
        );
        
        self.templates.insert("node".to_string(),
            ProjectTemplate::new("node", "JavaScript/TypeScript")
                .with_description("Node.js project with npm")
                .with_file("package.json", r#"{
  "name": "{{project_name}}",
  "version": "1.0.0",
  "main": "index.js",
  "scripts": {
    "start": "node index.js"
  }
}
"#)
                .with_file("index.js", r#"console.log('Hello, world!');
"#)
                .with_file(".gitignore", "node_modules/\n")
                .with_command("npm init -y")
        );
        
        self.templates.insert("python".to_string(),
            ProjectTemplate::new("python", "Python")
                .with_description("Python project")
                .with_file("main.py", r#"def main():
    print("Hello, world!")

if __name__ == "__main__":
    main()
"#)
                .with_file("requirements.txt", "")
                .with_file(".gitignore", "__pycache__/\n*.pyc\n.env\n")
        );
        
        self.templates.insert("basic".to_string(),
            ProjectTemplate::new("basic", "Generic")
                .with_description("Basic project structure")
                .with_file("README.md", "# {{project_name}}\n\nProject description.\n")
                .with_file(".gitignore", "")
        );
    }
    
    pub fn list_templates(&self) -> Vec<&ProjectTemplate> {
        self.templates.values().collect()
    }
    
    pub fn get_template(&self, name: &str) -> Option<&ProjectTemplate> {
        self.templates.get(name)
    }
    
    pub async fn init(&self, path: &PathBuf, name: &str, template_name: Option<&str>) -> anyhow::Result<()> {
        let template_name = template_name.unwrap_or(&self.config.default_template);
        let template = self.templates.get(template_name)
            .ok_or_else(|| anyhow::anyhow!("Template not found: {}", template_name))?;
        
        let project_path = path.join(name);
        tokio::fs::create_dir_all(&project_path).await?;
        
        println!("📁 Creating project: {} at {:?}", name, project_path);
        
        for file in &template.files {
            let file_path = project_path.join(&file.path);
            
            if let Some(parent) = file_path.parent() {
                tokio::fs::create_dir_all(parent).await?;
            }
            
            let content = self.render_template(&file.content, name);
            tokio::fs::write(&file_path, content).await?;
            
            println!("  ✓ Created: {}", file.path);
        }
        
        if self.config.enable_git {
            self.init_git(&project_path).await?;
        }
        
        if self.config.enable_vscode {
            self.init_vscode(&project_path, template).await?;
        }
        
        for command in &template.commands {
            println!("  ⚙️ Running: {}", command);
            let _ = tokio::process::Command::new("sh")
                .arg("-c")
                .arg(command)
                .current_dir(&project_path)
                .output()
                .await;
        }
        
        println!("✅ Project initialized: {}", name);
        
        Ok(())
    }
    
    fn render_template(&self, content: &str, project_name: &str) -> String {
        let mut result = content.to_string();
        result = result.replace("{{project_name}}", project_name);
        
        if let Some(ref author) = self.config.author_name {
            result = result.replace("{{author}}", author);
        }
        
        if let Some(ref email) = self.config.author_email {
            result = result.replace("{{email}}", email);
        }
        
        result
    }
    
    async fn init_git(&self, path: &PathBuf) -> anyhow::Result<()> {
        let gitignore_path = path.join(".gitignore");
        if !gitignore_path.exists() {
            tokio::fs::write(&gitignore_path, "").await?;
        }
        
        let output = tokio::process::Command::new("git")
            .arg("init")
            .current_dir(path)
            .output()
            .await?;
        
        if output.status.success() {
            println!("  ✓ Initialized git repository");
        }
        
        Ok(())
    }
    
    async fn init_vscode(&self, path: &PathBuf, template: &ProjectTemplate) -> anyhow::Result<()> {
        let vscode_path = path.join(".vscode");
        tokio::fs::create_dir_all(&vscode_path).await?;
        
        let settings = serde_json::json!({
            "files.exclude": {
                "**/.git": true,
                "**/.DS_Store": true
            }
        });
        
        tokio::fs::write(
            vscode_path.join("settings.json"),
            serde_json::to_string_pretty(&settings)?
        ).await?;
        
        let launch_config = match template.language.as_str() {
            "Rust" => serde_json::json!({
                "version": "0.2.0",
                "configurations": [{
                    "type": "lldb",
                    "request": "launch",
                    "name": "Debug",
                    "cargo": {
                        "args": ["build", "--bin=${fileBasenameNoExtension}"],
                    }
                }]
            }),
            "Python" => serde_json::json!({
                "version": "0.2.0",
                "configurations": [{
                    "name": "Python: Current File",
                    "type": "python",
                    "request": "launch",
                    "program": "${file}"
                }]
            }),
            _ => serde_json::json!({
                "version": "0.2.0",
                "configurations": []
            })
        };
        
        tokio::fs::write(
            vscode_path.join("launch.json"),
            serde_json::to_string_pretty(&launch_config)?
        ).await?;
        
        println!("  ✓ Created VS Code configuration");
        
        Ok(())
    }
    
    pub async fn add_template(&mut self, template: ProjectTemplate) {
        self.templates.insert(template.name.clone(), template);
    }
    
    pub async fn load_templates(&mut self) -> anyhow::Result<()> {
        if !self.config.templates_dir.exists() {
            return Ok(());
        }
        
        let mut dir = tokio::fs::read_dir(&self.config.templates_dir).await?;
        
        while let Some(entry) = dir.next_entry().await? {
            let path = entry.path();
            if path.extension().map(|e| e == "json").unwrap_or(false) {
                if let Ok(content) = tokio::fs::read_to_string(&path).await {
                    if let Ok(template) = serde_json::from_str::<ProjectTemplate>(&content) {
                        self.templates.insert(template.name.clone(), template);
                    }
                }
            }
        }
        
        Ok(())
    }
}

impl Default for ProjectInitializer {
    fn default() -> Self {
        Self::new(Default::default())
    }
}
