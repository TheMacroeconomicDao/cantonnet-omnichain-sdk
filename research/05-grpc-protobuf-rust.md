# gRPC and Protobuf Integration in Rust

**Актуальный источник и версия proto**: см. `crates/canton-ledger-api/proto/README.md` (v2, Daml SDK / digital-asset/daml или Canton).

## 1. Overview

This document covers the implementation of gRPC clients in Rust using Tonic and Prost for Canton Network SDK.

## 2. Tonic Stack Architecture

```
┌─────────────────────────────────────────────────────────────────────┐
│                        Application Layer                             │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  ┌─────────────────────────────────────────────────────────────────┐│
│  │                    Generated Client Stubs                        ││
│  │  (from .proto files via tonic-build)                            ││
│  └─────────────────────────────────────────────────────────────────┘│
│                              │                                       │
│  ┌─────────────────────────────────────────────────────────────────┐│
│  │                    Tonic (gRPC Framework)                        ││
│  │  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐          ││
│  │  │   Channel    │  │ Interceptors │  │   Codecs     │          ││
│  │  └──────────────┘  └──────────────┘  └──────────────┘          ││
│  └─────────────────────────────────────────────────────────────────┘│
│                              │                                       │
│  ┌─────────────────────────────────────────────────────────────────┐│
│  │                    Hyper (HTTP/2)                                ││
│  │  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐          ││
│  │  │  Connection  │  │   Streams    │  │   Frames     │          ││
│  │  └──────────────┘  └──────────────┘  └──────────────┘          ││
│  └─────────────────────────────────────────────────────────────────┘│
│                              │                                       │
│  ┌─────────────────────────────────────────────────────────────────┐│
│  │                    TLS (rustls/native-tls)                       ││
│  └─────────────────────────────────────────────────────────────────┘│
│                              │                                       │
│  ┌─────────────────────────────────────────────────────────────────┐│
│  │                    TCP (tokio)                                   ││
│  └─────────────────────────────────────────────────────────────────┘│
└─────────────────────────────────────────────────────────────────────┘
```

## 3. Proto File Organization

### 3.1 Directory Structure

```
crates/canton-ledger-api/
├── Cargo.toml
├── build.rs
├── proto/
│   ├── com/
│   │   └── daml/
│   │       └── ledger/
│   │           └── api/
│   │               └── v2/
│   │                   ├── command_service.proto
│   │                   ├── command_submission_service.proto
│   │                   ├── command_completion_service.proto
│   │                   ├── transaction_service.proto
│   │                   ├── active_contracts_service.proto
│   │                   ├── party_management_service.proto
│   │                   ├── package_service.proto
│   │                   ├── ledger_identity_service.proto
│   │                   ├── ledger_configuration_service.proto
│   │                   ├── version_service.proto
│   │                   ├── commands.proto
│   │                   ├── transaction.proto
│   │                   ├── event.proto
│   │                   ├── value.proto
│   │                   └── completion.proto
│   └── google/
│       ├── protobuf/
│       │   ├── any.proto
│       │   ├── duration.proto
│       │   ├── empty.proto
│       │   ├── timestamp.proto
│       │   └── wrappers.proto
│       └── rpc/
│           └── status.proto
└── src/
    ├── lib.rs
    └── generated/
        └── mod.rs
```

### 3.2 Build Script

```rust
// build.rs
use std::env;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out_dir = PathBuf::from(env::var("OUT_DIR")?);
    
    // Configure tonic-build
    tonic_build::configure()
        // Output directory for generated code
        .out_dir("src/generated")
        // Generate server code (not needed for client-only SDK)
        .build_server(false)
        // Generate client code
        .build_client(true)
        // Enable transport feature
        .build_transport(true)
        // Type attributes for generated types
        .type_attribute(".", "#[derive(serde::Serialize, serde::Deserialize)]")
        .type_attribute(".", "#[serde(rename_all = \"camelCase\")]")
        // Field attributes
        .field_attribute("Value.sum", "#[serde(flatten)]")
        // Extern paths for well-known types
        .extern_path(".google.protobuf.Timestamp", "::prost_types::Timestamp")
        .extern_path(".google.protobuf.Duration", "::prost_types::Duration")
        .extern_path(".google.protobuf.Empty", "()")
        // Compile proto files
        .compile(
            &[
                "proto/com/daml/ledger/api/v2/command_service.proto",
                "proto/com/daml/ledger/api/v2/command_submission_service.proto",
                "proto/com/daml/ledger/api/v2/command_completion_service.proto",
                "proto/com/daml/ledger/api/v2/transaction_service.proto",
                "proto/com/daml/ledger/api/v2/active_contracts_service.proto",
                "proto/com/daml/ledger/api/v2/party_management_service.proto",
                "proto/com/daml/ledger/api/v2/package_service.proto",
                "proto/com/daml/ledger/api/v2/ledger_identity_service.proto",
                "proto/com/daml/ledger/api/v2/ledger_configuration_service.proto",
                "proto/com/daml/ledger/api/v2/version_service.proto",
            ],
            &["proto/"],
        )?;
    
    // Rerun if proto files change
    println!("cargo:rerun-if-changed=proto/");
    
    Ok(())
}
```

### 3.3 Cargo.toml Configuration

