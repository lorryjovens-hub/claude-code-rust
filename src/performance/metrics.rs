//! 监控和遥测系统
//! 
//! 实现完整的性能指标收集系统

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};

const SAMPLE_INTERVAL_SECS: u64 = 5;
const AGGREGATION_INTERVAL_SECS: u64 = 60;
const RETENTION_DAYS: i64 = 30;

/// 指标收集器
pub struct MetricsCollector {
    counters: Arc<RwLock<HashMap<String, AttributedCounter>>>,
    snapshots: Arc<RwLock<Vec<MetricsSnapshot>>>,
    last_sample: Arc<RwLock<Option<Instant>>>,
    config: MetricsConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsConfig {
    pub sample_interval_secs: u64,
    pub aggregation_interval_secs: u64,
    pub retention_days: i64,
}

impl Default for MetricsConfig {
    fn default() -> Self {
        Self {
            sample_interval_secs: SAMPLE_INTERVAL_SECS,
            aggregation_interval_secs: AGGREGATION_INTERVAL_SECS,
            retention_days: RETENTION_DAYS,
        }
    }
}

impl MetricsCollector {
    pub fn new() -> Self {
        Self {
            counters: Arc::new(RwLock::new(HashMap::new())),
            snapshots: Arc::new(RwLock::new(Vec::new())),
            last_sample: Arc::new(RwLock::new(None)),
            config: MetricsConfig::default(),
        }
    }

    pub fn with_config(config: MetricsConfig) -> Self {
        Self {
            counters: Arc::new(RwLock::new(HashMap::new())),
            snapshots: Arc::new(RwLock::new(Vec::new())),
            last_sample: Arc::new(RwLock::new(None)),
            config,
        }
    }

    /// 初始化标准指标
    pub async fn init_standard_metrics(&self) {
        let mut counters = self.counters.write().await;

        counters.insert(
            "session_counter".to_string(),
            AttributedCounter::new(MetricType::SessionCounter),
        );
        counters.insert(
            "loc_counter".to_string(),
            AttributedCounter::new(MetricType::LocCounter),
        );
        counters.insert(
            "pr_counter".to_string(),
            AttributedCounter::new(MetricType::PrCounter),
        );
        counters.insert(
            "commit_counter".to_string(),
            AttributedCounter::new(MetricType::CommitCounter),
        );
        counters.insert(
            "cost_counter".to_string(),
            AttributedCounter::new(MetricType::CostCounter),
        );
        counters.insert(
            "token_counter".to_string(),
            AttributedCounter::new(MetricType::TokenCounter),
        );
        counters.insert(
            "code_edit_tool_decision_counter".to_string(),
            AttributedCounter::new(MetricType::CodeEditToolDecisionCounter),
        );
        counters.insert(
            "active_time_counter".to_string(),
            AttributedCounter::new(MetricType::ActiveTimeCounter),
        );
    }

    /// 采集指标样本
    pub async fn sample(&self) -> MetricsSnapshot {
        let counters = self.counters.read().await;
        let mut snapshot = MetricsSnapshot::new();

        for (name, counter) in counters.iter() {
            snapshot.values.insert(name.clone(), counter.value());
        }

        snapshot.timestamp = chrono::Utc::now().timestamp_millis();

        let mut last_sample = self.last_sample.write().await;
        *last_sample = Some(Instant::now());

        snapshot
    }

    /// 聚合指标
    pub async fn aggregate(&self) -> MetricsSnapshot {
        let snapshots = self.snapshots.write().await;
        let now = chrono::Utc::now().timestamp_millis();
        let window_start = now - (self.config.aggregation_interval_secs as i64 * 1000);

        let recent: Vec<MetricsSnapshot> = snapshots
            .iter()
            .filter(|s| s.timestamp >= window_start)
            .cloned()
            .collect();

        if recent.is_empty() {
            return MetricsSnapshot::new();
        }

        let mut aggregated = MetricsSnapshot::new();
        aggregated.timestamp = now;

        for snapshot in &recent {
            for (key, value) in &snapshot.values {
                let entry = aggregated.values.entry(key.clone()).or_insert(0);
                *entry += value;
            }
        }

        let count = recent.len() as f64;
        for value in aggregated.values.values_mut() {
            *value = (*value as f64 / count) as u64;
        }

        aggregated
    }

    /// 清理过期数据
    pub async fn cleanup_expired(&self) {
        let mut snapshots = self.snapshots.write().await;
        let now = chrono::Utc::now().timestamp_millis();
        let retention_ms = self.config.retention_days * 24 * 60 * 60 * 1000;

        snapshots.retain(|s| now - s.timestamp < retention_ms);
    }

    /// 获取计数器
    pub async fn get_counter(&self, name: &str) -> Option<AttributedCounter> {
        self.counters.read().await.get(name).cloned()
    }

    /// 增加计数器
    pub async fn increment(&self, name: &str, delta: u64) {
        if let Some(counter) = self.counters.write().await.get_mut(name) {
            counter.increment(delta);
        }
    }

