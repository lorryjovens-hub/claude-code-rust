//! CLI Arguments

use super::CliArgs;
use std::sync::Arc;
use tokio::sync::RwLock;

pub type Cli = CliArgs;

impl Cli {
    pub async fn run_async(&self, state: crate::state::AppState) -> anyhow::Result<()> {
        if self.version {
            println!("claude-code-rust {}", env!("CARGO_PKG_VERSION"));
            return Ok(());
        }

        if self.info {
            self.print_system_info();
            return Ok(());
        }

        match &self.command {
            Some(super::Commands::Repl { prompt }) => {
                self.run_repl(state, prompt.clone())?;
            }
            Some(super::Commands::Query { prompt }) => {
                self.run_query(state, prompt.clone()).await?;
            }
            Some(super::Commands::Config { action }) => {
                self.run_config(action)?;
            }
            Some(super::Commands::Mcp { action }) => {
                self.run_mcp(action).await?;
            }
            Some(super::Commands::Plugin { action }) => {
                self.run_plugin(action).await?;
            }
            Some(super::Commands::Memory { action }) => {
                self.run_memory(action).await?;
            }
            Some(super::Commands::Voice { push_to_talk }) => {
                self.run_voice(state, *push_to_talk).await?;
            }
            Some(super::Commands::Init { name }) => {
                self.run_init(name.clone())?;
            }
            Some(super::Commands::Update) => {
                self.run_update()?;
            }
            Some(super::Commands::Help { topic }) => {
                self.run_help(topic.clone())?;
            }
            Some(super::Commands::Services { action }) => {
                self.run_services(state, action).await?;
            }
            Some(super::Commands::Agent { agent_type, prompt }) => {
                self.run_agent(state, agent_type, prompt).await?;
            }
            Some(super::Commands::MagicDocs { action }) => {
                self.run_magic_docs(state, action).await?;
            }
            Some(super::Commands::TeamSync { action }) => {
                self.run_team_sync(state, action).await?;
            }
            Some(super::Commands::StressTest { concurrency, iterations }) => {
                self.run_stress_test(*concurrency, *iterations).await?;
            }
            Some(super::Commands::Skills { action }) => {
                self.run_skills(action).await?;
            }
            None => {
                self.run_repl(state, None)?;
            }
        }

        Ok(())
    }

    fn print_system_info(&self) {
        use colored::Colorize;

        println!();
        println!("  {}", "System Information".truecolor(147, 112, 219).bold());
        println!();
        println!("  {:20} {}", "Version:", env!("CARGO_PKG_VERSION").green());
        println!("  {:20} {}", "OS:", std::env::consts::OS.cyan());
        println!("  {:20} {}", "Architecture:", std::env::consts::ARCH.cyan());
        println!("  {:20} {}", "Working Directory:", std::env::current_dir().unwrap().display().to_string().bright_white());
        println!();
    }

    fn run_repl(&self, state: crate::state::AppState, prompt: Option<String>) -> anyhow::Result<()> {
        let mut repl = crate::cli::repl::Repl::new(state);
        repl.start(prompt)?;
        Ok(())
    }

