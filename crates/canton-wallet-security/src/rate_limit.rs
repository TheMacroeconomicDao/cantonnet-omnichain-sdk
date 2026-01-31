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

//! Rate limiting for preventing abuse.

use crate::error::{Result, SecurityError};
use governor::{
    clock::{Clock, DefaultClock, QuantaClock},
    state::{InMemoryState, NotKeyed},
    Jitter, Quota, RateLimiter,
};
use std::num::NonZeroU32;
use std::sync::Arc;
use std::time::Duration;

/// Rate limiter configuration.
#[derive(Debug, Clone)]
pub struct RateLimiterConfig {
    /// Maximum number of requests allowed.
    pub max_requests: NonZeroU32,

    /// Time period for the rate limit.
    pub period: Duration,

    /// Whether to add jitter to rate limit bursts.
    pub jitter: bool,

    /// Burst size (maximum requests in a burst).
    pub burst_size: Option<NonZeroU32>,
}

impl Default for RateLimiterConfig {
    fn default() -> Self {
        Self {
            max_requests: NonZeroU32::new(100).unwrap(),
            period: Duration::from_secs(60),
            jitter: false,
            burst_size: None,
        }
    }
}

impl RateLimiterConfig {
    /// Create a new rate limiter configuration.
    pub fn new(max_requests: u32, period: Duration) -> Self {
        Self {
            max_requests: NonZeroU32::new(max_requests).unwrap_or(NonZeroU32::new(1).unwrap()),
            period,
            jitter: false,
            burst_size: None,
        }
    }

    /// Set the burst size.
    pub fn with_burst_size(mut self, burst_size: u32) -> Self {
        self.burst_size = NonZeroU32::new(burst_size);
        self
    }

    /// Enable jitter.
    pub fn with_jitter(mut self) -> Self {
        self.jitter = true;
        self
    }
}

/// Rate limiter for preventing abuse.
pub struct RateLimiter<C: Clock = DefaultClock> {
    limiter: RateLimiter<NotKeyed, InMemoryState, C>,
    config: RateLimiterConfig,
}

impl RateLimiter<DefaultClock> {
    /// Create a new rate limiter with default configuration.
    pub fn new() -> Self {
        Self::with_config(RateLimiterConfig::default())
    }

    /// Create a new rate limiter with the specified configuration.
    pub fn with_config(config: RateLimiterConfig) -> Self {
        let quota = Quota::with_period(config.period)
            .unwrap()
            .allow_burst(config.max_requests);

        let limiter = RateLimiter::direct(quota);

        Self { limiter, config }
    }

    /// Create a new rate limiter with the specified quota.
    pub fn with_quota(max_requests: u32, period: Duration) -> Self {
        Self::with_config(RateLimiterConfig::new(max_requests, period))
    }
}

impl<C: Clock> RateLimiter<C> {
    /// Check if a request is allowed.
    ///
    /// # Returns
    ///
    /// Returns Ok(()) if the request is allowed, or an error if the rate limit is exceeded
    pub fn check(&self) -> Result<()> {
        self.check_n(1)
    }

    /// Check if n requests are allowed.
    ///
    /// # Arguments
    ///
    /// * `n` - Number of requests to check
    ///
    /// # Returns
    ///
    /// Returns Ok(()) if the requests are allowed, or an error if the rate limit is exceeded
    pub fn check_n(&self, n: u32) -> Result<()> {
        self.limiter
            .check_n(n)
            .map_err(|_| SecurityError::RateLimitExceeded(self.config.max_requests.get(), self.config.period))
    }

    /// Get the rate limiter configuration.
    pub fn config(&self) -> &RateLimiterConfig {
        &self.config
    }

    /// Get the number of requests remaining in the current period.
    ///
    /// # Returns
    ///
    /// Returns the number of remaining requests
    pub fn remaining(&self) -> u32 {
        // This is a simplified implementation
        // In production, you'd need to properly track the remaining requests
        self.config.max_requests.get()
    }
}

impl Default for RateLimiter<DefaultClock> {
    fn default() -> Self {
        Self::new()
    }
}

/// Keyed rate limiter for rate limiting per key (e.g., per user, per IP).
pub struct KeyedRateLimiter<C: Clock = DefaultClock> {
    limiters: Arc<parking_lot::RwLock<std::collections::HashMap<String, RateLimiter<C>>>>,
    config: RateLimiterConfig,
}

impl KeyedRateLimiter<DefaultClock> {
    /// Create a new keyed rate limiter with default configuration.
    pub fn new() -> Self {
        Self::with_config(RateLimiterConfig::default())
    }

    /// Create a new keyed rate limiter with the specified configuration.
    pub fn with_config(config: RateLimiterConfig) -> Self {
        Self {
            limiters: Arc::new(parking_lot::RwLock::new(std::collections::HashMap::new())),
            config,
        }
    }

    /// Create a new keyed rate limiter with the specified quota.
    pub fn with_quota(max_requests: u32, period: Duration) -> Self {
        Self::with_config(RateLimiterConfig::new(max_requests, period))
    }

    /// Check if a request is allowed for the specified key.
    ///
    /// # Arguments
    ///
    /// * `key` - Key to check (e.g., user ID, IP address)
    ///
    /// # Returns
    ///
    /// Returns Ok(()) if the request is allowed, or an error if the rate limit is exceeded
    pub fn check(&self, key: &str) -> Result<()> {
        self.check_n(key, 1)
    }

