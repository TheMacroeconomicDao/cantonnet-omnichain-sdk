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

use std::fmt;

/// Result type for observability operations
pub type ObservabilityResult<T> = Result<T, ObservabilityError>;

/// Errors that can occur in observability operations
#[derive(Debug, thiserror::Error)]
pub enum ObservabilityError {
    /// Logging initialization failed
    #[error("Logging initialization failed: {0}")]
    LoggingInitFailed(String),

    /// Metrics initialization failed
    #[error("Metrics initialization failed: {0}")]
    MetricsInitFailed(String),

    /// Tracing initialization failed
    #[error("Tracing initialization failed: {0}")]
    TracingInitFailed(String),

    /// Health check failed
    #[error("Health check failed: {0}")]
    HealthCheckFailed(String),

    /// Invalid configuration
    #[error("Invalid configuration: {0}")]
    InvalidConfiguration(String),

    /// Exporter error
    #[error("Exporter error: {0}")]
    ExporterError(String),

    /// Internal error
    #[error("Internal error: {0}")]
    InternalError(String),
}

impl fmt::Display for ObservabilityError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::LoggingInitFailed(msg) => write!(f, "Logging initialization failed: {}", msg),
            Self::MetricsInitFailed(msg) => write!(f, "Metrics initialization failed: {}", msg),
            Self::TracingInitFailed(msg) => write!(f, "Tracing initialization failed: {}", msg),
            Self::HealthCheckFailed(msg) => write!(f, "Health check failed: {}", msg),
            Self::InvalidConfiguration(msg) => write!(f, "Invalid configuration: {}", msg),
            Self::ExporterError(msg) => write!(f, "Exporter error: {}", msg),
            Self::InternalError(msg) => write!(f, "Internal error: {}", msg),
        }
    }
}
