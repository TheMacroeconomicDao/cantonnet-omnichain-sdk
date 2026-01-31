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

//! # Canton Wallet Observability
//!
//! This crate provides observability features for the Canton Wallet SDK, including
//! structured logging, metrics collection, distributed tracing, and health checks.
//!
//! ## Features
//!
//! - Structured logging with tracing
//! - Metrics collection with OpenTelemetry
//! - Distributed tracing with OpenTelemetry
//! - Health checks for monitoring
//! - Prometheus metrics export (optional)
//!
//! ## Example
//!
//! ```no_run
//! use canton_observability::{ObservabilityConfig, init_observability};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let config = ObservabilityConfig::default();
//! init_observability(config)?;
//! # Ok(())
//! # }
//! ```

pub mod error;
pub mod config;
pub mod logging;
pub mod metrics;
pub mod tracing;
pub mod health;

pub use error::{ObservabilityError, ObservabilityResult};
pub use config::ObservabilityConfig;
pub use logging::{init_logging, LogConfig};
pub use metrics::{init_metrics, MetricsConfig, MetricsRecorder};
pub use tracing::{init_tracing, TracingConfig};
pub use health::{HealthChecker, HealthStatus, HealthCheck};