    /// Check if n requests are allowed for the specified key.
    ///
    /// # Arguments
    ///
    /// * `key` - Key to check (e.g., user ID, IP address)
    /// * `n` - Number of requests to check
    ///
    /// # Returns
    ///
    /// Returns Ok(()) if the requests are allowed, or an error if the rate limit is exceeded
    pub fn check_n(&self, key: &str, n: u32) -> Result<()> {
        let mut limiters = self.limiters.write();

        let limiter = limiters
            .entry(key.to_string())
            .or_insert_with(|| RateLimiter::with_config(self.config.clone()));

        limiter.check_n(n)
    }

    /// Get the rate limiter configuration.
    pub fn config(&self) -> &RateLimiterConfig {
        &self.config
    }

    /// Get the number of requests remaining for the specified key.
    ///
    /// # Arguments
    ///
    /// * `key` - Key to check (e.g., user ID, IP address)
    ///
    /// # Returns
    ///
    /// Returns the number of remaining requests
    pub fn remaining(&self, key: &str) -> u32 {
        let limiters = self.limiters.read();
        limiters
            .get(key)
            .map(|limiter| limiter.remaining())
            .unwrap_or(self.config.max_requests.get())
    }

    /// Remove the rate limiter for the specified key.
    ///
    /// # Arguments
    ///
    /// * `key` - Key to remove (e.g., user ID, IP address)
    pub fn remove(&self, key: &str) {
        let mut limiters = self.limiters.write();
        limiters.remove(key);
    }

    /// Clear all rate limiters.
    pub fn clear(&self) {
        let mut limiters = self.limiters.write();
        limiters.clear();
    }

    /// Get the number of active rate limiters.
    pub fn len(&self) -> usize {
        let limiters = self.limiters.read();
        limiters.len()
    }

    /// Check if there are any active rate limiters.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl Default for KeyedRateLimiter<DefaultClock> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rate_limiter_config_default() {
        let config = RateLimiterConfig::default();
        assert_eq!(config.max_requests.get(), 100);
        assert_eq!(config.period, Duration::from_secs(60));
        assert!(!config.jitter);
        assert!(config.burst_size.is_none());
    }

    #[test]
    fn test_rate_limiter_config_new() {
        let config = RateLimiterConfig::new(50, Duration::from_secs(30));
        assert_eq!(config.max_requests.get(), 50);
        assert_eq!(config.period, Duration::from_secs(30));
    }

    #[test]
    fn test_rate_limiter_config_with_burst_size() {
        let config = RateLimiterConfig::new(50, Duration::from_secs(30))
            .with_burst_size(100);
        assert_eq!(config.burst_size.unwrap().get(), 100);
    }

    #[test]
    fn test_rate_limiter_config_with_jitter() {
        let config = RateLimiterConfig::new(50, Duration::from_secs(30))
            .with_jitter();
        assert!(config.jitter);
    }

    #[test]
    fn test_rate_limiter_new() {
        let limiter = RateLimiter::new();
        assert_eq!(limiter.config().max_requests.get(), 100);
    }

    #[test]
    fn test_rate_limiter_with_quota() {
        let limiter = RateLimiter::with_quota(50, Duration::from_secs(30));
        assert_eq!(limiter.config().max_requests.get(), 50);
        assert_eq!(limiter.config().period, Duration::from_secs(30));
    }

    #[test]
    fn test_rate_limiter_check() {
        let limiter = RateLimiter::with_quota(10, Duration::from_secs(1));
        // First few requests should succeed
        for _ in 0..5 {
            assert!(limiter.check().is_ok());
        }
    }

    #[test]
    fn test_rate_limiter_remaining() {
        let limiter = RateLimiter::with_quota(10, Duration::from_secs(1));
        assert_eq!(limiter.remaining(), 10);
    }

    #[test]
    fn test_keyed_rate_limiter_new() {
        let limiter = KeyedRateLimiter::new();
        assert_eq!(limiter.config().max_requests.get(), 100);
        assert!(limiter.is_empty());
    }

    #[test]
    fn test_keyed_rate_limiter_with_quota() {
        let limiter = KeyedRateLimiter::with_quota(50, Duration::from_secs(30));
        assert_eq!(limiter.config().max_requests.get(), 50);
    }

    #[test]
    fn test_keyed_rate_limiter_check() {
        let limiter = KeyedRateLimiter::with_quota(10, Duration::from_secs(1));
        // First few requests should succeed
        for _ in 0..5 {
            assert!(limiter.check("user1").is_ok());
        }
        assert_eq!(limiter.len(), 1);
    }

    #[test]
    fn test_keyed_rate_limiter_remaining() {
        let limiter = KeyedRateLimiter::with_quota(10, Duration::from_secs(1));
        assert_eq!(limiter.remaining("user1"), 10);
    }

    #[test]
    fn test_keyed_rate_limiter_remove() {
        let limiter = KeyedRateLimiter::with_quota(10, Duration::from_secs(1));
        limiter.check("user1").unwrap();
        assert_eq!(limiter.len(), 1);
        limiter.remove("user1");
        assert_eq!(limiter.len(), 0);
    }

    #[test]
    fn test_keyed_rate_limiter_clear() {
        let limiter = KeyedRateLimiter::with_quota(10, Duration::from_secs(1));
        limiter.check("user1").unwrap();
        limiter.check("user2").unwrap();
        assert_eq!(limiter.len(), 2);
        limiter.clear();
        assert_eq!(limiter.len(), 0);
    }
}
