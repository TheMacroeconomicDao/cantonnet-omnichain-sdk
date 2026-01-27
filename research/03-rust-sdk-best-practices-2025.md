# Rust SDK Best Practices 2025

## 1. Modern Rust Ecosystem Overview

### 1.1 Rust Edition and MSRV

```toml
# Cargo.toml - Recommended configuration for 2025
[package]
name = "canton-sdk"
version = "0.1.0"
edition = "2024"
rust-version = "1.85"  # MSRV - Minimum Supported Rust Version

[package.metadata.docs.rs]
all-features = true
rustdoc-args = ["--cfg", "docsrs"]
```

### 1.2 Key Dependencies for Production SDK (2025)

```toml
[dependencies]
# Async Runtime
tokio = { version = "1.45", features = ["full", "tracing"] }

# gRPC
tonic = { version = "0.13", features = ["tls", "tls-roots", "gzip", "zstd"] }
prost = "0.14"
prost-types = "0.14"

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
bincode = "2.0"

# Error Handling
thiserror = "2.0"
anyhow = "1.0"

# Logging & Tracing
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "json"] }
tracing-opentelemetry = "0.28"
opentelemetry = { version = "0.27", features = ["trace", "metrics"] }
opentelemetry-otlp = "0.27"

# Cryptography
ed25519-dalek = { version = "2.1", features = ["hazmat", "rand_core"] }
x25519-dalek = "2.0"
sha2 = "0.10"
blake3 = "1.5"
aes-gcm = "0.10"
rand = "0.8"
zeroize = { version = "1.8", features = ["derive"] }

# HTTP/TLS
rustls = { version = "0.23", features = ["ring"] }
tokio-rustls = "0.26"
webpki-roots = "0.26"

# Utilities
bytes = "1.7"
uuid = { version = "1.11", features = ["v4", "v7", "serde"] }
chrono = { version = "0.4", features = ["serde"] }
url = { version = "2.5", features = ["serde"] }
dashmap = "6.1"
parking_lot = "0.12"
arc-swap = "1.7"

# Configuration
config = "0.14"
dotenvy = "0.15"

# Validation
validator = { version = "0.19", features = ["derive"] }

[dev-dependencies]
tokio-test = "0.4"
criterion = { version = "0.6", features = ["async_tokio"] }
proptest = "1.5"
wiremock = "0.6"
testcontainers = "0.23"
fake = { version = "3.0", features = ["derive"] }
rstest = "0.23"
```

## 2. Project Structure Best Practices

### 2.1 Workspace Organization

```
canton-sdk/
├── Cargo.toml                    # Workspace root
├── rust-toolchain.toml           # Rust version pinning
├── .cargo/
│   └── config.toml               # Cargo configuration
├── deny.toml                     # cargo-deny configuration
├── clippy.toml                   # Clippy configuration
├── rustfmt.toml                  # Formatting configuration
│
├── crates/
│   ├── canton-sdk/               # Main SDK crate (facade)
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   └── prelude.rs
│   │   └── examples/
│   │
│   ├── canton-core/              # Core types and traits
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── types/
│   │       ├── traits/
│   │       └── error.rs
│   │
│   ├── canton-ledger-api/        # Ledger API client
│   │   ├── Cargo.toml
│   │   ├── build.rs              # Proto compilation
│   │   ├── proto/                # Proto files
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── client.rs
│   │       ├── services/
│   │       └── generated/
│   │
│   ├── canton-crypto/            # Cryptographic operations
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── keys.rs
│   │       ├── signing.rs
│   │       └── encryption.rs
│   │
│   ├── canton-transport/         # Transport layer
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── grpc.rs
│   │       ├── tls.rs
│   │       └── connection.rs
│   │
│   ├── canton-omnichain/         # OmniChain integration
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── adapters/
│   │       ├── bridge.rs
│   │       └── router.rs
│   │
│   └── canton-testing/           # Testing utilities
│       ├── Cargo.toml
│       └── src/
│           ├── lib.rs
│           ├── fixtures.rs
│           └── mocks.rs
│
├── tests/                        # Integration tests
│   ├── integration/
│   └── e2e/
│
├── benches/                      # Benchmarks
│   └── benchmarks.rs
│
├── docs/                         # Documentation
│   ├── architecture.md
│   └── api/
│
└── examples/                     # Example applications
    ├── basic_usage/
    └── cross_chain/
```

