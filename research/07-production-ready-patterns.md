# Production-Ready Patterns for Blockchain SDKs

## 1. Overview

This document covers production-ready patterns, reliability engineering, and operational excellence for blockchain SDKs.

## 2. Reliability Patterns

### 2.1 Circuit Breaker

```rust
//! Circuit breaker pattern for fault tolerance

use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::RwLock;
use std::time::{Duration, Instant};

/// Circuit breaker state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitState {
    /// Circuit is closed, requests flow normally
    Closed,
    /// Circuit is open, requests are rejected
    Open,
    /// Circuit is half-open, testing if service recovered
    HalfOpen,
}

/// Circuit breaker configuration
#[derive(Debug, Clone)]
pub struct CircuitBreakerConfig {
    /// Number of failures before opening circuit
    pub failure_threshold: u32,
    /// Number of successes in half-open to close circuit
    pub success_threshold: u32,
    /// Duration to wait before transitioning from open to half-open
    pub reset_timeout: Duration,
    /// Duration for the sliding window
    pub window_duration: Duration,
    /// Maximum concurrent requests in half-open state
    pub half_open_max_requests: u32,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            success_threshold: 3,
            reset_timeout: Duration::from_secs(30),
            window_duration: Duration::from_secs(60),
            half_open_max_requests: 3,
        }
    }
}

/// Circuit breaker implementation
pub struct CircuitBreaker {
    config: CircuitBreakerConfig,
    state: RwLock<CircuitState>,
    failure_count: AtomicU32,
    success_count: AtomicU32,
    last_failure_time: RwLock<Option<Instant>>,
    half_open_requests: AtomicU32,
    metrics: CircuitBreakerMetrics,
}

impl CircuitBreaker {
    pub fn new(config: CircuitBreakerConfig) -> Self {
        Self {
            config,
            state: RwLock::new(CircuitState::Closed),
            failure_count: AtomicU32::new(0),
            success_count: AtomicU32::new(0),
            last_failure_time: RwLock::new(None),
            half_open_requests: AtomicU32::new(0),
            metrics: CircuitBreakerMetrics::new(),
        }
    }
    
    /// Check if request is allowed
    pub async fn allow_request(&self) -> Result<CircuitBreakerGuard, CircuitBreakerError> {
        let state = *self.state.read().await;
        
        match state {
            CircuitState::Closed => {
                Ok(CircuitBreakerGuard::new(self))
            }
            CircuitState::Open => {
                // Check if we should transition to half-open
                let last_failure = self.last_failure_time.read().await;
                if let Some(time) = *last_failure {
                    if time.elapsed() >= self.config.reset_timeout {
                        drop(last_failure);
                        self.transition_to_half_open().await;
                        return self.allow_request().await;
                    }
                }
                
                self.metrics.record_rejected();
                Err(CircuitBreakerError::CircuitOpen)
            }
            CircuitState::HalfOpen => {
                let current = self.half_open_requests.fetch_add(1, Ordering::SeqCst);
                if current < self.config.half_open_max_requests {
                    Ok(CircuitBreakerGuard::new(self))
                } else {
                    self.half_open_requests.fetch_sub(1, Ordering::SeqCst);
                    self.metrics.record_rejected();
                    Err(CircuitBreakerError::CircuitHalfOpen)
                }
            }
        }
    }
    
    /// Record success
    pub async fn record_success(&self) {
        let state = *self.state.read().await;
        
        match state {
            CircuitState::Closed => {
                self.failure_count.store(0, Ordering::SeqCst);
            }
            CircuitState::HalfOpen => {
                self.half_open_requests.fetch_sub(1, Ordering::SeqCst);
                let successes = self.success_count.fetch_add(1, Ordering::SeqCst) + 1;
                
                if successes >= self.config.success_threshold {
                    self.transition_to_closed().await;
                }
            }
            CircuitState::Open => {}
        }
        
        self.metrics.record_success();
    }
    
    /// Record failure
    pub async fn record_failure(&self) {
        let state = *self.state.read().await;
        
        match state {
            CircuitState::Closed => {
                let failures = self.failure_count.fetch_add(1, Ordering::SeqCst) + 1;
                
                if failures >= self.config.failure_threshold {
                    self.transition_to_open().await;
                }
            }
            CircuitState::HalfOpen => {
                self.half_open_requests.fetch_sub(1, Ordering::SeqCst);
                self.transition_to_open().await;
            }
            CircuitState::Open => {}
        }
        
        self.metrics.record_failure();
    }
    
    async fn transition_to_open(&self) {
        let mut state = self.state.write().await;
        *state = CircuitState::Open;
        
        let mut last_failure = self.last_failure_time.write().await;
        *last_failure = Some(Instant::now());
        
        self.success_count.store(0, Ordering::SeqCst);
        
        tracing::warn!("Circuit breaker opened");
        self.metrics.record_state_change(CircuitState::Open);
    }
    
    async fn transition_to_half_open(&self) {
        let mut state = self.state.write().await;
        *state = CircuitState::HalfOpen;
        
        self.half_open_requests.store(0, Ordering::SeqCst);
        self.success_count.store(0, Ordering::SeqCst);
        
        tracing::info!("Circuit breaker half-open");
        self.metrics.record_state_change(CircuitState::HalfOpen);
    }
    
    async fn transition_to_closed(&self) {
        let mut state = self.state.write().await;
        *state = CircuitState::Closed;
        
        self.failure_count.store(0, Ordering::SeqCst);
        self.success_count.store(0, Ordering::SeqCst);
        
        tracing::info!("Circuit breaker closed");
        self.metrics.record_state_change(CircuitState::Closed);
    }
    
    /// Get current state
    pub async fn state(&self) -> CircuitState {
        *self.state.read().await
    }
}

/// Guard for circuit breaker request
pub struct CircuitBreakerGuard<'a> {
    breaker: &'a CircuitBreaker,
    completed: bool,
}

impl<'a> CircuitBreakerGuard<'a> {
    fn new(breaker: &'a CircuitBreaker) -> Self {
        Self {
            breaker,
            completed: false,
        }
    }
    
    /// Mark request as successful
    pub async fn success(mut self) {
        self.completed = true;
        self.breaker.record_success().await;
    }
    
    /// Mark request as failed
    pub async fn failure(mut self) {
        self.completed = true;
        self.breaker.record_failure().await;
    }
}

impl<'a> Drop for CircuitBreakerGuard<'a> {
    fn drop(&mut self) {
        if !self.completed {
            // If guard is dropped without explicit success/failure, treat as failure
            let breaker = self.breaker;
            tokio::spawn(async move {
                // Note: This is a simplified version; in production, handle this better
            });
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum CircuitBreakerError {
    #[error("Circuit is open")]
    CircuitOpen,
    #[error("Circuit is half-open and at capacity")]
    CircuitHalfOpen,
}

struct CircuitBreakerMetrics {
    // Metrics implementation
}

impl CircuitBreakerMetrics {
    fn new() -> Self { Self {} }
    fn record_success(&self) {}
    fn record_failure(&self) {}
    fn record_rejected(&self) {}
    fn record_state_change(&self, _state: CircuitState) {}
}
```