```toml
[package]
name = "canton-ledger-api"
version.workspace = true
edition.workspace = true

[dependencies]
# gRPC
tonic = { version = "0.13", features = ["tls", "tls-roots", "gzip", "zstd", "channel"] }
prost = "0.14"
prost-types = "0.14"

# Async
tokio = { version = "1.45", features = ["rt-multi-thread", "macros"] }
tokio-stream = "0.1"
futures = "0.3"

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# TLS
rustls = { version = "0.23", features = ["ring"] }
tokio-rustls = "0.26"
webpki-roots = "0.26"

# Utilities
bytes = "1.7"
http = "1.1"
tower = { version = "0.5", features = ["util", "timeout", "retry"] }

[build-dependencies]
tonic-build = { version = "0.13", features = ["prost"] }
```

## 4. Generated Code Integration

### 4.1 Module Organization

```rust
// src/generated/mod.rs

/// Daml Ledger API v2
pub mod com {
    pub mod daml {
        pub mod ledger {
            pub mod api {
                pub mod v2 {
                    include!("com.daml.ledger.api.v2.rs");
                }
            }
        }
    }
}

/// Google protobuf types
pub mod google {
    pub mod rpc {
        include!("google.rpc.rs");
    }
}

// Re-exports for convenience
pub use com::daml::ledger::api::v2::*;
```

### 4.2 Type Wrappers

```rust
// src/types/wrappers.rs

use crate::generated::*;
use std::convert::{TryFrom, TryInto};

/// Wrapper for generated Value type with ergonomic API
#[derive(Debug, Clone)]
pub struct DamlValue(pub(crate) value::Sum);

impl DamlValue {
    /// Create unit value
    pub fn unit() -> Self {
        Self(value::Sum::Unit(()))
    }
    
    /// Create bool value
    pub fn bool(v: bool) -> Self {
        Self(value::Sum::Bool(v))
    }
    
    /// Create int64 value
    pub fn int64(v: i64) -> Self {
        Self(value::Sum::Int64(v))
    }
    
    /// Create text value
    pub fn text(v: impl Into<String>) -> Self {
        Self(value::Sum::Text(v.into()))
    }
    
    /// Create party value
    pub fn party(v: impl Into<String>) -> Self {
        Self(value::Sum::Party(v.into()))
    }
    
    /// Create contract ID value
    pub fn contract_id(v: impl Into<String>) -> Self {
        Self(value::Sum::ContractId(v.into()))
    }
    
    /// Create numeric value
    pub fn numeric(v: impl ToString) -> Self {
        Self(value::Sum::Numeric(v.to_string()))
    }
    
    /// Create timestamp value
    pub fn timestamp(v: i64) -> Self {
        Self(value::Sum::Timestamp(v))
    }
    
    /// Create date value
    pub fn date(v: i32) -> Self {
        Self(value::Sum::Date(v))
    }
    
    /// Create list value
    pub fn list(values: impl IntoIterator<Item = DamlValue>) -> Self {
        Self(value::Sum::List(List {
            elements: values.into_iter().map(|v| v.into()).collect(),
        }))
    }
    
    /// Create optional value
    pub fn optional(value: Option<DamlValue>) -> Self {
        Self(value::Sum::Optional(Optional {
            value: value.map(|v| Box::new(v.into())),
        }))
    }
    
    /// Create record value
    pub fn record(fields: impl IntoIterator<Item = (impl Into<String>, DamlValue)>) -> Self {
        Self(value::Sum::Record(Record {
            record_id: None,
            fields: fields
                .into_iter()
                .map(|(label, value)| RecordField {
                    label: label.into(),
                    value: Some(value.into()),
                })
                .collect(),
        }))
    }
    
    /// Create variant value
    pub fn variant(constructor: impl Into<String>, value: DamlValue) -> Self {
        Self(value::Sum::Variant(Variant {
            variant_id: None,
            constructor: constructor.into(),
            value: Some(Box::new(value.into())),
        }))
    }
    
    /// Create enum value
    pub fn enum_value(constructor: impl Into<String>) -> Self {
        Self(value::Sum::Enum(Enum {
            enum_id: None,
            constructor: constructor.into(),
        }))
    }
    
    /// Create text map value
    pub fn text_map(entries: impl IntoIterator<Item = (impl Into<String>, DamlValue)>) -> Self {
        Self(value::Sum::Map(Map {
            entries: entries
                .into_iter()
                .map(|(k, v)| map::Entry {
                    key: k.into(),
                    value: Some(v.into()),
                })
                .collect(),
        }))
    }
    
    /// Create generic map value
    pub fn gen_map(entries: impl IntoIterator<Item = (DamlValue, DamlValue)>) -> Self {
        Self(value::Sum::GenMap(GenMap {
            entries: entries
                .into_iter()
                .map(|(k, v)| gen_map::Entry {
                    key: Some(k.into()),
                    value: Some(v.into()),
                })
                .collect(),
        }))
    }
    
    /// Try to get as bool
    pub fn as_bool(&self) -> Option<bool> {
        match &self.0 {
            value::Sum::Bool(v) => Some(*v),
            _ => None,
        }
    }
    
    /// Try to get as int64
    pub fn as_int64(&self) -> Option<i64> {
        match &self.0 {
            value::Sum::Int64(v) => Some(*v),
            _ => None,
        }
    }
    
    /// Try to get as text
    pub fn as_text(&self) -> Option<&str> {
        match &self.0 {
            value::Sum::Text(v) => Some(v),
            _ => None,
        }
    }
    
    /// Try to get as party
    pub fn as_party(&self) -> Option<&str> {
        match &self.0 {
            value::Sum::Party(v) => Some(v),
            _ => None,
        }
    }
    
    /// Try to get as contract ID
    pub fn as_contract_id(&self) -> Option<&str> {
        match &self.0 {
            value::Sum::ContractId(v) => Some(v),
            _ => None,
        }
    }
    
    /// Try to get as record
    pub fn as_record(&self) -> Option<&Record> {
        match &self.0 {
            value::Sum::Record(v) => Some(v),
            _ => None,
        }
    }
    
    /// Try to get as list
    pub fn as_list(&self) -> Option<&[Value]> {
        match &self.0 {
            value::Sum::List(v) => Some(&v.elements),
            _ => None,
        }
    }
}

impl From<DamlValue> for Value {
    fn from(v: DamlValue) -> Self {
        Value { sum: Some(v.0) }
    }
}

impl TryFrom<Value> for DamlValue {
    type Error = ConversionError;
    
    fn try_from(v: Value) -> Result<Self, Self::Error> {
        v.sum.map(DamlValue).ok_or(ConversionError::EmptyValue)
    }
}

// Implement From for common Rust types
impl From<bool> for DamlValue {
    fn from(v: bool) -> Self {
        Self::bool(v)
    }
}

impl From<i64> for DamlValue {
    fn from(v: i64) -> Self {
        Self::int64(v)
    }
}

impl From<&str> for DamlValue {
    fn from(v: &str) -> Self {
        Self::text(v)
    }
}

impl From<String> for DamlValue {
    fn from(v: String) -> Self {
        Self::text(v)
    }
}

impl<T: Into<DamlValue>> From<Option<T>> for DamlValue {
    fn from(v: Option<T>) -> Self {
        Self::optional(v.map(Into::into))
    }
}

impl<T: Into<DamlValue>> From<Vec<T>> for DamlValue {
    fn from(v: Vec<T>) -> Self {
        Self::list(v.into_iter().map(Into::into))
    }
}
```