### 2.2 Cargo Workspace Configuration

```toml
# Root Cargo.toml
[workspace]
resolver = "2"
members = [
    "crates/*",
]

[workspace.package]
version = "0.1.0"
edition = "2024"
rust-version = "1.85"
license = "Apache-2.0"
repository = "https://github.com/org/canton-sdk"
documentation = "https://docs.rs/canton-sdk"
keywords = ["canton", "blockchain", "sdk", "daml", "omnichain"]
categories = ["cryptography", "api-bindings", "asynchronous"]

[workspace.dependencies]
# Internal crates
canton-core = { path = "crates/canton-core" }
canton-ledger-api = { path = "crates/canton-ledger-api" }
canton-crypto = { path = "crates/canton-crypto" }
canton-transport = { path = "crates/canton-transport" }
canton-omnichain = { path = "crates/canton-omnichain" }
canton-testing = { path = "crates/canton-testing" }

# External dependencies (versions managed here)
tokio = { version = "1.45", default-features = false }
tonic = { version = "0.13", default-features = false }
# ... other dependencies

[workspace.lints.rust]
unsafe_code = "forbid"
missing_docs = "warn"
rust_2024_compatibility = "warn"

[workspace.lints.clippy]
all = "warn"
pedantic = "warn"
nursery = "warn"
cargo = "warn"
# Specific lints
unwrap_used = "deny"
expect_used = "deny"
panic = "deny"
todo = "deny"
unimplemented = "deny"
```

## 3. Error Handling Patterns

### 3.1 Error Type Design

```rust
//! Error handling module

use thiserror::Error;
use std::backtrace::Backtrace;

/// Main SDK error type
#[derive(Error, Debug)]
pub enum SdkError {
    /// Connection-related errors
    #[error("Connection error: {message}")]
    Connection {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
        backtrace: Backtrace,
    },

    /// Authentication errors
    #[error("Authentication failed: {reason}")]
    Authentication {
        reason: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// Transaction errors
    #[error("Transaction error: {kind}")]
    Transaction {
        kind: TransactionErrorKind,
        transaction_id: Option<String>,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// Serialization errors
    #[error("Serialization error: {0}")]
    Serialization(#[from] SerializationError),

    /// Cryptographic errors
    #[error("Cryptographic error: {0}")]
    Crypto(#[from] CryptoError),

    /// Configuration errors
    #[error("Configuration error: {0}")]
    Config(#[from] ConfigError),

    /// Timeout errors
    #[error("Operation timed out after {duration:?}")]
    Timeout {
        duration: std::time::Duration,
        operation: String,
    },

    /// Rate limit exceeded
    #[error("Rate limit exceeded: retry after {retry_after:?}")]
    RateLimited {
        retry_after: Option<std::time::Duration>,
    },

    /// Internal SDK error
    #[error("Internal error: {message}")]
    Internal {
        message: String,
        backtrace: Backtrace,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransactionErrorKind {
    InvalidCommand,
    ContractNotFound,
    ChoiceNotFound,
    AuthorizationFailed,
    Conflict,
    Timeout,
    Unknown,
}

impl std::fmt::Display for TransactionErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidCommand => write!(f, "Invalid command"),
            Self::ContractNotFound => write!(f, "Contract not found"),
            Self::ChoiceNotFound => write!(f, "Choice not found"),
            Self::AuthorizationFailed => write!(f, "Authorization failed"),
            Self::Conflict => write!(f, "Transaction conflict"),
            Self::Timeout => write!(f, "Transaction timeout"),
            Self::Unknown => write!(f, "Unknown error"),
        }
    }
}

/// Result type alias for SDK operations
pub type SdkResult<T> = Result<T, SdkError>;

/// Extension trait for Result types
pub trait ResultExt<T, E> {
    /// Add context to an error
    fn context(self, context: impl Into<String>) -> SdkResult<T>;
    
    /// Add lazy context to an error
    fn with_context<F>(self, f: F) -> SdkResult<T>
    where
        F: FnOnce() -> String;
}

impl<T, E: std::error::Error + Send + Sync + 'static> ResultExt<T, E> for Result<T, E> {
    fn context(self, context: impl Into<String>) -> SdkResult<T> {
        self.map_err(|e| SdkError::Internal {
            message: format!("{}: {}", context.into(), e),
            backtrace: Backtrace::capture(),
        })
    }
    
    fn with_context<F>(self, f: F) -> SdkResult<T>
    where
        F: FnOnce() -> String,
    {
        self.map_err(|e| SdkError::Internal {
            message: format!("{}: {}", f(), e),
            backtrace: Backtrace::capture(),
        })
    }
}
```

