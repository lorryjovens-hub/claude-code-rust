use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::{HashMap, HashSet};

/// Represents a tool call with its dependencies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallNode {
    pub id: String,
    pub name: String,
    pub input: Value,
    #[serde(default)]
    pub depends_on: Vec<String>,
    #[serde(default)]
    pub description: Option<String>,
}

/// Represents the result of executing a tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallResult {
    pub id: String,
    pub name: String,
    pub output: Value,
    pub success: bool,
    pub error: Option<String>,
    pub duration_ms: u64,
}

/// Dependency graph for parallel execution scheduling
#[derive(Debug, Clone)]
struct DagLayer {
    nodes: Vec<ToolCallNode>,
}

/// LRU cache key
type CacheKey = u64;

/// Tool cache entry with TTL
#[derive(Debug, Clone)]
struct CacheEntry {
    result: ToolCallResult,
    created_at: std::time::Instant,
}

/// Parallel tool execution engine with DAG scheduling and result caching
pub struct ParallelExecutor {
    cache: HashMap<CacheKey, CacheEntry>,
    cache_ttl: std::time::Duration,
    max_cache_entries: usize,
    stats: ExecutorStats,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct ExecutorStats {
    pub total_executed: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub parallel_layers: u64,
    pub total_duration_ms: u64,
}

impl ParallelExecutor {
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
            cache_ttl: std::time::Duration::from_secs(60),
            max_cache_entries: 1000,
            stats: ExecutorStats::default(),
        }
    }

    pub fn with_cache_ttl(mut self, ttl_secs: u64) -> Self {
        self.cache_ttl = std::time::Duration::from_secs(ttl_secs);
        self
    }

    /// Build a DAG from tool calls, analyzing dependencies between them
    pub fn build_dag(&self, tools: &[ToolCallNode]) -> Vec<DagLayer> {
        let mut remaining: HashSet<&str> = tools.iter().map(|t| t.id.as_str()).collect();
        let mut executed: HashSet<String> = HashSet::new();
        let mut layers: Vec<DagLayer> = vec![];

        while !remaining.is_empty() {
            let mut current_layer: Vec<ToolCallNode> = vec![];

            for tool in tools {
                if !remaining.contains(tool.id.as_str()) {
                    continue;
                }

                let all_deps_ready = tool
                    .depends_on
                    .iter()
                    .all(|dep_id| executed.contains(dep_id));

                if all_deps_ready {
                    current_layer.push(tool.clone());
                }
            }

            if current_layer.is_empty() {
                let pending: Vec<_> = remaining.iter().map(|s| s.to_string()).collect();
                tracing::warn!(
                    module = "ParallelExecutor",
                    "Circular or unsatisfiable dependency detected! Remaining: {:?}",
                    pending
                );
                for tool in tools {
                    if remaining.contains(tool.id.as_str()) {
                        current_layer.push(tool.clone());
                    }
                }
            }

            for node in &current_layer {
                remaining.remove(node.id.as_str());
                executed.insert(node.id.clone());
            }

            layers.push(DagLayer {
                nodes: current_layer,
            });
        }

        layers
    }

    /// Execute tools in parallel layers based on DAG analysis
    pub async fn execute(
        &mut self,
        tools: &[ToolCallNode],
        executor_fn: impl Fn(&ToolCallNode) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<ToolCallResult>> + Send + '_>> + Send + Sync + Clone + 'static,
    ) -> Result<(Vec<ToolCallResult>, ExecutorStats)> {
        let start = std::time::Instant::now();

        if tools.is_empty() {
            return Ok((vec![], self.stats.clone()));
        }

        if tools.len() == 1 {
            let result = self.execute_single(&tools[0], executor_fn).await?;
            self.stats.total_executed += 1;
            self.stats.total_duration_ms += result.duration_ms;
            return Ok((vec![result], self.stats.clone()));
        }

        let layers = self.build_dag(tools);
        self.stats.parallel_layers += layers.len() as u64;

        let mut all_results: HashMap<String, ToolCallResult> = HashMap::new();

        for (layer_idx, layer) in layers.iter().enumerate() {
            tracing::info!(
                module = "ParallelExecutor",
                "Layer {}/{}: {} tools",
                layer_idx + 1,
                layers.len(),
                layer.nodes.len()
            );

            if layer.nodes.len() == 1 {
                let result = self.execute_single(&layer.nodes[0], executor_fn.clone()).await?;
                all_results.insert(result.id.clone(), result);
            } else {
                let mut handles = vec![];
                for node in &layer.nodes {
                    let node = node.clone();
                    let fn_clone = executor_fn.clone();
                    handles.push(tokio::spawn(async move {
                        fn_clone(&node).await
                    }));
                }

                for handle in handles {
                    match handle.await {
                        Ok(Ok(result)) => {
                            all_results.insert(result.id.clone(), result);
                        }
                        Ok(Err(e)) => {
                            tracing::error!(
                                module = "ParallelExecutor",
                                "Tool execution failed: {}",
                                e
                            );
                            let failed = ToolCallResult {
                                id: "unknown".into(),
                                name: "unknown".into(),
                                output: json!({"error": e.to_string()}),
                                success: false,
                                error: Some(e.to_string()),
                                duration_ms: 0,
                            };
                            all_results.insert("failed".into(), failed);
                        }
                        Err(join_err) => {
                            tracing::error!(
                                module = "ParallelExecutor",
                                "Task join failed: {}",
                                join_err
                            );
                        }
                    }
                }
            }
        }

        self.stats.total_executed += tools.len() as u64;
        self.stats.total_duration_ms += start.elapsed().as_millis() as u64;

        let results: Vec<ToolCallResult> = tools
            .iter()
            .filter_map(|t| all_results.get(&t.id).cloned())
            .collect();

        Ok((results, self.stats.clone()))
    }

    /// Execute a single tool with caching
    async fn execute_single<
        F: Fn(&ToolCallNode) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<ToolCallResult>> + Send + '_>>,
    >(
        &mut self,
        node: &ToolCallNode,
        executor_fn: F,
    ) -> Result<ToolCallResult> {
        let cache_key = Self::hash_input(node);

        if let Some(entry) = self.cache.get(&cache_key) {
            if entry.created_at.elapsed() < self.cache_ttl {
                self.stats.cache_hits += 1;
                tracing::debug!(
                    module = "ParallelExecutor",
                    "Cache hit for tool: {} (key: {})",
                    node.name,
                    cache_key
                );
                return Ok(entry.result.clone());
            }
        }

        self.stats.cache_misses += 1;
        let result = executor_fn(node).await?;

        if self.cache.len() >= self.max_cache_entries {
            self.evict_oldest();
        }

        self.cache.insert(
            cache_key,
            CacheEntry {
                result: result.clone(),
                created_at: std::time::Instant::now(),
            },
        );

        Ok(result)
    }

    fn hash_input(node: &ToolCallNode) -> CacheKey {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        node.name.hash(&mut hasher);
        node.input.to_string().hash(&mut hasher);
        hasher.finish()
    }

    fn evict_oldest(&mut self) {
        if let Some(oldest_key) = self
            .cache
            .iter()
            .min_by_key(|(_, entry)| entry.created_at)
            .map(|(k, _)| *k)
        {
            self.cache.remove(&oldest_key);
        }
    }

    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    pub fn stats(&self) -> &ExecutorStats {
        &self.stats
    }
}

