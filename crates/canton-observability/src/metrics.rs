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
use parking_lot::RwLock;
use crate::{error::ObservabilityError, config::MetricsConfig, ObservabilityResult};

/// Metrics configuration
pub use crate::config::MetricsConfig;

/// Initialize metrics with the given configuration
pub fn init_metrics(config: MetricsConfig) -> ObservabilityResult<()> {
    if !config.enabled {
        return Ok(());
    }

    match config.exporter.as_str() {
        "otlp" => init_otlp_metrics(&config)?,
        "prometheus" => init_prometheus_metrics(&config)?,
        _ => return Err(ObservabilityError::InvalidConfiguration(
            format!("Unknown metrics exporter: {}", config.exporter)
        )),
    }

    Ok(())
}

/// Initialize OTLP metrics exporter
fn init_otlp_metrics(config: &MetricsConfig) -> ObservabilityResult<()> {
    let endpoint = config.endpoint.as_deref().unwrap_or("http://localhost:4317");
    
    let exporter = opentelemetry_otlp::new_exporter()
        .tonic()
        .with_endpoint(endpoint)
        .build_metrics_exporter(
            Box::new(opentelemetry_sdk::metrics::reader::DefaultTemporalitySelector::new()),
            Box::new(opentelemetry_sdk::metrics::reader::DefaultAggregationSelector::new()),
        )
        .map_err(|e| ObservabilityError::MetricsInitFailed(e.to_string()))?;

    let provider = opentelemetry_sdk::metrics::SdkMeterProvider::builder()
        .with_reader(exporter)
        .build();

    opentelemetry::global::set_meter_provider(provider);

    Ok(())
}

/// Initialize Prometheus metrics exporter
#[cfg(feature = "prometheus")]
fn init_prometheus_metrics(config: &MetricsConfig) -> ObservabilityResult<()> {
    let exporter = metrics_exporter_prometheus::PrometheusBuilder::new()
        .with_namespace(&config.namespace)
        .build()
        .map_err(|e| ObservabilityError::MetricsInitFailed(e.to_string()))?;

    metrics::set_global_recorder(exporter)
        .map_err(|e| ObservabilityError::MetricsInitFailed(e.to_string()))?;

    Ok(())
}

/// Initialize Prometheus metrics exporter (no-op without feature)
#[cfg(not(feature = "prometheus"))]
fn init_prometheus_metrics(_config: &MetricsConfig) -> ObservabilityResult<()> {
    Err(ObservabilityError::InvalidConfiguration(
        "Prometheus exporter requires 'prometheus' feature".to_string()
    ))
}

/// Metrics recorder for recording metrics
pub struct MetricsRecorder {
    namespace: String,
    prefix: String,
}

impl MetricsRecorder {
    /// Create a new metrics recorder
    pub fn new(namespace: impl Into<String>, prefix: impl Into<String>) -> Self {
        Self {
            namespace: namespace.into(),
            prefix: prefix.into(),
        }
    }

    /// Record a counter metric
    pub fn counter(&self, name: &str, value: u64, labels: &[(&str, &str)]) {
        let full_name = format!("{}.{}.{}", self.namespace, self.prefix, name);
        let mut labels_str = String::new();
        for (key, val) in labels {
            labels_str.push_str(&format!("{}={},", key, val));
        }
        tracing::debug!("Counter: {} = {} [{}]", full_name, value, labels_str);
    }

    /// Record a gauge metric
    pub fn gauge(&self, name: &str, value: f64, labels: &[(&str, &str)]) {
        let full_name = format!("{}.{}.{}", self.namespace, self.prefix, name);
        let mut labels_str = String::new();
        for (key, val) in labels {
            labels_str.push_str(&format!("{}={},", key, val));
        }
        tracing::debug!("Gauge: {} = {} [{}]", full_name, value, labels_str);
    }

    /// Record a histogram metric
    pub fn histogram(&self, name: &str, value: f64, labels: &[(&str, &str)]) {
        let full_name = format!("{}.{}.{}", self.namespace, self.prefix, name);
        let mut labels_str = String::new();
        for (key, val) in labels {
            labels_str.push_str(&format!("{}={},", key, val));
        }
        tracing::debug!("Histogram: {} = {} [{}]", full_name, value, labels_str);
    }

    /// Record a timing metric
    pub fn timing(&self, name: &str, duration_ms: u64, labels: &[(&str, &str)]) {
        self.histogram(name, duration_ms as f64, labels);
    }
}

impl Default for MetricsRecorder {
    fn default() -> Self {
        Self::new("canton_wallet", "canton_wallet")
    }
}

/// In-memory metrics store for testing
#[derive(Debug, Default)]
pub struct InMemoryMetricsStore {
    counters: Arc<RwLock<Vec<MetricEntry>>>,
    gauges: Arc<RwLock<Vec<MetricEntry>>>,
    histograms: Arc<RwLock<Vec<MetricEntry>>>,
}

impl InMemoryMetricsStore {
    /// Create a new in-memory metrics store
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a counter metric
    pub fn record_counter(&self, name: &str, value: u64, labels: &[(&str, &str)]) {
        let entry = MetricEntry {
            name: name.to_string(),
            value: MetricValue::Counter(value),
            labels: labels.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect(),
            timestamp: chrono::Utc::now(),
        };
        self.counters.write().push(entry);
    }

    /// Record a gauge metric
    pub fn record_gauge(&self, name: &str, value: f64, labels: &[(&str, &str)]) {
        let entry = MetricEntry {
            name: name.to_string(),
            value: MetricValue::Gauge(value),
            labels: labels.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect(),
            timestamp: chrono::Utc::now(),
        };
        self.gauges.write().push(entry);
    }

    /// Record a histogram metric
    pub fn record_histogram(&self, name: &str, value: f64, labels: &[(&str, &str)]) {
        let entry = MetricEntry {
            name: name.to_string(),
            value: MetricValue::Histogram(value),
            labels: labels.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect(),
            timestamp: chrono::Utc::now(),
        };
        self.histograms.write().push(entry);
    }

    /// Get all counters
    pub fn get_counters(&self) -> Vec<MetricEntry> {
        self.counters.read().clone()
    }

    /// Get all gauges
    pub fn get_gauges(&self) -> Vec<MetricEntry> {
        self.gauges.read().clone()
    }

    /// Get all histograms
    pub fn get_histograms(&self) -> Vec<MetricEntry> {
        self.histograms.read().clone()
    }

    /// Clear all metrics
    pub fn clear(&self) {
        self.counters.write().clear();
        self.gauges.write().clear();
        self.histograms.write().clear();
    }
}

/// Metric entry
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MetricEntry {
    /// Metric name
    pub name: String,
    /// Metric value
    pub value: MetricValue,
    /// Metric labels
    pub labels: std::collections::HashMap<String, String>,
    /// Timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Metric value
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type")]
pub enum MetricValue {
    /// Counter value
    Counter(u64),
    /// Gauge value
    Gauge(f64),
    /// Histogram value
    Histogram(f64),
}
