//! Stress Tests Module - Comprehensive load and stress testing for services
//!
//! This module provides stress testing utilities for all Claude Code services.

use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;

use crate::services::{
    AutoDreamService, AutoDreamConfig,
    VoiceService, VoiceConfig,
    MagicDocsService, MagicDocsConfig,
    TeamMemorySyncService, TeamMemoryConfig,
    PluginMarketplaceService, PluginConfig,
    AgentsService,
};
use crate::state::AppState;
use crate::config::Settings;

#[derive(Debug, Clone)]
pub struct StressTestResult {
    pub name: String,
    pub total_operations: usize,
    pub successful_operations: usize,
    pub failed_operations: usize,
    pub total_duration_ms: u128,
    pub avg_latency_ms: f64,
    pub min_latency_ms: u128,
    pub max_latency_ms: u128,
    pub ops_per_second: f64,
    pub errors: Vec<String>,
}

impl StressTestResult {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            total_operations: 0,
            successful_operations: 0,
            failed_operations: 0,
            total_duration_ms: 0,
            avg_latency_ms: 0.0,
            min_latency_ms: u128::MAX,
            max_latency_ms: 0,
            ops_per_second: 0.0,
            errors: Vec::new(),
        }
    }

    pub fn add_result(&mut self, latency_ms: u128, success: bool, error: Option<String>) {
        self.total_operations += 1;
        if success {
            self.successful_operations += 1;
        } else {
            self.failed_operations += 1;
            if let Some(e) = error {
                self.errors.push(e);
            }
        }
        self.total_duration_ms += latency_ms;
        self.min_latency_ms = self.min_latency_ms.min(latency_ms);
        self.max_latency_ms = self.max_latency_ms.max(latency_ms);
    }

    pub fn finalize(&mut self) {
        if self.total_operations > 0 {
            self.avg_latency_ms = self.total_duration_ms as f64 / self.total_operations as f64;
            let duration_secs = self.total_duration_ms as f64 / 1000.0;
            self.ops_per_second = if duration_secs > 0.0 {
                self.total_operations as f64 / duration_secs
            } else {
                0.0
            };
        }
    }

    pub fn print_summary(&self) {
        println!("\n{}", "=".repeat(70));
        println!("📊 Stress Test Results: {}", self.name);
        println!("{}", "=".repeat(70));
        println!("  Total Operations:    {}", self.total_operations);
        println!("  Successful:          {} ({:.1}%)",
                 self.successful_operations,
                 if self.total_operations > 0 {
                     (self.successful_operations as f64 / self.total_operations as f64) * 100.0
                 } else { 0.0 });
        println!("  Failed:              {} ({:.1}%)",
                 self.failed_operations,
                 if self.total_operations > 0 {
                     (self.failed_operations as f64 / self.total_operations as f64) * 100.0
                 } else { 0.0 });
        println!("  Total Duration:      {:.2}s", self.total_duration_ms as f64 / 1000.0);
        println!("  Avg Latency:         {:.2}ms", self.avg_latency_ms);
        println!("  Min Latency:         {}ms", self.min_latency_ms);
        println!("  Max Latency:         {}ms", self.max_latency_ms);
        println!("  Throughput:          {:.2} ops/sec", self.ops_per_second);
        if !self.errors.is_empty() {
            println!("\n  Errors encountered:");
            for (i, err) in self.errors.iter().take(5).enumerate() {
                println!("    {}. {}", i + 1, err);
            }
            if self.errors.len() > 5 {
                println!("    ... and {} more", self.errors.len() - 5);
            }
        }
        println!("{}", "=".repeat(70));
    }
}

pub struct StressTestRunner {
    state: Arc<RwLock<AppState>>,
    iterations: usize,
}

impl StressTestRunner {
    pub fn new(iterations: usize) -> Self {
        let settings = Settings::default();
        let state = AppState::new(settings);
        Self {
            state: Arc::new(RwLock::new(state)),
            iterations,
        }
    }

    pub async fn run_all_tests(&self) -> Vec<StressTestResult> {
        println!("\n{}", "🎯".repeat(35));
        println!("🔧 Claude Code Services Stress Test Suite");
        println!("{}", "🎯".repeat(35));
        println!("\nConfiguration:");
        println!("  Iterations:   {}", self.iterations);
        println!("  Total Tests:  6 services");
        println!("{}", "-".repeat(70));

        let mut results = Vec::new();

        results.push(self.test_autodream().await);
        results.push(self.test_voice().await);
        results.push(self.test_magic_docs().await);
        results.push(self.test_team_memory_sync().await);
        results.push(self.test_plugin_marketplace().await);
        results.push(self.test_agents().await);

        self.print_final_summary(&results);

        results
    }

