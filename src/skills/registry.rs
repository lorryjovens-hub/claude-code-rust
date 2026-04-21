//! Skill Registry
//!
//! Manages registration and lookup of skills.

use super::{Skill, SkillCategory, SkillError};
use std::collections::HashMap;
use std::sync::Arc;

/// Skill registry for managing skills
pub struct SkillRegistry {
    skills: HashMap<String, Arc<dyn Skill>>,
    categories: HashMap<SkillCategory, Vec<String>>,
}

impl SkillRegistry {
    /// Create a new skill registry
    pub fn new() -> Self {
        Self {
            skills: HashMap::new(),
            categories: HashMap::new(),
        }
    }

    /// Register a skill
    pub fn register(&mut self, skill: Arc<dyn Skill>, categories: Vec<SkillCategory>) {
        let name = skill.name().to_string();
        self.skills.insert(name.clone(), skill);

        for category in categories {
            self.categories
                .entry(category)
                .or_insert_with(Vec::new)
                .push(name.clone());
        }
    }

    /// Get a skill by name
    pub fn get(&self, name: &str) -> Option<Arc<dyn Skill>> {
        self.skills.get(name).cloned()
    }

    /// Check if a skill exists
    pub fn has(&self, name: &str) -> bool {
        self.skills.contains_key(name)
    }

    /// List all skill names
    pub fn list_names(&self) -> Vec<String> {
        self.skills.keys().cloned().collect()
    }

    /// List skills by category
    pub fn list_by_category(&self, category: SkillCategory) -> Vec<Arc<dyn Skill>> {
        self.categories
            .get(&category)
            .map(|names| {
                names
                    .iter()
                    .filter_map(|name| self.skills.get(name).cloned())
                    .collect()
            })
            .unwrap_or_default()
    }

    /// List all skills with their descriptions
    pub fn list_all(&self) -> Vec<(String, String)> {
        self.skills
            .iter()
            .map(|(name, skill)| (name.clone(), skill.description().to_string()))
            .collect()
    }

    /// Get skill categories
    pub fn get_categories(&self) -> Vec<SkillCategory> {
        self.categories.keys().cloned().collect()
    }

    /// Search skills by keyword
    pub fn search(&self, keyword: &str) -> Vec<Arc<dyn Skill>> {
        let keyword_lower = keyword.to_lowercase();
        self.skills
            .values()
            .filter(|skill| {
                skill.name().to_lowercase().contains(&keyword_lower) ||
                skill.description().to_lowercase().contains(&keyword_lower)
            })
            .cloned()
            .collect()
    }
}

impl Default for SkillRegistry {
    fn default() -> Self {
        Self::new()
    }
}