### 2.2 Bulkhead Pattern

```rust
//! Bulkhead pattern for resource isolation

use tokio::sync::Semaphore;
use std::sync::Arc;

/// Bulkhead configuration
#[derive(Debug, Clone)]
pub struct BulkheadConfig {
    /// Maximum concurrent executions
    pub max_concurrent: usize,
    /// Maximum wait queue size
    pub max_wait: usize,
    /// Timeout for acquiring permit
    pub timeout: Duration,
}

/// Bulkhead for resource isolation
pub struct Bulkhead {
    name: String,
    semaphore: Arc<Semaphore>,
    config: BulkheadConfig,
    metrics: BulkheadMetrics,
}

impl Bulkhead {
    pub fn new(name: impl Into<String>, config: BulkheadConfig) -> Self {
        Self {
            name: name.into(),
            semaphore: Arc::new(Semaphore::new(config.max_concurrent)),
            config,
            metrics: BulkheadMetrics::new(),
        }
    }
    
    /// Execute with bulkhead protection
    pub async fn execute<F, T, E>(&self, f: F) -> Result<T, BulkheadError<E>>
    where
        F: Future<Output = Result<T, E>>,
    {
        // Try to acquire permit with timeout
        let permit = tokio::time::timeout(
            self.config.timeout,
            self.semaphore.acquire(),
        )
        .await
        .map_err(|_| BulkheadError::Timeout)?
        .map_err(|_| BulkheadError::Closed)?;
        
        self.metrics.record_acquired();
        
        let result = f.await.map_err(BulkheadError::Inner);
        
        drop(permit);
        self.metrics.record_released();
        
        result
    }
    
    /// Get available permits
    pub fn available_permits(&self) -> usize {
        self.semaphore.available_permits()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum BulkheadError<E> {
    #[error("Bulkhead timeout")]
    Timeout,
    #[error("Bulkhead closed")]
    Closed,
    #[error("Inner error: {0}")]
    Inner(E),
}

struct BulkheadMetrics {
    // Metrics implementation
}

impl BulkheadMetrics {
    fn new() -> Self { Self {} }
    fn record_acquired(&self) {}
    fn record_released(&self) {}
}

/// Bulkhead registry for managing multiple bulkheads
pub struct BulkheadRegistry {
    bulkheads: RwLock<HashMap<String, Arc<Bulkhead>>>,
    default_config: BulkheadConfig,
}

impl BulkheadRegistry {
    pub fn new(default_config: BulkheadConfig) -> Self {
        Self {
            bulkheads: RwLock::new(HashMap::new()),
            default_config,
        }
    }
    
    /// Get or create bulkhead
    pub async fn get(&self, name: &str) -> Arc<Bulkhead> {
        {
            let bulkheads = self.bulkheads.read().await;
            if let Some(bulkhead) = bulkheads.get(name) {
                return bulkhead.clone();
            }
        }
        
        let mut bulkheads = self.bulkheads.write().await;
        bulkheads
            .entry(name.to_string())
            .or_insert_with(|| Arc::new(Bulkhead::new(name, self.default_config.clone())))
            .clone()
    }
}
```

### 2.3 Rate Limiter

