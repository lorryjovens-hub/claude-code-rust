use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenUsage {
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub total_tokens: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelCost {
    pub model: String,
    pub input_cost_per_1k: f64,
    pub output_cost_per_1k: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionCost {
    pub session_id: String,
    pub conversation_id: String,
    pub model: String,
    pub total_input_tokens: u64,
    pub total_output_tokens: u64,
    pub estimated_cost: f64,
    pub started_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostSummary {
    pub total_input_tokens: u64,
    pub total_output_tokens: u64,
    pub total_tokens: u64,
    pub estimated_total_cost: f64,
    pub sessions: Vec<SessionCost>,
    pub model_breakdown: HashMap<String, TokenUsage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BudgetCheckResult {
    WithinBudget,
    Warning(String, u64, u64),
    Exceeded(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyUsageRecord {
    pub date: String,
    pub total_tokens: u64,
    pub total_cost: f64,
    pub model_breakdown: HashMap<String, u64>,
}

pub struct CostTracker {
    store_dir: PathBuf,
    sessions: Arc<Mutex<HashMap<String, SessionCost>>>,
    model_costs: HashMap<String, ModelCost>,
    daily_budget: Arc<RwLock<Option<u64>>>,
    monthly_budget: Arc<RwLock<Option<u64>>>,
    daily_usage: Arc<RwLock<u64>>,
    monthly_usage: Arc<RwLock<u64>>,
    daily_records: Arc<Mutex<Vec<DailyUsageRecord>>>,
}

impl CostTracker {
    pub fn new(store_dir: PathBuf) -> Self {
        fs::create_dir_all(&store_dir).ok();
        let mut tracker = Self {
            store_dir,
            sessions: Arc::new(Mutex::new(HashMap::new())),
            model_costs: HashMap::new(),
            daily_budget: Arc::new(RwLock::new(None)),
            monthly_budget: Arc::new(RwLock::new(None)),
            daily_usage: Arc::new(RwLock::new(0)),
            monthly_usage: Arc::new(RwLock::new(0)),
            daily_records: Arc::new(Mutex::new(Vec::new())),
        };
        tracker.init_model_costs();
        tracker.load_budget_config();
        tracker
    }

    fn init_model_costs(&mut self) {
        let costs = vec![
            ModelCost {
                model: "claude-opus-4-5".to_string(),
                input_cost_per_1k: 0.015,
                output_cost_per_1k: 0.075,
            },
            ModelCost {
                model: "claude-sonnet-4-6".to_string(),
                input_cost_per_1k: 0.003,
                output_cost_per_1k: 0.015,
            },
            ModelCost {
                model: "claude-haiku-3-5".to_string(),
                input_cost_per_1k: 0.0008,
                output_cost_per_1k: 0.004,
            },
            ModelCost {
                model: "claude-sonnet-4-6".to_string(),
                input_cost_per_1k: 0.003,
                output_cost_per_1k: 0.015,
            },
            ModelCost {
                model: "claude-haiku-4-20250514".to_string(),
                input_cost_per_1k: 0.0008,
                output_cost_per_1k: 0.004,
            },
            ModelCost {
                model: "claude-opus-4-20250514".to_string(),
                input_cost_per_1k: 0.015,
                output_cost_per_1k: 0.075,
            },
            ModelCost {
                model: "gpt-4o".to_string(),
                input_cost_per_1k: 0.005,
                output_cost_per_1k: 0.015,
            },
            ModelCost {
                model: "gpt-4o-mini".to_string(),
                input_cost_per_1k: 0.00015,
                output_cost_per_1k: 0.0006,
            },
        ];

        for cost in costs {
            self.model_costs.insert(cost.model.clone(), cost);
        }
    }

    fn load_budget_config(&self) {
        let config_path = self.store_dir.join("budget_config.json");
        if let Ok(data) = fs::read_to_string(&config_path) {
            if let Ok(config) = serde_json::from_str::<serde_json::Value>(&data) {
                if let Some(daily) = config.get("daily_budget").and_then(|v| v.as_u64()) {
                    let db = self.daily_budget.clone();
                    tokio::spawn(async move {
                        let mut b = db.write().await;
                        *b = Some(daily);
                    });
                }
                if let Some(monthly) = config.get("monthly_budget").and_then(|v| v.as_u64()) {
                    let mb = self.monthly_budget.clone();
                    tokio::spawn(async move {
                        let mut b = mb.write().await;
                        *b = Some(monthly);
                    });
                }
            }
        }
    }

    fn save_budget_config(&self) {
        let config_path = self.store_dir.join("budget_config.json");
        let daily = self.daily_budget.blocking_read();
        let monthly = self.monthly_budget.blocking_read();
        let config = serde_json::json!({
            "daily_budget": *daily,
            "monthly_budget": *monthly,
        });
        let _ = fs::write(&config_path, serde_json::to_string_pretty(&config).unwrap_or_default());
    }

    pub fn check_budget(&self, additional_tokens: u64) -> BudgetCheckResult {
        let daily = self.daily_usage.blocking_read();
        let monthly = self.monthly_usage.blocking_read();
        let daily_budget = self.daily_budget.blocking_read();
        let monthly_budget = self.monthly_budget.blocking_read();

        let new_daily = *daily + additional_tokens;
        let new_monthly = *monthly + additional_tokens;

        if let Some(limit) = *monthly_budget {
            if new_monthly >= limit {
                return BudgetCheckResult::Exceeded("Monthly budget exceeded".to_string());
            }
            if new_monthly >= limit * 80 / 100 {
                return BudgetCheckResult::Warning("Monthly budget at 80%".to_string(), new_monthly, limit);
            }
        }

        if let Some(limit) = *daily_budget {
            if new_daily >= limit {
                return BudgetCheckResult::Exceeded("Daily budget exceeded".to_string());
            }
            if new_daily >= limit * 80 / 100 {
                return BudgetCheckResult::Warning("Daily budget at 80%".to_string(), new_daily, limit);
            }
        }

        BudgetCheckResult::WithinBudget
    }

    pub async fn record_usage(&self, tokens: u64) {
        {
            let mut daily = self.daily_usage.write().await;
            *daily += tokens;
        }
        {
            let mut monthly = self.monthly_usage.write().await;
            *monthly += tokens;
        }
        self.update_daily_record(tokens).await;
    }

    async fn update_daily_record(&self, tokens: u64) {
        let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
        let mut records = self.daily_records.lock().await;
        if let Some(record) = records.iter_mut().find(|r| r.date == today) {
            record.total_tokens += tokens;
        } else {
            records.push(DailyUsageRecord {
                date: today,
                total_tokens: tokens,
                total_cost: 0.0,
                model_breakdown: HashMap::new(),
            });
        }
        if records.len() > 90 {
            let keep = records.len() - 90;
            records.drain(0..keep);
        }
    }

    pub async fn set_daily_budget(&self, budget: Option<u64>) {
        {
            let mut b = self.daily_budget.write().await;
            *b = budget;
        }
        self.save_budget_config();
    }

    pub async fn set_monthly_budget(&self, budget: Option<u64>) {
        {
            let mut b = self.monthly_budget.write().await;
            *b = budget;
        }
        self.save_budget_config();
    }

    pub async fn get_usage_stats(&self) -> serde_json::Value {
        let daily = *self.daily_usage.read().await;
        let monthly = *self.monthly_usage.read().await;
        let daily_budget = *self.daily_budget.read().await;
        let monthly_budget = *self.monthly_budget.read().await;

        serde_json::json!({
            "daily_usage": daily,
            "monthly_usage": monthly,
            "daily_budget": daily_budget,
            "monthly_budget": monthly_budget,
            "daily_percent": daily_budget.map(|b| (daily as f64 / b as f64 * 100.0) as u32),
            "monthly_percent": monthly_budget.map(|b| (monthly as f64 / b as f64 * 100.0) as u32),
        })
    }

    pub async fn get_daily_records(&self, days: usize) -> Vec<DailyUsageRecord> {
        let records = self.daily_records.lock().await;
        records.iter().rev().take(days).cloned().collect()
    }

    pub fn calculate_cost(&self, model: &str, input_tokens: u64, output_tokens: u64) -> f64 {
        let model_cost = self.model_costs.get(model);
        if let Some(mc) = model_cost {
            let input_cost = (input_tokens as f64 / 1000.0) * mc.input_cost_per_1k;
            let output_cost = (output_tokens as f64 / 1000.0) * mc.output_cost_per_1k;
            input_cost + output_cost
        } else {
            0.0
        }
    }

    pub async fn create_session(&self, conversation_id: &str, model: &str) -> String {
        let session_id = uuid::Uuid::new_v4().to_string()[..12].to_string();
        let now = chrono::Utc::now().to_rfc3339();

        let session = SessionCost {
            session_id: session_id.clone(),
            conversation_id: conversation_id.to_string(),
            model: model.to_string(),
            total_input_tokens: 0,
            total_output_tokens: 0,
            estimated_cost: 0.0,
            started_at: now.clone(),
            updated_at: now,
        };

        let mut sessions = self.sessions.lock().await;
        sessions.insert(session_id.clone(), session);
        session_id
    }

    pub async fn add_usage(&self, session_id: &str, input_tokens: u64, output_tokens: u64) -> Result<f64> {
        let mut sessions = self.sessions.lock().await;
        if let Some(session) = sessions.get_mut(session_id) {
            session.total_input_tokens += input_tokens;
            session.total_output_tokens += output_tokens;
            session.estimated_cost = self.calculate_cost(
                &session.model,
                session.total_input_tokens,
                session.total_output_tokens,
            );
            session.updated_at = chrono::Utc::now().to_rfc3339();

            let total_tokens = input_tokens + output_tokens;
            drop(sessions);
            self.record_usage(total_tokens).await;

            let sessions = self.sessions.lock().await;
            if let Some(session) = sessions.get(session_id) {
                self.save_session(session)?;
                Ok(session.estimated_cost)
            } else {
                anyhow::bail!("Session not found after update: {}", session_id)
            }
        } else {
            anyhow::bail!("Session not found: {}", session_id)
        }
    }

    pub async fn get_session_cost(&self, session_id: &str) -> Option<SessionCost> {
        let sessions = self.sessions.lock().await;
        sessions.get(session_id).cloned()
    }

    pub async fn get_conversation_cost(&self, conversation_id: &str) -> CostSummary {
        let sessions = self.sessions.lock().await;
        let mut total_input = 0u64;
        let mut total_output = 0u64;
        let mut total_cost = 0.0;
        let mut model_breakdown: HashMap<String, TokenUsage> = HashMap::new();
        let mut session_list = Vec::new();

        for session in sessions.values() {
            if session.conversation_id == conversation_id {
                total_input += session.total_input_tokens;
                total_output += session.total_output_tokens;
                total_cost += session.estimated_cost;
                session_list.push(session.clone());

                let entry = model_breakdown.entry(session.model.clone()).or_insert(TokenUsage {
                    input_tokens: 0,
                    output_tokens: 0,
                    total_tokens: 0,
                });
                entry.input_tokens += session.total_input_tokens;
                entry.output_tokens += session.total_output_tokens;
                entry.total_tokens = entry.input_tokens + entry.output_tokens;
            }
        }

        CostSummary {
            total_input_tokens: total_input,
            total_output_tokens: total_output,
            total_tokens: total_input + total_output,
            estimated_total_cost: total_cost,
            sessions: session_list,
            model_breakdown,
        }
    }

    pub async fn get_all_sessions(&self) -> Vec<SessionCost> {
        let sessions = self.sessions.lock().await;
        sessions.values().cloned().collect()
    }

    fn save_session(&self, session: &SessionCost) -> Result<()> {
        let path = self.store_dir.join(format!("{}.json", session.session_id));
        let data = serde_json::to_string_pretty(session)?;
        fs::write(path, data)?;
        Ok(())
    }

    pub fn load_saved_sessions(&mut self) {
        if !self.store_dir.exists() {
            return;
        }

        if let Ok(entries) = fs::read_dir(&self.store_dir) {
            for entry in entries.flatten() {
                if entry.path().extension().and_then(|e| e.to_str()) == Some("json") {
                    if let Ok(data) = fs::read_to_string(entry.path()) {
                        if let Ok(session) = serde_json::from_str::<SessionCost>(&data) {
                            let mut sessions = self.sessions.blocking_lock();
                            sessions.insert(session.session_id.clone(), session);
                        }
                    }
                }
            }
        }
    }
}
