pub mod types;
pub mod cache;
pub mod monitor;
pub mod parallel;

pub use parallel::ParallelExecutionEngine;
pub use types::*;

pub async fn create_execution_engine(
    max_parallelism: usize,
) -> ParallelExecutionEngine {
    let engine = ParallelExecutionEngine::new(max_parallelism);
    engine
}