```rust
//! Rate limiting implementation

use std::collections::VecDeque;
use tokio::sync::Mutex;
use std::time::Instant;

/// Rate limiter configuration
#[derive(Debug, Clone)]
pub struct RateLimiterConfig {
    /// Maximum requests per window
    pub max_requests: u32,
    /// Window duration
    pub window: Duration,
    /// Strategy for rate limiting
    pub strategy: RateLimitStrategy,
}

#[derive(Debug, Clone, Copy)]
pub enum RateLimitStrategy {
    /// Fixed window
    FixedWindow,
    /// Sliding window log
    SlidingWindowLog,
    /// Token bucket
    TokenBucket,
    /// Leaky bucket
    LeakyBucket,
}

/// Rate limiter
pub struct RateLimiter {
    config: RateLimiterConfig,
    state: Mutex<RateLimiterState>,
}

struct RateLimiterState {
    /// Timestamps of requests (for sliding window)
    requests: VecDeque<Instant>,
    /// Token count (for token bucket)
    tokens: f64,
    /// Last refill time
    last_refill: Instant,
    /// Window start (for fixed window)
    window_start: Instant,
    /// Request count in current window
    window_count: u32,
}

impl RateLimiter {
    pub fn new(config: RateLimiterConfig) -> Self {
        let now = Instant::now();
        Self {
            config: config.clone(),
            state: Mutex::new(RateLimiterState {
                requests: VecDeque::new(),
                tokens: config.max_requests as f64,
                last_refill: now,
                window_start: now,
                window_count: 0,
            }),
        }
    }
    
    /// Try to acquire a permit
    pub async fn try_acquire(&self) -> Result<(), RateLimitError> {
        match self.config.strategy {
            RateLimitStrategy::FixedWindow => self.try_acquire_fixed_window().await,
            RateLimitStrategy::SlidingWindowLog => self.try_acquire_sliding_window().await,
            RateLimitStrategy::TokenBucket => self.try_acquire_token_bucket().await,
            RateLimitStrategy::LeakyBucket => self.try_acquire_leaky_bucket().await,
        }
    }
    
    /// Acquire a permit, waiting if necessary
    pub async fn acquire(&self) -> Result<(), RateLimitError> {
        loop {
            match self.try_acquire().await {
                Ok(()) => return Ok(()),
                Err(RateLimitError::RateLimited { retry_after }) => {
                    tokio::time::sleep(retry_after).await;
                }
                Err(e) => return Err(e),
            }
        }
    }
    
    async fn try_acquire_fixed_window(&self) -> Result<(), RateLimitError> {
        let mut state = self.state.lock().await;
        let now = Instant::now();
        
        // Check if we need to reset the window
        if now.duration_since(state.window_start) >= self.config.window {
            state.window_start = now;
            state.window_count = 0;
        }
        
        if state.window_count < self.config.max_requests {
            state.window_count += 1;
            Ok(())
        } else {
            let retry_after = self.config.window - now.duration_since(state.window_start);
            Err(RateLimitError::RateLimited { retry_after })
        }
    }
    
    async fn try_acquire_sliding_window(&self) -> Result<(), RateLimitError> {
        let mut state = self.state.lock().await;
        let now = Instant::now();
        let window_start = now - self.config.window;
        
        // Remove old requests
        while let Some(front) = state.requests.front() {
            if *front < window_start {
                state.requests.pop_front();
            } else {
                break;
            }
        }
        
        if state.requests.len() < self.config.max_requests as usize {
            state.requests.push_back(now);
            Ok(())
        } else {
            let oldest = state.requests.front().unwrap();
            let retry_after = self.config.window - now.duration_since(*oldest);
            Err(RateLimitError::RateLimited { retry_after })
        }
    }
    
    async fn try_acquire_token_bucket(&self) -> Result<(), RateLimitError> {
        let mut state = self.state.lock().await;
        let now = Instant::now();
        
        // Refill tokens
        let elapsed = now.duration_since(state.last_refill);
        let refill_rate = self.config.max_requests as f64 / self.config.window.as_secs_f64();
        let new_tokens = elapsed.as_secs_f64() * refill_rate;
        
        state.tokens = (state.tokens + new_tokens).min(self.config.max_requests as f64);
        state.last_refill = now;
        
        if state.tokens >= 1.0 {
            state.tokens -= 1.0;
            Ok(())
        } else {
            let tokens_needed = 1.0 - state.tokens;
            let retry_after = Duration::from_secs_f64(tokens_needed / refill_rate);
            Err(RateLimitError::RateLimited { retry_after })
        }
    }
    
    async fn try_acquire_leaky_bucket(&self) -> Result<(), RateLimitError> {
        // Similar to token bucket but with constant drain rate
        self.try_acquire_token_bucket().await
    }
}

#[derive(Debug, thiserror::Error)]
pub enum RateLimitError {
    #[error("Rate limited, retry after {retry_after:?}")]
    RateLimited { retry_after: Duration },
}
```

## 3. Observability

### 3.1 Structured Logging

