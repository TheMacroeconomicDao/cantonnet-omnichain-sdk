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

use crate::{error::ObservabilityError, config::TracingConfig, ObservabilityResult};

/// Tracing configuration
pub use crate::config::TracingConfig;

/// Initialize distributed tracing with the given configuration
pub fn init_tracing(config: TracingConfig) -> ObservabilityResult<()> {
    if !config.enabled {
        return Ok(());
    }

    match config.exporter.as_str() {
        "otlp" => init_otlp_tracing(&config)?,
        "jaeger" => init_jaeger_tracing(&config)?,
        "zipkin" => init_zipkin_tracing(&config)?,
        _ => return Err(ObservabilityError::InvalidConfiguration(
            format!("Unknown tracing exporter: {}", config.exporter)
        )),
    }

    Ok(())
}

/// Initialize OTLP tracing exporter
fn init_otlp_tracing(config: &TracingConfig) -> ObservabilityResult<()> {
    let endpoint = config.endpoint.as_deref().unwrap_or("http://localhost:4317");
    
    let exporter = opentelemetry_otlp::new_exporter()
        .tonic()
        .with_endpoint(endpoint)
        .build_span_exporter()
        .map_err(|e| ObservabilityError::TracingInitFailed(e.to_string()))?;

    let tracer = opentelemetry_sdk::trace::TracerProvider::builder()
        .with_simple_exporter(exporter)
        .with_batch_exporter(
            opentelemetry_sdk::trace::BatchConfig::default(),
            opentelemetry::trace::TracerProvider,
        )
        .build()
        .tracer(&config.service_name);

    let layer = tracing_opentelemetry::layer().with_tracer(tracer);

    tracing_subscriber::registry()
        .with(layer)
        .try_init()
        .map_err(|e| ObservabilityError::TracingInitFailed(e.to_string()))?;

    Ok(())
}

/// Initialize Jaeger tracing exporter
fn init_jaeger_tracing(config: &TracingConfig) -> ObservabilityResult<()> {
    let endpoint = config.endpoint.as_deref().unwrap_or("http://localhost:14268/api/traces");
    
    let exporter = opentelemetry_jaeger::new_pipeline()
        .with_endpoint(endpoint)
        .with_service_name(&config.service_name)
        .with_simple_exporter()
        .install_batch(opentelemetry_sdk::trace::BatchConfig::default())
        .map_err(|e| ObservabilityError::TracingInitFailed(e.to_string()))?;

    let tracer = opentelemetry::global::tracer(&config.service_name);
    let layer = tracing_opentelemetry::layer().with_tracer(tracer);

    tracing_subscriber::registry()
        .with(layer)
        .try_init()
        .map_err(|e| ObservabilityError::TracingInitFailed(e.to_string()))?;

    Ok(())
}

/// Initialize Zipkin tracing exporter
fn init_zipkin_tracing(config: &TracingConfig) -> ObservabilityResult<()> {
    let endpoint = config.endpoint.as_deref().unwrap_or("http://localhost:9411/api/v2/spans");
    
    let exporter = opentelemetry_zipkin::new_pipeline()
        .with_service_name(&config.service_name)
        .with_collector_endpoint(endpoint)
        .install_batch(opentelemetry_sdk::trace::BatchConfig::default())
        .map_err(|e| ObservabilityError::TracingInitFailed(e.to_string()))?;

    let tracer = opentelemetry::global::tracer(&config.service_name);
    let layer = tracing_opentelemetry::layer().with_tracer(tracer);

    tracing_subscriber::registry()
        .with(layer)
        .try_init()
        .map_err(|e| ObservabilityError::TracingInitFailed(e.to_string()))?;

    Ok(())
}

/// Create a tracing span with the given name
#[macro_export]
macro_rules! trace_span {
    ($name:expr) => {
        tracing::span!(tracing::Level::TRACE, $name)
    };
    ($name:expr, $($key:ident = $value:expr),* $(,)?) => {
        tracing::span!(tracing::Level::TRACE, $name, $($key = $value),*)
    };
}

/// Create a debug span with the given name
#[macro_export]
macro_rules! debug_span {
    ($name:expr) => {
        tracing::span!(tracing::Level::DEBUG, $name)
    };
    ($name:expr, $($key:ident = $value:expr),* $(,)?) => {
        tracing::span!(tracing::Level::DEBUG, $name, $($key = $value),*)
    };
}

/// Create an info span with the given name
#[macro_export]
macro_rules! info_span {
    ($name:expr) => {
        tracing::span!(tracing::Level::INFO, $name)
    };
    ($name:expr, $($key:ident = $value:expr),* $(,)?) => {
        tracing::span!(tracing::Level::INFO, $name, $($key = $value),*)
    };
}

/// Create a warn span with the given name
#[macro_export]
macro_rules! warn_span {
    ($name:expr) => {
        tracing::span!(tracing::Level::WARN, $name)
    };
    ($name:expr, $($key:ident = $value:expr),* $(,)?) => {
        tracing::span!(tracing::Level::WARN, $name, $($key = $value),*)
    };
}

/// Create an error span with the given name
#[macro_export]
macro_rules! error_span {
    ($name:expr) => {
        tracing::span!(tracing::Level::ERROR, $name)
    };
    ($name:expr, $($key:ident = $value:expr),* $(,)?) => {
        tracing::span!(tracing::Level::ERROR, $name, $($key = $value),*)
    };
}