### 3.2 Error Recovery Patterns

```rust
/// Retry configuration
#[derive(Debug, Clone)]
pub struct RetryConfig {
    pub max_attempts: u32,
    pub initial_delay: Duration,
    pub max_delay: Duration,
    pub multiplier: f64,
    pub jitter: bool,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(30),
            multiplier: 2.0,
            jitter: true,
        }
    }
}

/// Retry with exponential backoff
pub async fn retry_with_backoff<T, E, F, Fut>(
    config: &RetryConfig,
    mut operation: F,
) -> Result<T, E>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T, E>>,
    E: std::fmt::Debug,
{
    let mut attempt = 0;
    let mut delay = config.initial_delay;
    
    loop {
        attempt += 1;
        
        match operation().await {
            Ok(result) => return Ok(result),
            Err(e) if attempt >= config.max_attempts => {
                tracing::error!(
                    attempt = attempt,
                    error = ?e,
                    "Operation failed after max attempts"
                );
                return Err(e);
            }
            Err(e) => {
                tracing::warn!(
                    attempt = attempt,
                    error = ?e,
                    delay = ?delay,
                    "Operation failed, retrying"
                );
                
                let actual_delay = if config.jitter {
                    let jitter = rand::random::<f64>() * 0.3 + 0.85;
                    delay.mul_f64(jitter)
                } else {
                    delay
                };
                
                tokio::time::sleep(actual_delay).await;
                
                delay = std::cmp::min(
                    delay.mul_f64(config.multiplier),
                    config.max_delay,
                );
            }
        }
    }
}
```

## 4. Async Patterns

### 4.1 Async Trait Design

```rust
use std::future::Future;
use std::pin::Pin;

/// Async service trait using RPITIT (Return Position Impl Trait in Trait)
/// Available in Rust 1.75+
pub trait AsyncService: Send + Sync {
    type Error: std::error::Error + Send + Sync + 'static;
    
    /// Health check
    fn health_check(&self) -> impl Future<Output = Result<(), Self::Error>> + Send;
    
    /// Start the service
    fn start(&self) -> impl Future<Output = Result<(), Self::Error>> + Send;
    
    /// Stop the service
    fn stop(&self) -> impl Future<Output = Result<(), Self::Error>> + Send;
}

/// For object-safe traits, use boxed futures
pub trait DynAsyncService: Send + Sync {
    fn health_check_boxed(&self) -> Pin<Box<dyn Future<Output = Result<(), SdkError>> + Send + '_>>;
}

impl<T: AsyncService<Error = SdkError>> DynAsyncService for T {
    fn health_check_boxed(&self) -> Pin<Box<dyn Future<Output = Result<(), SdkError>> + Send + '_>> {
        Box::pin(self.health_check())
    }
}
```

### 4.2 Stream Processing