## 5. Channel Configuration

### 5.1 Channel Builder

```rust
// src/transport/channel.rs

use tonic::transport::{Channel, ClientTlsConfig, Endpoint};
use std::time::Duration;
use tower::ServiceBuilder;

/// Channel configuration
#[derive(Debug, Clone)]
pub struct ChannelConfig {
    /// Endpoint URL
    pub endpoint: String,
    /// TLS configuration
    pub tls: Option<TlsConfig>,
    /// Connection timeout
    pub connect_timeout: Duration,
    /// Request timeout
    pub request_timeout: Duration,
    /// Keep-alive interval
    pub keep_alive_interval: Option<Duration>,
    /// Keep-alive timeout
    pub keep_alive_timeout: Option<Duration>,
    /// HTTP/2 adaptive window
    pub http2_adaptive_window: bool,
    /// Initial connection window size
    pub initial_connection_window_size: Option<u32>,
    /// Initial stream window size
    pub initial_stream_window_size: Option<u32>,
    /// Concurrency limit
    pub concurrency_limit: Option<usize>,
    /// Rate limit (requests per second)
    pub rate_limit: Option<u32>,
}

impl Default for ChannelConfig {
    fn default() -> Self {
        Self {
            endpoint: String::new(),
            tls: None,
            connect_timeout: Duration::from_secs(10),
            request_timeout: Duration::from_secs(30),
            keep_alive_interval: Some(Duration::from_secs(30)),
            keep_alive_timeout: Some(Duration::from_secs(10)),
            http2_adaptive_window: true,
            initial_connection_window_size: Some(1024 * 1024), // 1MB
            initial_stream_window_size: Some(1024 * 1024),     // 1MB
            concurrency_limit: Some(100),
            rate_limit: None,
        }
    }
}

/// TLS configuration
#[derive(Debug, Clone)]
pub struct TlsConfig {
    /// CA certificate (PEM)
    pub ca_cert: Option<Vec<u8>>,
    /// Client certificate (PEM)
    pub client_cert: Option<Vec<u8>>,
    /// Client key (PEM)
    pub client_key: Option<Vec<u8>>,
    /// Domain name for verification
    pub domain_name: Option<String>,
}

/// Channel builder
pub struct ChannelBuilder {
    config: ChannelConfig,
}

impl ChannelBuilder {
    pub fn new(endpoint: impl Into<String>) -> Self {
        Self {
            config: ChannelConfig {
                endpoint: endpoint.into(),
                ..Default::default()
            },
        }
    }
    
    /// Set TLS configuration
    pub fn tls(mut self, tls: TlsConfig) -> Self {
        self.config.tls = Some(tls);
        self
    }
    
    /// Set connect timeout
    pub fn connect_timeout(mut self, timeout: Duration) -> Self {
        self.config.connect_timeout = timeout;
        self
    }
    
    /// Set request timeout
    pub fn request_timeout(mut self, timeout: Duration) -> Self {
        self.config.request_timeout = timeout;
        self
    }
    
    /// Set keep-alive interval
    pub fn keep_alive_interval(mut self, interval: Duration) -> Self {
        self.config.keep_alive_interval = Some(interval);
        self
    }
    
    /// Set concurrency limit
    pub fn concurrency_limit(mut self, limit: usize) -> Self {
        self.config.concurrency_limit = Some(limit);
        self
    }
    
    /// Set rate limit
    pub fn rate_limit(mut self, requests_per_second: u32) -> Self {
        self.config.rate_limit = Some(requests_per_second);
        self
    }
    
    /// Build the channel
    pub async fn build(self) -> Result<Channel, ChannelError> {
        let mut endpoint = Endpoint::from_shared(self.config.endpoint.clone())
            .map_err(|e| ChannelError::InvalidEndpoint(e.to_string()))?;
        
        // Configure timeouts
        endpoint = endpoint
            .connect_timeout(self.config.connect_timeout)
            .timeout(self.config.request_timeout);
        
        // Configure keep-alive
        if let Some(interval) = self.config.keep_alive_interval {
            endpoint = endpoint.keep_alive_while_idle(true);
            endpoint = endpoint.http2_keep_alive_interval(interval);
        }
        if let Some(timeout) = self.config.keep_alive_timeout {
            endpoint = endpoint.keep_alive_timeout(timeout);
        }
        
        // Configure HTTP/2
        if self.config.http2_adaptive_window {
            endpoint = endpoint.http2_adaptive_window(true);
        }
        if let Some(size) = self.config.initial_connection_window_size {
            endpoint = endpoint.initial_connection_window_size(size);
        }
        if let Some(size) = self.config.initial_stream_window_size {
            endpoint = endpoint.initial_stream_window_size(size);
        }
        
        // Configure TLS
        if let Some(tls) = &self.config.tls {
            let tls_config = Self::build_tls_config(tls)?;
            endpoint = endpoint.tls_config(tls_config)
                .map_err(|e| ChannelError::TlsError(e.to_string()))?;
        }
        
        // Connect
        let channel = endpoint.connect().await
            .map_err(|e| ChannelError::ConnectionFailed(e.to_string()))?;
        
        Ok(channel)
    }
    
    fn build_tls_config(tls: &TlsConfig) -> Result<ClientTlsConfig, ChannelError> {
        let mut config = ClientTlsConfig::new();
        
        // Set CA certificate
        if let Some(ca_cert) = &tls.ca_cert {
            let cert = tonic::transport::Certificate::from_pem(ca_cert);
            config = config.ca_certificate(cert);
        } else {
            // Use system roots
            config = config.with_webpki_roots();
        }
        
        // Set client certificate
        if let (Some(cert), Some(key)) = (&tls.client_cert, &tls.client_key) {
            let identity = tonic::transport::Identity::from_pem(cert, key);
            config = config.identity(identity);
        }
        
        // Set domain name
        if let Some(domain) = &tls.domain_name {
            config = config.domain_name(domain);
        }
        
        Ok(config)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ChannelError {
    #[error("Invalid endpoint: {0}")]
    InvalidEndpoint(String),
    #[error("TLS error: {0}")]
    TlsError(String),
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),
}
```

