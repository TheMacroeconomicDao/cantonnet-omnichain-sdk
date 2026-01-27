# Canton OmniChain SDK - Production Development Prompt

> **Version**: 1.0.0  
> **Target**: Rust 2024 Edition, MSRV 1.85  
> **Standard**: Production-Ready, Enterprise-Grade  
> **Last Updated**: January 2025

---

## 🎯 MISSION

You are a Senior Rust Developer and Blockchain Architect tasked with implementing a **production-ready SDK platform for OmniChain integration with Canton Network**. This SDK must meet enterprise standards for reliability, security, performance, and maintainability.

---

## 📋 PROJECT OVERVIEW

### Product Definition

**Canton OmniChain SDK** is a comprehensive Rust library that provides:

1. **Canton Network Integration** - Full Ledger API client for Daml smart contracts
2. **OmniChain Capabilities** - Cross-chain interoperability with Ethereum, Cosmos, Substrate
3. **Enterprise Features** - Production-ready reliability, observability, and security patterns

### Key Characteristics

- **Type-Safe**: Leverage Rust's type system for compile-time guarantees
- **Async-First**: Built on Tokio for high-performance async operations
- **Modular**: Feature-flagged components for flexible deployment
- **Observable**: Comprehensive metrics, tracing, and logging
- **Resilient**: Circuit breakers, rate limiting, retry policies
- **Secure**: HSM support, secure key management, encryption

---

## 🏗️ ARCHITECTURE

### Workspace Structure

```
canton-sdk/
├── Cargo.toml                          # Workspace root
├── rust-toolchain.toml                 # Rust 1.85+
├── deny.toml                           # cargo-deny config
├── clippy.toml                         # Clippy config
├── rustfmt.toml                        # Format config
│
├── crates/
│   ├── canton-sdk/                     # Main facade (re-exports)
│   ├── canton-core/                    # Core types, traits, errors
│   ├── canton-ledger-api/              # gRPC Ledger API client
│   ├── canton-crypto/                  # Cryptographic operations
│   ├── canton-transport/               # gRPC transport layer
│   ├── canton-reliability/             # Circuit breaker, rate limiter
│   ├── canton-observability/           # Logging, metrics, tracing
│   ├── canton-omnichain/               # Cross-chain integration
│   └── canton-testing/                 # Test utilities
│
├── tests/                              # Integration tests
├── benches/                            # Benchmarks
├── examples/                           # Usage examples
└── docs/                               # Documentation
```

### Layer Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                      Public API Layer                            │
│  CantonSdk, CantonSdkBuilder, OmniChainClient                   │
├─────────────────────────────────────────────────────────────────┤
│                      Core Domain Layer                           │
│  DamlValue, Commands, Transactions, Events, Identifiers         │
├─────────────────────────────────────────────────────────────────┤
│                      Service Layer                               │
│  CommandService, TransactionService, PartyService, BridgeService│
├─────────────────────────────────────────────────────────────────┤
│                      Infrastructure Layer                        │
│  Transport, Crypto, Cache, Metrics, Reliability                 │
├─────────────────────────────────────────────────────────────────┤
│                      Adapter Layer                               │
│  CantonAdapter, EthereumAdapter, CosmosAdapter, SubstrateAdapter│
└─────────────────────────────────────────────────────────────────┘
```

---

## 📦 CRATE SPECIFICATIONS

### 1. canton-core

**Purpose**: Core types, traits, and error definitions

```toml
[package]
name = "canton-core"
version = "0.1.0"
edition = "2024"

