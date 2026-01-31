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

use tracing_subscriber::{EnvFilter, fmt, prelude::*};
use crate::{error::ObservabilityError, config::LogConfig, ObservabilityResult};

/// Initialize logging with the given configuration
pub fn init_logging(config: LogConfig) -> ObservabilityResult<()> {
    let env_filter = build_env_filter(&config)?;

    let registry = tracing_subscriber::registry().with(env_filter);

    let registry = if config.stdout {
        let layer = build_stdout_layer(&config)?;
        registry.with(layer)
    } else {
        registry
    };

    let registry = if let Some(file_path) = &config.file {
        let layer = build_file_layer(file_path, &config)?;
        registry.with(layer)
    } else {
        registry
    };

    registry.try_init()
        .map_err(|e| ObservabilityError::LoggingInitFailed(e.to_string()))?;

    Ok(())
}

/// Build environment filter from configuration
fn build_env_filter(config: &LogConfig) -> ObservabilityResult<EnvFilter> {
    let filter_str = config.filter.as_deref().unwrap_or("info");
    EnvFilter::try_new(filter_str)
        .map_err(|e| ObservabilityError::LoggingInitFailed(e.to_string()))
}

/// Build stdout layer from configuration
fn build_stdout_layer(config: &LogConfig) -> ObservabilityResult<fmt::Layer<tracing_subscriber::Registry>> {
    let layer = match config.format.as_str() {
        "json" => fmt::layer().json(),
        "pretty" => fmt::layer().pretty(),
        "compact" => fmt::layer().compact(),
        _ => fmt::layer().json(),
    };

    Ok(layer)
}

/// Build file layer from configuration
fn build_file_layer(
    file_path: &str,
    config: &LogConfig,
) -> ObservabilityResult<fmt::Layer<tracing_subscriber::Registry, fmt::format::DefaultFields, fmt::format::Format, fn() -> std::io::Stdout>> {
    let file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(file_path)
        .map_err(|e| ObservabilityError::LoggingInitFailed(format!("Failed to open log file: {}", e)))?;

    let layer = match config.format.as_str() {
        "json" => fmt::layer().json().with_writer(file),
        "pretty" => fmt::layer().pretty().with_writer(file),
        "compact" => fmt::layer().compact().with_writer(file),
        _ => fmt::layer().json().with_writer(file),
    };

    Ok(layer)
}

/// Initialize observability with the given configuration
pub fn init_observability(config: crate::config::ObservabilityConfig) -> ObservabilityResult<()> {
    init_logging(config.logging)?;
    Ok(())
}