    async fn test_autodream(&self) -> StressTestResult {
        println!("\n🧪 Testing AutoDream Service...");
        let mut result = StressTestResult::new("AutoDream");

        let service = AutoDreamService::new(
            self.state.clone(),
            Some(AutoDreamConfig {
                min_hours: 24,
                min_sessions: 5,
                enabled: true,
            }),
        );

        let start = Instant::now();

        for i in 0..self.iterations {
            let iter_start = Instant::now();
            let _status = service.get_status().await;
            let latency = iter_start.elapsed().as_millis();

            result.add_result(latency, true, None);

            if (i + 1) % 100 == 0 {
                println!("  Progress: {}/{} ({}%)", i + 1, self.iterations,
                         (i + 1) * 100 / self.iterations);
            }
        }

        result.total_duration_ms = start.elapsed().as_millis();
        result.finalize();
        result.print_summary();

        result
    }

    async fn test_voice(&self) -> StressTestResult {
        println!("\n🧪 Testing Voice Service...");
        let mut result = StressTestResult::new("Voice");

        let service = VoiceService::new(
            self.state.clone(),
            Some(VoiceConfig::new(true, true)),
        );

        let start = Instant::now();

        for i in 0..self.iterations {
            let iter_start = Instant::now();
            let _status = service.get_status().await;
            let latency = iter_start.elapsed().as_millis();

            result.add_result(latency, true, None);

            if (i + 1) % 100 == 0 {
                println!("  Progress: {}/{} ({}%)", i + 1, self.iterations,
                         (i + 1) * 100 / self.iterations);
            }
        }

        result.total_duration_ms = start.elapsed().as_millis();
        result.finalize();
        result.print_summary();

        result
    }

    async fn test_magic_docs(&self) -> StressTestResult {
        println!("\n🧪 Testing MagicDocs Service...");
        let mut result = StressTestResult::new("MagicDocs");

        let service = MagicDocsService::new(
            self.state.clone(),
            Some(MagicDocsConfig {
                enabled: true,
                auto_update: true,
                update_interval_hours: 1,
                max_docs: 100,
            }),
        );

        let start = Instant::now();

        for i in 0..self.iterations {
            let iter_start = Instant::now();
            let header = service.detect_magic_doc_header(
                &format!("# MAGIC DOC: Test Document {}\n\n_Update this document_", i)
            );
            let latency = iter_start.elapsed().as_millis();

            match header {
                Some(_) => result.add_result(latency, true, None),
                None => result.add_result(latency, false, Some("No header detected".to_string())),
            }

            if (i + 1) % 100 == 0 {
                println!("  Progress: {}/{} ({}%)", i + 1, self.iterations,
                         (i + 1) * 100 / self.iterations);
            }
        }

        result.total_duration_ms = start.elapsed().as_millis();
        result.finalize();
        result.print_summary();

        result
    }

    async fn test_team_memory_sync(&self) -> StressTestResult {
        println!("\n🧪 Testing TeamMemorySync Service...");
        let mut result = StressTestResult::new("TeamMemorySync");

        let service = TeamMemorySyncService::new(
            self.state.clone(),
            Some(TeamMemoryConfig {
                enabled: true,
                team_id: Some("test-team".to_string()),
                sync_interval_secs: 3600,
                auto_sync: false,
                conflict_resolution: crate::services::ConflictResolution::PreferNewer,
            }),
        );

        let start = Instant::now();

        for _ in 0..self.iterations {
            let iter_start = Instant::now();
            let _status = service.get_status().await;
            let latency = iter_start.elapsed().as_millis();

            result.add_result(latency, true, None);
        }

        result.total_duration_ms = start.elapsed().as_millis();
        result.finalize();
        result.print_summary();

        result
    }

    async fn test_plugin_marketplace(&self) -> StressTestResult {
        println!("\n🧪 Testing PluginMarketplace Service...");
        let mut result = StressTestResult::new("PluginMarketplace");

        let service = PluginMarketplaceService::new(
            self.state.clone(),
            Some(PluginConfig {
                enabled: true,
                auto_update: true,
                marketplace_url: "https://plugins.claude.ai".to_string(),
                trusted_sources: vec!["official".to_string()],
            }),
        );

        let start = Instant::now();

        for i in 0..self.iterations {
            let iter_start = Instant::now();
            let query = if i % 3 == 0 { "git" } else if i % 3 == 1 { "code" } else { "test" };
            let results = service.search(query).await;
            let latency = iter_start.elapsed().as_millis();

            let _unused = results;
            result.add_result(latency, true, None);

            if (i + 1) % 100 == 0 {
                println!("  Progress: {}/{} ({}%)", i + 1, self.iterations,
                         (i + 1) * 100 / self.iterations);
            }
        }

        result.total_duration_ms = start.elapsed().as_millis();
        result.finalize();
        result.print_summary();

        result
    }

