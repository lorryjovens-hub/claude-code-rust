//! 性能优化模块
//! 
//! 这个模块实现了完整的性能优化系统，//! 
//! - 启动优化：快速路径、懒加载、预连接、缓存
//! - 运行时优化：提示缓存、流式处理、并行执行、内存管理
//! - 监控和遥测：指标收集、采样、聚合

pub mod startup;
pub mod runtime;
pub mod metrics;

pub use startup::{StartupOptimizer, LazyLoader, ConnectionPool, DiskCache};
pub use runtime::{PromptCache, StreamProcessor, ParallelExecutor, MemoryManager};
pub use metrics::{MetricsCollector, MetricType, AttributedCounter, MetricsSnapshot, PerformanceMonitor, PerformanceReport};
