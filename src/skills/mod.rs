//! Skills Framework
//!
//! A framework for defining and executing high-level skills that combine
//! multiple tools and operations. Skills are similar to macros or workflows
//! that can be triggered by specific commands (e.g., /commit, /review).
//!
//! Features:
//! - Skill trait for defining custom skills
//! - Skill registry for managing skills
//! - Skill executor with context and parameter parsing
//! - Built-in skills for common operations
//! - Skill chaining and composition

pub mod registry;
pub mod executor;
pub mod builtin;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json;
use std::collections::HashMap;

/// Skill execution context
#[derive(Clone)]
pub struct SkillContext {
    /// Current working directory
    pub cwd: String,
    /// Environment variables
    pub env: HashMap<String, String>,
    /// Tool registry for executing tools
    pub tool_registry: Option<std::sync::Arc<crate::tools::ToolRegistry>>,
    /// Additional context data
    pub data: HashMap<String, serde_json::Value>,
}

impl std::fmt::Debug for SkillContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SkillContext")
            .field("cwd", &self.cwd)
            .field("env", &self.env)
            .field("tool_registry", &"[ToolRegistry]")
            .field("data", &self.data)
            .finish()
    }
}

/// Skill parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillParams {
    /// Raw input string
    pub raw_input: String,
    /// Parsed arguments
    pub args: Vec<String>,
    /// Named parameters
    pub named_params: HashMap<String, String>,
    /// Flags
    pub flags: HashMap<String, bool>,
}

/// Skill result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillResult {
    /// Success flag
    pub success: bool,
    /// Output message
    pub message: String,
    /// Detailed output
    pub output: Option<serde_json::Value>,
    /// Metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Skill error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillError {
    /// Error message
    pub message: String,
    /// Error code
    pub code: String,
    /// Details
    pub details: Option<serde_json::Value>,
}

/// Skill trait
#[async_trait]
pub trait Skill: Send + Sync {
    /// Skill name (e.g., "commit", "review")
    fn name(&self) -> &str;

    /// Skill description
    fn description(&self) -> &str;

    /// Skill usage examples
    fn examples(&self) -> Vec<String>;

    /// Skill parameter schema
    fn parameter_schema(&self) -> serde_json::Value;

    /// Execute the skill
    async fn execute(&self, params: SkillParams, context: SkillContext) -> Result<SkillResult, SkillError>;
}

/// Skill category
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SkillCategory {
    /// Git operations
    Git,
    /// Code review
    CodeReview,
    /// Testing
    Testing,
    /// Documentation
    Documentation,
    /// Project setup
    ProjectSetup,
    /// Debugging
    Debugging,
    /// Utility
    Utility,
}

impl std::fmt::Display for SkillCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SkillCategory::Git => write!(f, "git"),
            SkillCategory::CodeReview => write!(f, "code_review"),
            SkillCategory::Testing => write!(f, "testing"),
            SkillCategory::Documentation => write!(f, "documentation"),
            SkillCategory::ProjectSetup => write!(f, "project_setup"),
            SkillCategory::Debugging => write!(f, "debugging"),
            SkillCategory::Utility => write!(f, "utility"),
        }
    }
}

/// Re-exports
pub use registry::SkillRegistry;
pub use executor::SkillExecutor;
pub use builtin::BuiltinSkills;