### 5.2 Connection Pool

```rust
// src/transport/pool.rs

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, Semaphore};
use tonic::transport::Channel;

/// Connection pool for managing multiple channels
pub struct ConnectionPool {
    channels: RwLock<HashMap<String, PooledChannel>>,
    config: PoolConfig,
    semaphore: Arc<Semaphore>,
}

struct PooledChannel {
    channel: Channel,
    created_at: std::time::Instant,
    last_used: std::time::Instant,
}

#[derive(Debug, Clone)]
pub struct PoolConfig {
    /// Maximum number of connections
    pub max_connections: usize,
    /// Connection idle timeout
    pub idle_timeout: Duration,
    /// Connection max lifetime
    pub max_lifetime: Duration,
}

impl Default for PoolConfig {
    fn default() -> Self {
        Self {
            max_connections: 10,
            idle_timeout: Duration::from_secs(300),
            max_lifetime: Duration::from_secs(3600),
        }
    }
}

impl ConnectionPool {
    pub fn new(config: PoolConfig) -> Self {
        Self {
            channels: RwLock::new(HashMap::new()),
            semaphore: Arc::new(Semaphore::new(config.max_connections)),
            config,
        }
    }
    
    /// Get or create a channel for the given endpoint
    pub async fn get(&self, endpoint: &str) -> Result<Channel, PoolError> {
        // Try to get existing channel
        {
            let channels = self.channels.read().await;
            if let Some(pooled) = channels.get(endpoint) {
                if self.is_valid(pooled) {
                    return Ok(pooled.channel.clone());
                }
            }
        }
        
        // Create new channel
        self.create_channel(endpoint).await
    }
    
    async fn create_channel(&self, endpoint: &str) -> Result<Channel, PoolError> {
        // Acquire permit
        let _permit = self.semaphore.acquire().await
            .map_err(|_| PoolError::PoolExhausted)?;
        
        // Double-check after acquiring permit
        {
            let channels = self.channels.read().await;
            if let Some(pooled) = channels.get(endpoint) {
                if self.is_valid(pooled) {
                    return Ok(pooled.channel.clone());
                }
            }
        }
        
        // Create new channel
        let channel = ChannelBuilder::new(endpoint)
            .build()
            .await
            .map_err(|e| PoolError::ChannelError(e))?;
        
        // Store in pool
        let now = std::time::Instant::now();
        let pooled = PooledChannel {
            channel: channel.clone(),
            created_at: now,
            last_used: now,
        };
        
        self.channels.write().await.insert(endpoint.to_string(), pooled);
        
        Ok(channel)
    }
    
    fn is_valid(&self, pooled: &PooledChannel) -> bool {
        let now = std::time::Instant::now();
        let age = now - pooled.created_at;
        let idle = now - pooled.last_used;
        
        age < self.config.max_lifetime && idle < self.config.idle_timeout
    }
    
    /// Clean up expired connections
    pub async fn cleanup(&self) {
        let mut channels = self.channels.write().await;
        channels.retain(|_, pooled| self.is_valid(pooled));
    }
    
    /// Start background cleanup task
    pub fn start_cleanup_task(self: Arc<Self>, interval: Duration) {
        tokio::spawn(async move {
            let mut ticker = tokio::time::interval(interval);
            loop {
                ticker.tick().await;
                self.cleanup().await;
            }
        });
    }
}

#[derive(Debug, thiserror::Error)]
pub enum PoolError {
    #[error("Connection pool exhausted")]
    PoolExhausted,
    #[error("Channel error: {0}")]
    ChannelError(#[from] ChannelError),
}
```