```rust
//! Structured logging configuration

use tracing_subscriber::{
    fmt::{self, format::FmtSpan},
    layer::SubscriberExt,
    util::SubscriberInitExt,
    EnvFilter,
};

/// Logging configuration
#[derive(Debug, Clone)]
pub struct LoggingConfig {
    /// Log level
    pub level: String,
    /// Output format
    pub format: LogFormat,
    /// Include span events
    pub span_events: bool,
    /// Include file/line info
    pub file_info: bool,
    /// Include target
    pub target: bool,
    /// JSON output
    pub json: bool,
}

#[derive(Debug, Clone, Copy)]
pub enum LogFormat {
    Pretty,
    Compact,
    Json,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            format: LogFormat::Pretty,
            span_events: true,
            file_info: false,
            target: true,
            json: false,
        }
    }
}

/// Initialize logging
pub fn init_logging(config: &LoggingConfig) -> Result<(), LoggingError> {
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(&config.level));
    
    let span_events = if config.span_events {
        FmtSpan::NEW | FmtSpan::CLOSE
    } else {
        FmtSpan::NONE
    };
    
    match config.format {
        LogFormat::Pretty => {
            let fmt_layer = fmt::layer()
                .with_span_events(span_events)
                .with_file(config.file_info)
                .with_line_number(config.file_info)
                .with_target(config.target)
                .pretty();
            
            tracing_subscriber::registry()
                .with(env_filter)
                .with(fmt_layer)
                .try_init()
                .map_err(|e| LoggingError::InitError(e.to_string()))?;
        }
        LogFormat::Compact => {
            let fmt_layer = fmt::layer()
                .with_span_events(span_events)
                .with_file(config.file_info)
                .with_line_number(config.file_info)
                .with_target(config.target)
                .compact();
            
            tracing_subscriber::registry()
                .with(env_filter)
                .with(fmt_layer)
                .try_init()
                .map_err(|e| LoggingError::InitError(e.to_string()))?;
        }
        LogFormat::Json => {
            let fmt_layer = fmt::layer()
                .with_span_events(span_events)
                .json();
            
            tracing_subscriber::registry()
                .with(env_filter)
                .with(fmt_layer)
                .try_init()
                .map_err(|e| LoggingError::InitError(e.to_string()))?;
        }
    }
    
    Ok(())
}

#[derive(Debug, thiserror::Error)]
pub enum LoggingError {
    #[error("Failed to initialize logging: {0}")]
    InitError(String),
}

/// Logging macros with context
#[macro_export]
macro_rules! log_operation {
    ($level:ident, $operation:expr, $($field:tt)*) => {
        tracing::$level!(
            operation = $operation,
            $($field)*
        )
    };
}

/// Log SDK operation
pub fn log_sdk_operation(
    operation: &str,
    duration: Duration,
    success: bool,
    details: &HashMap<String, String>,
) {
    if success {
        tracing::info!(
            operation = operation,
            duration_ms = duration.as_millis() as u64,
            success = true,
            ?details,
            "SDK operation completed"
        );
    } else {
        tracing::error!(
            operation = operation,
            duration_ms = duration.as_millis() as u64,
            success = false,
            ?details,
            "SDK operation failed"
        );
    }
}
```

### 3.2 Metrics Collection

```rust
//! Metrics collection and export

use opentelemetry::{
    global,
    metrics::{Counter, Histogram, Meter, UpDownCounter},
    KeyValue,
};
use opentelemetry_otlp::WithExportConfig;
use std::time::Duration;

/// Metrics configuration
#[derive(Debug, Clone)]
pub struct MetricsConfig {
    /// OTLP endpoint
    pub endpoint: String,
    /// Export interval
    pub export_interval: Duration,
    /// Service name
    pub service_name: String,
    /// Additional attributes
    pub attributes: HashMap<String, String>,
}

/// Initialize metrics
pub fn init_metrics(config: &MetricsConfig) -> Result<Meter, MetricsError> {
    let exporter = opentelemetry_otlp::new_exporter()
        .tonic()
        .with_endpoint(&config.endpoint);
    
    let meter_provider = opentelemetry_otlp::new_pipeline()
        .metrics(opentelemetry_sdk::runtime::Tokio)
        .with_exporter(exporter)
        .with_period(config.export_interval)
        .build()
        .map_err(|e| MetricsError::InitError(e.to_string()))?;
    
    global::set_meter_provider(meter_provider);
    
    Ok(global::meter(&config.service_name))
}

/// SDK metrics
pub struct SdkMetrics {
    /// Command submissions
    pub commands_submitted: Counter<u64>,
    /// Command latency
    pub command_latency: Histogram<f64>,
    /// Active connections
    pub active_connections: UpDownCounter<i64>,
    /// Transaction events processed
    pub events_processed: Counter<u64>,
    /// Errors by type
    pub errors: Counter<u64>,
    /// Cache hits/misses
    pub cache_hits: Counter<u64>,
    pub cache_misses: Counter<u64>,
    /// Circuit breaker state
    pub circuit_breaker_state: UpDownCounter<i64>,
}

impl SdkMetrics {
    pub fn new(meter: &Meter) -> Self {
        Self {
            commands_submitted: meter
                .u64_counter("canton_sdk_commands_submitted")
                .with_description("Total commands submitted")
                .init(),
            command_latency: meter
                .f64_histogram("canton_sdk_command_latency_seconds")
                .with_description("Command latency in seconds")
                .init(),
            active_connections: meter
                .i64_up_down_counter("canton_sdk_active_connections")
                .with_description("Number of active connections")
                .init(),
            events_processed: meter
                .u64_counter("canton_sdk_events_processed")
                .with_description("Total events processed")
                .init(),
            errors: meter
                .u64_counter("canton_sdk_errors")
                .with_description("Total errors by type")
                .init(),
            cache_hits: meter
                .u64_counter("canton_sdk_cache_hits")
                .with_description("Cache hits")
                .init(),
            cache_misses: meter
                .u64_counter("canton_sdk_cache_misses")
                .with_description("Cache misses")
                .init(),
            circuit_breaker_state: meter
                .i64_up_down_counter("canton_sdk_circuit_breaker_state")
                .with_description("Circuit breaker state (0=closed, 1=half-open, 2=open)")
                .init(),
        }
    }
    
    pub fn record_command(&self, template: &str, success: bool, latency: Duration) {
        let attributes = [
            KeyValue::new("template", template.to_string()),
            KeyValue::new("success", success.to_string()),
        ];
        
        self.commands_submitted.add(1, &attributes);
        self.command_latency.record(latency.as_secs_f64(), &attributes);
    }
    
    pub fn record_error(&self, error_type: &str) {
        self.errors.add(1, &[KeyValue::new("type", error_type.to_string())]);
    }
}

#[derive(Debug, thiserror::Error)]
pub enum MetricsError {
    #[error("Failed to initialize metrics: {0}")]
    InitError(String),
}
```

