use claude_desktop_tauri_lib::native_engine::provider_manager::{
    ApiFormat, ModelConfig, Provider, ResolvedProvider,
};
use serde_json::{json, Value};

fn serialized_body(base_url: &str, model: &str, body: Value) -> Value {
    let resolved = ResolvedProvider {
        provider: Provider {
            id: "fixture".into(),
            name: "Fixture".into(),
            base_url: base_url.into(),
            api_key: String::new(),
            api_format: ApiFormat::OpenAI,
            models: vec![],
            enabled: true,
            web_search_strategy: None,
        },
        model: ModelConfig {
            id: model.into(),
            name: model.into(),
            enabled: true,
            max_tokens: None,
            supports_vision: true,
            supports_web_search: false,
        },
    };
    // Exercise the same reqwest JSON serialization used by both native clients.
    // Building a request does not send it or require credentials.
    let request = reqwest::Client::new()
        .post(base_url)
        .json(&resolved.prepare_request(body))
        .build()
        .unwrap();
    serde_json::from_slice(request.body().unwrap().as_bytes().unwrap()).unwrap()
}

#[test]
fn direct_flash_serialization_keeps_history_and_bounds_output() {
    for host in ["api.minimax.io", "api.minimaxi.com"] {
        for path in ["v1/chat/completions", "anthropic/v1/messages"] {
            for stream in [false, true] {
                let body = json!({
                    "model": "MiniMax-M3.1-Flash",
                    "max_tokens": 1_000_000,
                    "stream": stream,
                    "thinking": { "type": "enabled", "budget_tokens": 10000 },
                    "messages": [{
                        "role": "assistant", "content": "Inspecting",
                        "reasoning_content": "Keep this reasoning",
                        "tool_calls": [{"id": "call-1", "type": "function", "function": {"name": "inspect", "arguments": "{}"}}]
                    }],
                    "tools": [{"name": "inspect", "input_schema": {"type": "object"}}]
                });
                let actual = serialized_body(
                    &format!("https://{host}/{path}"),
                    "MiniMax-M3.1-Flash",
                    body.clone(),
                );
                assert_eq!(actual["model"], body["model"]);
                assert_eq!(actual["max_tokens"], 524_288);
                assert_eq!(actual["thinking"], json!({"type": "adaptive"}));
                assert_eq!(actual["messages"], body["messages"]);
                assert_eq!(actual["tools"], body["tools"]);
                assert_eq!(actual["stream"], body["stream"]);
                assert!(actual.get("service_tier").is_none());
            }
        }
    }
}

#[test]
fn explicit_models_and_gateway_payloads_are_preserved() {
    let body = json!({"max_tokens": 4096, "messages": [], "thinking": {"type": "disabled"}});
    for model in ["MiniMax-M3", "MiniMax-M2.7", "custom"] {
        assert_eq!(
            serialized_body("https://api.minimax.io/v1", model, body.clone()),
            body
        );
    }
    assert_eq!(
        serialized_body(
            "https://gateway.example/v1",
            "MiniMax-M3.1-Flash",
            body.clone()
        ),
        body
    );
    let bounded = serialized_body(
        "https://api.minimax.io/v1",
        "MiniMax-M3.1-Flash",
        json!({"max_tokens": 4096}),
    );
    assert_eq!(bounded["max_tokens"], 4096);
}