## 6. Interceptors

### 6.1 Authentication Interceptor

```rust
// src/transport/interceptors.rs

use tonic::{Request, Status};
use tonic::service::Interceptor;

/// Authentication interceptor
#[derive(Clone)]
pub struct AuthInterceptor {
    token: Arc<RwLock<String>>,
}

impl AuthInterceptor {
    pub fn new(token: impl Into<String>) -> Self {
        Self {
            token: Arc::new(RwLock::new(token.into())),
        }
    }
    
    /// Update the token
    pub async fn set_token(&self, token: impl Into<String>) {
        *self.token.write().await = token.into();
    }
}

impl Interceptor for AuthInterceptor {
    fn call(&mut self, mut request: Request<()>) -> Result<Request<()>, Status> {
        // Get token synchronously (blocking)
        let token = self.token.blocking_read();
        
        let header_value = format!("Bearer {}", token)
            .parse()
            .map_err(|_| Status::internal("Invalid token format"))?;
        
        request.metadata_mut().insert("authorization", header_value);
        
        Ok(request)
    }
}

/// Tracing interceptor
#[derive(Clone)]
pub struct TracingInterceptor;

impl Interceptor for TracingInterceptor {
    fn call(&mut self, mut request: Request<()>) -> Result<Request<()>, Status> {
        // Inject trace context
        let span = tracing::Span::current();
        
        if let Some(trace_id) = span.context().span().span_context().trace_id() {
            let header_value = trace_id.to_string()
                .parse()
                .map_err(|_| Status::internal("Invalid trace ID"))?;
            request.metadata_mut().insert("x-trace-id", header_value);
        }
        
        Ok(request)
    }
}

/// Retry interceptor configuration
#[derive(Clone)]
pub struct RetryInterceptor {
    max_retries: u32,
    initial_backoff: Duration,
    max_backoff: Duration,
}

impl RetryInterceptor {
    pub fn new(max_retries: u32) -> Self {
        Self {
            max_retries,
            initial_backoff: Duration::from_millis(100),
            max_backoff: Duration::from_secs(10),
        }
    }
    
    pub fn with_backoff(mut self, initial: Duration, max: Duration) -> Self {
        self.initial_backoff = initial;
        self.max_backoff = max;
        self
    }
}

/// Combine multiple interceptors
pub fn combine_interceptors<I1, I2>(i1: I1, i2: I2) -> impl Interceptor
where
    I1: Interceptor + Clone,
    I2: Interceptor + Clone,
{
    move |request: Request<()>| {
        let mut i1 = i1.clone();
        let mut i2 = i2.clone();
        i1.call(request).and_then(|r| i2.call(r))
    }
}
```

### 6.2 Metrics Interceptor

```rust
// src/transport/metrics.rs

use std::time::Instant;
use tonic::{Request, Response, Status};
use tower::{Layer, Service};
use std::task::{Context, Poll};
use std::pin::Pin;
use std::future::Future;

/// Metrics layer for gRPC calls
#[derive(Clone)]
pub struct MetricsLayer {
    metrics: Arc<GrpcMetrics>,
}

impl MetricsLayer {
    pub fn new(metrics: Arc<GrpcMetrics>) -> Self {
        Self { metrics }
    }
}

impl<S> Layer<S> for MetricsLayer {
    type Service = MetricsService<S>;
    
    fn layer(&self, service: S) -> Self::Service {
        MetricsService {
            inner: service,
            metrics: self.metrics.clone(),
        }
    }
}

#[derive(Clone)]
pub struct MetricsService<S> {
    inner: S,
    metrics: Arc<GrpcMetrics>,
}

impl<S, ReqBody, ResBody> Service<http::Request<ReqBody>> for MetricsService<S>
where
    S: Service<http::Request<ReqBody>, Response = http::Response<ResBody>>,
    S::Future: Send + 'static,
    S::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;
    
    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }
    
    fn call(&mut self, request: http::Request<ReqBody>) -> Self::Future {
        let method = request.uri().path().to_string();
        let metrics = self.metrics.clone();
        let start = Instant::now();
        
        let future = self.inner.call(request);
        
        Box::pin(async move {
            let result = future.await;
            let duration = start.elapsed();
            
            match &result {
                Ok(response) => {
                    let status = response.status();
                    metrics.record_request(&method, status.as_u16(), duration);
                }
                Err(_) => {
                    metrics.record_error(&method, duration);
                }
            }
            
            result
        })
    }
}

/// gRPC metrics
pub struct GrpcMetrics {
    requests_total: Counter<u64>,
    request_duration: Histogram<f64>,
    errors_total: Counter<u64>,
}

impl GrpcMetrics {
    pub fn new(meter: &opentelemetry::metrics::Meter) -> Self {
        Self {
            requests_total: meter
                .u64_counter("grpc_client_requests_total")
                .with_description("Total number of gRPC requests")
                .init(),
            request_duration: meter
                .f64_histogram("grpc_client_request_duration_seconds")
                .with_description("gRPC request duration in seconds")
                .init(),
            errors_total: meter
                .u64_counter("grpc_client_errors_total")
                .with_description("Total number of gRPC errors")
                .init(),
        }
    }
    
    pub fn record_request(&self, method: &str, status: u16, duration: Duration) {
        let attributes = [
            KeyValue::new("method", method.to_string()),
            KeyValue::new("status", status.to_string()),
        ];
        
        self.requests_total.add(1, &attributes);
        self.request_duration.record(duration.as_secs_f64(), &attributes);
    }
    
    pub fn record_error(&self, method: &str, duration: Duration) {
        let attributes = [
            KeyValue::new("method", method.to_string()),
        ];
        
        self.errors_total.add(1, &attributes);
        self.request_duration.record(duration.as_secs_f64(), &attributes);
    }
}
```