### 3.3 Distributed Tracing

```rust
//! Distributed tracing configuration

use opentelemetry::{
    global,
    trace::{Tracer, TracerProvider},
    KeyValue,
};
use opentelemetry_otlp::WithExportConfig;
use tracing_opentelemetry::OpenTelemetryLayer;

/// Tracing configuration
#[derive(Debug, Clone)]
pub struct TracingConfig {
    /// OTLP endpoint
    pub endpoint: String,
    /// Service name
    pub service_name: String,
    /// Sample rate (0.0 - 1.0)
    pub sample_rate: f64,
    /// Additional attributes
    pub attributes: HashMap<String, String>,
}

/// Initialize distributed tracing
pub fn init_tracing(config: &TracingConfig) -> Result<(), TracingError> {
    let exporter = opentelemetry_otlp::new_exporter()
        .tonic()
        .with_endpoint(&config.endpoint);
    
    let tracer = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(exporter)
        .with_trace_config(
            opentelemetry_sdk::trace::config()
                .with_sampler(opentelemetry_sdk::trace::Sampler::TraceIdRatioBased(
                    config.sample_rate,
                ))
                .with_resource(opentelemetry_sdk::Resource::new(vec![
                    KeyValue::new("service.name", config.service_name.clone()),
                ])),
        )
        .install_batch(opentelemetry_sdk::runtime::Tokio)
        .map_err(|e| TracingError::InitError(e.to_string()))?;
    
    let telemetry_layer = OpenTelemetryLayer::new(tracer);
    
    // Combine with existing subscriber
    // This should be called during logging initialization
    
    Ok(())
}

/// Span builder for SDK operations
pub struct SpanBuilder {
    name: String,
    attributes: Vec<KeyValue>,
}

impl SpanBuilder {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            attributes: Vec::new(),
        }
    }
    
    pub fn attribute(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.attributes.push(KeyValue::new(key.into(), value.into()));
        self
    }
    
    pub fn start(self) -> tracing::Span {
        let span = tracing::info_span!(
            "sdk_operation",
            otel.name = %self.name,
        );
        
        // Add attributes to span
        for attr in self.attributes {
            span.record(attr.key.as_str(), attr.value.as_str());
        }
        
        span
    }
}

#[derive(Debug, thiserror::Error)]
pub enum TracingError {
    #[error("Failed to initialize tracing: {0}")]
    InitError(String),
}
```

## 4. Health Checks

### 4.1 Health Check System

```rust
//! Health check system

use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;

/// Health status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
}

/// Health check result
#[derive(Debug, Clone)]
pub struct HealthCheckResult {
    pub status: HealthStatus,
    pub message: Option<String>,
    pub details: HashMap<String, String>,
    pub checked_at: chrono::DateTime<chrono::Utc>,
}

/// Health check trait
#[async_trait]
pub trait HealthCheck: Send + Sync {
    fn name(&self) -> &str;
    async fn check(&self) -> HealthCheckResult;
}

/// Health check registry
pub struct HealthCheckRegistry {
    checks: RwLock<HashMap<String, Arc<dyn HealthCheck>>>,
}

impl HealthCheckRegistry {
    pub fn new() -> Self {
        Self {
            checks: RwLock::new(HashMap::new()),
        }
    }
    
    /// Register a health check
    pub async fn register(&self, check: Arc<dyn HealthCheck>) {
        let name = check.name().to_string();
        self.checks.write().await.insert(name, check);
    }
    
    /// Run all health checks
    pub async fn check_all(&self) -> OverallHealth {
        let checks = self.checks.read().await;
        let mut results = HashMap::new();
        let mut overall_status = HealthStatus::Healthy;
        
        for (name, check) in checks.iter() {
            let result = check.check().await;
            
            match result.status {
                HealthStatus::Unhealthy => {
                    overall_status = HealthStatus::Unhealthy;
                }
                HealthStatus::Degraded if overall_status == HealthStatus::Healthy => {
                    overall_status = HealthStatus::Degraded;
                }
                _ => {}
            }
            
            results.insert(name.clone(), result);
        }
        
        OverallHealth {
            status: overall_status,
            checks: results,
            checked_at: chrono::Utc::now(),
        }
    }
    
    /// Run specific health check
    pub async fn check(&self, name: &str) -> Option<HealthCheckResult> {
        let checks = self.checks.read().await;
        if let Some(check) = checks.get(name) {
            Some(check.check().await)
        } else {
            None
        }
    }
}

/// Overall health status
#[derive(Debug, Clone)]
pub struct OverallHealth {
    pub status: HealthStatus,
    pub checks: HashMap<String, HealthCheckResult>,
    pub checked_at: chrono::DateTime<chrono::Utc>,
}

/// Connection health check
pub struct ConnectionHealthCheck {
    name: String,
    client: Arc<LedgerClient>,
}

impl ConnectionHealthCheck {
    pub fn new(name: impl Into<String>, client: Arc<LedgerClient>) -> Self {
        Self {
            name: name.into(),
            client,
        }
    }
}

#[async_trait]
impl HealthCheck for ConnectionHealthCheck {
    fn name(&self) -> &str {
        &self.name
    }
    
    async fn check(&self) -> HealthCheckResult {
        let start = std::time::Instant::now();
        
        match self.client.get_ledger_end().await {
            Ok(offset) => {
                let latency = start.elapsed();
                HealthCheckResult {
                    status: if latency < Duration::from_secs(1) {
                        HealthStatus::Healthy
                    } else {
                        HealthStatus::Degraded
                    },
                    message: Some(format!("Connected, offset: {:?}", offset)),
                    details: [
                        ("latency_ms".to_string(), latency.as_millis().to_string()),
                    ].into_iter().collect(),
                    checked_at: chrono::Utc::now(),
                }
            }
            Err(e) => {
                HealthCheckResult {
                    status: HealthStatus::Unhealthy,
                    message: Some(format!("Connection failed: {}", e)),
                    details: HashMap::new(),
                    checked_at: chrono::Utc::now(),
                }
            }
        }
    }
}
```