    /// 带属性增加计数器
    pub async fn increment_with_attributes(
        &self,
        name: &str,
        delta: u64,
        attributes: HashMap<String, String>,
    ) {
        if let Some(counter) = self.counters.write().await.get_mut(name) {
            counter.increment_with_attributes(delta, attributes);
        }
    }
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

/// 指标类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MetricType {
    SessionCounter,
    LocCounter,
    PrCounter,
    CommitCounter,
    CostCounter,
    TokenCounter,
    CodeEditToolDecisionCounter,
    ActiveTimeCounter,
}

impl MetricType {
    pub fn description(&self) -> &'static str {
        match self {
            MetricType::SessionCounter => "会话计数，精确到秒级",
            MetricType::LocCounter => "代码行数，支持按语言分类统计",
            MetricType::PrCounter => "PR计数，关联代码变更量",
            MetricType::CommitCounter => "提交计数，包含提交频率分析",
            MetricType::CostCounter => "成本计数，精确到0.01单位",
            MetricType::TokenCounter => "Token计数，区分输入/输出token",
            MetricType::CodeEditToolDecisionCounter => "编辑决策，按操作类型分类",
            MetricType::ActiveTimeCounter => "活跃时间，精确到分钟级",
        }
    }
}

/// 带属性的计数器
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttributedCounter {
    metric_type: MetricType,
    value: u64,
    attributes: HashMap<String, String>,
    history: Vec<CounterEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CounterEntry {
    pub value: u64,
    pub timestamp: i64,
    pub attributes: HashMap<String, String>,
}

impl AttributedCounter {
    pub fn new(metric_type: MetricType) -> Self {
        Self {
            metric_type,
            value: 0,
            attributes: HashMap::new(),
            history: Vec::new(),
        }
    }

    pub fn value(&self) -> u64 {
        self.value
    }

    pub fn increment(&mut self, delta: u64) {
        self.value += delta;
        self.record_entry(delta, HashMap::new());
    }

    pub fn increment_with_attributes(
        &mut self,
        delta: u64,
        mut attributes: HashMap<String, String>,
    ) {
        self.value += delta;
        self.record_entry(delta, attributes);
    }

    fn record_entry(&mut self, delta: u64, attributes: HashMap<String, String>) {
        self.history.push(CounterEntry {
            value: delta,
            timestamp: chrono::Utc::now().timestamp_millis(),
            attributes,
        });

        if self.history.len() > 1000 {
            self.history.remove(0);
        }
    }

    pub fn metric_type(&self) -> &MetricType {
        &self.metric_type
    }

    pub fn attributes(&self) -> &HashMap<String, String> {
        &self.attributes
    }

    pub fn history(&self) -> &[CounterEntry] {
        &self.history
    }
}

/// 指标快照
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsSnapshot {
    pub timestamp: i64,
    pub values: HashMap<String, u64>,
}

impl MetricsSnapshot {
    pub fn new() -> Self {
        Self {
            timestamp: chrono::Utc::now().timestamp_millis(),
            values: HashMap::new(),
        }
    }

    pub fn get(&self, name: &str) -> Option<u64> {
        self.values.get(name).copied()
    }

    pub fn set(&mut self, name: String, value: u64) {
        self.values.insert(name, value);
    }
}

impl Default for MetricsSnapshot {
    fn default() -> Self {
        Self::new()
    }
}

/// 性能监控器
pub struct PerformanceMonitor {
    metrics: MetricsCollector,
    start_time: Instant,
}

impl PerformanceMonitor {
    pub fn new() -> Self {
        Self {
            metrics: MetricsCollector::new(),
            start_time: Instant::now(),
        }
    }

    /// 记录会话开始
    pub async fn record_session_start(&self) {
        self.metrics.increment("session_counter", 1).await;
    }

    /// 记录代码行数
    pub async fn record_loc(&self, lines: u64, language: &str) {
        let mut attrs = HashMap::new();
        attrs.insert("language".to_string(), language.to_string());
        self.metrics
            .increment_with_attributes("loc_counter", lines, attrs)
            .await;
    }

    /// 记录 PR
    pub async fn record_pr(&self, changes: u64) {
        let mut attrs = HashMap::new();
        attrs.insert("changes".to_string(), changes.to_string());
        self.metrics
            .increment_with_attributes("pr_counter", 1, attrs)
            .await;
    }

    /// 记录提交
    pub async fn record_commit(&self) {
        self.metrics.increment("commit_counter", 1).await;
    }

    /// 记录成本
    pub async fn record_cost(&self, cost: f64) {
        let cents = (cost * 100.0) as u64;
        self.metrics.increment("cost_counter", cents).await;
    }

    /// 记录 Token 使用
    pub async fn record_tokens(&self, input: u64, output: u64) {
        let mut attrs = HashMap::new();
        attrs.insert("type".to_string(), "input".to_string());
        self.metrics
            .increment_with_attributes("token_counter", input, attrs)
            .await;

        let mut attrs = HashMap::new();
        attrs.insert("type".to_string(), "output".to_string());
        self.metrics
            .increment_with_attributes("token_counter", output, attrs)
            .await;
    }

    /// 记录编辑决策
    pub async fn record_edit_decision(&self, operation: &str) {
        let mut attrs = HashMap::new();
        attrs.insert("operation".to_string(), operation.to_string());
        self.metrics
            .increment_with_attributes("code_edit_tool_decision_counter", 1, attrs)
            .await;
    }

    /// 记录活跃时间
    pub async fn record_active_time(&self, minutes: u64) {
        self.metrics.increment("active_time_counter", minutes).await;
    }

    /// 获取运行时间
    pub fn uptime(&self) -> Duration {
        self.start_time.elapsed()
    }

    /// 获取指标收集器
    pub fn metrics(&self) -> &MetricsCollector {
        &self.metrics
    }

    /// 生成报告
    pub async fn generate_report(&self) -> PerformanceReport {
        let counters = self.metrics.counters.read().await;

        let mut report = PerformanceReport {
            uptime_secs: self.uptime().as_secs(),
            session_count: 0,
            total_loc: 0,
            total_prs: 0,
            total_commits: 0,
            total_cost: 0.0,
            total_tokens_input: 0,
            total_tokens_output: 0,
            edit_decisions: HashMap::new(),
            active_time_minutes: 0,
        };

        for (name, counter) in counters.iter() {
            match name.as_str() {
                "session_counter" => report.session_count = counter.value(),
                "loc_counter" => report.total_loc = counter.value(),
                "pr_counter" => report.total_prs = counter.value(),
                "commit_counter" => report.total_commits = counter.value(),
                "cost_counter" => report.total_cost = counter.value() as f64 / 100.0,
                "token_counter" => {
                    if let Some(entry) = counter.history().last() {
                        if entry.attributes.get("type") == Some(&"input".to_string()) {
                            report.total_tokens_input += entry.value;
                        } else {
                            report.total_tokens_output += entry.value;
                        }
                    }
                }
                "active_time_counter" => report.active_time_minutes = counter.value(),
                _ => {}
            }
        }

        report
    }
}

impl Default for PerformanceMonitor {
    fn default() -> Self {
        Self::new()
    }
}

/// 性能报告
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceReport {
    pub uptime_secs: u64,
    pub session_count: u64,
    pub total_loc: u64,
    pub total_prs: u64,
    pub total_commits: u64,
    pub total_cost: f64,
    pub total_tokens_input: u64,
    pub total_tokens_output: u64,
    pub edit_decisions: HashMap<String, u64>,
    pub active_time_minutes: u64,
}

impl std::fmt::Display for PerformanceReport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "=== Performance Report ===")?;
        writeln!(f, "Uptime: {}s", self.uptime_secs)?;
        writeln!(f, "Sessions: {}", self.session_count)?;
        writeln!(f, "Lines of Code: {}", self.total_loc)?;
        writeln!(f, "PRs: {}", self.total_prs)?;
        writeln!(f, "Commits: {}", self.total_commits)?;
        writeln!(f, "Cost: ${:.2}", self.total_cost)?;
        writeln!(
            f,
            "Tokens: {} in / {} out",
            self.total_tokens_input, self.total_tokens_output
        )?;
        writeln!(f, "Active Time: {}m", self.active_time_minutes)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_metrics_collector() {
        let collector = MetricsCollector::new();
        collector.init_standard_metrics().await;

        collector.increment("session_counter", 1).await;
        collector.increment("loc_counter", 100).await;

        let session = collector.get_counter("session_counter").await;
        assert!(session.is_some());
        assert_eq!(session.unwrap().value(), 1);
    }

    #[tokio::test]
    async fn test_attributed_counter() {
        let mut counter = AttributedCounter::new(MetricType::LocCounter);

        counter.increment(100);
        assert_eq!(counter.value(), 100);

        let mut attrs = HashMap::new();
        attrs.insert("language".to_string(), "rust".to_string());
        counter.increment_with_attributes(50, attrs);

        assert_eq!(counter.value(), 150);
        assert_eq!(counter.history().len(), 2);
    }

    #[tokio::test]
    async fn test_metrics_snapshot() {
        let collector = MetricsCollector::new();
        collector.init_standard_metrics().await;

        collector.increment("session_counter", 5).await;
        collector.increment("loc_counter", 500).await;

        let snapshot = collector.sample().await;

        assert_eq!(snapshot.get("session_counter"), Some(5));
        assert_eq!(snapshot.get("loc_counter"), Some(500));
    }

    #[tokio::test]
    async fn test_performance_monitor() {
        let monitor = PerformanceMonitor::new();

        monitor.metrics.init_standard_metrics().await;
        monitor.record_session_start().await;
        monitor.record_loc(100, "rust").await;
        monitor.record_tokens(1000, 500).await;

        let report = monitor.generate_report().await;

        assert_eq!(report.session_count, 1);
        assert_eq!(report.total_loc, 100);
    }
}