## 7. Streaming Implementation

### 7.1 Bidirectional Streaming

```rust
// src/transport/streaming.rs

use futures::{Stream, StreamExt, Sink, SinkExt};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;

/// Streaming client wrapper
pub struct StreamingClient<T> {
    sender: mpsc::Sender<T>,
    receiver: ReceiverStream<Result<T, Status>>,
}

impl<T: Send + 'static> StreamingClient<T> {
    /// Create new streaming client
    pub fn new(buffer_size: usize) -> (Self, mpsc::Receiver<T>, mpsc::Sender<Result<T, Status>>) {
        let (tx_out, rx_out) = mpsc::channel(buffer_size);
        let (tx_in, rx_in) = mpsc::channel(buffer_size);
        
        let client = Self {
            sender: tx_out,
            receiver: ReceiverStream::new(rx_in),
        };
        
        (client, rx_out, tx_in)
    }
    
    /// Send a message
    pub async fn send(&self, message: T) -> Result<(), SendError> {
        self.sender.send(message).await
            .map_err(|_| SendError::ChannelClosed)
    }
    
    /// Receive next message
    pub async fn recv(&mut self) -> Option<Result<T, Status>> {
        self.receiver.next().await
    }
    
    /// Get the receiver stream
    pub fn into_stream(self) -> impl Stream<Item = Result<T, Status>> {
        self.receiver
    }
}

#[derive(Debug, thiserror::Error)]
pub enum SendError {
    #[error("Channel closed")]
    ChannelClosed,
}

/// Transaction stream with automatic reconnection
pub struct ReconnectingStream<T> {
    endpoint: String,
    config: StreamConfig,
    current_stream: Option<Box<dyn Stream<Item = Result<T, Status>> + Send + Unpin>>,
    reconnect_attempts: u32,
}

#[derive(Debug, Clone)]
pub struct StreamConfig {
    pub max_reconnect_attempts: u32,
    pub reconnect_delay: Duration,
    pub max_reconnect_delay: Duration,
    pub backoff_multiplier: f64,
}

impl Default for StreamConfig {
    fn default() -> Self {
        Self {
            max_reconnect_attempts: 10,
            reconnect_delay: Duration::from_millis(100),
            max_reconnect_delay: Duration::from_secs(30),
            backoff_multiplier: 2.0,
        }
    }
}

impl<T> ReconnectingStream<T> {
    pub fn new(endpoint: impl Into<String>, config: StreamConfig) -> Self {
        Self {
            endpoint: endpoint.into(),
            config,
            current_stream: None,
            reconnect_attempts: 0,
        }
    }
    
    fn calculate_delay(&self) -> Duration {
        let delay = self.config.reconnect_delay.as_secs_f64()
            * self.config.backoff_multiplier.powi(self.reconnect_attempts as i32);
        
        Duration::from_secs_f64(delay.min(self.config.max_reconnect_delay.as_secs_f64()))
    }
}

impl<T: Send + 'static> Stream for ReconnectingStream<T> {
    type Item = Result<T, Status>;
    
    fn poll_next(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        // Implementation would handle reconnection logic
        todo!()
    }
}
```

## 8. Error Handling

### 8.1 gRPC Status Mapping

