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

use serde::{Deserialize, Serialize};

/// Observability configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObservabilityConfig {
    /// Logging configuration
    pub logging: LogConfig,
    /// Metrics configuration
    pub metrics: MetricsConfig,
    /// Tracing configuration
    pub tracing: TracingConfig,
    /// Health check configuration
    pub health: HealthConfig,
}

impl ObservabilityConfig {
    /// Create a new observability configuration
    pub fn new() -> Self {
        Self {
            logging: LogConfig::default(),
            metrics: MetricsConfig::default(),
            tracing: TracingConfig::default(),
            health: HealthConfig::default(),
        }
    }

    /// Set logging configuration
    pub fn with_logging(mut self, logging: LogConfig) -> Self {
        self.logging = logging;
        self
    }

    /// Set metrics configuration
    pub fn with_metrics(mut self, metrics: MetricsConfig) -> Self {
        self.metrics = metrics;
        self
    }

    /// Set tracing configuration
    pub fn with_tracing(mut self, tracing: TracingConfig) -> Self {
        self.tracing = tracing;
        self
    }

    /// Set health check configuration
    pub fn with_health(mut self, health: HealthConfig) -> Self {
        self.health = health;
        self
    }
}

impl Default for ObservabilityConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// Logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogConfig {
    /// Log level
    pub level: String,
    /// Log format (json, pretty, compact)
    pub format: String,
    /// Log to stdout
    pub stdout: bool,
    /// Log to file
    pub file: Option<String>,
    /// Log filter
    pub filter: Option<String>,
}

impl LogConfig {
    /// Create a new log configuration
    pub fn new() -> Self {
        Self {
            level: "info".to_string(),
            format: "json".to_string(),
            stdout: true,
            file: None,
            filter: None,
        }
    }

    /// Set log level
    pub fn with_level(mut self, level: impl Into<String>) -> Self {
        self.level = level.into();
        self
    }

    /// Set log format
    pub fn with_format(mut self, format: impl Into<String>) -> Self {
        self.format = format.into();
        self
    }

    /// Enable stdout logging
    pub fn with_stdout(mut self, stdout: bool) -> Self {
        self.stdout = stdout;
        self
    }

    /// Set log file
    pub fn with_file(mut self, file: impl Into<String>) -> Self {
        self.file = Some(file.into());
        self
    }

    /// Set log filter
    pub fn with_filter(mut self, filter: impl Into<String>) -> Self {
        self.filter = Some(filter.into());
        self
    }
}

impl Default for LogConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// Metrics configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsConfig {
    /// Enable metrics
    pub enabled: bool,
    /// Metrics exporter (prometheus, otlp)
    pub exporter: String,
    /// Metrics endpoint
    pub endpoint: Option<String>,
    /// Metrics namespace
    pub namespace: String,
    /// Metrics prefix
    pub prefix: String,
}

impl MetricsConfig {
    /// Create a new metrics configuration
    pub fn new() -> Self {
        Self {
            enabled: true,
            exporter: "otlp".to_string(),
            endpoint: None,
            namespace: "canton_wallet".to_string(),
            prefix: "canton_wallet".to_string(),
        }
    }

    /// Enable or disable metrics
    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Set metrics exporter
    pub fn with_exporter(mut self, exporter: impl Into<String>) -> Self {
        self.exporter = exporter.into();
        self
    }

    /// Set metrics endpoint
    pub fn with_endpoint(mut self, endpoint: impl Into<String>) -> Self {
        self.endpoint = Some(endpoint.into());
        self
    }

    /// Set metrics namespace
    pub fn with_namespace(mut self, namespace: impl Into<String>) -> Self {
        self.namespace = namespace.into();
        self
    }

    /// Set metrics prefix
    pub fn with_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.prefix = prefix.into();
        self
    }
}

impl Default for MetricsConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// Tracing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TracingConfig {
    /// Enable tracing
    pub enabled: bool,
    /// Tracing exporter (otlp, jaeger, zipkin)
    pub exporter: String,
    /// Tracing endpoint
    pub endpoint: Option<String>,
    /// Service name
    pub service_name: String,
    /// Sample rate (0.0 to 1.0)
    pub sample_rate: f64,
}

impl TracingConfig {
    /// Create a new tracing configuration
    pub fn new() -> Self {
        Self {
            enabled: true,
            exporter: "otlp".to_string(),
            endpoint: None,
            service_name: "canton-wallet-sdk".to_string(),
            sample_rate: 1.0,
        }
    }

    /// Enable or disable tracing
    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Set tracing exporter
    pub fn with_exporter(mut self, exporter: impl Into<String>) -> Self {
        self.exporter = exporter.into();
        self
    }

    /// Set tracing endpoint
    pub fn with_endpoint(mut self, endpoint: impl Into<String>) -> Self {
        self.endpoint = Some(endpoint.into());
        self
    }

    /// Set service name
    pub fn with_service_name(mut self, service_name: impl Into<String>) -> Self {
        self.service_name = service_name.into();
        self
    }

    /// Set sample rate
    pub fn with_sample_rate(mut self, sample_rate: f64) -> Self {
        self.sample_rate = sample_rate.clamp(0.0, 1.0);
        self
    }
}

impl Default for TracingConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// Health check configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthConfig {
    /// Enable health checks
    pub enabled: bool,
    /// Health check interval in seconds
    pub interval_seconds: u64,
    /// Health check timeout in seconds
    pub timeout_seconds: u64,
    /// Health check endpoint
    pub endpoint: String,
}

impl HealthConfig {
    /// Create a new health check configuration
    pub fn new() -> Self {
        Self {
            enabled: true,
            interval_seconds: 30,
            timeout_seconds: 5,
            endpoint: "/health".to_string(),
        }
    }

    /// Enable or disable health checks
    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Set health check interval
    pub fn with_interval(mut self, interval: u64) -> Self {
        self.interval_seconds = interval;
        self
    }

    /// Set health check timeout
    pub fn with_timeout(mut self, timeout: u64) -> Self {
        self.timeout_seconds = timeout;
        self
    }

    /// Set health check endpoint
    pub fn with_endpoint(mut self, endpoint: impl Into<String>) -> Self {
        self.endpoint = endpoint.into();
        self
    }
}

impl Default for HealthConfig {
    fn default() -> Self {
        Self::new()
    }
}
