//! Consolidation - Memory consolidation engine

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

use super::{MemoryEntry, MemoryType};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsolidationConfig {
    pub max_memories: usize,
    pub importance_threshold: f32,
    pub age_threshold_hours: u64,
    pub consolidation_interval_hours: u64,
    pub enable_auto_consolidation: bool,
}

impl Default for ConsolidationConfig {
    fn default() -> Self {
        Self {
            max_memories: 10000,
            importance_threshold: 0.3,
            age_threshold_hours: 24,
            consolidation_interval_hours: 6,
            enable_auto_consolidation: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsolidationResult {
    pub memories_before: usize,
    pub memories_after: usize,
    pub memories_consolidated: usize,
    pub memories_removed: usize,
    pub insights_generated: usize,
    pub duration_ms: u64,
    pub timestamp: DateTime<Utc>,
}

pub struct ConsolidationEngine {
    config: ConsolidationConfig,
    last_consolidation: Option<DateTime<Utc>>,
}

impl ConsolidationEngine {
    pub fn new(config: ConsolidationConfig) -> Self {
        Self {
            config,
            last_consolidation: None,
        }
    }
    
    pub async fn consolidate(&self, memories: &[MemoryEntry]) -> anyhow::Result<Vec<MemoryEntry>> {
        let start = std::time::Instant::now();
        let memories_before = memories.len();
        
        let mut consolidated = Vec::new();
        let mut to_merge: Vec<&MemoryEntry> = Vec::new();
        let _insights: Vec<String> = Vec::new();
        
        let mut sorted_memories: Vec<_> = memories.iter().collect();
        sorted_memories.sort_by(|a, b| b.importance.partial_cmp(&a.importance).unwrap_or(std::cmp::Ordering::Equal));
        
        for memory in sorted_memories {
            if consolidated.len() >= self.config.max_memories {
                break;
            }
            
            if self.should_keep(memory) {
                if self.is_similar_to_any(memory, &consolidated) {
                    to_merge.push(memory);
                } else {
                    consolidated.push(memory.clone());
                }
            }
        }
        
        if !to_merge.is_empty() {
            let merged = self.merge_memories(&to_merge);
            consolidated.push(merged);
        }
        
        let insights_generated = self.extract_insights(&memories, &mut consolidated);
        
        let result = ConsolidationResult {
            memories_before,
            memories_after: consolidated.len(),
            memories_consolidated: to_merge.len(),
            memories_removed: memories_before - consolidated.len(),
            insights_generated,
            duration_ms: start.elapsed().as_millis() as u64,
            timestamp: Utc::now(),
        };
        
        println!("🧠 Consolidation complete: {} -> {} memories ({} insights)", 
            result.memories_before, result.memories_after, result.insights_generated);
        
        Ok(consolidated)
    }
    
    fn should_keep(&self, memory: &MemoryEntry) -> bool {
        if memory.importance >= self.config.importance_threshold {
            return true;
        }
        
        let age = (Utc::now() - memory.timestamp).num_hours() as u64;
        if age < self.config.age_threshold_hours {
            return true;
        }
        
        matches!(memory.memory_type, MemoryType::Knowledge | MemoryType::Preference)
    }
    
    fn is_similar_to_any(&self, memory: &MemoryEntry, others: &[MemoryEntry]) -> bool {
        others.iter().any(|other| {
            self.calculate_similarity(memory, other) > 0.8
        })
    }
    
    fn calculate_similarity(&self, a: &MemoryEntry, b: &MemoryEntry) -> f32 {
        if a.memory_type != b.memory_type {
            return 0.0;
        }
        
        let a_words: std::collections::HashSet<&str> = a.content.split_whitespace().collect();
        let b_words: std::collections::HashSet<&str> = b.content.split_whitespace().collect();
        
        if a_words.is_empty() || b_words.is_empty() {
            return 0.0;
        }
        
        let intersection = a_words.intersection(&b_words).count();
        let union = a_words.union(&b_words).count();
        
        intersection as f32 / union as f32
    }
    
    fn merge_memories(&self, memories: &[&MemoryEntry]) -> MemoryEntry {
        let mut combined_content = String::new();
        let mut max_importance: f32 = 0.0;
        let mut all_tags: Vec<String> = Vec::new();
        
        for memory in memories {
            if !combined_content.is_empty() {
                combined_content.push('\n');
            }
            combined_content.push_str(&memory.content);
            max_importance = max_importance.max(memory.importance);
            all_tags.extend(memory.tags.clone());
        }
        
        all_tags.sort();
        all_tags.dedup();
        
        MemoryEntry::new(memories[0].memory_type.clone(), &combined_content)
            .with_importance(max_importance + 0.1)
            .with_tags(all_tags)
    }
    
    fn extract_insights(&self, memories: &[MemoryEntry], consolidated: &mut Vec<MemoryEntry>) -> usize {
        let mut insights = 0;
        
        let mut type_counts: std::collections::HashMap<MemoryType, usize> = std::collections::HashMap::new();
        for memory in memories {
            *type_counts.entry(memory.memory_type.clone()).or_insert(0) += 1;
        }
        
        for (memory_type, count) in type_counts {
            if count >= 10 {
                let insight_content = format!(
                    "Pattern detected: {} {} memories recorded. Consider reviewing for optimization.",
                    count, 
                    match memory_type {
                        MemoryType::Session => "session",
                        MemoryType::Conversation => "conversation",
                        MemoryType::Knowledge => "knowledge",
                        MemoryType::Preference => "preference",
                        MemoryType::Task => "task",
                        MemoryType::Error => "error",
                        MemoryType::Insight => "insight",
                    }
                );
                
                consolidated.push(
                    MemoryEntry::new(MemoryType::Insight, &insight_content)
                        .with_importance(0.7)
                );
                insights += 1;
            }
        }
        
        let error_memories: Vec<_> = memories.iter()
            .filter(|m| m.memory_type == MemoryType::Error)
            .collect();
        
        if error_memories.len() >= 3 {
            let error_patterns: Vec<String> = error_memories.iter()
                .take(5)
                .map(|m| m.content.chars().take(100).collect::<String>())
                .collect();
            
            consolidated.push(
                MemoryEntry::new(MemoryType::Insight, &format!(
                    "Recurring errors detected: {} errors. Recent: {}",
                    error_memories.len(),
                    error_patterns.join("; ")
                ))
                .with_importance(0.8)
            );
            insights += 1;
        }
        
        insights
    }
    
    pub fn should_consolidate(&self, memory_count: usize) -> bool {
        memory_count >= self.config.max_memories
    }
    
    pub fn last_consolidation(&self) -> Option<DateTime<Utc>> {
        self.last_consolidation
    }
    
    pub fn config(&self) -> &ConsolidationConfig {
        &self.config
    }
}

impl Default for ConsolidationEngine {
    fn default() -> Self {
        Self::new(Default::default())
    }
}