[dependencies]
thiserror = "2.0"
serde = { version = "1.0", features = ["derive"] }
chrono = { version = "0.4", features = ["serde"] }
rust_decimal = { version = "1.36", features = ["serde"] }
uuid = { version = "1.11", features = ["v4", "v7", "serde"] }
bytes = "1.7"
```

**Key Types**:

```rust
// Value types
pub enum DamlValue {
    Unit,
    Bool(bool),
    Int64(i64),
    Numeric(Decimal),
    Text(String),
    Timestamp(DateTime<Utc>),
    Date(NaiveDate),
    Party(PartyId),
    ContractId(ContractId),
    List(Vec<DamlValue>),
    Optional(Option<Box<DamlValue>>),
    TextMap(HashMap<String, DamlValue>),
    GenMap(Vec<(DamlValue, DamlValue)>),
    Record(DamlRecord),
    Variant(DamlVariant),
    Enum(DamlEnum),
}

// Identifiers
pub struct Identifier { package_id, module_name, entity_name }
pub struct PartyId(String);  // Validated
pub struct ContractId(String);

// Commands
pub enum Command { Create, Exercise, ExerciseByKey, CreateAndExercise }
pub struct Commands { workflow_id, application_id, command_id, act_as, read_as, commands }

// Events
pub enum Event { Created(CreatedEvent), Archived(ArchivedEvent) }
pub struct CreatedEvent { event_id, contract_id, template_id, create_arguments, ... }

// Errors
pub enum SdkError {
    Connection { message, source, backtrace },
    Authentication { reason, source },
    Transaction { kind, transaction_id, details, source },
    Validation { field, message },
    Config(String),
    Timeout { duration, operation },
    RateLimited { retry_after },
    CircuitOpen,
    CrossChain { message, source_chain, target_chain, source },
    Internal { message, backtrace },
}
```

### 2. canton-ledger-api

**Purpose**: gRPC client for Canton Ledger API

```toml
[package]
name = "canton-ledger-api"

[dependencies]
canton-core = { workspace = true }
tonic = { version = "0.13", features = ["tls", "tls-roots", "gzip", "zstd"] }
prost = "0.14"
prost-types = "0.14"
tokio = { version = "1.45", features = ["rt-multi-thread"] }
tokio-stream = "0.1"
futures = "0.3"

[build-dependencies]
tonic-build = "0.13"
```

**Proto Files Required**:
- `com/daml/ledger/api/v2/command_service.proto`
- `com/daml/ledger/api/v2/command_submission_service.proto`
- `com/daml/ledger/api/v2/command_completion_service.proto`
- `com/daml/ledger/api/v2/transaction_service.proto`
- `com/daml/ledger/api/v2/active_contracts_service.proto`
- `com/daml/ledger/api/v2/party_management_service.proto`
- `com/daml/ledger/api/v2/package_service.proto`
- `com/daml/ledger/api/v2/ledger_identity_service.proto`

**Key Implementation**:

```rust
pub struct LedgerClient {
    channel: Channel,
    ledger_id: String,
    services: LedgerServices,
    reliability: ReliabilityComponents,
    metrics: Arc<SdkMetrics>,
}

impl LedgerClient {
    pub async fn connect(config: &CantonConfig) -> SdkResult<Self>;
    pub fn ledger_id(&self) -> &str;
    
    // Command operations
    pub async fn submit_and_wait(&self, commands: Commands) -> SdkResult<Transaction>;
    pub async fn submit(&self, commands: Commands) -> SdkResult<()>;
    
    // Transaction streaming
    pub fn get_transactions(&self, begin, end, filter) -> impl Stream<Item = SdkResult<Transaction>>;
    pub fn get_transaction_trees(&self, begin, end, filter) -> impl Stream<Item = SdkResult<TransactionTree>>;
    
    // Active contracts
    pub async fn get_active_contracts(&self, filter) -> SdkResult<Vec<CreatedEvent>>;
    
    // Party management
    pub async fn allocate_party(&self, hint, display_name) -> SdkResult<PartyDetails>;
    pub async fn list_known_parties(&self) -> SdkResult<Vec<PartyDetails>>;
    