## 5. Configuration Management

### 5.1 Configuration System

```rust
//! Configuration management

use config::{Config, ConfigError, Environment, File};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// SDK configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SdkConfig {
    /// Canton connection settings
    pub canton: CantonConfig,
    /// OmniChain settings
    pub omnichain: OmniChainConfig,
    /// Reliability settings
    pub reliability: ReliabilityConfig,
    /// Observability settings
    pub observability: ObservabilityConfig,
    /// Security settings
    pub security: SecurityConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CantonConfig {
    /// Ledger API endpoint
    pub endpoint: String,
    /// TLS configuration
    pub tls: Option<TlsConfig>,
    /// Connection timeout
    #[serde(with = "humantime_serde")]
    pub connect_timeout: Duration,
    /// Request timeout
    #[serde(with = "humantime_serde")]
    pub request_timeout: Duration,
    /// Keep-alive interval
    #[serde(with = "humantime_serde")]
    pub keep_alive_interval: Duration,
    /// Maximum concurrent requests
    pub max_concurrent_requests: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OmniChainConfig {
    /// Enabled chain adapters
    pub enabled_chains: Vec<String>,
    /// Chain-specific configurations
    pub chains: HashMap<String, ChainAdapterConfig>,
    /// Bridge configuration
    pub bridge: BridgeConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReliabilityConfig {
    /// Circuit breaker settings
    pub circuit_breaker: CircuitBreakerConfig,
    /// Rate limiter settings
    pub rate_limiter: RateLimiterConfig,
    /// Retry settings
    pub retry: RetryConfig,
    /// Bulkhead settings
    pub bulkhead: BulkheadConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObservabilityConfig {
    /// Logging settings
    pub logging: LoggingConfig,
    /// Metrics settings
    pub metrics: MetricsConfig,
    /// Tracing settings
    pub tracing: TracingConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// Authentication method
    pub auth: AuthConfig,
    /// Key store configuration
    pub key_store: KeyStoreConfig,
    /// Encryption settings
    pub encryption: EncryptionConfig,
}

impl SdkConfig {
    /// Load configuration from files and environment
    pub fn load() -> Result<Self, ConfigError> {
        Self::load_from_paths(&["config/default", "config/local"])
    }
    
    /// Load configuration from specific paths
    pub fn load_from_paths(paths: &[&str]) -> Result<Self, ConfigError> {
        let mut builder = Config::builder();
        
        // Add configuration files
        for path in paths {
            builder = builder.add_source(
                File::with_name(path).required(false)
            );
        }
        
        // Add environment variables with prefix CANTON_SDK_
        builder = builder.add_source(
            Environment::with_prefix("CANTON_SDK")
                .separator("__")
                .try_parsing(true)
        );
        
        let config = builder.build()?;
        config.try_deserialize()
    }
    
    /// Load from a specific file
    pub fn load_from_file(path: impl AsRef<Path>) -> Result<Self, ConfigError> {
        let config = Config::builder()
            .add_source(File::from(path.as_ref()))
            .build()?;
        config.try_deserialize()
    }
    
    /// Validate configuration
    pub fn validate(&self) -> Result<(), ValidationError> {
        // Validate Canton config
        if self.canton.endpoint.is_empty() {
            return Err(ValidationError::MissingField("canton.endpoint".into()));
        }
        
        // Validate timeouts
        if self.canton.connect_timeout.is_zero() {
            return Err(ValidationError::InvalidValue(
                "canton.connect_timeout".into(),
                "must be greater than 0".into(),
            ));
        }
        
        // Validate reliability settings
        if self.reliability.circuit_breaker.failure_threshold == 0 {
            return Err(ValidationError::InvalidValue(
                "reliability.circuit_breaker.failure_threshold".into(),
                "must be greater than 0".into(),
            ));
        }
        
        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ValidationError {
    #[error("Missing required field: {0}")]
    MissingField(String),
    #[error("Invalid value for {0}: {1}")]
    InvalidValue(String, String),
}
```

