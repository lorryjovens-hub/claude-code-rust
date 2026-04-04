//! 重试策略
//!
//! 实现查询重试机制，包括指数退避、错误类型判断等。

use std::time::Duration;
use thiserror::Error;

/// 重试错误
#[derive(Debug, Error)]
pub enum RetryError {
    /// 超出最大重试次数
    #[error("Exceeded maximum retry attempts ({0})")]
    MaxRetriesExceeded(u32),

    /// 不可重试的错误
    #[error("Non-retryable error: {0}")]
    NonRetryable(String),

    /// 超时错误
    #[error("Timeout after {0:?}")]
    Timeout(Duration),
}

/// 重试策略
pub trait RetryStrategy: Send + Sync {
    /// 检查是否应该重试
    fn should_retry(&self, error: &RetryError, attempt: u32) -> bool;

    /// 获取退避时间
    fn get_backoff(&self, attempt: u32) -> Duration;

    /// 获取最大重试次数
    fn max_retries(&self) -> u32;
}

/// 指数退避策略
pub struct ExponentialBackoffStrategy {
    max_retries: u32,
    initial_backoff: Duration,
    max_backoff: Duration,
    backoff_factor: f32,
    retry_status_codes: Vec<u16>,
}

impl ExponentialBackoffStrategy {
    /// 创建新的指数退避策略
    pub fn new(
        max_retries: u32,
        initial_backoff: Duration,
        max_backoff: Duration,
        backoff_factor: f32,
        retry_status_codes: Vec<u16>,
    ) -> Self {
        Self {
            max_retries,
            initial_backoff,
            max_backoff,
            backoff_factor,
            retry_status_codes,
        }
    }

    /// 创建默认策略
    pub fn default() -> Self {
        Self::new(
            3, // max_retries
            Duration::from_millis(100), // initial_backoff
            Duration::from_secs(5), // max_backoff
            2.0, // backoff_factor
            vec![429, 500, 502, 503, 504], // retry_status_codes
        )
    }
}

impl RetryStrategy for ExponentialBackoffStrategy {
    fn should_retry(&self, error: &RetryError, attempt: u32) -> bool {
        if attempt >= self.max_retries {
            return false;
        }

        match error {
            RetryError::MaxRetriesExceeded(_) => false,
            RetryError::NonRetryable(_) => false,
            RetryError::Timeout(_) => true, // 超时可以重试
        }
    }

    fn get_backoff(&self, attempt: u32) -> Duration {
        if attempt == 0 {
            return Duration::ZERO;
        }

        let backoff = self.initial_backoff.as_secs_f32() * self.backoff_factor.powi(attempt as i32 - 1);
        let backoff = Duration::from_secs_f32(backoff);

        // 限制最大退避时间
        backoff.min(self.max_backoff)
    }

    fn max_retries(&self) -> u32 {
        self.max_retries
    }
}

/// 重试策略构建器
pub struct RetryPolicyBuilder {
    max_retries: u32,
    initial_backoff: Duration,
    max_backoff: Duration,
    backoff_factor: f32,
    retry_status_codes: Vec<u16>,
}

impl RetryPolicyBuilder {
    pub fn new() -> Self {
        Self {
            max_retries: 3,
            initial_backoff: Duration::from_millis(100),
            max_backoff: Duration::from_secs(5),
            backoff_factor: 2.0,
            retry_status_codes: vec![429, 500, 502, 503, 504],
        }
    }

    pub fn max_retries(mut self, max_retries: u32) -> Self {
        self.max_retries = max_retries;
        self
    }

    pub fn initial_backoff(mut self, initial_backoff: Duration) -> Self {
        self.initial_backoff = initial_backoff;
        self
    }

    pub fn max_backoff(mut self, max_backoff: Duration) -> Self {
        self.max_backoff = max_backoff;
        self
    }

    pub fn backoff_factor(mut self, backoff_factor: f32) -> Self {
        self.backoff_factor = backoff_factor;
        self
    }

    pub fn retry_status_codes(mut self, retry_status_codes: Vec<u16>) -> Self {
        self.retry_status_codes = retry_status_codes;
        self
    }

    pub fn build(self) -> ExponentialBackoffStrategy {
        ExponentialBackoffStrategy::new(
            self.max_retries,
            self.initial_backoff,
            self.max_backoff,
            self.backoff_factor,
            self.retry_status_codes,
        )
    }
}

impl Default for RetryPolicyBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// 执行带重试的操作
pub async fn with_retry<T, E, F, Fut>(
    operation: F,
    retry_strategy: &dyn RetryStrategy,
) -> Result<T, RetryError>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = Result<T, E>>,
    E: std::fmt::Display,
{
    let mut attempt = 0;
    let mut last_error = None;

    loop {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(err) => {
                attempt += 1;
                last_error = Some(err.to_string());

                // 将错误转换为 RetryError
                let retry_error = RetryError::NonRetryable(err.to_string());

                if !retry_strategy.should_retry(&retry_error, attempt) {
                    return Err(RetryError::MaxRetriesExceeded(attempt));
                }

                let backoff = retry_strategy.get_backoff(attempt);
                if backoff > Duration::ZERO {
                    tokio::time::sleep(backoff).await;
                }
            }
        }
    }
}

/// 指数退避函数
pub fn exponential_backoff(attempt: u32) -> Duration {
    let base = Duration::from_millis(100);
    let max = Duration::from_secs(5);
    let factor = 2.0;

    let backoff = base.as_secs_f32() * factor.powi(attempt as i32);
    let backoff = Duration::from_secs_f32(backoff);

    backoff.min(max)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_exponential_backoff() {
        let strategy = ExponentialBackoffStrategy::default();

        assert_eq!(strategy.max_retries(), 3);
        assert!(strategy.get_backoff(1) > Duration::ZERO);
        assert!(strategy.get_backoff(2) > strategy.get_backoff(1));
    }

    #[tokio::test]
    async fn test_with_retry_success() {
        let strategy = ExponentialBackoffStrategy::default();

        let mut attempts = 0;
        let result = with_retry(
            || async {
                attempts += 1;
                if attempts < 2 {
                    Err::<(), _>("Temporary error")
                } else {
                    Ok(())
                }
            },
            &strategy,
        ).await;

        assert!(result.is_ok());
        assert_eq!(attempts, 2);
    }

    #[tokio::test]
    async fn test_with_retry_failure() {
        let strategy = ExponentialBackoffStrategy::default();

        let result = with_retry(
            || async { Err::<(), _>("Permanent error") },
            &strategy,
        ).await;

        assert!(result.is_err());
    }
}