// Copyright 2025 Canton Wallet SDK Contributors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Retry policy for transaction operations

use std::time::Duration;
use tracing::{debug, warn};
use crate::error::{TransactionError, TransactionResult};

/// Retry policy
#[derive(Debug, Clone)]
pub struct RetryPolicy {
    max_attempts: usize,
    initial_delay: Duration,
    max_delay: Duration,
    backoff_multiplier: f64,
    retryable_errors: Vec<String>,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self::new()
    }
}

impl RetryPolicy {
    pub fn new() -> Self {
        Self {
            max_attempts: 3,
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(10),
            backoff_multiplier: 2.0,
            retryable_errors: vec![
                "network".to_string(),
                "timeout".to_string(),
                "temporary".to_string(),
                "unavailable".to_string(),
            ],
        }
    }

    pub fn with_max_attempts(mut self, attempts: usize) -> Self {
        self.max_attempts = attempts;
        self
    }

    pub fn with_initial_delay(mut self, delay: Duration) -> Self {
        self.initial_delay = delay;
        self
    }

    pub fn with_max_delay(mut self, delay: Duration) -> Self {
        self.max_delay = delay;
        self
    }

    pub fn with_backoff_multiplier(mut self, multiplier: f64) -> Self {
        self.backoff_multiplier = multiplier;
        self
    }

    pub fn with_retryable_errors(mut self, errors: Vec<String>) -> Self {
        self.retryable_errors = errors;
        self
    }

    pub fn max_attempts(&self) -> usize {
        self.max_attempts
    }

    pub fn initial_delay(&self) -> Duration {
        self.initial_delay
    }

    pub fn max_delay(&self) -> Duration {
        self.max_delay
    }

    pub fn backoff_multiplier(&self) -> f64 {
        self.backoff_multiplier
    }

    pub fn is_retryable(&self, error: &TransactionError) -> bool {
        let error_str = error.to_string().to_lowercase();
        self.retryable_errors.iter().any(|retryable| {
            error_str.contains(&retryable.to_lowercase())
        })
    }

    pub fn calculate_delay(&self, attempt: usize) -> Duration {
        let delay_ms = self.initial_delay.as_millis() as f64
            * self.backoff_multiplier.powi(attempt as i32);
        let delay_ms = delay_ms.min(self.max_delay.as_millis() as f64);
        Duration::from_millis(delay_ms as u64)
    }

    pub async fn retry<F, Fut, T>(&self, mut operation: F) -> TransactionResult<T>
    where
        F: FnMut() -> Fut,
        Fut: std::future::Future<Output = TransactionResult<T>>,
    {
        let mut last_error = None;

        for attempt in 0..self.max_attempts {
            match operation().await {
                Ok(result) => {
                    if attempt > 0 {
                        debug!("Operation succeeded after {} attempts", attempt + 1);
                    }
                    return Ok(result);
                }
                Err(error) => {
                    last_error = Some(error.clone());

                    if !self.is_retryable(&error) {
                        warn!("Error is not retryable: {}", error);
                        return Err(error);
                    }

                    if attempt < self.max_attempts - 1 {
                        let delay = self.calculate_delay(attempt);
                        debug!(
                            "Attempt {} failed, retrying after {:?}: {}",
                            attempt + 1,
                            delay,
                            error
                        );
                        tokio::time::sleep(delay).await;
                    } else {
                        warn!(
                            "All {} attempts failed, last error: {}",
                            self.max_attempts,
                            error
                        );
                    }
                }
            }
        }

        Err(last_error.unwrap_or_else(|| {
            TransactionError::InternalError("Unknown error during retry".to_string())
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_retry_policy_defaults() {
        let policy = RetryPolicy::new();
        assert_eq!(policy.max_attempts(), 3);
        assert_eq!(policy.initial_delay(), Duration::from_millis(100));
        assert_eq!(policy.max_delay(), Duration::from_secs(10));
        assert_eq!(policy.backoff_multiplier(), 2.0);
    }

    #[test]
    fn test_retry_policy_custom() {
        let policy = RetryPolicy::new()
            .with_max_attempts(5)
            .with_initial_delay(Duration::from_millis(200))
            .with_max_delay(Duration::from_secs(20))
            .with_backoff_multiplier(3.0);

        assert_eq!(policy.max_attempts(), 5);
        assert_eq!(policy.initial_delay(), Duration::from_millis(200));
        assert_eq!(policy.max_delay(), Duration::from_secs(20));
        assert_eq!(policy.backoff_multiplier(), 3.0);
    }

    #[test]
    fn test_is_retryable() {
        let policy = RetryPolicy::new();
        assert!(policy.is_retryable(&TransactionError::NetworkError("test".to_string())));
        assert!(policy.is_retryable(&TransactionError::Timeout));
        assert!(!policy.is_retryable(&TransactionError::MissingPartyId));
    }

    #[test]
    fn test_calculate_delay() {
        let policy = RetryPolicy::new()
            .with_initial_delay(Duration::from_millis(100))
            .with_backoff_multiplier(2.0);

        assert_eq!(policy.calculate_delay(0), Duration::from_millis(100));
        assert_eq!(policy.calculate_delay(1), Duration::from_millis(200));
        assert_eq!(policy.calculate_delay(2), Duration::from_millis(400));
    }

    #[tokio::test]
    async fn test_retry_success() {
        let policy = RetryPolicy::new().with_max_attempts(3);
        let mut attempt_count = 0;

        let result = policy
            .retry(|| {
                attempt_count += 1;
                async move {
                    if attempt_count < 2 {
                        Err(TransactionError::NetworkError("test".to_string()))
                    } else {
                        Ok("success")
                    }
                }
            })
            .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "success");
        assert_eq!(attempt_count, 2);
    }

    #[tokio::test]
    async fn test_retry_failure() {
        let policy = RetryPolicy::new().with_max_attempts(2);
        let mut attempt_count = 0;

        let result = policy
            .retry(|| {
                attempt_count += 1;
                async move {
                    Err(TransactionError::NetworkError("test".to_string()))
                }
            })
            .await;

        assert!(result.is_err());
        assert_eq!(attempt_count, 2);
    }

    #[tokio::test]
    async fn test_retry_non_retryable() {
        let policy = RetryPolicy::new().with_max_attempts(3);
        let mut attempt_count = 0;

        let result = policy
            .retry(|| {
                attempt_count += 1;
                async move {
                    Err(TransactionError::MissingPartyId)
                }
            })
            .await;

        assert!(result.is_err());
        assert_eq!(attempt_count, 1);
    }
}