```rust
use futures::{Stream, StreamExt, TryStreamExt};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;

/// Event stream processor
pub struct EventProcessor<E> {
    buffer_size: usize,
    _phantom: std::marker::PhantomData<E>,
}

impl<E: Send + 'static> EventProcessor<E> {
    pub fn new(buffer_size: usize) -> Self {
        Self {
            buffer_size,
            _phantom: std::marker::PhantomData,
        }
    }
    
    /// Process events with backpressure
    pub fn process_stream<S, F, Fut, R>(
        &self,
        stream: S,
        processor: F,
    ) -> impl Stream<Item = Result<R, SdkError>>
    where
        S: Stream<Item = Result<E, SdkError>> + Send + 'static,
        F: Fn(E) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<R, SdkError>> + Send,
        R: Send + 'static,
    {
        let (tx, rx) = mpsc::channel(self.buffer_size);
        
        tokio::spawn(async move {
            let mut stream = std::pin::pin!(stream);
            
            while let Some(result) = stream.next().await {
                match result {
                    Ok(event) => {
                        match processor(event).await {
                            Ok(result) => {
                                if tx.send(Ok(result)).await.is_err() {
                                    break;
                                }
                            }
                            Err(e) => {
                                let _ = tx.send(Err(e)).await;
                                break;
                            }
                        }
                    }
                    Err(e) => {
                        let _ = tx.send(Err(e)).await;
                        break;
                    }
                }
            }
        });
        
        ReceiverStream::new(rx)
    }
    
    /// Batch processing with configurable batch size and timeout
    pub fn batch_process<S, F, Fut, R>(
        &self,
        stream: S,
        batch_size: usize,
        timeout: Duration,
        processor: F,
    ) -> impl Stream<Item = Result<Vec<R>, SdkError>>
    where
        S: Stream<Item = Result<E, SdkError>> + Send + 'static,
        F: Fn(Vec<E>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<Vec<R>, SdkError>> + Send,
        R: Send + 'static,
    {
        let (tx, rx) = mpsc::channel(self.buffer_size);
        
        tokio::spawn(async move {
            let mut stream = std::pin::pin!(stream);
            let mut batch = Vec::with_capacity(batch_size);
            let mut interval = tokio::time::interval(timeout);
            
            loop {
                tokio::select! {
                    item = stream.next() => {
                        match item {
                            Some(Ok(event)) => {
                                batch.push(event);
                                if batch.len() >= batch_size {
                                    let events = std::mem::take(&mut batch);
                                    match processor(events).await {
                                        Ok(results) => {
                                            if tx.send(Ok(results)).await.is_err() {
                                                break;
                                            }
                                        }
                                        Err(e) => {
                                            let _ = tx.send(Err(e)).await;
                                            break;
                                        }
                                    }
                                }
                            }
                            Some(Err(e)) => {
                                let _ = tx.send(Err(e)).await;
                                break;
                            }
                            None => {
                                if !batch.is_empty() {
                                    let events = std::mem::take(&mut batch);
                                    let _ = processor(events).await
                                        .map(|r| tx.send(Ok(r)));
                                }
                                break;
                            }
                        }
                    }
                    _ = interval.tick() => {
                        if !batch.is_empty() {
                            let events = std::mem::take(&mut batch);
                            match processor(events).await {
                                Ok(results) => {
                                    if tx.send(Ok(results)).await.is_err() {
                                        break;
                                    }
                                }
                                Err(e) => {
                                    let _ = tx.send(Err(e)).await;
                                    break;
                                }
                            }
                        }
                    }
                }
            }
        });
        
        ReceiverStream::new(rx)
    }
}
```

## 5. Builder Pattern

### 5.1 Type-State Builder