```rust
// src/transport/error.rs

use tonic::{Code, Status};

/// Map gRPC status to SDK error
pub fn map_grpc_error(status: Status) -> SdkError {
    let code = status.code();
    let message = status.message().to_string();
    let details = extract_error_details(&status);
    
    match code {
        Code::Ok => unreachable!("OK status should not be an error"),
        
        Code::Cancelled => SdkError::Cancelled { message },
        
        Code::Unknown => SdkError::Internal {
            message: format!("Unknown error: {}", message),
            backtrace: std::backtrace::Backtrace::capture(),
        },
        
        Code::InvalidArgument => SdkError::InvalidArgument {
            field: details.field,
            message,
        },
        
        Code::DeadlineExceeded => SdkError::Timeout {
            duration: Duration::from_secs(0),
            operation: message,
        },
        
        Code::NotFound => SdkError::NotFound {
            resource_type: details.resource_type,
            resource_id: details.resource_id,
        },
        
        Code::AlreadyExists => SdkError::AlreadyExists {
            resource_type: details.resource_type,
            resource_id: details.resource_id,
        },
        
        Code::PermissionDenied => SdkError::PermissionDenied {
            action: details.action,
            resource: details.resource_id,
        },
        
        Code::ResourceExhausted => SdkError::RateLimited {
            retry_after: details.retry_after,
        },
        
        Code::FailedPrecondition => SdkError::FailedPrecondition {
            condition: message,
            details: details.additional_info,
        },
        
        Code::Aborted => SdkError::Aborted {
            reason: message,
            retry_info: details.retry_info,
        },
        
        Code::OutOfRange => SdkError::OutOfRange {
            field: details.field,
            message,
        },
        
        Code::Unimplemented => SdkError::Unimplemented {
            feature: message,
        },
        
        Code::Internal => SdkError::Internal {
            message,
            backtrace: std::backtrace::Backtrace::capture(),
        },
        
        Code::Unavailable => SdkError::Unavailable {
            service: details.service,
            message,
        },
        
        Code::DataLoss => SdkError::DataLoss {
            message,
        },
        
        Code::Unauthenticated => SdkError::Unauthenticated {
            message,
        },
    }
}

struct ErrorDetails {
    field: Option<String>,
    resource_type: Option<String>,
    resource_id: Option<String>,
    action: Option<String>,
    service: Option<String>,
    retry_after: Option<Duration>,
    retry_info: Option<RetryInfo>,
    additional_info: HashMap<String, String>,
}

fn extract_error_details(status: &Status) -> ErrorDetails {
    // Parse error details from status metadata
    let mut details = ErrorDetails {
        field: None,
        resource_type: None,
        resource_id: None,
        action: None,
        service: None,
        retry_after: None,
        retry_info: None,
        additional_info: HashMap::new(),
    };
    
    // Extract from metadata
    if let Some(value) = status.metadata().get("x-resource-type") {
        details.resource_type = value.to_str().ok().map(String::from);
    }
    if let Some(value) = status.metadata().get("x-resource-id") {
        details.resource_id = value.to_str().ok().map(String::from);
    }
    if let Some(value) = status.metadata().get("retry-after") {
        if let Ok(secs) = value.to_str().unwrap_or("0").parse::<u64>() {
            details.retry_after = Some(Duration::from_secs(secs));
        }
    }
    
    details
}
```

## 9. Testing gRPC Clients

### 9.1 Mock Server

```rust
// src/testing/mock_server.rs

use tonic::{Request, Response, Status};
use std::sync::Arc;
use tokio::sync::Mutex;

/// Mock gRPC server for testing
pub struct MockLedgerServer {
    responses: Arc<Mutex<MockResponses>>,
}

struct MockResponses {
    submit_responses: Vec<Result<SubmitAndWaitResponse, Status>>,
    transaction_responses: Vec<Result<GetTransactionsResponse, Status>>,
    party_responses: Vec<Result<AllocatePartyResponse, Status>>,
}

impl MockLedgerServer {
    pub fn new() -> Self {
        Self {
            responses: Arc::new(Mutex::new(MockResponses {
                submit_responses: Vec::new(),
                transaction_responses: Vec::new(),
                party_responses: Vec::new(),
            })),
        }
    }
    
    /// Queue a submit response
    pub async fn queue_submit_response(&self, response: Result<SubmitAndWaitResponse, Status>) {
        self.responses.lock().await.submit_responses.push(response);
    }
    
    /// Queue a transaction response
    pub async fn queue_transaction_response(&self, response: Result<GetTransactionsResponse, Status>) {
        self.responses.lock().await.transaction_responses.push(response);
    }
    
    /// Start the mock server
    pub async fn start(self, addr: std::net::SocketAddr) -> Result<(), Box<dyn std::error::Error>> {
        tonic::transport::Server::builder()
            .add_service(CommandServiceServer::new(self.clone()))
            .add_service(TransactionServiceServer::new(self.clone()))
            .serve(addr)
            .await?;
        
        Ok(())
    }
}

#[tonic::async_trait]
impl CommandService for MockLedgerServer {
    async fn submit_and_wait(
        &self,
        request: Request<SubmitAndWaitRequest>,
    ) -> Result<Response<SubmitAndWaitResponse>, Status> {
        let mut responses = self.responses.lock().await;
        
        if let Some(response) = responses.submit_responses.pop() {
            response.map(Response::new)
        } else {
            Err(Status::internal("No mock response configured"))
        }
    }
    
    // ... other methods
}

/// Test helper for creating mock transactions
pub fn create_mock_transaction(
    transaction_id: &str,
    events: Vec<Event>,
) -> Transaction {
    Transaction {
        transaction_id: transaction_id.to_string(),
        command_id: uuid::Uuid::new_v4().to_string(),
        workflow_id: String::new(),
        effective_at: Some(prost_types::Timestamp::default()),
        events,
        offset: "0".to_string(),
    }
}

/// Test helper for creating mock created events
pub fn create_mock_created_event(
    contract_id: &str,
    template_id: Identifier,
    arguments: Record,
) -> Event {
    Event {
        event: Some(event::Event::Created(CreatedEvent {
            event_id: uuid::Uuid::new_v4().to_string(),
            contract_id: contract_id.to_string(),
            template_id: Some(template_id),
            contract_key: None,
            create_arguments: Some(arguments),
            witness_parties: vec![],
            signatories: vec![],
            observers: vec![],
            created_at: Some(prost_types::Timestamp::default()),
        })),
    }
}
```

### 9.2 Integration Tests

