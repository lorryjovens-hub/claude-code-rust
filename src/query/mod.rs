//! 查询引擎模块
//!
//! 根据 Claude Code 源码深度分析文档实现完整的查询管道系统。
//! 核心功能：系统提示词组装、消息处理管道、API调用、工具执行循环、错误恢复。

pub mod engine;
pub mod context;
pub mod message;
pub mod result;
pub mod pipeline;
pub mod compressor;
pub mod retry;

// 重新导出主要类型
pub use engine::{QueryEngine, QueryEngineBuilder, QueryEngineConfig};
pub use context::QueryContext;
pub use message::{Message, MessageRole, MessageContent, ToolCall, ToolResult};
pub use result::{QueryResult, QueryError, QueryStatus};
pub use pipeline::{QueryPipeline, PipelineStage};
pub use compressor::{ContextCompressor, CompressorConfig};
pub use retry::{RetryPolicy, RetryStrategy, RetryError, exponential_backoff};

/// 查询系统初始化
pub async fn init() -> anyhow::Result<QueryEngine> {
    QueryEngine::default().await
}