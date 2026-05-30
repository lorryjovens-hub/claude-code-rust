use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlashCommand {
    pub name: String,
    pub description: String,
    pub category: String,
    pub handler: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlashCommandResult {
    pub success: bool,
    pub message: String,
    pub data: Option<serde_json::Value>,
}

pub struct SlashCommandRegistry {
    commands: HashMap<String, SlashCommand>,
}

impl SlashCommandRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            commands: HashMap::new(),
        };
        registry.register_default_commands();
        registry
    }

    fn register_default_commands(&mut self) {
        let defaults = vec![
            SlashCommand {
                name: "/help".to_string(),
                description: "Show available commands".to_string(),
                category: "general".to_string(),
                handler: "help".to_string(),
            },
            SlashCommand {
                name: "/model".to_string(),
                description: "Show or switch model".to_string(),
                category: "general".to_string(),
                handler: "model".to_string(),
            },
            SlashCommand {
                name: "/cost".to_string(),
                description: "Show token usage and cost".to_string(),
                category: "general".to_string(),
                handler: "cost".to_string(),
            },
            SlashCommand {
                name: "/compact".to_string(),
                description: "Compact conversation to free context".to_string(),
                category: "context".to_string(),
                handler: "compact".to_string(),
            },
            SlashCommand {
                name: "/clear".to_string(),
                description: "Clear conversation history".to_string(),
                category: "context".to_string(),
                handler: "clear".to_string(),
            },
            SlashCommand {
                name: "/rewind".to_string(),
                description: "Rewind to previous message".to_string(),
                category: "context".to_string(),
                handler: "rewind".to_string(),
            },
            SlashCommand {
                name: "/branch".to_string(),
                description: "Branch conversation from a message".to_string(),
                category: "context".to_string(),
                handler: "branch".to_string(),
            },
            SlashCommand {
                name: "/stats".to_string(),
                description: "Show usage statistics".to_string(),
                category: "analytics".to_string(),
                handler: "stats".to_string(),
            },
            SlashCommand {
                name: "/usage".to_string(),
                description: "Show detailed usage breakdown".to_string(),
                category: "analytics".to_string(),
                handler: "usage".to_string(),
            },
            SlashCommand {
                name: "/theme".to_string(),
                description: "Switch theme (light/dark/system)".to_string(),
                category: "settings".to_string(),
                handler: "theme".to_string(),
            },
            SlashCommand {
                name: "/config".to_string(),
                description: "Show or modify configuration".to_string(),
                category: "settings".to_string(),
                handler: "config".to_string(),
            },
            SlashCommand {
                name: "/skills".to_string(),
                description: "List available skills".to_string(),
                category: "general".to_string(),
                handler: "skills".to_string(),
            },
            SlashCommand {
                name: "/tasks".to_string(),
                description: "Show active tasks and agents".to_string(),
                category: "general".to_string(),
                handler: "tasks".to_string(),
            },
            SlashCommand {
                name: "/doctor".to_string(),
                description: "Run system diagnostics".to_string(),
                category: "settings".to_string(),
                handler: "doctor".to_string(),
            },
            SlashCommand {
                name: "/resume".to_string(),
                description: "Resume last conversation".to_string(),
                category: "context".to_string(),
                handler: "resume".to_string(),
            },
            SlashCommand {
                name: "/plan".to_string(),
                description: "Toggle plan mode".to_string(),
                category: "general".to_string(),
                handler: "plan".to_string(),
            },
            SlashCommand {
                name: "/mcp".to_string(),
                description: "Manage MCP servers".to_string(),
                category: "settings".to_string(),
                handler: "mcp".to_string(),
            },
            SlashCommand {
                name: "/worktree".to_string(),
                description: "Manage Git worktrees".to_string(),
                category: "general".to_string(),
                handler: "worktree".to_string(),
            },
            SlashCommand {
                name: "/openspace".to_string(),
                description: "Analyze requirements and generate specs".to_string(),
                category: "orchestration".to_string(),
                handler: "openspace".to_string(),
            },
            SlashCommand {
                name: "/superpowers".to_string(),
                description: "Validate plans with engineering rules".to_string(),
                category: "orchestration".to_string(),
                handler: "superpowers".to_string(),
            },
            SlashCommand {
                name: "/gstack".to_string(),
                description: "Generate and manage work plans".to_string(),
                category: "orchestration".to_string(),
                handler: "gstack".to_string(),
            },
            SlashCommand {
                name: "/workflow".to_string(),
                description: "Execute multi-agent workflow".to_string(),
                category: "orchestration".to_string(),
                handler: "workflow".to_string(),
            },
        ];

        for cmd in defaults {
            self.commands.insert(cmd.name.clone(), cmd);
        }
    }

    pub fn list_commands(&self) -> Vec<&SlashCommand> {
        self.commands.values().collect()
    }

    pub fn get_command(&self, name: &str) -> Option<&SlashCommand> {
        self.commands.get(name)
    }

    pub fn search_commands(&self, query: &str) -> Vec<&SlashCommand> {
        let query_lower = query.to_lowercase();
        self.commands
            .values()
            .filter(|cmd| {
                cmd.name.to_lowercase().starts_with(&query_lower)
                    || cmd.description.to_lowercase().contains(&query_lower)
            })
            .collect()
    }

    pub fn get_commands_by_category(&self, category: &str) -> Vec<&SlashCommand> {
        self.commands
            .values()
            .filter(|cmd| cmd.category == category)
            .collect()
    }

    pub fn get_categories(&self) -> Vec<String> {
        let mut categories: Vec<String> = self.commands.values()
            .map(|c| c.category.clone())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();
        categories.sort();
        categories
    }
}

impl Default for SlashCommandRegistry {
    fn default() -> Self {
        Self::new()
    }
}
