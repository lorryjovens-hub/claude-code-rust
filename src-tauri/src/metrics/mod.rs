use prometheus::{
    Registry, Counter, Histogram, IntGauge, IntCounter,
    histogram_opts, Encoder, TextEncoder,
};
use std::sync::LazyLock;

macro_rules! register_metric {
    ($metric:expr, $name:literal) => {{
        let m = $metric.expect(concat!("Failed to create metric: ", $name));
        REGISTRY.register(Box::new(m.clone()))
            .expect(concat!("Failed to register metric: ", $name, " — duplicate registration"));
        m
    }};
}

pub static REGISTRY: LazyLock<Registry> = LazyLock::new(|| Registry::new());

pub static HTTP_REQUESTS_TOTAL: LazyLock<Counter> = LazyLock::new(|| {
    register_metric!(Counter::new("http_requests_total", "Total HTTP requests"), "http_requests_total")
});

pub static HTTP_REQUEST_DURATION: LazyLock<Histogram> = LazyLock::new(|| {
    register_metric!(
        Histogram::with_opts(histogram_opts!("http_request_duration_seconds", "HTTP request duration",
            vec![0.01, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0])),
        "http_request_duration_seconds"
    )
});

pub static ACTIVE_SSE_CONNECTIONS: LazyLock<IntGauge> = LazyLock::new(|| {
    register_metric!(IntGauge::new("active_sse_connections", "Active SSE connections"), "active_sse_connections")
});

pub static TOOL_CALLS_TOTAL: LazyLock<Counter> = LazyLock::new(|| {
    register_metric!(Counter::new("tool_calls_total", "Total tool calls"), "tool_calls_total")
});

pub static TOOL_CALL_DURATION: LazyLock<Histogram> = LazyLock::new(|| {
    register_metric!(
        Histogram::with_opts(histogram_opts!("tool_call_duration_seconds", "Tool call duration",
            vec![0.1, 0.5, 1.0, 2.5, 5.0, 10.0, 30.0, 60.0])),
        "tool_call_duration_seconds"
    )
});

pub static TOKENS_CONSUMED: LazyLock<Counter> = LazyLock::new(|| {
    register_metric!(Counter::new("tokens_consumed_total", "Total tokens consumed"), "tokens_consumed_total")
});

pub static TOKENS_INPUT: LazyLock<IntCounter> = LazyLock::new(|| {
    register_metric!(IntCounter::new("tokens_input_total", "Total input tokens"), "tokens_input_total")
});

pub static TOKENS_OUTPUT: LazyLock<IntCounter> = LazyLock::new(|| {
    register_metric!(IntCounter::new("tokens_output_total", "Total output tokens"), "tokens_output_total")
});

pub static TOKENS_SAVED_COMPRESSION: LazyLock<IntCounter> = LazyLock::new(|| {
    register_metric!(IntCounter::new("tokens_saved_compression_total", "Tokens saved through compression"), "tokens_saved_compression_total")
});

pub static TOKENS_PROCESSED: LazyLock<IntCounter> = LazyLock::new(|| {
    register_metric!(IntCounter::new("tokens_processed_total", "Total tokens processed"), "tokens_processed_total")
});

pub static MEMORY_SEGMENTS: LazyLock<IntGauge> = LazyLock::new(|| {
    register_metric!(IntGauge::new("memory_segments_total", "Total memory segments"), "memory_segments_total")
});

pub static RLM_FEEDBACK_COUNT: LazyLock<IntCounter> = LazyLock::new(|| {
    register_metric!(IntCounter::new("rlm_feedback_total", "Total RLM feedback events"), "rlm_feedback_total")
});

pub fn gather_metrics() -> String {
    let encoder = TextEncoder::new();
    let metric_families = REGISTRY.gather();
    let mut buffer = Vec::new();
    encoder.encode(&metric_families, &mut buffer).expect("Failed to encode metrics");
    String::from_utf8(buffer).expect("Metrics output is not valid UTF-8")
}

/// Convenience function to record token usage
pub fn record_tokens(input: u64, output: u64) {
    TOKENS_INPUT.inc_by(input);
    TOKENS_OUTPUT.inc_by(output);
    TOKENS_CONSUMED.inc_by((input + output) as f64);
}

/// Convenience function to record tokens saved by compression
pub fn record_tokens_saved(saved: u64, processed: u64) {
    TOKENS_SAVED_COMPRESSION.inc_by(saved);
    TOKENS_PROCESSED.inc_by(processed);
}

/// Convenience function to update memory segments count
pub fn set_memory_segments(count: i64) {
    MEMORY_SEGMENTS.set(count);
}

/// Convenience function to record RLM feedback
pub fn record_rlm_feedback() {
    RLM_FEEDBACK_COUNT.inc();
}