/// Analyze tool dependencies from their inputs
/// Returns a map of tool_id -> list of dependent tool_ids
pub fn analyze_tool_dependencies(tools: &[ToolCallNode]) -> HashMap<String, Vec<String>> {
    let mut deps: HashMap<String, Vec<String>> = HashMap::new();

    for tool in tools {
        let mut tool_deps = vec![];
        let input_str = tool.input.to_string();

        for other in tools {
            if other.id == tool.id {
                continue;
            }
            // Check if this tool's input references another tool's output
            if input_str.contains(&format!("{{{{{}}}}}", other.id))
                || input_str.contains(&format!("tool_result_{}", other.id))
            {
                tool_deps.push(other.id.clone());
            }
        }

        deps.insert(tool.id.clone(), tool_deps);
    }

    deps
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dag_simple() {
        let tools = vec![
            ToolCallNode {
                id: "1".into(),
                name: "read_file".into(),
                input: json!({"path": "a.txt"}),
                depends_on: vec![],
                description: None,
            },
            ToolCallNode {
                id: "2".into(),
                name: "read_file".into(),
                input: json!({"path": "b.txt"}),
                depends_on: vec![],
                description: None,
            },
        ];

        let executor = ParallelExecutor::new();
        let layers = executor.build_dag(&tools);
        assert_eq!(layers.len(), 1, "Both tools should be in same layer");
        assert_eq!(layers[0].nodes.len(), 2);
    }

    #[test]
    fn test_dag_with_dependency() {
        let tools = vec![
            ToolCallNode {
                id: "1".into(),
                name: "read_file".into(),
                input: json!({"path": "a.txt"}),
                depends_on: vec![],
                description: None,
            },
            ToolCallNode {
                id: "2".into(),
                name: "edit_file".into(),
                input: json!({"path": "a.txt", "content": "new"}),
                depends_on: vec!["1".into()],
                description: None,
            },
            ToolCallNode {
                id: "3".into(),
                name: "search".into(),
                input: json!({"query": "test"}),
                depends_on: vec![],
                description: None,
            },
        ];

        let executor = ParallelExecutor::new();
        let layers = executor.build_dag(&tools);

        assert_eq!(layers.len(), 2, "Should have 2 layers");
        assert_eq!(layers[0].nodes.len(), 2, "Layer 1: read_file + search (parallel)");
        assert_eq!(layers[0].nodes.iter().map(|n| n.name.as_str()).collect::<HashSet<_>>(),
            vec!["read_file", "search"].into_iter().collect::<HashSet<_>>());
        assert_eq!(layers[1].nodes.len(), 1, "Layer 2: edit_file (depends on read_file)");
        assert_eq!(layers[1].nodes[0].name, "edit_file");
    }

    #[test]
    fn test_cache_hit() {
        let executor = ParallelExecutor::new();
        let node = ToolCallNode {
            id: "1".into(),
            name: "test".into(),
            input: json!({"x": 1}),
            depends_on: vec![],
            description: None,
        };
        let key = ParallelExecutor::hash_input(&node);

        let node2 = ToolCallNode {
            id: "2".into(),
            name: "test".into(),
            input: json!({"x": 1}),
            depends_on: vec![],
            description: None,
        };
        let key2 = ParallelExecutor::hash_input(&node2);

        assert_eq!(key, key2, "Same input should produce same cache key");
    }

    #[test]
    fn test_dependency_analysis() {
        let tools = vec![
            ToolCallNode {
                id: "tool_1".into(),
                name: "read".into(),
                input: json!({"content": "use {{tool_2}} result"}),
                depends_on: vec![],
                description: None,
            },
            ToolCallNode {
                id: "tool_2".into(),
                name: "write".into(),
                input: json!({"data": "hello"}),
                depends_on: vec![],
                description: None,
            },
        ];

        let deps = analyze_tool_dependencies(&tools);
        assert_eq!(deps["tool_1"].len(), 1);
        assert_eq!(deps["tool_1"][0], "tool_2");
    }
}