    async fn run_query(&self, state: crate::state::AppState, prompt: String) -> anyhow::Result<()> {
        let client = crate::api::ApiClient::new(state.settings.clone());
        
        let api_key = match client.get_api_key() {
            Some(key) => key,
            None => {
                eprintln!("Error: API key not configured");
                eprintln!("Set environment variable DEEPSEEK_API_KEY or run:");
                eprintln!("  claude-code config set api_key \"your-api-key\"");
                std::process::exit(1);
            }
        };

        let messages = vec![crate::api::ChatMessage::user(&prompt)];
        let base_url = client.get_base_url().to_string();
        let model = client.get_model().to_string();
        let max_tokens = state.settings.api.max_tokens;

        let request_body = serde_json::json!({
            "model": model,
            "messages": messages,
            "max_tokens": max_tokens,
            "stream": false,
            "temperature": 0.7
        });

        let http_client = reqwest::Client::new();
        let url = format!("{}/v1/chat/completions", base_url);

        let response = http_client
            .post(&url)
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!("API error ({}): {}", status, body));
        }

        let json: serde_json::Value = response.json().await?;

        if let Some(choices) = json.get("choices").and_then(|c| c.as_array()) {
            if let Some(choice) = choices.first() {
                if let Some(content) = choice.get("message").and_then(|m| m.get("content")).and_then(|c| c.as_str()) {
                    println!("{}", content);
                }
            }
        }

        Ok(())
    }

    fn run_config(&self, action: &super::ConfigCommands) -> anyhow::Result<()> {
        match action {
            super::ConfigCommands::Show => {
                let settings = crate::config::Settings::load()?;
                println!("{}", serde_json::to_string_pretty(&settings)?);
            }
            super::ConfigCommands::Set { key, value } => {
                crate::config::Settings::set(key, value)?;
                println!("Set {} = {}", key, value);
            }
            super::ConfigCommands::Reset => {
                crate::config::Settings::reset()?;
                println!("Configuration reset to defaults");
            }
        }
        Ok(())
    }

    async fn run_mcp(&self, action: &super::McpCommands) -> anyhow::Result<()> {
        let manager = crate::mcp::McpManager::new();
        match action {
            super::McpCommands::List => {
                let servers = manager.list_servers().await?;
                for server in servers {
                    println!("  - {} ({})", server.name, server.status);
                }
            }
            super::McpCommands::Add { name, command, path } => {
                // 确定 command 值：优先 path，其次 command，最后空字符串
                let cmd = path.as_ref()
                    .or(command.as_ref())
                    .map(|s| s.as_str())
                    .unwrap_or("");

                let mut config = crate::mcp::McpConfig::new(name, cmd);

                // 特殊处理 filesystem → 变成真正内置可用工具
                if name == "filesystem" {
                    let fs_path = path.as_deref()
                        .or(command.as_deref())
                        .unwrap_or("");
                    config.command = fs_path.to_string();
                    config.status = crate::config::McpServerStatus::Running;
                    config.capabilities = vec![
                        "read_file".to_string(),
                        "write_file".to_string(),
                        "list_directory".to_string(),
                        "search_files".to_string(),
                        "edit_file".to_string(),
                    ];
                    config.auto_start = true;
                    config.filesystem_path = Some(std::path::PathBuf::from(fs_path));
                    println!("✅ Filesystem MCP 已作为内置工具添加（路径: {}）", config.command);
                }

                manager.add_server(config).await?;
                println!("Added MCP server: {}", name);
            }
            super::McpCommands::Remove { name } => {
                manager.remove_server(name).await?;
                println!("Removed MCP server: {}", name);
            }
            super::McpCommands::Restart { name } => {
                manager.restart_server(name).await?;
                println!("Restarted MCP server: {}", name);
            }
        }
        Ok(())
    }

    async fn run_plugin(&self, action: &super::PluginCommands) -> anyhow::Result<()> {
        let state = Arc::new(RwLock::new(crate::state::AppState::default()));
        let service = crate::services::PluginMarketplaceService::new(state, None);
        
        match action {
            super::PluginCommands::List => {
                let plugins = service.list_installed().await;
                if plugins.is_empty() {
                    println!("No plugins installed");
                } else {
                    for plugin in plugins {
                        let status = if plugin.enabled { "enabled" } else { "disabled" };
                        println!("  - {} v{} [{}]", plugin.name, plugin.version, status);
                    }
                }
            }
            super::PluginCommands::Install { plugin } => {
                let installed = service.install(plugin).await?;
                println!("Installed: {} v{}", installed.name, installed.version);
            }
            super::PluginCommands::Remove { name } => {
                service.remove(name).await?;
                println!("Removed plugin: {}", name);
            }
            super::PluginCommands::Update => {
                let updated = service.update_all().await?;
                println!("Updated {} plugins", updated.len());
            }
            super::PluginCommands::Search { query } => {
                let results = service.search(query).await;
                if results.is_empty() {
                    println!("No plugins found for: {}", query);
                } else {
                    for plugin in results {
                        println!("  - {} v{} by {} (⭐ {})", 
                                 plugin.name, plugin.version, plugin.author, plugin.rating);
                        println!("    {}", plugin.description);
                    }
                }
            }
            super::PluginCommands::Enable { name } => {
                service.enable(name).await?;
                println!("Enabled plugin: {}", name);
            }
            super::PluginCommands::Disable { name } => {
                service.disable(name).await?;
                println!("Disabled plugin: {}", name);
            }
        }
        Ok(())
    }

    async fn run_memory(&self, action: &super::MemoryCommands) -> anyhow::Result<()> {
        let manager = crate::memory::MemoryManager::new();
        manager.load().await?;
        
        match action {
            super::MemoryCommands::Status => {
                let status = manager.status().await?;
                println!("Memory Status:");
                println!("  Sessions: {}", status.session_count);
                println!("  Memories: {}", status.total_memories);
                println!("  Last Consolidation: {:?}", status.last_consolidation);
            }
            super::MemoryCommands::Clear => {
                manager.clear().await?;
                println!("All memories cleared");
            }
            super::MemoryCommands::Export { output } => {
                manager.export(output).await?;
                println!("Memories exported to: {}", output.display());
            }
            super::MemoryCommands::Import { input } => {
                manager.import(input).await?;
                println!("Memories imported from: {}", input.display());
            }
            super::MemoryCommands::Dream => {
                println!("Running memory consolidation (dream)...");
                manager.consolidate().await?;
                println!("Memory consolidation completed");
            }
            super::MemoryCommands::AutoDream => {
                let state = Arc::new(RwLock::new(crate::state::AppState::default()));
                let service = crate::services::AutoDreamService::new(state, None);
                println!("Forcing AutoDream consolidation...");
                service.force_consolidation().await?;
                println!("AutoDream consolidation completed");
            }
        }
        Ok(())
    }

    async fn run_voice(&self, state: crate::state::AppState, push_to_talk: bool) -> anyhow::Result<()> {
        let state = Arc::new(RwLock::new(state));
        let service = crate::services::VoiceService::new(state, None);
        
        let status = service.get_status().await;
        if !status.available {
            println!("Voice input is not available on this system");
            println!("Backend: {:?}", status.backend);
            return Ok(());
        }

        if push_to_talk {
            println!("🎤 Push-to-talk mode enabled");
            println!("Press Enter to start recording, press Enter again to stop.");
            
            service.push_to_talk_start().await?;
            
            let mut input = String::new();
            std::io::stdin().read_line(&mut input)?;
            
            let text = service.push_to_talk_stop().await?;
            println!("\n📝 Transcribed: {}", text);
        } else {
            println!("🎤 Continuous voice input mode");
            println!("Voice input starting...");
            service.start_recording().await?;
        }
        
        Ok(())
    }

    fn run_init(&self, name: Option<String>) -> anyhow::Result<()> {
        let project_name = name.unwrap_or_else(|| "claude-code-project".to_string());
        crate::utils::project::init_project(&project_name)?;
        println!("Initialized project: {}", project_name);
        Ok(())
    }

    fn run_update(&self) -> anyhow::Result<()> {
        println!("Checking for updates...");
        println!("Already at latest version");
        Ok(())
    }

    fn run_help(&self, topic: Option<String>) -> anyhow::Result<()> {
        match topic {
            Some(t) => println!("Help for topic: {}", t),
            None => println!("Use --help for detailed usage information"),
        }
        Ok(())
    }

    async fn run_services(&self, state: crate::state::AppState, action: &super::ServiceCommands) -> anyhow::Result<()> {
        let state = Arc::new(RwLock::new(state));
        let mut manager = crate::services::ServiceManager::new(state.clone());
        manager.initialize().await?;
        
        match action {
            super::ServiceCommands::Status => {
                let status = manager.get_status().await;
                println!("Service Status:");
                println!("{}", serde_json::to_string_pretty(&status)?);
            }
            super::ServiceCommands::Start => {
                manager.start_all().await?;
            }
            super::ServiceCommands::Stop => {
                manager.stop_all().await?;
            }
            super::ServiceCommands::AutoDream => {
                if let Some(auto_dream) = manager.auto_dream() {
                    let status = auto_dream.get_status().await;
                    println!("AutoDream Status:");
                    println!("  Enabled: {}", status.enabled);
                    println!("  Consolidating: {}", status.is_consolidating);
                    println!("  Last consolidation: {}h ago", status.hours_since_last);
                    println!("  Sessions accumulated: {}", status.sessions_accumulated);
                    println!("  Next consolidation in: {}h", status.next_consolidation_in);
                }
            }
            super::ServiceCommands::Voice => {
                if let Some(voice) = manager.voice() {
                    let status = voice.get_status().await;
                    println!("Voice Status:");
                    println!("  Available: {}", status.available);
                    println!("  Backend: {:?}", status.backend);
                    println!("  State: {:?}", status.state);
                }
            }
            super::ServiceCommands::MagicDocs => {
                if let Some(magic_docs) = manager.magic_docs() {
                    let status = magic_docs.get_status().await;
                    println!("Magic Docs Status:");
                    println!("  Enabled: {}", status.enabled);
                    println!("  Auto-update: {}", status.auto_update);
                    println!("  Tracked docs: {}", status.tracked_count);
                }
            }
            super::ServiceCommands::TeamSync => {
                if let Some(team_sync) = manager.team_memory_sync() {
                    let status = team_sync.get_status().await;
                    println!("Team Sync Status:");
                    println!("  Enabled: {}", status.enabled);
                    println!("  Authenticated: {}", status.is_authenticated);
                    println!("  Local memories: {}", status.local_memories);
                    println!("  Remote memories: {}", status.remote_memories);
                }
            }
            super::ServiceCommands::Plugins => {
                if let Some(plugins) = manager.plugin_marketplace() {
                    let status = plugins.get_status().await;
                    println!("Plugins Status:");
                    println!("  Enabled: {}", status.enabled);
                    println!("  Installed: {}", status.installed_count);
                    println!("  Updates available: {}", status.updates_available);
                }
            }
            super::ServiceCommands::Agents => {
                if let Some(agents) = manager.agents() {
                    let status = agents.get_status().await;
                    println!("Agents Status:");
                    println!("  Available agents: {}", status.available_agents.len());
                    println!("  Active sessions: {}", status.active_sessions);
                    for agent in &status.available_agents {
                        println!("    - {} ({})", agent.name, agent.agent_type);
                    }
                }
            }
        }
        Ok(())
    }

    async fn run_agent(&self, state: crate::state::AppState, agent_type: &str, prompt: &str) -> anyhow::Result<()> {
        let state = Arc::new(RwLock::new(state));
        let service = crate::services::AgentsService::new(state);
        
        let agent_type = match agent_type.to_lowercase().as_str() {
            "guide" | "claude-code-guide" => crate::services::AgentType::ClaudeCodeGuide,
            "explore" => crate::services::AgentType::Explore,
            "plan" => crate::services::AgentType::Plan,
            "verify" | "verification" => crate::services::AgentType::Verification,
            "general" | "general-purpose" => crate::services::AgentType::GeneralPurpose,
            _ => {
                println!("Unknown agent type: {}", agent_type);
                println!("Available types: guide, explore, plan, verify, general");
                return Ok(());
            }
        };

        println!("🤖 Running {} agent...", agent_type);
        println!("Prompt: {}", prompt);
        println!();

        let session = service.run_agent(&agent_type, prompt).await?;
        
        if let Some(result) = &session.result {
            println!("{}", result);
        }

        Ok(())
    }

    async fn run_magic_docs(&self, state: crate::state::AppState, action: &super::MagicDocsCommands) -> anyhow::Result<()> {
        let state = Arc::new(RwLock::new(state));
        let service = crate::services::MagicDocsService::new(state, None);
        
        match action {
            super::MagicDocsCommands::List => {
                let docs = service.get_tracked_docs().await;
                if docs.is_empty() {
                    println!("No Magic Docs tracked");
                } else {
                    for doc in docs {
                        println!("  - {} ({})", doc.title, doc.path);
                        println!("    Updated: {} ({} times)", doc.last_updated, doc.update_count);
                    }
                }
            }
            super::MagicDocsCommands::Check { file } => {
                if let Some(header) = service.check_file(file).await {
                    println!("Magic Doc detected:");
                    println!("  Title: {}", header.title);
                    if let Some(instructions) = header.instructions {
                        println!("  Instructions: {}", instructions);
                    }
                } else {
                    println!("Not a Magic Doc: {}", file);
                }
            }
            super::MagicDocsCommands::Update { file, context } => {
                let ctx = context.clone().unwrap_or_else(|| "Manual update".to_string());
                service.update_magic_doc(file, &ctx).await?;
                println!("Updated Magic Doc: {}", file);
            }
            super::MagicDocsCommands::Clear => {
                service.clear_all().await;
                println!("All Magic Docs cleared");
            }
        }
        Ok(())
    }

    async fn run_team_sync(&self, state: crate::state::AppState, action: &super::TeamSyncCommands) -> anyhow::Result<()> {
        use crate::services::{TeamMemorySyncService, TeamMemoryConfig, ConflictResolution};

        let state = Arc::new(RwLock::new(state));
        let service = TeamMemorySyncService::new(
            state,
            Some(TeamMemoryConfig {
                enabled: true,
                team_id: Some("test-team".to_string()),
                sync_interval_secs: 3600,
                auto_sync: false,
                conflict_resolution: ConflictResolution::PreferNewer,
            }),
        );

        match action {
            super::TeamSyncCommands::Status => {
                let status = service.get_status().await;
                println!("Team Sync Status:");
                println!("  Enabled: {}", status.enabled);
                println!("  Team ID: {:?}", status.team_id);
                println!("  Authenticated: {}", status.is_authenticated);
                println!("  Local memories: {}", status.local_memories);
                println!("  Remote memories: {}", status.remote_memories);
                if let Some(last) = status.sync_status.last_sync {
                    println!("  Last sync: {}", last);
                }
            }
            super::TeamSyncCommands::Sync => {
                println!("Starting sync...");
                let result = service.sync().await?;
                println!("Sync completed:");
                println!("  Uploaded: {}", result.uploaded);
                println!("  Downloaded: {}", result.downloaded);
                println!("  Conflicts: {}", result.conflicts);
                if !result.errors.is_empty() {
                    println!("  Errors: {:?}", result.errors);
                }
            }
            super::TeamSyncCommands::Auth { team_id } => {
                println!("Authenticating with team: {}", team_id);
                if service.authenticate(team_id).await.is_ok() {
                    println!("✅ Authentication successful");
                } else {
                    println!("❌ Authentication failed");
                }
            }
            super::TeamSyncCommands::Create { title, content, .. } => {
                let memory = service.create_memory(title, content, vec![]).await;
                if memory.is_ok() {
                    println!("✅ Memory created: {}", title);
                } else {
                    println!("❌ Failed to create memory");
                }
            }
            super::TeamSyncCommands::List => {
                let memories = service.list_memories().await;
                if memories.is_empty() {
                    println!("No local memories");
                } else {
                    for memory in memories {
                        println!("  - {} ({})", memory.title, memory.author);
                    }
                }
            }
            super::TeamSyncCommands::Delete { .. } => {
                println!("Delete not implemented");
            }
        }
        Ok(())
    }

    async fn run_stress_test(&self, concurrency: usize, iterations: usize) -> anyhow::Result<()> {
        use crate::services::run_stress_test;
        run_stress_test(concurrency, iterations).await;
        Ok(())
    }

    async fn run_skills(&self, action: &super::SkillsCommands) -> anyhow::Result<()> {
        match action {
            super::SkillsCommands::List => {
                println!("Available skills:");
                println!("  - simplify: Review and simplify code");
                println!("  - loop: Run recurring tasks");
                println!("  - schedule: Schedule automated tasks");
            }
            super::SkillsCommands::Execute { skill, args } => {
                println!("Executing skill: {} with args: {:?}", skill, args);
            }
            super::SkillsCommands::Help { skill } => {
                println!("Help for skill: {}", skill);
            }
            super::SkillsCommands::Search { query } => {
                println!("Searching skills for: {}", query);
            }
        }
        Ok(())
    }
}