    // Utilities
    pub async fn get_ledger_end(&self) -> SdkResult<LedgerOffset>;
}
```

### 3. canton-crypto

**Purpose**: Cryptographic operations and key management

```toml
[dependencies]
ed25519-dalek = { version = "2.1", features = ["hazmat", "rand_core"] }
x25519-dalek = "2.0"
p256 = { version = "0.13", features = ["ecdsa"] }
k256 = { version = "0.13", features = ["ecdsa"] }
sha2 = "0.10"
sha3 = "0.10"
blake2 = "0.10"
blake3 = "1.5"
aes-gcm = "0.10"
hkdf = "0.12"
argon2 = "0.5"
rand = "0.8"
zeroize = { version = "1.8", features = ["derive"] }
```

**Key Components**:

```rust
// Key types
pub enum KeyAlgorithm { Ed25519, EcdsaP256, EcdsaSecp256k1, X25519 }
pub enum KeyPurpose { Signing, Encryption, NamespaceDelegation, IdentityBinding }

// Key store trait
#[async_trait]
pub trait KeyStore: Send + Sync {
    async fn generate_key(&self, algorithm, purpose, metadata) -> Result<KeyFingerprint>;
    async fn sign(&self, fingerprint, data) -> Result<Signature>;
    async fn verify(&self, fingerprint, data, signature) -> Result<bool>;
    async fn export_public_key(&self, fingerprint) -> Result<Vec<u8>>;
}

// Implementations
pub struct InMemoryKeyStore;      // Development
pub struct FileKeyStore;          // File-based
pub struct HsmKeyStore;           // HSM integration (Vault, CloudHSM)

// Encryption
pub fn encrypt(key: &EncryptionKey, plaintext: &[u8]) -> Result<EncryptedData>;
pub fn decrypt(key: &EncryptionKey, encrypted: &EncryptedData) -> Result<Vec<u8>>;
pub fn ecies_encrypt(public_key: &[u8; 32], plaintext: &[u8]) -> Result<EciesMessage>;

// Hashing
pub fn hash(algorithm: HashAlgorithm, data: &[u8]) -> Vec<u8>;
pub struct MerkleTree { /* Merkle tree with proof generation */ }
```

### 4. canton-transport

**Purpose**: gRPC transport, connection pooling, interceptors

```toml
[dependencies]
tonic = { version = "0.13", features = ["tls", "tls-roots", "gzip", "zstd"] }
tower = { version = "0.5", features = ["util", "timeout", "retry"] }
rustls = { version = "0.23", features = ["ring"] }
tokio-rustls = "0.26"
hyper = "1.5"
http = "1.1"
```

**Key Components**:

```rust
// Channel builder
pub struct ChannelBuilder {
    pub fn new(endpoint: impl Into<String>) -> Self;
    pub fn tls(self, config: TlsConfig) -> Self;
    pub fn connect_timeout(self, timeout: Duration) -> Self;
    pub fn request_timeout(self, timeout: Duration) -> Self;
    pub fn keep_alive_interval(self, interval: Duration) -> Self;
    pub async fn build(self) -> Result<Channel>;
}

// Connection pool
pub struct ConnectionPool {
    pub fn new(config: PoolConfig) -> Self;
    pub async fn get(&self, endpoint: &str) -> Result<Channel>;
    pub async fn cleanup(&self);
}

// Interceptors
pub struct AuthInterceptor;       // JWT/mTLS authentication
pub struct TracingInterceptor;    // Distributed tracing
pub struct MetricsInterceptor;    // Request metrics
```

### 5. canton-reliability

**Purpose**: Fault tolerance patterns

```toml
[dependencies]
tokio = { version = "1.45", features = ["sync", "time"] }
```

**Key Components**:

```rust
// Circuit breaker
pub struct CircuitBreaker {
    pub fn new(config: CircuitBreakerConfig) -> Self;
    pub async fn allow_request(&self) -> Result<CircuitBreakerGuard>;
    pub async fn state(&self) -> CircuitState;
}

pub enum CircuitState { Closed, Open, HalfOpen }

pub struct CircuitBreakerConfig {
    pub failure_threshold: u32,      // Default: 5
    pub success_threshold: u32,      // Default: 3
    pub reset_timeout: Duration,     // Default: 30s
    pub window_duration: Duration,   // Default: 60s
}

