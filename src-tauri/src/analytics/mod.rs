use anyhow::Result;
use chrono::{Duration, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::memory::CavemanRTKStats;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsEvent {
    pub id: String,
    pub event_type: String,
    pub timestamp: String,
    pub properties: serde_json::Value,
    pub session_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackEventRequest {
    pub event_type: String,
    pub properties: Option<serde_json::Value>,
    pub session_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyStats {
    pub date: String,
    pub messages_sent: u64,
    pub conversations_created: u64,
    pub tokens_input: u64,
    pub tokens_output: u64,
    pub tools_executed: u64,
    pub errors: u64,
    pub voice_inputs: u64,
    pub slash_commands: u64,
    pub files_uploaded: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageSummary {
    pub total_days: u64,
    pub total_messages: u64,
    pub total_conversations: u64,
    pub total_tokens_input: u64,
    pub total_tokens_output: u64,
    pub total_tools: u64,
    pub total_errors: u64,
    pub total_voice_inputs: u64,
    pub avg_daily_messages: f64,
    pub avg_daily_tokens: f64,
    pub streak_days: u64,
    pub most_active_day: String,
    pub most_used_model: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelUsage {
    pub model: String,
    pub count: u64,
    pub total_input_tokens: u64,
    pub total_output_tokens: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventTypeCount {
    pub event_type: String,
    pub count: u64,
}

/// Combined dashboard stats including Caveman RTK
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardStats {
    pub daily: DailyStats,
    pub summary: UsageSummary,
    pub caveman: Option<CavemanRTKStats>,
    pub event_types: Vec<EventTypeCount>,
    pub recent_events: Vec<AnalyticsEvent>,
}

pub struct AnalyticsStore {
    store_dir: PathBuf,
    cache: Arc<Mutex<HashMap<String, DailyStats>>>,
}

impl AnalyticsStore {
    pub fn new(store_dir: PathBuf) -> Self {
        fs::create_dir_all(&store_dir).ok();
        let cache = Arc::new(Mutex::new(HashMap::new()));
        Self { store_dir, cache }
    }

    pub fn today_key(&self) -> String {
        Utc::now().format("%Y-%m-%d").to_string()
    }

    fn stats_path(&self, date: &str) -> PathBuf {
        self.store_dir.join(format!("{}.json", date))
    }

    async fn load_stats(&self, date: &str) -> DailyStats {
        {
            let cache = self.cache.lock().await;
            if let Some(stats) = cache.get(date) {
                return stats.clone();
            }
        }

        let path = self.stats_path(date);
        if path.exists() {
            if let Ok(data) = fs::read_to_string(&path) {
                if let Ok(stats) = serde_json::from_str::<DailyStats>(&data) {
                    return stats;
                }
            }
        }

        DailyStats {
            date: date.to_string(),
            messages_sent: 0,
            conversations_created: 0,
            tokens_input: 0,
            tokens_output: 0,
            tools_executed: 0,
            errors: 0,
            voice_inputs: 0,
            slash_commands: 0,
            files_uploaded: 0,
        }
    }

    async fn save_stats(&self, stats: &DailyStats) -> Result<()> {
        let path = self.stats_path(&stats.date);
        let data = serde_json::to_string_pretty(stats)?;
        fs::write(&path, data)?;

        let mut cache = self.cache.lock().await;
        cache.insert(stats.date.clone(), stats.clone());

        Ok(())
    }

    pub async fn track_event(&self, event: &TrackEventRequest) -> Result<()> {
        let today = self.today_key();
        let mut stats = self.load_stats(&today).await;

        match event.event_type.as_str() {
            "message_sent" => stats.messages_sent += 1,
            "conversation_created" => stats.conversations_created += 1,
            "tokens_used" => {
                if let Some(props) = &event.properties {
                    stats.tokens_input += props.get("input_tokens").and_then(|v| v.as_u64()).unwrap_or(0);
                    stats.tokens_output += props.get("output_tokens").and_then(|v| v.as_u64()).unwrap_or(0);
                }
            }
            "tool_executed" => stats.tools_executed += 1,
            "error" => stats.errors += 1,
            "voice_input" => stats.voice_inputs += 1,
            "slash_command" => stats.slash_commands += 1,
            "file_uploaded" => stats.files_uploaded += 1,
            _ => {}
        }

        self.save_stats(&stats).await?;

        let events_dir = self.store_dir.join("events");
        fs::create_dir_all(&events_dir).ok();

        let event_record = AnalyticsEvent {
            id: uuid::Uuid::new_v4().to_string()[..12].to_string(),
            event_type: event.event_type.clone(),
            timestamp: Utc::now().to_rfc3339(),
            properties: event.properties.clone().unwrap_or(serde_json::json!({})),
            session_id: event.session_id.clone(),
        };

        let event_file = events_dir.join(format!("{}-{}.json", today, event_record.id));
        let event_data = serde_json::to_string(&event_record)?;
        fs::write(&event_file, event_data)?;

        Ok(())
    }

    pub async fn get_daily_stats(&self, date: &str) -> Option<DailyStats> {
        Some(self.load_stats(date).await)
    }

    pub async fn get_stats_range(&self, from: &str, to: &str) -> Vec<DailyStats> {
        let mut result = Vec::new();
        let mut current = from.to_string();
        let to_owned = to.to_string();

        while current <= to_owned {
            result.push(self.load_stats(&current).await);
            if let Ok(d) = NaiveDate::parse_from_str(&current, "%Y-%m-%d") {
                let next = d + Duration::days(1);
                current = next.format("%Y-%m-%d").to_string();
            } else {
                break;
            }
        }

        result
    }

    pub async fn get_usage_summary(&self, days: u32) -> UsageSummary {
        let end = Utc::now();
        let start = end - Duration::days(days as i64);

        let stats = self.get_stats_range(
            &start.format("%Y-%m-%d").to_string(),
            &end.format("%Y-%m-%d").to_string(),
        ).await;

        let total_messages: u64 = stats.iter().map(|s| s.messages_sent).sum();
        let total_conversations: u64 = stats.iter().map(|s| s.conversations_created).sum();
        let total_tokens_input: u64 = stats.iter().map(|s| s.tokens_input).sum();
        let total_tokens_output: u64 = stats.iter().map(|s| s.tokens_output).sum();
        let total_tools: u64 = stats.iter().map(|s| s.tools_executed).sum();
        let total_errors: u64 = stats.iter().map(|s| s.errors).sum();
        let total_voice_inputs: u64 = stats.iter().map(|s| s.voice_inputs).sum();

        let _active_days = stats.iter().filter(|s| s.messages_sent > 0).count() as u64;

        let mut streak: u64 = 0;
        for s in stats.iter().rev() {
            if s.messages_sent > 0 {
                streak += 1;
            } else {
                break;
            }
        }

        let most_active_day = stats
            .iter()
            .max_by_key(|s| s.messages_sent)
            .map(|s| s.date.clone())
            .unwrap_or_default();

        UsageSummary {
            total_days: days as u64,
            total_messages,
            total_conversations,
            total_tokens_input,
            total_tokens_output,
            total_tools,
            total_errors,
            total_voice_inputs,
            avg_daily_messages: if days > 0 { total_messages as f64 / days as f64 } else { 0.0 },
            avg_daily_tokens: if days > 0 { (total_tokens_input + total_tokens_output) as f64 / days as f64 } else { 0.0 },
            streak_days: streak,
            most_active_day,
            most_used_model: String::new(),
        }
    }

    pub fn get_event_type_counts(&self, days: u32) -> Vec<EventTypeCount> {
        let end = Utc::now();
        let start = end - Duration::days(days as i64);

        let events_dir = self.store_dir.join("events");
        if !events_dir.exists() {
            return vec![];
        }

        let mut counts: HashMap<String, u64> = HashMap::new();

        let start_str = start.format("%Y-%m-%d").to_string();
        let end_str = end.format("%Y-%m-%d").to_string();

        if let Ok(entries) = fs::read_dir(&events_dir) {
            for entry in entries.flatten() {
                if let Ok(data) = fs::read_to_string(entry.path()) {
                    if let Ok(event) = serde_json::from_str::<AnalyticsEvent>(&data) {
                        if event.timestamp.len() >= 10 {
                            let event_date = event.timestamp[..10].to_string();
                            if event_date >= start_str && event_date <= end_str
                            {
                                *counts.entry(event.event_type).or_insert(0) += 1;
                            }
                        }
                    }
                }
            }
        }

        let mut result: Vec<EventTypeCount> = counts
            .into_iter()
            .map(|(event_type, count)| EventTypeCount { event_type, count })
            .collect();
        result.sort_by(|a, b| b.count.cmp(&a.count));
        result
    }

    pub fn get_recent_events(&self, limit: usize) -> Vec<AnalyticsEvent> {
        let events_dir = self.store_dir.join("events");
        if !events_dir.exists() {
            return vec![];
        }

        let mut events: Vec<AnalyticsEvent> = Vec::new();

        if let Ok(entries) = fs::read_dir(&events_dir) {
            let mut files: Vec<_> = entries.flatten().collect();
            files.sort_by(|a, b| {
                b.metadata().and_then(|m| m.modified()).ok()
                    .cmp(&a.metadata().and_then(|m| m.modified()).ok())
            });

            for entry in files.iter().take(limit * 3) {
                if let Ok(data) = fs::read_to_string(entry.path()) {
                    if let Ok(event) = serde_json::from_str::<AnalyticsEvent>(&data) {
                        events.push(event);
                    }
                }
            }
        }

        events.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        events.truncate(limit);
        events
    }

    /// Get complete dashboard stats with optional Caveman RTK stats
    pub async fn get_dashboard_stats(&self, days: u32, caveman_stats: Option<CavemanRTKStats>) -> DashboardStats {
        let today = self.today_key();
        let daily = self.load_stats(&today).await;
        let summary = self.get_usage_summary(days).await;
        let event_types = self.get_event_type_counts(days);
        let recent_events = self.get_recent_events(50);

        DashboardStats {
            daily,
            summary,
            caveman: caveman_stats,
            event_types,
            recent_events,
        }
    }
}
