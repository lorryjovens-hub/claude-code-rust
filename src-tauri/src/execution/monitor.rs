use crate::execution::types::ToolCallResult;
use dashmap::DashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

pub struct ExecutionMonitor {
    total_executions: AtomicU64,
    total_duration: AtomicU64,
    tool_usage: DashMap<String, ToolUsageStats>,
    recent_durations: Arc<tokio::sync::Mutex<Vec<Duration>>>,
}

#[derive(Debug, Clone)]
struct ToolUsageStats {
    count: u64,
    total_duration: Duration,
    success_count: u64,
    fail_count: u64,
}

impl ExecutionMonitor {
    pub fn new() -> Self {
        Self {
            total_executions: AtomicU64::new(0),
            total_duration: AtomicU64::new(0),
            tool_usage: DashMap::new(),
            recent_durations: Arc::new(tokio::sync::Mutex::new(Vec::with_capacity(1000))),
        }
    }

    pub async fn record_tool_execution(&self, result: &ToolCallResult) {
        self.total_executions.fetch_add(1, Ordering::SeqCst);

        let duration_ns = result.duration.as_nanos() as u64;
        self.total_duration.fetch_add(duration_ns, Ordering::SeqCst);

        {
            let mut recent = self.recent_durations.lock().await;
            recent.push(result.duration);
            if recent.len() > 1000 {
                recent.remove(0);
            }
        }

        self.tool_usage
            .entry(result.tool_name.clone())
            .and_modify(|stats| {
                stats.count += 1;
                stats.total_duration += result.duration;
                if result.success {
                    stats.success_count += 1;
                } else {
                    stats.fail_count += 1;
                }
            })
            .or_insert(ToolUsageStats {
                count: 1,
                total_duration: result.duration,
                success_count: if result.success { 1 } else { 0 },
                fail_count: if result.success { 0 } else { 1 },
            });
    }

    pub async fn total_executions(&self) -> u64 {
        self.total_executions.load(Ordering::SeqCst)
    }

    pub async fn average_duration(&self) -> Duration {
        let total = self.total_executions.load(Ordering::SeqCst);
        if total == 0 {
            return Duration::ZERO;
        }

        let total_ns = self.total_duration.load(Ordering::SeqCst);
        Duration::from_nanos(total_ns / total)
    }

    pub async fn recent_average_duration(&self) -> Duration {
        let recent = self.recent_durations.lock().await;
        if recent.is_empty() {
            return Duration::ZERO;
        }

        let total: Duration = recent.iter().sum();
        total / recent.len() as u32
    }

    pub fn tool_usage_stats(&self) -> Vec<(String, u64)> {
        self.tool_usage
            .iter()
            .map(|entry| (entry.key().clone(), entry.value().count))
            .collect()
    }

    pub fn slow_tools(&self, threshold_ms: u64) -> Vec<(String, Duration)> {
        let mut slow = Vec::new();
        let threshold = Duration::from_millis(threshold_ms);

        for entry in self.tool_usage.iter() {
            let avg = entry.value().total_duration / entry.value().count as u32;
            if avg > threshold {
                slow.push((entry.key().clone(), avg));
            }
        }

        slow.sort_by(|a, b| b.1.cmp(&a.1));
        slow
    }
}

impl Default for ExecutionMonitor {
    fn default() -> Self {
        Self::new()
    }
}