// Rate limiter
pub struct RateLimiter {
    pub fn new(config: RateLimiterConfig) -> Self;
    pub async fn try_acquire(&self) -> Result<()>;
    pub async fn acquire(&self) -> Result<()>;
}

pub enum RateLimitStrategy { FixedWindow, SlidingWindowLog, TokenBucket, LeakyBucket }

// Retry policy
pub struct RetryPolicy {
    pub max_attempts: u32,
    pub initial_delay: Duration,
    pub max_delay: Duration,
    pub multiplier: f64,
    pub jitter: bool,
}

impl RetryPolicy {
    pub async fn execute<F, T, E>(&self, operation: F) -> Result<T, E>
    where F: FnMut() -> Future<Output = Result<T, E>>;
}

// Bulkhead
pub struct Bulkhead {
    pub fn new(name: &str, config: BulkheadConfig) -> Self;
    pub async fn execute<F, T, E>(&self, f: F) -> Result<T, BulkheadError<E>>;
}
```

### 6. canton-observability

**Purpose**: Logging, metrics, tracing, health checks

```toml
[dependencies]
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "json"] }
tracing-opentelemetry = "0.28"
opentelemetry = { version = "0.27", features = ["trace", "metrics"] }
opentelemetry-otlp = "0.27"
```

**Key Components**:

```rust
// Logging
pub fn init_logging(config: &LoggingConfig) -> Result<()>;

pub struct LoggingConfig {
    pub level: String,           // "info", "debug", etc.
    pub format: LogFormat,       // Pretty, Compact, Json
    pub span_events: bool,
    pub file_info: bool,
}

// Metrics
pub struct SdkMetrics {
    pub commands_submitted: Counter<u64>,
    pub command_latency: Histogram<f64>,
    pub active_connections: UpDownCounter<i64>,
    pub events_processed: Counter<u64>,
    pub errors: Counter<u64>,
    pub cache_hits: Counter<u64>,
    pub cache_misses: Counter<u64>,
}

// Health checks
#[async_trait]
pub trait HealthCheck: Send + Sync {
    fn name(&self) -> &str;
    async fn check(&self) -> HealthCheckResult;
}

pub struct HealthCheckRegistry {
    pub async fn register(&self, check: Arc<dyn HealthCheck>);
    pub async fn check_all(&self) -> OverallHealth;
}
```

### 7. canton-omnichain

**Purpose**: Cross-chain integration

```toml
[dependencies]
canton-core = { workspace = true }
canton-ledger-api = { workspace = true }
canton-crypto = { workspace = true }

[features]
default = []
ethereum = ["ethers"]
cosmos = ["cosmrs"]
substrate = ["subxt"]
```

**Key Components**:

```rust
// OmniChain client
pub struct OmniChainClient {
    pub async fn new(config: &OmniChainConfig, canton: Arc<LedgerClient>) -> SdkResult<Self>;
    pub fn canton(&self) -> &LedgerClient;
    pub fn adapter(&self, chain_id: &ChainId) -> Option<&dyn ChainAdapter>;
    
    pub async fn transfer(&self, request: CrossChainTransferRequest) -> SdkResult<CrossChainTransferResult>;
    pub async fn call(&self, request: CrossChainCallRequest) -> SdkResult<CrossChainCallResult>;
    pub async fn query(&self, request: CrossChainQueryRequest) -> SdkResult<CrossChainQueryResult>;
    pub fn subscribe_events(&self, filter: CrossChainEventFilter) -> impl Stream<Item = SdkResult<CrossChainEvent>>;
}