```rust
use std::marker::PhantomData;

/// Type-state markers
pub mod state {
    pub struct NoEndpoint;
    pub struct HasEndpoint;
    pub struct NoAuth;
    pub struct HasAuth;
}

/// Client builder with type-state pattern
pub struct ClientBuilder<Endpoint, Auth> {
    endpoint: Option<String>,
    auth: Option<AuthConfig>,
    timeout: Duration,
    retry_config: RetryConfig,
    _state: PhantomData<(Endpoint, Auth)>,
}

impl ClientBuilder<state::NoEndpoint, state::NoAuth> {
    pub fn new() -> Self {
        Self {
            endpoint: None,
            auth: None,
            timeout: Duration::from_secs(30),
            retry_config: RetryConfig::default(),
            _state: PhantomData,
        }
    }
}

impl<Auth> ClientBuilder<state::NoEndpoint, Auth> {
    pub fn endpoint(self, endpoint: impl Into<String>) -> ClientBuilder<state::HasEndpoint, Auth> {
        ClientBuilder {
            endpoint: Some(endpoint.into()),
            auth: self.auth,
            timeout: self.timeout,
            retry_config: self.retry_config,
            _state: PhantomData,
        }
    }
}

impl<Endpoint> ClientBuilder<Endpoint, state::NoAuth> {
    pub fn auth(self, auth: AuthConfig) -> ClientBuilder<Endpoint, state::HasAuth> {
        ClientBuilder {
            endpoint: self.endpoint,
            auth: Some(auth),
            timeout: self.timeout,
            retry_config: self.retry_config,
            _state: PhantomData,
        }
    }
}

impl<Endpoint, Auth> ClientBuilder<Endpoint, Auth> {
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }
    
    pub fn retry_config(mut self, config: RetryConfig) -> Self {
        self.retry_config = config;
        self
    }
}

impl ClientBuilder<state::HasEndpoint, state::HasAuth> {
    pub async fn build(self) -> SdkResult<Client> {
        let endpoint = self.endpoint.expect("endpoint is set");
        let auth = self.auth.expect("auth is set");
        
        Client::connect(endpoint, auth, self.timeout, self.retry_config).await
    }
}
```

## 6. Testing Patterns

### 6.1 Test Organization

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use rstest::*;
    use proptest::prelude::*;
    
    /// Fixtures using rstest
    #[fixture]
    fn client_config() -> ClientConfig {
        ClientConfig::default()
    }
    
    #[fixture]
    async fn connected_client(client_config: ClientConfig) -> Client {
        Client::connect(client_config).await.unwrap()
    }
    
    /// Parameterized tests
    #[rstest]
    #[case("valid_party_1", true)]
    #[case("valid_party_2", true)]
    #[case("", false)]
    #[case("invalid party!", false)]
    fn test_party_id_validation(#[case] input: &str, #[case] expected: bool) {
        assert_eq!(PartyId::is_valid(input), expected);
    }
    
    /// Async tests with fixtures
    #[rstest]
    #[tokio::test]
    async fn test_submit_command(#[future] connected_client: Client) {
        let client = connected_client.await;
        let command = CreateCommand::new("template", "args");
        
        let result = client.submit(command).await;
        assert!(result.is_ok());
    }
    
    /// Property-based tests
    proptest! {
        #[test]
        fn test_serialization_roundtrip(value: DamlValue) {
            let encoded = value.encode();
            let decoded = DamlValue::decode(&encoded).unwrap();
            prop_assert_eq!(value, decoded);
        }
        
        #[test]
        fn test_contract_id_format(s in "[a-zA-Z0-9]{1,64}") {
            let id = ContractId::new(&s);
            prop_assert!(id.is_ok());
        }
    }
}

/// Integration tests module
#[cfg(test)]
mod integration_tests {
    use super::*;
    use testcontainers::{clients::Cli, Container};
    
    struct CantonContainer {
        // Container configuration
    }
    
    #[tokio::test]
    #[ignore = "requires Docker"]
    async fn test_full_workflow() {
        let docker = Cli::default();
        let canton = docker.run(CantonContainer::default());
        
        let client = Client::connect(canton.endpoint()).await.unwrap();
        
        // Test full workflow
        let party = client.allocate_party("test-party").await.unwrap();
        let contract = client.create_contract(&party, "Template", args).await.unwrap();
        let result = client.exercise(&contract, "Choice", args).await.unwrap();
        
        assert!(result.is_success());
    }
}
```

### 6.2 Mock Implementations

```rust
use mockall::automock;