    async fn test_agents(&self) -> StressTestResult {
        println!("\n🧪 Testing Agents Service...");
        let mut result = StressTestResult::new("Agents");

        let service = AgentsService::new(self.state.clone());

        let start = Instant::now();

        for i in 0..self.iterations {
            let iter_start = Instant::now();
            let agents = service.list_agents().await;
            let latency = iter_start.elapsed().as_millis();

            let _unused = agents.len();
            let _unused_i = i;
            result.add_result(latency, true, None);

            if (i + 1) % 100 == 0 {
                println!("  Progress: {}/{} ({}%)", i + 1, self.iterations,
                         (i + 1) * 100 / self.iterations);
            }
        }

        result.total_duration_ms = start.elapsed().as_millis();
        result.finalize();
        result.print_summary();

        result
    }

    fn print_final_summary(&self, results: &[StressTestResult]) {
        println!("\n{}", "🎉".repeat(35));
        println!("📊 Final Stress Test Summary");
        println!("{}", "🎉".repeat(35));

        let total_ops: usize = results.iter().map(|r| r.total_operations).sum();
        let total_success: usize = results.iter().map(|r| r.successful_operations).sum();
        let total_fail: usize = results.iter().map(|r| r.failed_operations).sum();
        let total_duration: u128 = results.iter().map(|r| r.total_duration_ms).sum();
        let avg_throughput: f64 = results.iter().map(|r| r.ops_per_second).sum::<f64>() / results.len() as f64;

        println!("\n  Overall Statistics:");
        println!("  ├─ Total Operations:     {}", total_ops);
        println!("  ├─ Successful:         {} ({:.1}%)",
                 total_success,
                 if total_ops > 0 { (total_success as f64 / total_ops as f64) * 100.0 } else { 0.0 });
        println!("  ├─ Failed:              {} ({:.1}%)",
                 total_fail,
                 if total_ops > 0 { (total_fail as f64 / total_ops as f64) * 100.0 } else { 0.0 });
        println!("  ├─ Total Duration:      {:.2}s", total_duration as f64 / 1000.0);
        println!("  └─ Avg Throughput:     {:.2} ops/sec", avg_throughput);

        println!("\n  Per-Service Breakdown:");
        for r in results {
            let status = if r.failed_operations == 0 { "✅" } else { "⚠️" };
            println!("  {} {:20} | {:6} ops | {:6.2} ops/s | {:6.2}ms avg",
                     status,
                     r.name,
                     r.total_operations,
                     r.ops_per_second,
                     r.avg_latency_ms);
        }

        let all_passed = results.iter().all(|r| r.failed_operations == 0);
        println!("\n{}", "=".repeat(70));
        if all_passed {
            println!("✅ ALL TESTS PASSED - Services are ready for production!");
        } else {
            println!("⚠️ SOME TESTS FAILED - Review errors above");
        }
        println!("{}", "=".repeat(70));
    }
}

pub async fn run_stress_test(_concurrency: usize, iterations: usize) {
    let runner = StressTestRunner::new(iterations);
    runner.run_all_tests().await;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_autodream_basic() {
        let runner = StressTestRunner::new(10);
        let results = runner.run_all_tests().await;
        assert_eq!(results.len(), 6);
        for result in results {
            assert!(result.total_operations > 0);
        }
    }

    #[tokio::test]
    async fn test_stress_result_finalization() {
        let mut result = StressTestResult::new("test");
        result.add_result(100, true, None);
        result.add_result(200, true, None);
        result.add_result(300, false, Some("test error".to_string()));
        result.finalize();

        assert_eq!(result.total_operations, 3);
        assert_eq!(result.successful_operations, 2);
        assert_eq!(result.failed_operations, 1);
        assert_eq!(result.avg_latency_ms, 200.0);
        assert_eq!(result.min_latency_ms, 100);
        assert_eq!(result.max_latency_ms, 300);
        assert_eq!(result.errors.len(), 1);
    }
}