// Chain adapter trait
#[async_trait]
pub trait ChainAdapter: Send + Sync {
    fn chain_id(&self) -> ChainId;
    fn chain_type(&self) -> ChainType;
    async fn is_connected(&self) -> bool;
    async fn block_height(&self) -> SdkResult<u64>;
    async fn submit_transaction(&self, tx: ChainTransaction) -> SdkResult<TransactionReceipt>;
    async fn query_state(&self, query: StateQuery) -> SdkResult<StateQueryResult>;
    async fn verify_proof(&self, proof: &ChainProof) -> SdkResult<bool>;
    async fn generate_proof(&self, request: ProofRequest) -> SdkResult<ChainProof>;
    fn subscribe_events(&self, filter: EventFilter) -> Box<dyn Stream<Item = SdkResult<ChainEvent>>>;
}

// Adapters
pub struct CantonAdapter;
pub struct EthereumAdapter;   // feature = "ethereum"
pub struct CosmosAdapter;     // feature = "cosmos"
pub struct SubstrateAdapter;  // feature = "substrate"

// Bridge
pub struct Bridge {
    pub async fn execute_transfer(&self, request: CrossChainTransferRequest) -> SdkResult<CrossChainTransferResult>;
    pub async fn status(&self) -> SdkResult<BridgeStatus>;
}

// Message router
pub struct MessageRouter {
    pub async fn route_call(&self, request: CrossChainCallRequest) -> SdkResult<CrossChainCallResult>;
    pub async fn route_query(&self, request: CrossChainQueryRequest) -> SdkResult<CrossChainQueryResult>;
}
```

### 8. canton-sdk (Facade)

**Purpose**: Main entry point, re-exports

```rust
// lib.rs
pub use canton_core as core;
pub use canton_ledger_api as ledger_api;
pub use canton_crypto as crypto;
pub use canton_transport as transport;
pub use canton_reliability as reliability;
pub use canton_observability as observability;

#[cfg(feature = "omnichain")]
pub use canton_omnichain as omnichain;

pub mod prelude;

pub struct CantonSdk {
    config: SdkConfig,
    ledger: Arc<LedgerClient>,
    omnichain: Option<Arc<OmniChainClient>>,
    shutdown: Arc<ShutdownCoordinator>,
}

impl CantonSdk {
    pub fn builder() -> CantonSdkBuilder;
    pub fn ledger(&self) -> &LedgerClient;
    pub fn omnichain(&self) -> Option<&OmniChainClient>;
    pub async fn shutdown(&self, timeout: Duration);
}

pub struct CantonSdkBuilder {
    pub fn new() -> Self;
    pub fn config(self, config: SdkConfig) -> Self;
    pub fn config_file(self, path: impl Into<String>) -> Self;
    pub fn with_omnichain(self) -> Self;
    pub async fn build(self) -> SdkResult<CantonSdk>;
}
```

---

## 🔧 IMPLEMENTATION REQUIREMENTS

### Code Quality Standards

1. **No `unwrap()` or `expect()` in library code** - Use proper error handling
2. **No `panic!()` or `todo!()` in production code** - All paths must be handled
3. **All public APIs must be documented** - With examples where appropriate
4. **All types must implement `Debug`** - For troubleshooting
5. **Sensitive data must use `Zeroize`** - Keys, tokens, secrets
6. **All async functions must be `Send + Sync`** - For multi-threaded use

### Error Handling

```rust
// Always use Result with SdkError
pub async fn operation() -> SdkResult<T> {
    // Use ? operator for propagation
    let result = inner_operation().await?;
    
    // Add context when needed
    let data = parse_data(&input)
        .map_err(|e| SdkError::Validation {
            field: "input".into(),
            message: e.to_string(),
        })?;
    
    Ok(result)
}
```

### Logging Standards

```rust
// Use structured logging with tracing
#[tracing::instrument(skip(self, commands), fields(command_id = %commands.command_id))]
pub async fn submit_and_wait(&self, commands: Commands) -> SdkResult<Transaction> {
    tracing::debug!("Submitting command");
    
    let result = self.do_submit(&commands).await;
    
    match &result {
        Ok(tx) => tracing::info!(transaction_id = %tx.transaction_id, "Command succeeded"),
        Err(e) => tracing::error!(error = %e, "Command failed"),
    }
    
    result
}
```

### Testing Requirements

1. **Unit tests** for all public functions
2. **Integration tests** with mock servers
3. **Property-based tests** for serialization
4. **Benchmark tests** for performance-critical paths

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use rstest::*;
    use proptest::prelude::*;
    
    #[rstest]
    #[case("valid_party", true)]
    #[case("", false)]
    fn test_party_id_validation(#[case] input: &str, #[case] expected: bool) {
        assert_eq!(PartyId::new(input).is_ok(), expected);
    }
    
    proptest! {
        #[test]
        fn test_value_roundtrip(value in arb_daml_value()) {
            let encoded = value.to_proto();
            let decoded = DamlValue::from_proto(encoded).unwrap();
            prop_assert_eq!(value, decoded);
        }
    }
}
```

