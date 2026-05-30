use serde::{Deserialize, Serialize};
use std::sync::Arc;
use crate::knowledge::{KnowledgeBase, KnowledgeNode};

/* ═══════════════════════════════════════════════
   Knowledge Flow Engine
   Connects conversations, agents, skills, and
   knowledge into a self-reinforcing innovation loop.
   ═══════════════════════════════════════════════ */

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InsightEvent {
    pub source: String,          // 'chat' | 'agent' | 'code' | 'feishu' | 'skill'
    pub source_id: String,       // conversation_id / agent_session_id / etc.
    pub content: String,         // The insight/decision/pattern
    pub context: String,         // Surrounding context
    pub tags: Vec<String>,
    pub project_id: Option<String>,
    pub participants: Vec<String>,
}

pub struct KnowledgeFlow {
    pub kb: Arc<KnowledgeBase>,
}

impl KnowledgeFlow {
    pub fn new(kb: Arc<KnowledgeBase>) -> Self {
        Self { kb }
    }

    /// Extract structured knowledge from a conversation exchange.
    /// Called automatically when a message stream completes.
    pub async fn ingest_conversation(
        &self,
        conv_id: &str,
        messages: &[serde_json::Value],
        _project_id: Option<&str>,
    ) -> Result<Vec<String>, String> {
        let mut created_ids = Vec::new();

        // Find key exchanges: user asks a question, assistant provides answer
        for (i, msg) in messages.iter().enumerate() {
            let role = msg.get("role").and_then(|r| r.as_str()).unwrap_or("");
            if role != "user" { continue; }

            let user_text = msg.get("content").and_then(|c| c.as_str()).unwrap_or("");
            if user_text.len() < 30 { continue; } // Skip trivial messages

            // Get the assistant response
            let assistant_response = messages.get(i + 1)
                .and_then(|m| m.get("content"))
                .and_then(|c| c.as_str())
                .unwrap_or("");

            if assistant_response.is_empty() { continue; }

            // Check if this exchange contains a decision or insight
            let is_decision = has_decision_pattern(user_text) || has_decision_pattern(assistant_response);

            if is_decision {
                let title = extract_title(user_text);
                let tags = extract_tags(user_text, assistant_response);
                let now = chrono::Utc::now().to_rfc3339();
                let node = KnowledgeNode {
                    id: uuid::Uuid::new_v4().to_string(),
                    title: format!("[决策] {}", title),
                    content: format!("## 问题\n{}\n\n## 决策\n{}\n\n## 来源\n对话: {}", user_text, assistant_response, conv_id),
                    tags: { let mut t = tags; t.push("auto-captured".to_string()); t.push("decision".to_string()); t },
                    links: vec![],
                    backlinks: vec![],
                    created_at: now.clone(),
                    updated_at: now,
                    source: "chat_auto".to_string(),
                    file_path: None,
                };
                self.kb.add_node(node).await?;
                created_ids.push(format!("decision:{}", &conv_id[..8]));
            }
        }
        Ok(created_ids)
    }

    /// Inject relevant knowledge into a conversation before it starts.
    /// Returns a system-prompt appendix with context from the knowledge base.
    pub async fn prepare_context(&self, query: &str, _project_id: Option<&str>) -> String {
        let nodes = self.kb.search(query).await;
        if nodes.is_empty() { return String::new(); }

        let mut context = String::from("\n\n## 相关知识\n");
        for node in nodes.iter().take(5) {
            context.push_str(&format!("\n- **{}**", node.title));
            if !node.tags.is_empty() {
                context.push_str(&format!(" ({})", node.tags.join(", ")));
            }
            let preview = node.content.chars().take(150).collect::<String>();
            context.push_str(&format!("\n  {}", preview.replace('\n', " ")));
        }
        context
    }

    /// Generate a weekly innovation digest from accumulated knowledge.
    pub async fn generate_digest(&self) -> String {
        let nodes = self.kb.list_nodes().await;
        let recent: Vec<_> = nodes.iter()
            .filter(|n| n.source == "chat_auto")
            .collect();

        if recent.is_empty() {
            return String::from("本周暂无自动捕获的知识。开始对话后，决策和洞察会自动记录。");
        }

        let mut digest = String::from("## 📊 本周知识快照\n\n");
        digest.push_str(&format!("本周捕获 **{}** 条知识\n\n", recent.len()));

        let decisions: Vec<_> = recent.iter().filter(|n| n.tags.contains(&"decision".to_string())).collect();
        if !decisions.is_empty() {
            digest.push_str("### 决策记录\n");
            for d in decisions.iter().take(5) {
                digest.push_str(&format!("- {}\n", d.title));
            }
            digest.push('\n');
        }

        let patterns = find_patterns(&nodes);
        if !patterns.is_empty() {
            digest.push_str("### 发现模式\n");
            for p in patterns.iter().take(3) {
                digest.push_str(&format!("- {}\n", p));
            }
        }

        digest
    }

    /// Cross-project pattern discovery
    pub async fn discover_patterns(&self) -> Vec<String> {
        let nodes = self.kb.list_nodes().await;
        find_patterns(&nodes)
    }
}

fn has_decision_pattern(text: &str) -> bool {
    let lower = text.to_lowercase();
    let signals = ["决定", "选择", "采用", "使用", "改用", "弃用", "迁移",
        "decided", "chose", "selected", "migrate", "replace",
        "最优", "方案", "架构", "设计", "重构", "优化"];
    signals.iter().any(|s| lower.contains(s))
}

fn extract_title(text: &str) -> String {
    let lines: Vec<&str> = text.lines().collect();
    let first = lines.first().and_then(|l| {
        let t = l.trim();
        if t.len() > 100 { Some(&t[..100]) } else { Some(t) }
    }).unwrap_or("Untitled");
    first.to_string()
}

fn extract_tags(user: &str, assistant: &str) -> Vec<String> {
    let combined = format!("{} {}", user, assistant).to_lowercase();
    let mut tags = Vec::new();
    let tech_keywords = ["rust", "typescript", "python", "react", "node", "docker", "kubernetes",
        "api", "database", "frontend", "backend", "devops", "ai", "machine learning"];
    for kw in tech_keywords {
        if combined.contains(kw) { tags.push(kw.to_string()); }
    }
    tags
}

fn find_patterns(nodes: &[KnowledgeNode]) -> Vec<String> {
    let mut tech_counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    for node in nodes {
        for tag in &node.tags {
            *tech_counts.entry(tag.clone()).or_default() += 1;
        }
    }
    let mut patterns = Vec::new();
    for (tag, count) in tech_counts.iter().filter(|(_, c)| **c >= 2) {
        patterns.push(format!("团队在 **{}** 方向有 {} 次决策/讨论，值得沉淀最佳实践", tag, count));
    }
    patterns
}