```rust
// tests/grpc_integration.rs

use canton_ledger_api::*;
use testcontainers::{clients::Cli, Container};

struct CantonContainer {
    // Container configuration
}

impl testcontainers::Image for CantonContainer {
    // Implementation
}

#[tokio::test]
#[ignore = "requires Docker"]
async fn test_submit_command() {
    let docker = Cli::default();
    let canton = docker.run(CantonContainer::default());
    
    let endpoint = format!("http://localhost:{}", canton.get_host_port_ipv4(6865));
    
    let client = LedgerClient::connect(ClientConfig {
        endpoint,
        ..Default::default()
    }).await.unwrap();
    
    // Allocate party
    let party = client.allocate_party(Some("test-party"), Some("Test Party"))
        .await
        .unwrap();
    
    // Create command
    let commands = CommandBuilder::new("test-app")
        .act_as(party.party.parse().unwrap())
        .create(
            Identifier::parse("package:Module.Template").unwrap(),
            daml_record! {
                owner: party.party.as_str(),
                value: 100i64,
            },
        )
        .build()
        .unwrap();
    
    // Submit and wait
    let result = client.submit_and_wait(commands).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_with_mock_server() {
    let mock = MockLedgerServer::new();
    
    // Queue expected response
    mock.queue_submit_response(Ok(SubmitAndWaitResponse {
        transaction: Some(create_mock_transaction("tx-1", vec![])),
    })).await;
    
    // Start mock server
    let addr = "127.0.0.1:0".parse().unwrap();
    let server_handle = tokio::spawn(mock.clone().start(addr));
    
    // Create client
    let client = LedgerClient::connect(ClientConfig {
        endpoint: format!("http://{}", addr),
        ..Default::default()
    }).await.unwrap();
    
    // Test
    let commands = CommandBuilder::new("test-app")
        .act_as(PartyId::new("party").unwrap())
        .create(
            Identifier::new("pkg", "Module", "Template"),
            DamlRecord::default(),
        )
        .build()
        .unwrap();
    
    let result = client.submit_and_wait(commands).await;
    assert!(result.is_ok());
    
    server_handle.abort();
}
```

## 10. Performance Optimization

### 10.1 Request Batching

```rust
// src/transport/batching.rs

use std::collections::VecDeque;
use tokio::sync::{mpsc, oneshot};
use tokio::time::{interval, Duration};

/// Batched request sender
pub struct BatchedSender<Req, Res> {
    tx: mpsc::Sender<BatchedRequest<Req, Res>>,
}

struct BatchedRequest<Req, Res> {
    request: Req,
    response_tx: oneshot::Sender<Result<Res, Status>>,
}

/// Batch processor
pub struct BatchProcessor<Req, Res, F> {
    rx: mpsc::Receiver<BatchedRequest<Req, Res>>,
    batch_size: usize,
    batch_timeout: Duration,
    processor: F,
}

impl<Req, Res, F, Fut> BatchProcessor<Req, Res, F>
where
    Req: Send + 'static,
    Res: Send + 'static,
    F: Fn(Vec<Req>) -> Fut + Send + 'static,
    Fut: Future<Output = Vec<Result<Res, Status>>> + Send,
{
    pub async fn run(mut self) {
        let mut batch: Vec<BatchedRequest<Req, Res>> = Vec::with_capacity(self.batch_size);
        let mut ticker = interval(self.batch_timeout);
        
        loop {
            tokio::select! {
                Some(request) = self.rx.recv() => {
                    batch.push(request);
                    
                    if batch.len() >= self.batch_size {
                        self.process_batch(&mut batch).await;
                    }
                }
                _ = ticker.tick() => {
                    if !batch.is_empty() {
                        self.process_batch(&mut batch).await;
                    }
                }
                else => break,
            }
        }
    }
    
    async fn process_batch(&self, batch: &mut Vec<BatchedRequest<Req, Res>>) {
        let requests: Vec<Req> = batch.iter().map(|r| r.request.clone()).collect();
        let results = (self.processor)(requests).await;
        
        for (request, result) in batch.drain(..).zip(results) {
            let _ = request.response_tx.send(result);
        }
    }
}
```

### 10.2 Compression

```rust
// src/transport/compression.rs

use tonic::codec::CompressionEncoding;

/// Configure compression for channel
pub fn configure_compression(
    mut client: impl tonic::client::GrpcService<tonic::body::BoxBody>,
) -> impl tonic::client::GrpcService<tonic::body::BoxBody> {
    // Enable gzip compression for requests
    client = client.send_compressed(CompressionEncoding::Gzip);
    
    // Accept gzip and zstd compressed responses
    client = client.accept_compressed(CompressionEncoding::Gzip);
    client = client.accept_compressed(CompressionEncoding::Zstd);
    
    client
}

/// Compression configuration
#[derive(Debug, Clone)]
pub struct CompressionConfig {
    /// Enable request compression
    pub compress_requests: bool,
    /// Compression algorithm for requests
    pub request_encoding: CompressionEncoding,
    /// Accept compressed responses
    pub accept_compressed: Vec<CompressionEncoding>,
    /// Minimum size for compression (bytes)
    pub min_compression_size: usize,
}

impl Default for CompressionConfig {
    fn default() -> Self {
        Self {
            compress_requests: true,
            request_encoding: CompressionEncoding::Gzip,
            accept_compressed: vec![
                CompressionEncoding::Gzip,
                CompressionEncoding::Zstd,
            ],
            min_compression_size: 1024, // 1KB
        }
    }
}
```