---

## 📝 CONFIGURATION

### Default Configuration (YAML)

```yaml
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

reliability:
  circuit_breaker:
    failure_threshold: 5
    success_threshold: 3
    reset_timeout: "30s"
  rate_limiter:
    max_requests: 100
    window: "1s"
    strategy: "token_bucket"
  retry:
    max_attempts: 3
    initial_delay: "100ms"
    max_delay: "10s"
    multiplier: 2.0

observability:
  logging:
    level: "info"
    format: "json"
  metrics:
    endpoint: "http://otel-collector:4317"
    export_interval: "10s"
  tracing:
    endpoint: "http://otel-collector:4317"
    sample_rate: 0.1

omnichain:
  enabled_chains:
    - ethereum
    - cosmos
  chains:
    ethereum:
      rpc_url: "https://eth.example.com"
      chain_id: 1
    cosmos:
      rpc_url: "https://cosmos.example.com"
      chain_id: "cosmoshub-4"
```

---

## 🚀 USAGE EXAMPLES

### Basic Usage

```rust
use canton_sdk::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize SDK
    let sdk = CantonSdk::builder()
        .config_file("config/production.yaml")
        .build()
        .await?;
    
    // Allocate party
    let party = sdk.ledger()
        .allocate_party(Some("my-party"), Some("My Party"))
        .await?;
    
    // Create contract
    let commands = CommandBuilder::new("my-app")
        .act_as(party.party.parse()?)
        .create(
            "package:Module.Template".parse()?,
            daml_record! {
                owner: party.party.as_str(),
                amount: 100i64,
            },
        )
        .build()?;
    
    let transaction = sdk.ledger().submit_and_wait(commands).await?;
    println!("Created contract: {:?}", transaction);
    
    // Graceful shutdown
    sdk.shutdown(Duration::from_secs(30)).await;
    
    Ok(())
}
```

### Transaction Streaming

```rust
use canton_sdk::prelude::*;
use futures::StreamExt;

async fn process_transactions(sdk: &CantonSdk, party: &PartyId) -> SdkResult<()> {
    let filter = TransactionFilterBuilder::new()
        .for_party(party.clone())
            .template("package:Module.Template".parse()?)
            .done()
        .build();
    
    let mut stream = sdk.ledger().get_transactions(
        LedgerOffset::Begin,
        None,
        filter,
    );
    
    while let Some(result) = stream.next().await {
        let transaction = result?;
        
        for event in transaction.events {
            match event {
                Event::Created(created) => {
                    println!("Contract created: {}", created.contract_id);
                }
                Event::Archived(archived) => {
                    println!("Contract archived: {}", archived.contract_id);
                }
            }
        }
    }
    
    Ok(())
}
```

### Cross-Chain Transfer

```rust
use canton_sdk::prelude::*;

async fn cross_chain_transfer(sdk: &CantonSdk) -> SdkResult<()> {
    let omnichain = sdk.omnichain()
        .ok_or(SdkError::Config("OmniChain not enabled".into()))?;
    
    let request = CrossChainTransferRequest {
        source_chain: ChainId::Canton,
        target_chain: ChainId::Ethereum,
        asset: UniversalAsset {
            id: "asset-123".into(),
            amount: 1000.into(),
            ..Default::default()
        },
        sender: "canton-party".into(),
        recipient: "0x1234...".into(),
        timeout: Duration::from_secs(300),
    };
    
    let result = omnichain.transfer(request).await?;
    println!("Transfer completed: {:?}", result);
    
    Ok(())
}
```