### 5.2 Configuration File Example

```yaml
# config/default.yaml

canton:
  endpoint: "https://canton.example.com:6865"
  tls:
    ca_cert_path: "/etc/ssl/certs/ca.pem"
    client_cert_path: "/etc/ssl/certs/client.pem"
    client_key_path: "/etc/ssl/private/client.key"
  connect_timeout: "10s"
  request_timeout: "30s"
  keep_alive_interval: "30s"
  max_concurrent_requests: 100

omnichain:
  enabled_chains:
    - ethereum
    - cosmos
  chains:
    ethereum:
      rpc_url: "https://eth.example.com"
      chain_id: 1
      confirmations: 12
    cosmos:
      rpc_url: "https://cosmos.example.com"
      chain_id: "cosmoshub-4"
      gas_price: "0.025uatom"
  bridge:
    enabled: true
    min_confirmations: 6

reliability:
  circuit_breaker:
    failure_threshold: 5
    success_threshold: 3
    reset_timeout: "30s"
    window_duration: "60s"
  rate_limiter:
    max_requests: 100
    window: "1s"
    strategy: "token_bucket"
  retry:
    max_attempts: 3
    initial_delay: "100ms"
    max_delay: "10s"
    multiplier: 2.0
  bulkhead:
    max_concurrent: 50
    max_wait: 100
    timeout: "5s"

observability:
  logging:
    level: "info"
    format: "json"
    span_events: true
  metrics:
    endpoint: "http://otel-collector:4317"
    export_interval: "10s"
    service_name: "canton-sdk"
  tracing:
    endpoint: "http://otel-collector:4317"
    service_name: "canton-sdk"
    sample_rate: 0.1

security:
  auth:
    method: "jwt"
    token_path: "/var/run/secrets/token"
  key_store:
    type: "vault"
    url: "https://vault.example.com"
    mount_path: "transit"
  encryption:
    algorithm: "aes-256-gcm"
    key_rotation_interval: "30d"
```

## 6. Graceful Shutdown

```rust
//! Graceful shutdown handling

use tokio::sync::{broadcast, watch};
use std::sync::Arc;

/// Shutdown coordinator
pub struct ShutdownCoordinator {
    /// Shutdown signal sender
    shutdown_tx: broadcast::Sender<()>,
    /// Shutdown complete receiver
    complete_rx: watch::Receiver<bool>,
    /// Shutdown complete sender
    complete_tx: watch::Sender<bool>,
    /// Active tasks counter
    active_tasks: Arc<AtomicU32>,
}

impl ShutdownCoordinator {
    pub fn new() -> Self {
        let (shutdown_tx, _) = broadcast::channel(1);
        let (complete_tx, complete_rx) = watch::channel(false);
        
        Self {
            shutdown_tx,
            complete_rx,
            complete_tx,
            active_tasks: Arc::new(AtomicU32::new(0)),
        }
    }
    
    /// Get shutdown signal receiver
    pub fn subscribe(&self) -> broadcast::Receiver<()> {
        self.shutdown_tx.subscribe()
    }
    
    /// Register a task
    pub fn register_task(&self) -> TaskGuard {
        self.active_tasks.fetch_add(1, Ordering::SeqCst);
        TaskGuard {
            active_tasks: self.active_tasks.clone(),
        }
    }
    
    /// Initiate shutdown
    pub async fn shutdown(&self, timeout: Duration) {
        tracing::info!("Initiating graceful shutdown");
        
        // Send shutdown signal
        let _ = self.shutdown_tx.send(());
        
        // Wait for tasks to complete with timeout
        let deadline = tokio::time::Instant::now() + timeout;
        
        loop {
            let active = self.active_tasks.load(Ordering::SeqCst);
            if active == 0 {
                break;
            }
            
            if tokio::time::Instant::now() >= deadline {
                tracing::warn!(
                    active_tasks = active,
                    "Shutdown timeout reached, forcing shutdown"
                );
                break;
            }
            
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
        
        // Mark shutdown as complete
        let _ = self.complete_tx.send(true);
        
        tracing::info!("Graceful shutdown complete");
    }
    
    /// Wait for shutdown to complete
    pub async fn wait_for_shutdown(&self) {
        let mut rx = self.complete_rx.clone();
        while !*rx.borrow() {
            rx.changed().await.ok();
        }
    }
}

/// Guard for tracking active tasks
pub struct TaskGuard {
    active_tasks: Arc<AtomicU32>,
}

impl Drop for TaskGuard {
    fn drop(&mut self) {
        self.active_tasks.fetch_sub(1, Ordering::SeqCst);
    }
}

/// Shutdown-aware service wrapper
pub struct ShutdownAwareService<S> {
    inner: S,
    shutdown: broadcast::Receiver<()>,
}

impl<S> ShutdownAwareService<S> {
    pub fn new(inner: S, coordinator: &ShutdownCoordinator) -> Self {
        Self {
            inner,
            shutdown: coordinator.subscribe(),
        }
    }
    
    /// Run with shutdown awareness
    pub async fn run<F, Fut>(&mut self, f: F) -> Result<(), ShutdownError>
    where
        F: FnOnce(&mut S) -> Fut,
        Fut: Future<Output = ()>,
    {
        tokio::select! {
            _ = f(&mut self.inner) => Ok(()),
            _ = self.shutdown.recv() => {
                tracing::info!("Received shutdown signal");
                Err(ShutdownError::ShutdownRequested)
            }
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ShutdownError {
    #[error("Shutdown requested")]
    ShutdownRequested,
}
```

