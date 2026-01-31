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

use std::sync::Arc;
use std::collections::HashMap;
use parking_lot::RwLock;
use crate::{error::ObservabilityError, config::HealthConfig, ObservabilityResult};

/// Health status
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum HealthStatus {
    /// Healthy
    Healthy,
    /// Degraded
    Degraded,
    /// Unhealthy
    Unhealthy,
}

impl HealthStatus {
    /// Check if status is healthy
    pub fn is_healthy(&self) -> bool {
        matches!(self, Self::Healthy)
    }

    /// Check if status is degraded or unhealthy
    pub fn is_unhealthy(&self) -> bool {
        matches!(self, Self::Degraded | Self::Unhealthy)
    }
}

/// Health check trait
#[async_trait::async_trait]
pub trait HealthCheck: Send + Sync {
    /// Get the name of the health check
    fn name(&self) -> &str;

    /// Perform the health check
    async fn check(&self) -> HealthCheckResult;
}

/// Health check result
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct HealthCheckResult {
    /// Check name
    pub name: String,
    /// Health status
    pub status: HealthStatus,
    /// Message
    pub message: Option<String>,
    /// Duration in milliseconds
    pub duration_ms: u64,
    /// Timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl HealthCheckResult {
    /// Create a new healthy result
    pub fn healthy(name: impl Into<String>, duration_ms: u64) -> Self {
        Self {
            name: name.into(),
            status: HealthStatus::Healthy,
            message: None,
            duration_ms,
            timestamp: chrono::Utc::now(),
        }
    }

    /// Create a new degraded result
    pub fn degraded(name: impl Into<String>, message: impl Into<String>, duration_ms: u64) -> Self {
        Self {
            name: name.into(),
            status: HealthStatus::Degraded,
            message: Some(message.into()),
            duration_ms,
            timestamp: chrono::Utc::now(),
        }
    }

    /// Create a new unhealthy result
    pub fn unhealthy(name: impl Into<String>, message: impl Into<String>, duration_ms: u64) -> Self {
        Self {
            name: name.into(),
            status: HealthStatus::Unhealthy,
            message: Some(message.into()),
            duration_ms,
            timestamp: chrono::Utc::now(),
        }
    }
}

/// Health checker for managing multiple health checks
pub struct HealthChecker {
    checks: Arc<RwLock<HashMap<String, Arc<dyn HealthCheck>>>>,
    config: HealthConfig,
}

impl HealthChecker {
    /// Create a new health checker
    pub fn new(config: HealthConfig) -> Self {
        Self {
            checks: Arc::new(RwLock::new(HashMap::new())),
            config,
        }
    }

    /// Add a health check
    pub fn add_check(&self, check: Arc<dyn HealthCheck>) {
        let name = check.name().to_string();
        self.checks.write().insert(name, check);
    }

    /// Remove a health check
    pub fn remove_check(&self, name: &str) {
        self.checks.write().remove(name);
    }

    /// Get all health check names
    pub fn check_names(&self) -> Vec<String> {
        self.checks.read().keys().cloned().collect()
    }

    /// Perform all health checks
    pub async fn check_all(&self) -> ObservabilityResult<HealthReport> {
        let checks = self.checks.read().clone();
        let mut results = Vec::new();

        for (name, check) in checks.iter() {
            let start = std::time::Instant::now();
            let result = check.check().await;
            let duration = start.elapsed().as_millis() as u64;

            let result = HealthCheckResult {
                name: name.clone(),
                status: result.status,
                message: result.message,
                duration_ms: duration,
                timestamp: chrono::Utc::now(),
            };

            results.push(result);
        }

        let overall_status = self.determine_overall_status(&results);

        Ok(HealthReport {
            status: overall_status,
            checks: results,
            timestamp: chrono::Utc::now(),
        })
    }

    /// Perform a specific health check
    pub async fn check_one(&self, name: &str) -> ObservabilityResult<HealthCheckResult> {
        let checks = self.checks.read();
        let check = checks
            .get(name)
            .ok_or_else(|| ObservabilityError::HealthCheckFailed(format!("Check not found: {}", name)))?;

        let start = std::time::Instant::now();
        let result = check.check().await;
        let duration = start.elapsed().as_millis() as u64;

        Ok(HealthCheckResult {
            name: name.to_string(),
            status: result.status,
            message: result.message,
            duration_ms: duration,
            timestamp: chrono::Utc::now(),
        })
    }

    /// Determine overall health status
    fn determine_overall_status(&self, results: &[HealthCheckResult]) -> HealthStatus {
        if results.is_empty() {
            return HealthStatus::Healthy;
        }

        let has_unhealthy = results.iter().any(|r| r.status == HealthStatus::Unhealthy);
        if has_unhealthy {
            return HealthStatus::Unhealthy;
        }

        let has_degraded = results.iter().any(|r| r.status == HealthStatus::Degraded);
        if has_degraded {
            return HealthStatus::Degraded;
        }

        HealthStatus::Healthy
    }
}

/// Health report
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct HealthReport {
    /// Overall health status
    pub status: HealthStatus,
    /// Individual check results
    pub checks: Vec<HealthCheckResult>,
    /// Report timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl HealthReport {
    /// Check if the system is healthy
    pub fn is_healthy(&self) -> bool {
        self.status.is_healthy()
    }

    /// Get JSON representation
    pub fn to_json(&self) -> ObservabilityResult<String> {
        serde_json::to_string(self)
            .map_err(|e| ObservabilityError::InternalError(e.to_string()))
    }
}

/// Simple health check implementation
pub struct SimpleHealthCheck {
    name: String,
    check_fn: Arc<dyn Fn() -> HealthCheckResult + Send + Sync>,
}

impl SimpleHealthCheck {
    /// Create a new simple health check
    pub fn new(
        name: impl Into<String>,
        check_fn: impl Fn() -> HealthCheckResult + Send + Sync + 'static,
    ) -> Self {
        Self {
            name: name.into(),
            check_fn: Arc::new(check_fn),
        }
    }
}

#[async_trait::async_trait]
impl HealthCheck for SimpleHealthCheck {
    fn name(&self) -> &str {
        &self.name
    }

    async fn check(&self) -> HealthCheckResult {
        (self.check_fn)()
    }
}

/// Async health check implementation
pub struct AsyncHealthCheck {
    name: String,
    check_fn: Arc<dyn Fn() -> HealthCheckResult + Send + Sync>,
}

impl AsyncHealthCheck {
    /// Create a new async health check
    pub fn new(
        name: impl Into<String>,
        check_fn: impl Fn() -> HealthCheckResult + Send + Sync + 'static,
    ) -> Self {
        Self {
            name: name.into(),
            check_fn: Arc::new(check_fn),
        }
    }
}

#[async_trait::async_trait]
impl HealthCheck for AsyncHealthCheck {
    fn name(&self) -> &str {
        &self.name
    }

    async fn check(&self) -> HealthCheckResult {
        (self.check_fn)()
    }
}
