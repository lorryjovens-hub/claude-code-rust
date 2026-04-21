//! Services Module - Background services for Claude Code
//!
//! This module provides various background services including:
//! - AutoDream: Automatic memory consolidation
//! - Voice: Voice input and transcription
//! - MagicDocs: Automatic documentation maintenance
//! - TeamMemorySync: Team memory synchronization
//! - PluginMarketplace: Plugin management
//! - Agents: Built-in agent system

use std::sync::Arc;
use tokio::sync::RwLock;
use crate::state::AppState;

pub mod auto_dream;
pub mod voice;
pub mod magic_docs;
pub mod team_memory_sync;
pub mod plugin_marketplace;
pub mod agents;
pub mod stress_tests;

pub use auto_dream::{AutoDreamService, AutoDreamConfig, AutoDreamStatus};
pub use voice::{VoiceService, VoiceConfig, VoiceBackend, VoiceStatus, RecordingState};
pub use magic_docs::{MagicDocsService, MagicDocsConfig, MagicDocInfo, MagicDocHeader};
pub use team_memory_sync::{TeamMemorySyncService, TeamMemoryConfig, TeamMemorySyncStatus, TeamMemory, ConflictResolution};
pub use plugin_marketplace::{PluginMarketplaceService, PluginConfig, Plugin, MarketplacePlugin};
pub use agents::{AgentsService, AgentDefinition, AgentType, AgentSession, AgentStatus};
pub use stress_tests::{StressTestRunner, StressTestResult, run_stress_test};

/// Background service manager
pub struct ServiceManager {
    state: Arc<RwLock<AppState>>,
    auto_dream: Option<Arc<AutoDreamService>>,
    voice: Option<Arc<VoiceService>>,
    magic_docs: Option<Arc<MagicDocsService>>,
    team_memory_sync: Option<Arc<TeamMemorySyncService>>,
    plugin_marketplace: Option<Arc<PluginMarketplaceService>>,
    agents: Option<Arc<AgentsService>>,
}

impl ServiceManager {
    /// Create a new service manager
    pub fn new(state: Arc<RwLock<AppState>>) -> Self {
        Self {
            state,
            auto_dream: None,
            voice: None,
            magic_docs: None,
            team_memory_sync: None,
            plugin_marketplace: None,
            agents: None,
        }
    }

    /// Initialize all services
    pub async fn initialize(&mut self) -> anyhow::Result<()> {
        println!("🔧 Initializing services...");

        self.auto_dream = Some(Arc::new(AutoDreamService::new(self.state.clone(), None)));
        self.voice = Some(Arc::new(VoiceService::new(self.state.clone(), None)));
        self.magic_docs = Some(Arc::new(MagicDocsService::new(self.state.clone(), None)));
        self.team_memory_sync = Some(Arc::new(TeamMemorySyncService::new(self.state.clone(), None)));
        self.plugin_marketplace = Some(Arc::new(PluginMarketplaceService::new(self.state.clone(), None)));
        self.agents = Some(Arc::new(AgentsService::new(self.state.clone())));

        if let Some(magic_docs) = &self.magic_docs {
            magic_docs.load_state().await?;
        }

        println!("✅ Services initialized");
        Ok(())
    }
    
    /// Start all background services
    pub async fn start_all(&self) -> anyhow::Result<()> {
        println!("🚀 Starting background services...");

        if let Some(auto_dream) = &self.auto_dream {
            let status = auto_dream.get_status().await;
            println!("   🌙 AutoDream: {} (last: {}h ago)", 
                     if status.enabled { "enabled" } else { "disabled" },
                     status.hours_since_last);
        }

        if let Some(voice) = &self.voice {
            let status = voice.get_status().await;
            println!("   🎤 Voice: {} ({:?})", 
                     if status.available { "available" } else { "unavailable" },
                     status.backend);
        }

        if let Some(magic_docs) = &self.magic_docs {
            let status = magic_docs.get_status().await;
            println!("   📚 MagicDocs: {} docs tracked", status.tracked_count);
        }

        if let Some(team_sync) = &self.team_memory_sync {
            let status = team_sync.get_status().await;
            println!("   👥 TeamSync: {} local, {} remote", 
                     status.local_memories, status.remote_memories);
        }

        if let Some(plugins) = &self.plugin_marketplace {
            let status = plugins.get_status().await;
            println!("   🔌 Plugins: {} installed", status.installed_count);
        }

        if let Some(agents) = &self.agents {
            let status = agents.get_status().await;
            println!("   🤖 Agents: {} available, {} active", 
                     status.available_agents.len(), status.active_sessions);
        }
        
        println!("✅ All services started");
        Ok(())
    }
    
    /// Stop all background services
    pub async fn stop_all(&self) -> anyhow::Result<()> {
        println!("🛑 Stopping background services...");

        if let Some(magic_docs) = &self.magic_docs {
            magic_docs.save_state().await?;
        }
        
        println!("✅ All services stopped");
        Ok(())
    }

    pub fn auto_dream(&self) -> Option<Arc<AutoDreamService>> {
        self.auto_dream.clone()
    }

    pub fn voice(&self) -> Option<Arc<VoiceService>> {
        self.voice.clone()
    }

    pub fn magic_docs(&self) -> Option<Arc<MagicDocsService>> {
        self.magic_docs.clone()
    }

    pub fn team_memory_sync(&self) -> Option<Arc<TeamMemorySyncService>> {
        self.team_memory_sync.clone()
    }

    pub fn plugin_marketplace(&self) -> Option<Arc<PluginMarketplaceService>> {
        self.plugin_marketplace.clone()
    }

    pub fn agents(&self) -> Option<Arc<AgentsService>> {
        self.agents.clone()
    }

    pub async fn get_status(&self) -> ServiceStatus {
        ServiceStatus {
            auto_dream: self.auto_dream.as_ref().map(|s| futures::executor::block_on(s.get_status())),
            voice: self.voice.as_ref().map(|s| futures::executor::block_on(s.get_status())),
            magic_docs: self.magic_docs.as_ref().map(|s| futures::executor::block_on(s.get_status())),
            team_sync: self.team_memory_sync.as_ref().map(|s| futures::executor::block_on(s.get_status())),
            plugins: self.plugin_marketplace.as_ref().map(|s| futures::executor::block_on(s.get_status())),
            agents: self.agents.as_ref().map(|s| futures::executor::block_on(s.get_status())),
        }
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ServiceStatus {
    pub auto_dream: Option<AutoDreamStatus>,
    pub voice: Option<VoiceStatus>,
    pub magic_docs: Option<magic_docs::MagicDocsStatus>,
    pub team_sync: Option<TeamMemorySyncStatus>,
    pub plugins: Option<plugin_marketplace::PluginStatus>,
    pub agents: Option<agents::AgentStatusReport>,
}
