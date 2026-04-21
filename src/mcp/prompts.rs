//! MCP Prompts - Prompt system

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Prompt {
    pub name: String,
    pub description: String,
    pub arguments: Vec<PromptArgument>,
    pub template: String,
    pub server_name: Option<String>,
}

impl Prompt {
    pub fn new(name: &str, description: &str, template: &str) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            arguments: Vec::new(),
            template: template.to_string(),
            server_name: None,
        }
    }
    
    pub fn with_argument(mut self, name: &str, description: &str, required: bool) -> Self {
        self.arguments.push(PromptArgument {
            name: name.to_string(),
            description: description.to_string(),
            required,
        });
        self
    }
    
    pub fn with_server(mut self, server_name: &str) -> Self {
        self.server_name = Some(server_name.to_string());
        self
    }
    
    pub fn render(&self, args: &HashMap<String, String>) -> String {
        let mut result = self.template.clone();
        for (key, value) in args {
            result = result.replace(&format!("{{{{{}}}}}", key), value);
        }
        result
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptArgument {
    pub name: String,
    pub description: String,
    pub required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptMessage {
    pub role: String,
    pub content: PromptContent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptContent {
    #[serde(rename = "type")]
    pub content_type: String,
    pub text: String,
}

pub struct PromptManager {
    prompts: Arc<RwLock<HashMap<String, Prompt>>>,
}

impl PromptManager {
    pub fn new() -> Self {
        Self {
            prompts: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    pub async fn register(&self, prompt: Prompt) {
        let mut prompts = self.prompts.write().await;
        prompts.insert(prompt.name.clone(), prompt);
    }
    
    pub async fn unregister(&self, name: &str) {
        let mut prompts = self.prompts.write().await;
        prompts.remove(name);
    }
    
    pub async fn get(&self, name: &str) -> Option<Prompt> {
        let prompts = self.prompts.read().await;
        prompts.get(name).cloned()
    }
    
    pub async fn list(&self) -> Vec<Prompt> {
        let prompts = self.prompts.read().await;
        prompts.values().cloned().collect()
    }
    
    pub async fn render(&self, name: &str, args: HashMap<String, String>) -> anyhow::Result<String> {
        let prompts = self.prompts.read().await;
        let prompt = prompts.get(name)
            .ok_or_else(|| anyhow::anyhow!("Prompt not found: {}", name))?;
        
        for arg in &prompt.arguments {
            if arg.required && !args.contains_key(&arg.name) {
                return Err(anyhow::anyhow!("Missing required argument: {}", arg.name));
            }
        }
        
        Ok(prompt.render(&args))
    }
    
    pub async fn register_builtin_prompts(&self) {
        self.register(
            Prompt::new(
                "code_review",
                "Review code for issues and improvements",
                "Please review the following code and provide feedback on:\n1. Code quality\n2. Potential bugs\n3. Performance issues\n4. Security concerns\n\nCode:\n```\n{{code}}\n```\n\nFocus areas: {{focus}}"
            )
            .with_argument("code", "The code to review", true)
            .with_argument("focus", "Specific areas to focus on", false)
        ).await;
        
        self.register(
            Prompt::new(
                "explain_code",
                "Explain what a piece of code does",
                "Please explain the following code in detail:\n\n```\n{{code}}\n```\n\nExplain:\n1. What it does\n2. How it works\n3. Key concepts used\n4. Potential improvements\n\nContext: {{context}}"
            )
            .with_argument("code", "The code to explain", true)
            .with_argument("context", "Additional context", false)
        ).await;
        
        self.register(
            Prompt::new(
                "generate_tests",
                "Generate unit tests for code",
                "Generate comprehensive unit tests for the following code:\n\n```\n{{code}}\n```\n\nRequirements:\n- Test framework: {{framework}}\n- Coverage goal: {{coverage}}\n- Include edge cases\n- Include error handling tests"
            )
            .with_argument("code", "The code to test", true)
            .with_argument("framework", "Test framework to use", false)
            .with_argument("coverage", "Coverage goal percentage", false)
        ).await;
        
        self.register(
            Prompt::new(
                "refactor",
                "Refactor code for better quality",
                "Refactor the following code to improve:\n- Readability\n- Maintainability\n- Performance\n- Error handling\n\nOriginal code:\n```\n{{code}}\n```\n\nGoals: {{goals}}\n\nConstraints: {{constraints}}"
            )
            .with_argument("code", "The code to refactor", true)
            .with_argument("goals", "Refactoring goals", false)
            .with_argument("constraints", "Constraints to follow", false)
        ).await;
        
        self.register(
            Prompt::new(
                "debug",
                "Debug code issues",
                "Help debug the following issue:\n\nProblem: {{problem}}\n\nCode:\n```\n{{code}}\n```\n\nError message:\n{{error}}\n\nExpected behavior: {{expected}}"
            )
            .with_argument("problem", "Description of the problem", true)
            .with_argument("code", "Related code", false)
            .with_argument("error", "Error message", false)
            .with_argument("expected", "Expected behavior", false)
        ).await;
        
        self.register(
            Prompt::new(
                "document",
                "Generate documentation for code",
                "Generate documentation for the following code:\n\n```\n{{code}}\n```\n\nDocumentation style: {{style}}\nInclude:\n- Description\n- Parameters\n- Return value\n- Examples\n- Edge cases"
            )
            .with_argument("code", "The code to document", true)
            .with_argument("style", "Documentation style (rustdoc, jsdoc, etc.)", false)
        ).await;
    }
}

impl Default for PromptManager {
    fn default() -> Self {
        Self::new()
    }
}
