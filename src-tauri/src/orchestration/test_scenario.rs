use super::*;
use crate::native_engine::provider_manager::{Provider, ResolvedProvider, ModelConfig, ApiFormat};
use std::path::PathBuf;

fn create_mock_provider() -> ResolvedProvider {
    let provider = Provider {
        id: "test-anthropic".to_string(),
        name: "Anthropic".to_string(),
        base_url: "https://api.anthropic.com".to_string(),
        api_key: "test_key".to_string(),
        api_format: ApiFormat::Anthropic,
        models: vec![ModelConfig {
            id: "claude-3-sonnet".to_string(),
            name: "Claude 3 Sonnet".to_string(),
            enabled: true,
            max_tokens: Some(8192),
            supports_vision: false,
            supports_web_search: false,
        }],
        enabled: true,
        web_search_strategy: None,
    };
    
    ResolvedProvider {
        provider,
        model: ModelConfig {
            id: "claude-3-sonnet".to_string(),
            name: "Claude 3 Sonnet".to_string(),
            enabled: true,
            max_tokens: Some(8192),
            supports_vision: false,
            supports_web_search: false,
        },
    }
}

pub async fn run_complex_workflow_test() {
    eprintln!("\n\n=== MULTI-AGENT WORKFLOW TEST SCENARIO ===\n");
    eprintln!("Test Goal: Build a comprehensive e-commerce backend system with microservices architecture");
    eprintln!("Expected Flow: OpenSpace -> GStack -> Superpowers -> Task Execution\n");

    let data_dir = PathBuf::from("./test_data");
    let _ = std::fs::create_dir_all(&data_dir);

    let config = OrchestratorConfig {
        max_concurrent_agents: 8,
        enable_priority_scheduling: true,
        priority_adjust_interval_ms: 5000,
        aging_factor: 0.1,
        ..Default::default()
    };

    let orchestrator = MultiAgentOrchestrator::new(config, &data_dir);
    let mock_provider = create_mock_provider();

    let goal = r#"Build a comprehensive e-commerce backend system with the following requirements:
1. User authentication and authorization system
2. Product catalog management
3. Shopping cart functionality
4. Order processing system
5. Payment integration
6. Inventory management
7. Analytics dashboard
8. RESTful API with GraphQL support
9. Real-time notifications
10. Docker containerization for deployment"#;

    eprintln!("Step 1: Executing workflow with goal...");
    eprintln!("Goal length: {} characters", goal.len());
    eprintln!("=");

    let result = orchestrator.execute_workflow(goal, &mock_provider).await;

    match result {
        Ok(output) => {
            eprintln!("\n=== WORKFLOW EXECUTION COMPLETED ===");
            eprintln!("Plan ID: {}", output.get("plan_id").unwrap_or(&serde_json::Value::Null));
            eprintln!("Total Tasks: {}", output.get("total_tasks").unwrap_or(&serde_json::Value::Null));
            eprintln!("Completed Tasks: {}", output.get("completed_tasks").unwrap_or(&serde_json::Value::Null));
            
            if let Some(results) = output.get("results") {
                eprintln!("\nTask Results Summary:");
                eprintln!("-------------------");
                if let serde_json::Value::Object(map) = results {
                    for (task_id, result) in map {
                        let status = if result.get("error").is_some() { "FAILED" } else { "SUCCESS" };
                        eprintln!("  {}: {}", task_id, status);
                    }
                }
            }
        }
        Err(e) => {
            eprintln!("\n=== WORKFLOW EXECUTION FAILED ===");
            eprintln!("Error: {}", e);
        }
    }

    let stats = orchestrator.get_scheduling_stats().await;
    eprintln!("\n=== SCHEDULING STATISTICS ===");
    eprintln!("{}", serde_json::to_string_pretty(&stats).unwrap());

    eprintln!("\n=== TEST COMPLETE ===");
}

pub async fn run_priority_scheduling_test() {
    eprintln!("\n\n=== PRIORITY SCHEDULING TEST ===");
    
    let data_dir = PathBuf::from("./test_data");
    let _ = std::fs::create_dir_all(&data_dir);

    let config = OrchestratorConfig {
        max_concurrent_agents: 3,
        enable_priority_scheduling: true,
        priority_adjust_interval_ms: 2000,
        aging_factor: 0.5,
        ..Default::default()
    };

    let orchestrator = MultiAgentOrchestrator::new(config, &data_dir);
    let mock_provider = create_mock_provider();

    let goal = r#"Create a CI/CD pipeline for a web application with:
1. Automated testing suite
2. Code quality analysis
3. Security scanning
4. Container build and push
5. Deployment to staging
6. Deployment to production
7. Rollback procedures"#;

    eprintln!("Testing priority scheduling with limited concurrent agents (3)...");
    let _ = orchestrator.execute_workflow(goal, &mock_provider).await;

    eprintln!("=== PRIORITY SCHEDULING TEST COMPLETE ===");
}

pub async fn run_dependency_test() {
    eprintln!("\n\n=== DEPENDENCY RESOLUTION TEST ===");
    
    let data_dir = PathBuf::from("./test_data");
    let _ = std::fs::create_dir_all(&data_dir);

    let config = OrchestratorConfig {
        max_concurrent_agents: 5,
        enable_priority_scheduling: true,
        ..Default::default()
    };

    let orchestrator = MultiAgentOrchestrator::new(config, &data_dir);
    let mock_provider = create_mock_provider();

    let goal = r#"Build a data processing pipeline with:
1. Data ingestion from multiple sources
2. Data cleaning and validation
3. Data transformation
4. Data analysis and reporting
5. Data visualization dashboard
6. Alerting and notification system"#;

    eprintln!("Testing task dependency resolution...");
    let _ = orchestrator.execute_workflow(goal, &mock_provider).await;

    eprintln!("=== DEPENDENCY RESOLUTION TEST COMPLETE ===");
}