#[automock]
pub trait LedgerClient: Send + Sync {
    async fn submit_command(&self, command: Command) -> SdkResult<CommandResult>;
    async fn get_transaction(&self, id: &str) -> SdkResult<Transaction>;
    fn transaction_stream(&self) -> Box<dyn Stream<Item = Transaction> + Send>;
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_with_mock() {
        let mut mock = MockLedgerClient::new();
        
        mock.expect_submit_command()
            .times(1)
            .returning(|_| Ok(CommandResult::success()));
        
        let service = MyService::new(Arc::new(mock));
        let result = service.do_something().await;
        
        assert!(result.is_ok());
    }
}
```

## 7. Documentation Standards

### 7.1 Module Documentation

```rust
//! # Canton SDK
//!
//! A production-ready Rust SDK for interacting with Canton Network.
//!
//! ## Features
//!
//! - Full Ledger API support
//! - OmniChain integration
//! - Type-safe command building
//! - Async/await support
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use canton_sdk::prelude::*;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let client = CantonClient::builder()
//!         .endpoint("https://canton.example.com")
//!         .auth(AuthConfig::token("your-token"))
//!         .build()
//!         .await?;
//!
//!     let party = client.allocate_party("my-party").await?;
//!     println!("Allocated party: {}", party.id());
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Architecture
//!
//! The SDK is organized into several layers:
//!
//! - **Core**: Basic types and traits
//! - **Transport**: gRPC communication
//! - **Ledger API**: Canton Ledger API client
//! - **OmniChain**: Cross-chain integration
//!
//! ## Error Handling
//!
//! All operations return [`SdkResult<T>`], which is an alias for
//! `Result<T, SdkError>`. See [`SdkError`] for error variants.

/// Creates a new contract on the ledger.
///
/// # Arguments
///
/// * `template_id` - The fully qualified template identifier
/// * `arguments` - The contract arguments as a Daml record
///
/// # Returns
///
/// Returns the created contract ID on success.
///
/// # Errors
///
/// Returns [`SdkError::Transaction`] if:
/// - The template is not found
/// - Authorization fails
/// - The arguments are invalid
///
/// # Examples
///
/// ```rust,no_run
/// # use canton_sdk::prelude::*;
/// # async fn example(client: &CantonClient) -> SdkResult<()> {
/// let contract_id = client.create_contract(
///     "MyModule:MyTemplate",
///     daml_args! {
///         owner: "party-123",
///         amount: 100,
///     },
/// ).await?;
///
/// println!("Created contract: {}", contract_id);
/// # Ok(())
/// # }
/// ```
///
/// # See Also
///
/// - [`exercise_choice`] - Exercise a choice on a contract
/// - [`archive_contract`] - Archive a contract
pub async fn create_contract(
    &self,
    template_id: impl Into<TemplateId>,
    arguments: DamlRecord,
) -> SdkResult<ContractId> {
    // Implementation
}
```

## 8. Performance Optimization

### 8.1 Connection Pooling

```rust
use std::sync::Arc;
use dashmap::DashMap;
use tokio::sync::Semaphore;

/// Connection pool for gRPC channels
pub struct ConnectionPool {
    connections: DashMap<String, Arc<Channel>>,
    semaphore: Arc<Semaphore>,
    config: PoolConfig,
}

#[derive(Debug, Clone)]
pub struct PoolConfig {
    pub max_connections: usize,
    pub idle_timeout: Duration,
    pub connection_timeout: Duration,
}

impl ConnectionPool {
    pub fn new(config: PoolConfig) -> Self {
        Self {
            connections: DashMap::new(),
            semaphore: Arc::new(Semaphore::new(config.max_connections)),
            config,
        }
    }
    
