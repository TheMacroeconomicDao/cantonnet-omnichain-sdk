//! Configuration types.
//! See research/08, DEVELOPMENT_PROMPT § CONFIGURATION.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::time::Duration;

use crate::error::{SdkError, SdkResult};

/// Root SDK configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SdkConfig {
    pub canton: CantonConfig,
    #[serde(default)]
    pub reliability: ReliabilityConfig,
    #[serde(default)]
    pub observability: ObservabilityConfig,
    #[serde(default)]
    pub omnichain: Option<OmniChainConfig>,
}

impl SdkConfig {
    /// Load from default paths (e.g. config.yaml, environment).
    pub fn load() -> SdkResult<Self> {
        let path = std::env::var("CANTON_SDK_CONFIG").unwrap_or_else(|_| "config.yaml".to_string());
        Self::load_from_file(&path)
    }

    /// Load from file.
    pub fn load_from_file(path: impl AsRef<Path>) -> SdkResult<Self> {
        let content = std::fs::read_to_string(path.as_ref())
            .map_err(|e| SdkError::Config(e.to_string()))?;
        let config: Self = serde_yaml::from_str(&content).map_err(|e| SdkError::Config(e.to_string()))?;
        config.validate()?;
        Ok(config)
    }

    /// Validate configuration.
    pub fn validate(&self) -> SdkResult<()> {
        self.canton.validate()?;
        if let Some(ref o) = self.omnichain {
            for chain_id in &o.enabled_chains {
                if !o.chains.contains_key(chain_id) {
                    return Err(SdkError::Config(format!("OmniChain chain config missing for: {}", chain_id)));
                }
            }
        }
        Ok(())
    }
}

/// Canton ledger connection config.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CantonConfig {
    pub endpoint: String,
    #[serde(default)]
    pub tls: Option<TlsConfig>,
    #[serde(default)]
    pub connect_timeout_secs: Option<u64>,
    #[serde(default)]
    pub request_timeout_secs: Option<u64>,
    #[serde(default)]
    pub keep_alive_interval_secs: Option<u64>,
    #[serde(default)]
    pub reliability: CantonReliabilityConfig,
}

impl CantonConfig {
    pub fn validate(&self) -> SdkResult<()> {
        if self.endpoint.is_empty() {
            return Err(SdkError::Config("canton.endpoint must be set".into()));
        }
        Ok(())
    }

    pub fn connect_timeout(&self) -> Duration {
        self.connect_timeout_secs
            .map(Duration::from_secs)
            .unwrap_or(Duration::from_secs(10))
    }

    pub fn request_timeout(&self) -> Duration {
        self.request_timeout_secs
            .map(Duration::from_secs)
            .unwrap_or(Duration::from_secs(30))
    }

    pub fn keep_alive_interval(&self) -> Duration {
        self.keep_alive_interval_secs
            .map(Duration::from_secs)
            .unwrap_or(Duration::from_secs(30))
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CantonReliabilityConfig {
    #[serde(default)]
    pub circuit_breaker: Option<CircuitBreakerConfig>,
    #[serde(default)]
    pub rate_limiter: Option<RateLimiterConfig>,
    #[serde(default)]
    pub retry: Option<RetryConfig>,
}

/// TLS config.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TlsConfig {
    pub ca_cert_path: Option<String>,
    pub client_cert_path: Option<String>,
    pub client_key_path: Option<String>,
}

/// Reliability (circuit breaker, retry) config.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ReliabilityConfig {
    #[serde(default)]
    pub circuit_breaker: Option<CircuitBreakerConfig>,
    #[serde(default)]
    pub rate_limiter: Option<RateLimiterConfig>,
    #[serde(default)]
    pub retry: Option<RetryConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitBreakerConfig {
    #[serde(default = "default_failure_threshold")]
    pub failure_threshold: u32,
    #[serde(default = "default_success_threshold")]
    pub success_threshold: u32,
    #[serde(default = "default_reset_timeout_secs")]
    pub reset_timeout_secs: u64,
    #[serde(default = "default_window_duration_secs")]
    pub window_duration_secs: u64,
    #[serde(default = "default_half_open_max_requests")]
    pub half_open_max_requests: u32,
}

fn default_failure_threshold() -> u32 { 5 }
fn default_success_threshold() -> u32 { 3 }
fn default_reset_timeout_secs() -> u64 { 30 }
fn default_window_duration_secs() -> u64 { 60 }
fn default_half_open_max_requests() -> u32 { 3 }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimiterConfig {
    pub max_requests: u64,
    pub window_secs: u64,
    #[serde(default)]
    pub strategy: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    pub max_attempts: u32,
    pub initial_delay_ms: u64,
    pub max_delay_ms: u64,
    #[serde(default)]
    pub multiplier: f64,
    #[serde(default = "default_true")]
    pub jitter: bool,
}

fn default_true() -> bool { true }

/// Observability (logging, metrics) config.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ObservabilityConfig {
    pub logging_level: Option<String>,
    pub logging_format: Option<String>,
    pub metrics_endpoint: Option<String>,
    pub tracing_endpoint: Option<String>,
    pub tracing_sample_rate: Option<f64>,
}

/// OmniChain (EVM, Cosmos, etc.) config.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OmniChainConfig {
    pub enabled_chains: Vec<String>,
    pub chains: HashMap<String, ChainConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainConfig {
    pub rpc_url: String,
    pub chain_id: String,
}