## 7. Testing Patterns

### 7.1 Integration Test Framework

```rust
//! Integration testing framework

use testcontainers::{clients::Cli, Container, Image};

/// Test environment
pub struct TestEnvironment {
    docker: Cli,
    canton: Option<Container<'static, CantonImage>>,
    client: Option<Arc<LedgerClient>>,
}

impl TestEnvironment {
    pub async fn new() -> Self {
        Self {
            docker: Cli::default(),
            canton: None,
            client: None,
        }
    }
    
    /// Start Canton container
    pub async fn start_canton(&mut self) -> Result<(), TestError> {
        let canton = self.docker.run(CantonImage::default());
        let endpoint = format!(
            "http://localhost:{}",
            canton.get_host_port_ipv4(6865)
        );
        
        // Wait for Canton to be ready
        self.wait_for_canton(&endpoint).await?;
        
        // Create client
        let client = LedgerClient::connect(ClientConfig {
            endpoint,
            ..Default::default()
        }).await?;
        
        self.canton = Some(canton);
        self.client = Some(Arc::new(client));
        
        Ok(())
    }
    
    /// Get client
    pub fn client(&self) -> Arc<LedgerClient> {
        self.client.clone().expect("Canton not started")
    }
    
    async fn wait_for_canton(&self, endpoint: &str) -> Result<(), TestError> {
        let max_attempts = 30;
        let delay = Duration::from_secs(1);
        
        for attempt in 1..=max_attempts {
            match LedgerClient::connect(ClientConfig {
                endpoint: endpoint.to_string(),
                connect_timeout: Duration::from_secs(5),
                ..Default::default()
            }).await {
                Ok(_) => return Ok(()),
                Err(e) if attempt == max_attempts => {
                    return Err(TestError::StartupTimeout(e.to_string()));
                }
                Err(_) => {
                    tokio::time::sleep(delay).await;
                }
            }
        }
        
        Ok(())
    }
}

/// Canton Docker image
struct CantonImage {
    version: String,
}

impl Default for CantonImage {
    fn default() -> Self {
        Self {
            version: "latest".to_string(),
        }
    }
}

impl Image for CantonImage {
    type Args = Vec<String>;
    
    fn name(&self) -> String {
        "digitalasset/canton-open-source".to_string()
    }
    
    fn tag(&self) -> String {
        self.version.clone()
    }
    
    fn ready_conditions(&self) -> Vec<testcontainers::core::WaitFor> {
        vec![
            testcontainers::core::WaitFor::message_on_stdout("Started"),
        ]
    }
}

#[derive(Debug, thiserror::Error)]
pub enum TestError {
    #[error("Startup timeout: {0}")]
    StartupTimeout(String),
    #[error("SDK error: {0}")]
    SdkError(#[from] SdkError),
}

/// Test fixture macro
#[macro_export]
macro_rules! integration_test {
    ($name:ident, $body:expr) => {
        #[tokio::test]
        #[ignore = "requires Docker"]
        async fn $name() {
            let mut env = TestEnvironment::new().await;
            env.start_canton().await.expect("Failed to start Canton");
            
            let client = env.client();
            $body(client).await;
        }
    };
}
```

### 7.2 Property-Based Testing

```rust
//! Property-based testing utilities

use proptest::prelude::*;

/// Generate arbitrary Daml values
pub fn arb_daml_value() -> impl Strategy<Value = DamlValue> {
    prop_oneof![
        Just(DamlValue::Unit),
        any::<bool>().prop_map(DamlValue::Bool),
        any::<i64>().prop_map(DamlValue::Int64),
        "[a-zA-Z0-9]{1,100}".prop_map(DamlValue::Text),
        arb_daml_list(),
        arb_daml_optional(),
        arb_daml_record(),
    ]
}

fn arb_daml_list() -> impl Strategy<Value = DamlValue> {
    prop::collection::vec(arb_daml_value(), 0..10)
        .prop_map(DamlValue::List)
}

fn arb_daml_optional() -> impl Strategy<Value = DamlValue> {
    prop::option::of(arb_daml_value().prop_map(Box::new))
        .prop_map(DamlValue::Optional)
}

fn arb_daml_record() -> impl Strategy<Value = DamlValue> {
    prop::collection::vec(
        ("[a-z]{1,20}", arb_daml_value()),
        0..5,
    )
    .prop_map(|fields| {
        DamlValue::Record(DamlRecord {
            record_id: None,
            fields: fields
                .into_iter()
                .map(|(label, value)| RecordField { label, value })
                .collect(),
        })
    })
}

proptest! {
    #[test]
    fn test_value_serialization_roundtrip(value in arb_daml_value()) {
        let encoded = value.encode();
        let decoded = DamlValue::decode(&encoded).unwrap();
        prop_assert_eq!(value, decoded);
    }
    
    #[test]
    fn test_command_builder_validation(
        app_id in "[a-zA-Z0-9]{1,50}",
        party in "[a-zA-Z0-9]{1,50}",
    ) {
        let result = CommandBuilder::new(&app_id)
            .act_as(PartyId::new(&party).unwrap())
            .create(
                Identifier::new("pkg", "Module", "Template"),
                DamlRecord::default(),
            )
            .build();
        
        prop_assert!(result.is_ok());
    }
}
```