    pub async fn get_connection(&self, endpoint: &str) -> SdkResult<Arc<Channel>> {
        // Check existing connection
        if let Some(conn) = self.connections.get(endpoint) {
            return Ok(conn.clone());
        }
        
        // Acquire permit
        let _permit = self.semaphore.acquire().await
            .map_err(|_| SdkError::Internal {
                message: "Connection pool exhausted".into(),
                backtrace: std::backtrace::Backtrace::capture(),
            })?;
        
        // Double-check after acquiring permit
        if let Some(conn) = self.connections.get(endpoint) {
            return Ok(conn.clone());
        }
        
        // Create new connection
        let channel = Channel::from_shared(endpoint.to_string())
            .map_err(|e| SdkError::Connection {
                message: format!("Invalid endpoint: {}", e),
                source: Some(Box::new(e)),
                backtrace: std::backtrace::Backtrace::capture(),
            })?
            .connect_timeout(self.config.connection_timeout)
            .connect()
            .await
            .map_err(|e| SdkError::Connection {
                message: format!("Failed to connect: {}", e),
                source: Some(Box::new(e)),
                backtrace: std::backtrace::Backtrace::capture(),
            })?;
        
        let channel = Arc::new(channel);
        self.connections.insert(endpoint.to_string(), channel.clone());
        
        Ok(channel)
    }
}
```

### 8.2 Caching

```rust
use std::hash::Hash;
use std::time::{Duration, Instant};
use dashmap::DashMap;

/// Time-based cache with automatic expiration
pub struct Cache<K, V> {
    entries: DashMap<K, CacheEntry<V>>,
    default_ttl: Duration,
}

struct CacheEntry<V> {
    value: V,
    expires_at: Instant,
}

impl<K: Eq + Hash, V: Clone> Cache<K, V> {
    pub fn new(default_ttl: Duration) -> Self {
        Self {
            entries: DashMap::new(),
            default_ttl,
        }
    }
    
    pub fn get(&self, key: &K) -> Option<V> {
        self.entries.get(key).and_then(|entry| {
            if entry.expires_at > Instant::now() {
                Some(entry.value.clone())
            } else {
                None
            }
        })
    }
    
    pub fn insert(&self, key: K, value: V) {
        self.insert_with_ttl(key, value, self.default_ttl);
    }
    
    pub fn insert_with_ttl(&self, key: K, value: V, ttl: Duration) {
        self.entries.insert(key, CacheEntry {
            value,
            expires_at: Instant::now() + ttl,
        });
    }
    
    pub fn invalidate(&self, key: &K) {
        self.entries.remove(key);
    }
    
    pub fn clear(&self) {
        self.entries.clear();
    }
    
    /// Remove expired entries
    pub fn cleanup(&self) {
        let now = Instant::now();
        self.entries.retain(|_, entry| entry.expires_at > now);
    }
}
```

## 9. Observability

### 9.1 Tracing Integration

```rust
use tracing::{instrument, Span};
use tracing_opentelemetry::OpenTelemetrySpanExt;

/// Instrumented client wrapper
pub struct InstrumentedClient {
    inner: Client,
}

impl InstrumentedClient {
    #[instrument(
        name = "canton.submit_command",
        skip(self, command),
        fields(
            command_id = %command.id(),
            template_id = %command.template_id(),
            otel.kind = "client",
        )
    )]
    pub async fn submit_command(&self, command: Command) -> SdkResult<CommandResult> {
        let span = Span::current();
        
        // Add custom attributes
        span.set_attribute("canton.party", command.party().to_string());
        
        let result = self.inner.submit_command(command).await;
        
        match &result {
            Ok(r) => {
                span.set_attribute("canton.transaction_id", r.transaction_id().to_string());
                span.set_status(opentelemetry::trace::Status::Ok);
            }
            Err(e) => {
                span.set_status(opentelemetry::trace::Status::error(e.to_string()));
                span.record_error(e);
            }
        }
        
        result
    }
}
```

### 9.2 Metrics

```rust
use opentelemetry::metrics::{Counter, Histogram, Meter};
use std::sync::Arc;

/// SDK metrics
pub struct SdkMetrics {
    commands_submitted: Counter<u64>,
    command_latency: Histogram<f64>,
    active_connections: Counter<i64>,
    errors: Counter<u64>,
}

