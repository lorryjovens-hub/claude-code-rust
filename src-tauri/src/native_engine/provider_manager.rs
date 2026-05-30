use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use tokio::time::timeout;

const DEFAULT_REQUEST_TIMEOUT: Duration = Duration::from_secs(60);
const MAX_RETRIES: u32 = 3;
const RETRY_BASE_DELAY: Duration = Duration::from_millis(500);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Provider {
    pub id: String,
    pub name: String,
    pub base_url: String,
    pub api_key: String,
    pub api_format: ApiFormat,
    pub models: Vec<ModelConfig>,
    pub enabled: bool,
    pub web_search_strategy: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ApiFormat {
    #[serde(rename = "anthropic")]
    Anthropic,
    #[serde(rename = "openai")]
    OpenAI,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    pub id: String,
    pub name: String,
    pub enabled: bool,
    pub max_tokens: Option<u32>,
    pub supports_vision: bool,
    pub supports_web_search: bool,
}

#[derive(Debug, Clone)]
pub struct ResolvedProvider {
    pub provider: Provider,
    pub model: ModelConfig,
}

#[derive(Debug, Clone)]
struct ProviderCacheEntry {
    pub provider: Provider,
    pub model: ModelConfig,
    pub cached_at: Instant,
}

#[derive(Debug)]
pub struct ProviderManager {
    providers: Vec<Provider>,
    config_path: PathBuf,
    resolve_cache: Mutex<HashMap<String, ProviderCacheEntry>>,
    cache_ttl: Duration,
    request_timeout: Duration,
    max_retries: u32,
}

impl ProviderManager {
    pub fn new(config_path: PathBuf) -> Self {
        let mut manager = Self {
            providers: Vec::new(),
            config_path,
            resolve_cache: Mutex::new(HashMap::new()),
            cache_ttl: Duration::from_secs(300),
            request_timeout: DEFAULT_REQUEST_TIMEOUT,
            max_retries: MAX_RETRIES,
        };
        manager.load();
        manager
    }

    pub fn from_providers(providers: Vec<Provider>, config_path: PathBuf) -> Self {
        let manager = Self {
            providers,
            config_path,
            resolve_cache: Mutex::new(HashMap::new()),
            cache_ttl: Duration::from_secs(300),
            request_timeout: DEFAULT_REQUEST_TIMEOUT,
            max_retries: MAX_RETRIES,
        };
        if let Err(e) = manager.save() {
            tracing::error!(module = "ProviderManager", "Failed to save synced providers: {}", e);
        }
        manager
    }

    pub fn with_cache_ttl(mut self, ttl: Duration) -> Self {
        self.cache_ttl = ttl;
        self
    }

    pub fn with_request_timeout(mut self, timeout: Duration) -> Self {
        self.request_timeout = timeout;
        self
    }

    pub fn with_max_retries(mut self, retries: u32) -> Self {
        self.max_retries = retries;
        self
    }

    pub fn config_path(&self) -> &PathBuf {
        &self.config_path
    }

    pub fn get_request_timeout(&self) -> Duration {
        self.request_timeout
    }

    pub fn load(&mut self) {
        if self.config_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&self.config_path) {
                if let Ok(providers) = serde_json::from_str::<Vec<Provider>>(&content) {
                    self.providers = providers;
                    self.invalidate_cache();
                    tracing::info!(module = "ProviderManager", "Loaded {} providers from {}", self.providers.len(), self.config_path.display());
                }
            }
        }
    }

    pub fn save(&self) -> Result<(), anyhow::Error> {
        if let Some(parent) = self.config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(&self.providers)?;
        std::fs::write(&self.config_path, content)?;
        Ok(())
    }

    fn invalidate_cache(&self) {
        tracing::info!(module = "ProviderManager", "Resolve cache invalidated (sync)");
    }

    pub async fn invalidate_cache_async(&self) {
        let mut guard = self.resolve_cache.lock().await;
        guard.clear();
        tracing::info!(module = "ProviderManager", "Resolve cache invalidated");
    }

    pub async fn resolve_provider(&self, model_id: &str) -> Option<ResolvedProvider> {
        let now = Instant::now();

        {
            let cache = self.resolve_cache.lock().await;
            if let Some(entry) = cache.get(model_id) {
                if now.duration_since(entry.cached_at) < self.cache_ttl {
                    tracing::info!(
                        module = "ProviderManager",
                        "Cache hit for \"{}\" (age: {:.1}s)",
                        model_id,
                        now.duration_since(entry.cached_at).as_secs_f64()
                    );
                    return Some(ResolvedProvider {
                        provider: entry.provider.clone(),
                        model: entry.model.clone(),
                    });
                }
            }
        }

        tracing::info!(module = "ProviderManager", "Cache miss for \"{}\", resolving...", model_id);
        let result = self.resolve_provider_internal(model_id);

        if let Some(ref resolved) = result {
            let mut cache = self.resolve_cache.lock().await;
            cache.insert(
                model_id.to_string(),
                ProviderCacheEntry {
                    provider: resolved.provider.clone(),
                    model: resolved.model.clone(),
                    cached_at: Instant::now(),
                },
            );
        }

        result
    }

    fn resolve_provider_internal(&self, model_id: &str) -> Option<ResolvedProvider> {
        let mut first_match: Option<ResolvedProvider> = None;
        
        let aliases: std::collections::HashMap<&str, Vec<&str>> = [
            ("deepseek-v4-pro", vec!["deepseek-chat"]),
            ("deepseek-v4-flash", vec!["deepseek-reasoner"]),
        ].iter().cloned().collect();
        
        let mut ids_to_try = vec![model_id];
        if let Some(alias_list) = aliases.get(model_id) {
            ids_to_try.extend(alias_list.iter().copied());
        }
        for (canonical, alias_list) in &aliases {
            if alias_list.contains(&model_id) {
                ids_to_try.push(canonical);
            }
        }
        
        for try_id in &ids_to_try {
            for provider in &self.providers {
                if !provider.enabled {
                    continue;
                }
                
                for model in &provider.models {
                    if model.id == *try_id && model.enabled {
                        if first_match.is_none() {
                            first_match = Some(ResolvedProvider {
                                provider: provider.clone(),
                                model: model.clone(),
                            });
                        } else {
                            tracing::warn!(
                                module = "ProviderManager",
                                "Model \"{}\" exists in multiple providers. Using first match.",
                                model_id
                            );
                        }
                        break;
                    }
                }
                
                if first_match.is_some() {
                    break;
                }
            }
            if first_match.is_some() {
                break;
            }
        }

        if let Some(resolved) = &first_match {
            tracing::info!(
                module = "ProviderManager",
                "Resolved \"{}\" → \"{}\" ({})",
                model_id, resolved.provider.name, resolved.provider.base_url
            );
        } else {
            tracing::warn!(module = "ProviderManager", "No provider found for \"{}\"", model_id);
        }

        first_match
    }

    pub async fn execute_with_retry<F, Fut, T>(&self, operation: F) -> Result<T>
    where
        F: Fn() -> Fut,
        Fut: std::future::Future<Output = Result<T>>,
    {
        let mut last_error = None;
        
        for attempt in 0..self.max_retries {
            let result = timeout(self.request_timeout, operation()).await;
            
            match result {
                Ok(Ok(value)) => return Ok(value),
                Ok(Err(e)) => {
                    tracing::warn!(
                        module = "ProviderManager",
                        "Request failed (attempt {}/{}): {}",
                        attempt + 1,
                        self.max_retries,
                        e
                    );
                    last_error = Some(e);
                }
                Err(_) => {
                    tracing::warn!(
                        module = "ProviderManager",
                        "Request timeout after {}s (attempt {}/{})",
                        self.request_timeout.as_secs(),
                        attempt + 1,
                        self.max_retries,
                    );
                    last_error = Some(anyhow!("Request timed out"));
                }
            }

            if attempt < self.max_retries - 1 {
                let delay = RETRY_BASE_DELAY * 2u32.pow(attempt);
                tracing::info!(
                    module = "ProviderManager",
                    "Retrying in {:.1}s...",
                    delay.as_secs_f64()
                );
                tokio::time::sleep(delay).await;
            }
        }

        Err(last_error.unwrap_or_else(|| anyhow!("All retries exhausted")))
    }

    pub fn normalize_base_url(url: &str) -> String {
        let clean = url.trim_end_matches('/');
        let clean = clean
            .strip_suffix("/chat/completions")
            .or_else(|| clean.strip_suffix("/messages"))
            .or_else(|| clean.strip_suffix("/anthropic"))
            .or_else(|| clean.strip_suffix("/v1"))
            .unwrap_or(clean);
        clean.trim_end_matches('/').to_string()
    }

    pub fn list_providers(&self) -> &[Provider] {
        &self.providers
    }

    pub fn update_provider(&mut self, id: &str, provider: Provider) {
        if let Some(idx) = self.providers.iter().position(|p| p.id == id) {
            self.providers[idx] = provider;
            tracing::info!(module = "ProviderManager", "Updated provider: {}", id);
        } else {
            self.providers.push(provider);
            tracing::info!(module = "ProviderManager", "Added new provider: {}", id);
        }
        self.invalidate_cache();
        if let Err(e) = self.save() {
            tracing::error!(module = "ProviderManager", "Failed to save providers: {}", e);
        } else {
            tracing::info!(module = "ProviderManager", "Providers saved successfully to {}", self.config_path.display());
        }
    }

    pub fn delete_provider(&mut self, id: &str) {
        self.providers.retain(|p| p.id != id);
        tracing::info!(module = "ProviderManager", "Deleted provider: {}", id);
        self.invalidate_cache();
        if let Err(e) = self.save() {
            tracing::error!(module = "ProviderManager", "Failed to save providers after deletion: {}", e);
        } else {
            tracing::info!(module = "ProviderManager", "Providers saved successfully after deletion");
        }
    }

    pub async fn get_cache_stats(&self) -> HashMap<String, String> {
        let cache = self.resolve_cache.lock().await;
        let mut stats = HashMap::new();
        stats.insert("cache_size".to_string(), cache.len().to_string());
        stats.insert(
            "cache_ttl".to_string(),
            format!("{}s", self.cache_ttl.as_secs()),
        );
        stats
    }
}