---

## ✅ ACCEPTANCE CRITERIA

### Functional Requirements

- [ ] Full Canton Ledger API v2 support
- [ ] Command submission (sync and async)
- [ ] Transaction streaming with filtering
- [ ] Active contracts query
- [ ] Party management
- [ ] Package management
- [ ] Cross-chain transfers (Canton ↔ Ethereum/Cosmos/Substrate)
- [ ] Cross-chain contract calls
- [ ] Cross-chain state queries

### Non-Functional Requirements

- [ ] < 10ms overhead per operation
- [ ] Support for 1000+ concurrent connections
- [ ] 99.9% uptime with proper error handling
- [ ] Memory-safe with no leaks
- [ ] Thread-safe for multi-threaded use
- [ ] Graceful shutdown support

### Quality Requirements

- [ ] 80%+ test coverage
- [ ] All clippy warnings resolved
- [ ] Documentation for all public APIs
- [ ] Examples for common use cases
- [ ] Benchmarks for critical paths

---

## 📚 REFERENCE MATERIALS

The following research documents provide detailed specifications:

1. **Canton Network Architecture** - Protocol details, domain structure, transaction flow
2. **OmniChain Integration Patterns** - Bridge patterns, message routing, state sync
3. **Rust SDK Best Practices 2025** - Modern Rust patterns, async design, testing
4. **Daml Ledger API** - gRPC services, protobuf messages, streaming
5. **gRPC/Protobuf in Rust** - Tonic configuration, interceptors, compression
6. **Cryptographic Requirements** - Key management, signing, encryption, HSM
7. **Production-Ready Patterns** - Circuit breakers, rate limiting, observability
8. **SDK Architecture Design** - Crate structure, type hierarchy, error handling

---

## 🎯 DEVELOPMENT WORKFLOW

### Phase 1: Foundation (Week 1-2)
1. Set up workspace structure
2. Implement canton-core types
3. Implement canton-crypto basics
4. Set up CI/CD pipeline

### Phase 2: Ledger API (Week 3-4)
1. Proto compilation setup
2. Implement canton-transport
3. Implement canton-ledger-api
4. Integration tests with Canton

### Phase 3: Reliability (Week 5-6)
1. Implement canton-reliability
2. Implement canton-observability
3. Add metrics and tracing
4. Performance testing

### Phase 4: OmniChain (Week 7-8)
1. Implement canton-omnichain core
2. Implement chain adapters
3. Implement bridge logic
4. Cross-chain testing

### Phase 5: Polish (Week 9-10)
1. Documentation
2. Examples
3. Performance optimization
4. Security audit

---

## ⚠️ CRITICAL CONSTRAINTS

1. **NO MOCKS OR STUBS** - All code must be production-ready
2. **NO PLACEHOLDER IMPLEMENTATIONS** - Every function must be complete
3. **NO UNSAFE CODE** - Unless absolutely necessary with documentation
4. **NO BLOCKING IN ASYNC** - Use proper async patterns
5. **NO HARDCODED VALUES** - Everything must be configurable
6. **NO SECRETS IN CODE** - Use environment variables or config files

---

## 🔐 SECURITY REQUIREMENTS

1. All network communication must use TLS 1.3
2. All secrets must be zeroized after use
3. All cryptographic operations must use audited libraries
4. All user input must be validated
5. All errors must not leak sensitive information
6. HSM support must be available for production keys

---

**END OF PROMPT**

This prompt contains all necessary information to implement a production-ready Canton OmniChain SDK in Rust. Follow the specifications exactly and refer to the research documents for detailed implementation guidance.