impl SdkMetrics {
    pub fn new(meter: &Meter) -> Self {
        Self {
            commands_submitted: meter
                .u64_counter("canton_sdk_commands_submitted")
                .with_description("Total number of commands submitted")
                .init(),
            command_latency: meter
                .f64_histogram("canton_sdk_command_latency_seconds")
                .with_description("Command submission latency in seconds")
                .init(),
            active_connections: meter
                .i64_counter("canton_sdk_active_connections")
                .with_description("Number of active connections")
                .init(),
            errors: meter
                .u64_counter("canton_sdk_errors")
                .with_description("Total number of errors")
                .init(),
        }
    }
    
    pub fn record_command(&self, latency: Duration, success: bool) {
        self.commands_submitted.add(1, &[]);
        self.command_latency.record(latency.as_secs_f64(), &[]);
        
        if !success {
            self.errors.add(1, &[]);
        }
    }
}
```

## 10. Security Best Practices

### 10.1 Secret Management

```rust
use zeroize::{Zeroize, ZeroizeOnDrop};

/// Secure secret storage
#[derive(Zeroize, ZeroizeOnDrop)]
pub struct Secret {
    #[zeroize(skip)]
    _marker: std::marker::PhantomData<()>,
    value: Vec<u8>,
}

impl Secret {
    pub fn new(value: impl Into<Vec<u8>>) -> Self {
        Self {
            _marker: std::marker::PhantomData,
            value: value.into(),
        }
    }
    
    pub fn expose(&self) -> &[u8] {
        &self.value
    }
}

impl std::fmt::Debug for Secret {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Secret")
            .field("value", &"[REDACTED]")
            .finish()
    }
}

/// Secure configuration
pub struct SecureConfig {
    pub endpoint: String,
    pub token: Secret,
    pub private_key: Secret,
}
```

### 10.2 Input Validation

```rust
use validator::{Validate, ValidationError};

#[derive(Debug, Validate)]
pub struct CommandRequest {
    #[validate(length(min = 1, max = 256))]
    pub command_id: String,
    
    #[validate(custom(function = "validate_template_id"))]
    pub template_id: String,
    
    #[validate(length(min = 1, max = 64))]
    pub party: String,
    
    #[validate]
    pub arguments: DamlRecord,
}

fn validate_template_id(id: &str) -> Result<(), ValidationError> {
    let parts: Vec<&str> = id.split(':').collect();
    if parts.len() != 2 {
        return Err(ValidationError::new("invalid_template_id"));
    }
    
    for part in parts {
        if part.is_empty() || !part.chars().all(|c| c.is_alphanumeric() || c == '.') {
            return Err(ValidationError::new("invalid_template_id"));
        }
    }
    
    Ok(())
}
```

## 11. Feature Flags

```toml
# Cargo.toml feature configuration
[features]
default = ["tls", "compression"]

# TLS support
tls = ["tonic/tls", "tonic/tls-roots", "rustls"]

# Compression
compression = ["tonic/gzip", "tonic/zstd"]

# OmniChain adapters
omnichain = ["canton-omnichain"]
ethereum = ["omnichain", "ethers"]
cosmos = ["omnichain", "cosmrs"]

# Observability
telemetry = ["tracing-opentelemetry", "opentelemetry", "opentelemetry-otlp"]

# Development features
dev = ["testcontainers", "wiremock"]
```

## 12. CI/CD Configuration

```yaml
# .github/workflows/ci.yml
name: CI

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

env:
  CARGO_TERM_COLOR: always
  RUSTFLAGS: -Dwarnings

jobs:
  check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - run: cargo check --all-features

  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - run: cargo test --all-features

  clippy:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: clippy
      - uses: Swatinem/rust-cache@v2
      - run: cargo clippy --all-features -- -D warnings

  fmt:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt
      - run: cargo fmt --all -- --check

  docs:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - run: cargo doc --all-features --no-deps
        env:
          RUSTDOCFLAGS: -Dwarnings

  security:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: rustsec/audit-check@v2
        with:
          token: ${{ secrets.GITHUB_TOKEN }}

  deny:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: EmbarkStudios/cargo-deny-action@v2
```
