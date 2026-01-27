# Canton OmniChain SDK Architecture Design

## 1. High-Level Architecture

```
┌─────────────────────────────────────────────────────────────────────────────────┐
│                           Canton OmniChain SDK                                   │
├─────────────────────────────────────────────────────────────────────────────────┤
│                                                                                  │
│  ┌────────────────────────────────────────────────────────────────────────────┐ │
│  │                         Public API Layer                                    │ │
│  │  ┌──────────────┐ ┌──────────────┐ ┌──────────────┐ ┌──────────────┐      │ │
│  │  │ CantonClient │ │OmniChainClient│ │ EventStream  │ │  Builders    │      │ │
│  │  └──────────────┘ └──────────────┘ └──────────────┘ └──────────────┘      │ │
│  └────────────────────────────────────────────────────────────────────────────┘ │
│                                      │                                           │
│  ┌────────────────────────────────────────────────────────────────────────────┐ │
│  │                         Core Domain Layer                                   │ │
│  │  ┌──────────────┐ ┌──────────────┐ ┌──────────────┐ ┌──────────────┐      │ │
│  │  │   Commands   │ │ Transactions │ │   Contracts  │ │   Parties    │      │ │
│  │  └──────────────┘ └──────────────┘ └──────────────┘ └──────────────┘      │ │
│  │  ┌──────────────┐ ┌──────────────┐ ┌──────────────┐ ┌──────────────┐      │ │
│  │  │    Events    │ │    Values    │ │  Identifiers │ │    Errors    │      │ │
│  │  └──────────────┘ └──────────────┘ └──────────────┘ └──────────────┘      │ │
│  └────────────────────────────────────────────────────────────────────────────┘ │
│                                      │                                           │
│  ┌────────────────────────────────────────────────────────────────────────────┐ │
│  │                         Service Layer                                       │ │
│  │  ┌──────────────┐ ┌──────────────┐ ┌──────────────┐ ┌──────────────┐      │ │
│  │  │CommandService│ │TransactionSvc│ │ ContractSvc  │ │  PartySvc    │      │ │
│  │  └──────────────┘ └──────────────┘ └──────────────┘ └──────────────┘      │ │
│  │  ┌──────────────┐ ┌──────────────┐ ┌──────────────┐ ┌──────────────┐      │ │
│  │  │ PackageSvc   │ │  HealthSvc   │ │  AdminSvc    │ │  BridgeSvc   │      │ │
│  │  └──────────────┘ └──────────────┘ └──────────────┘ └──────────────┘      │ │
│  └────────────────────────────────────────────────────────────────────────────┘ │
│                                      │                                           │
│  ┌────────────────────────────────────────────────────────────────────────────┐ │
│  │                         Infrastructure Layer                                │ │
│  │  ┌──────────────┐ ┌──────────────┐ ┌──────────────┐ ┌──────────────┐      │ │
│  │  │   Transport  │ │    Crypto    │ │    Cache     │ │   Metrics    │      │ │
│  │  │    (gRPC)    │ │              │ │              │ │              │      │ │
│  │  └──────────────┘ └──────────────┘ └──────────────┘ └──────────────┘      │ │
│  │  ┌──────────────┐ ┌──────────────┐ ┌──────────────┐ ┌──────────────┐      │ │
│  │  │  Reliability │ │   Tracing    │ │   Config     │ │   KeyStore   │      │ │
│  │  └──────────────┘ └──────────────┘ └──────────────┘ └──────────────┘      │ │
│  └────────────────────────────────────────────────────────────────────────────┘ │
│                                      │                                           │
│  ┌────────────────────────────────────────────────────────────────────────────┐ │
│  │                         OmniChain Adapter Layer                             │ │
│  │  ┌──────────────┐ ┌──────────────┐ ┌──────────────┐ ┌──────────────┐      │ │
│  │  │CantonAdapter │ │EthereumAdapter│ │ CosmosAdapter│ │SubstrateAdapter│    │ │
│  │  └──────────────┘ └──────────────┘ └──────────────┘ └──────────────┘      │ │
│  └────────────────────────────────────────────────────────────────────────────┘ │
│                                                                                  │
└─────────────────────────────────────────────────────────────────────────────────┘
```

## 2. Crate Structure

```
canton-sdk/
├── Cargo.toml                          # Workspace root
├── rust-toolchain.toml
├── .cargo/config.toml
│
├── crates/
│   │
│   ├── canton-sdk/                     # Main facade crate
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs                  # Re-exports
│   │       └── prelude.rs              # Common imports
│   │
│   ├── canton-core/                    # Core types and traits
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── types/
│   │       │   ├── mod.rs
│   │       │   ├── value.rs            # DamlValue
│   │       │   ├── identifier.rs       # Identifier, ContractId, PartyId
│   │       │   ├── command.rs          # Command types
│   │       │   ├── transaction.rs      # Transaction types
│   │       │   ├── event.rs            # Event types
│   │       │   ├── filter.rs           # TransactionFilter
│   │       │   └── offset.rs           # LedgerOffset
│   │       ├── traits/
│   │       │   ├── mod.rs
│   │       │   ├── client.rs           # Client traits
│   │       │   ├── service.rs          # Service traits
│   │       │   └── adapter.rs          # Adapter traits
│   │       ├── error.rs                # Error types
│   │       └── config.rs               # Configuration types
│   │
│   ├── canton-ledger-api/              # Ledger API client
│   │   ├── Cargo.toml
│   │   ├── build.rs                    # Proto compilation
│   │   ├── proto/                      # Proto files
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── generated/              # Generated code
│   │       ├── client.rs               # LedgerClient
│   │       ├── services/
│   │       │   ├── mod.rs
│   │       │   ├── command.rs
│   │       │   ├── transaction.rs
│   │       │   ├── active_contracts.rs
│   │       │   ├── party.rs
│   │       │   ├── package.rs
│   │       │   └── completion.rs
│   │       ├── streaming.rs            # Stream handling
│   │       └── conversion.rs           # Type conversions
│   │
│   ├── canton-crypto/                  # Cryptographic operations
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── keys/
│   │       │   ├── mod.rs
│   │       │   ├── signing.rs
│   │       │   ├── encryption.rs
│   │       │   └── derivation.rs
│   │       ├── keystore/
│   │       │   ├── mod.rs
│   │       │   ├── memory.rs
│   │       │   ├── file.rs
│   │       │   └── hsm.rs
│   │       ├── hash.rs
│   │       ├── merkle.rs
│   │       └── random.rs
│   │
│   ├── canton-transport/               # Transport layer
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── channel.rs              # Channel management
│   │       ├── pool.rs                 # Connection pooling
│   │       ├── tls.rs                  # TLS configuration
│   │       ├── interceptors/
│   │       │   ├── mod.rs
│   │       │   ├── auth.rs
│   │       │   ├── tracing.rs
│   │       │   └── metrics.rs
│   │       └── compression.rs
│   │
│   ├── canton-reliability/             # Reliability patterns
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── circuit_breaker.rs
│   │       ├── rate_limiter.rs
│   │       ├── bulkhead.rs
│   │       ├── retry.rs
│   │       └── timeout.rs
│   │
│   ├── canton-observability/           # Observability
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── logging.rs
│   │       ├── metrics.rs
│   │       ├── tracing.rs
│   │       └── health.rs
│   │
│   ├── canton-omnichain/               # OmniChain integration
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── client.rs               # OmniChainClient
│   │       ├── router.rs               # Message router
│   │       ├── bridge.rs               # Bridge logic
│   │       ├── adapters/
│   │       │   ├── mod.rs
│   │       │   ├── canton.rs
│   │       │   ├── ethereum.rs
│   │       │   ├── cosmos.rs
│   │       │   └── substrate.rs
│   │       ├── types/
│   │       │   ├── mod.rs
│   │       │   ├── message.rs
│   │       │   ├── asset.rs
│   │       │   └── proof.rs
│   │       └── sync.rs                 # State synchronization
│   │
│   └── canton-testing/                 # Testing utilities
│       ├── Cargo.toml
│       └── src/
│           ├── lib.rs
│           ├── fixtures.rs
│           ├── mocks.rs
│           ├── containers.rs
│           └── generators.rs
│
├── tests/                              # Integration tests
│   ├── integration/
│   └── e2e/
│
├── benches/                            # Benchmarks
│   └── benchmarks.rs
│
├── examples/                           # Examples
│   ├── basic_usage/
│   ├── streaming/
│   ├── cross_chain/
│   └── advanced/
│
└── docs/                               # Documentation
    ├── architecture.md
    ├── getting_started.md
    └── api/
```

## 3. Core Types Design

### 3.1 Type Hierarchy

```rust
// canton-core/src/types/mod.rs

/// Core type exports
pub mod value;
pub mod identifier;
pub mod command;
pub mod transaction;
pub mod event;
pub mod filter;
pub mod offset;

pub use value::*;
pub use identifier::*;
pub use command::*;
pub use transaction::*;
pub use event::*;
pub use filter::*;
pub use offset::*;
```

### 3.2 Value Types

```rust
// canton-core/src/types/value.rs

use rust_decimal::Decimal;
use chrono::{DateTime, NaiveDate, Utc};
use std::collections::HashMap;

/// Daml value representation
#[derive(Debug, Clone, PartialEq)]
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

#[derive(Debug, Clone, PartialEq)]
pub struct DamlRecord {
    pub record_id: Option<Identifier>,
    pub fields: Vec<RecordField>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RecordField {
    pub label: String,
    pub value: DamlValue,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DamlVariant {
    pub variant_id: Option<Identifier>,
    pub constructor: String,
    pub value: Box<DamlValue>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DamlEnum {
    pub enum_id: Option<Identifier>,
    pub constructor: String,
}

impl DamlValue {
    // Constructors
    pub fn unit() -> Self { Self::Unit }
    pub fn bool(v: bool) -> Self { Self::Bool(v) }
    pub fn int64(v: i64) -> Self { Self::Int64(v) }
    pub fn text(v: impl Into<String>) -> Self { Self::Text(v.into()) }
    pub fn party(v: PartyId) -> Self { Self::Party(v) }
    pub fn contract_id(v: ContractId) -> Self { Self::ContractId(v) }
    
    // Type checking
    pub fn is_unit(&self) -> bool { matches!(self, Self::Unit) }
    pub fn is_bool(&self) -> bool { matches!(self, Self::Bool(_)) }
    pub fn is_int64(&self) -> bool { matches!(self, Self::Int64(_)) }
    pub fn is_text(&self) -> bool { matches!(self, Self::Text(_)) }
    pub fn is_record(&self) -> bool { matches!(self, Self::Record(_)) }
    
    // Accessors
    pub fn as_bool(&self) -> Option<bool> {
        match self { Self::Bool(v) => Some(*v), _ => None }
    }
    pub fn as_int64(&self) -> Option<i64> {
        match self { Self::Int64(v) => Some(*v), _ => None }
    }
    pub fn as_text(&self) -> Option<&str> {
        match self { Self::Text(v) => Some(v), _ => None }
    }
    pub fn as_record(&self) -> Option<&DamlRecord> {
        match self { Self::Record(v) => Some(v), _ => None }
    }
    
    // Record field access
    pub fn get_field(&self, name: &str) -> Option<&DamlValue> {
        self.as_record()?.fields.iter()
            .find(|f| f.label == name)
            .map(|f| &f.value)
    }
}

impl DamlRecord {
    pub fn new() -> Self {
        Self { record_id: None, fields: Vec::new() }
    }
    
    pub fn with_id(mut self, id: Identifier) -> Self {
        self.record_id = Some(id);
        self
    }
    
    pub fn field(mut self, label: impl Into<String>, value: impl Into<DamlValue>) -> Self {
        self.fields.push(RecordField {
            label: label.into(),
            value: value.into(),
        });
        self
    }
    
    pub fn get(&self, name: &str) -> Option<&DamlValue> {
        self.fields.iter()
            .find(|f| f.label == name)
            .map(|f| &f.value)
    }
}

// From implementations for ergonomic construction
impl From<bool> for DamlValue {
    fn from(v: bool) -> Self { Self::Bool(v) }
}

impl From<i64> for DamlValue {
    fn from(v: i64) -> Self { Self::Int64(v) }
}

impl From<&str> for DamlValue {
    fn from(v: &str) -> Self { Self::Text(v.to_string()) }
}

impl From<String> for DamlValue {
    fn from(v: String) -> Self { Self::Text(v) }
}

impl<T: Into<DamlValue>> From<Vec<T>> for DamlValue {
    fn from(v: Vec<T>) -> Self {
        Self::List(v.into_iter().map(Into::into).collect())
    }
}

impl<T: Into<DamlValue>> From<Option<T>> for DamlValue {
    fn from(v: Option<T>) -> Self {
        Self::Optional(v.map(|x| Box::new(x.into())))
    }
}

impl From<DamlRecord> for DamlValue {
    fn from(v: DamlRecord) -> Self { Self::Record(v) }
}
```

### 3.3 Identifier Types

```rust
// canton-core/src/types/identifier.rs

use std::fmt;
use std::str::FromStr;

/// Template/type identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Identifier {
    pub package_id: String,
    pub module_name: String,
    pub entity_name: String,
}

impl Identifier {
    pub fn new(
        package_id: impl Into<String>,
        module_name: impl Into<String>,
        entity_name: impl Into<String>,
    ) -> Self {
        Self {
            package_id: package_id.into(),
            module_name: module_name.into(),
            entity_name: entity_name.into(),
        }
    }
    
    pub fn qualified_name(&self) -> String {
        format!("{}:{}.{}", self.package_id, self.module_name, self.entity_name)
    }
}

impl FromStr for Identifier {
    type Err = ParseError;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split(':').collect();
        if parts.len() != 2 {
            return Err(ParseError::InvalidFormat("expected package:module.entity".into()));
        }
        
        let package_id = parts[0];
        let module_entity: Vec<&str> = parts[1].rsplitn(2, '.').collect();
        
        if module_entity.len() != 2 {
            return Err(ParseError::InvalidFormat("expected module.entity".into()));
        }
        
        Ok(Self {
            package_id: package_id.to_string(),
            module_name: module_entity[1].to_string(),
            entity_name: module_entity[0].to_string(),
        })
    }
}

impl fmt::Display for Identifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.qualified_name())
    }
}

/// Party identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PartyId(String);

impl PartyId {
    pub fn new(id: impl Into<String>) -> Result<Self, ValidationError> {
        let id = id.into();
        Self::validate(&id)?;
        Ok(Self(id))
    }
    
    pub fn new_unchecked(id: impl Into<String>) -> Self {
        Self(id.into())
    }
    
    fn validate(id: &str) -> Result<(), ValidationError> {
        if id.is_empty() {
            return Err(ValidationError::Empty("party_id"));
        }
        if id.len() > 256 {
            return Err(ValidationError::TooLong("party_id", 256));
        }
        if !id.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_' || c == ':') {
            return Err(ValidationError::InvalidCharacters("party_id"));
        }
        Ok(())
    }
    
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for PartyId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for PartyId {
    type Err = ValidationError;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s)
    }
}

/// Contract identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ContractId(String);

impl ContractId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
    
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ContractId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for ContractId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for ContractId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("Invalid format: {0}")]
    InvalidFormat(String),
}

#[derive(Debug, thiserror::Error)]
pub enum ValidationError {
    #[error("{0} cannot be empty")]
    Empty(&'static str),
    #[error("{0} exceeds maximum length of {1}")]
    TooLong(&'static str, usize),
    #[error("{0} contains invalid characters")]
    InvalidCharacters(&'static str),
}
```

## 4. Client Architecture

### 4.1 Main Client

```rust
// canton-sdk/src/lib.rs

//! Canton OmniChain SDK
//!
//! A production-ready Rust SDK for Canton Network with OmniChain integration.

pub use canton_core as core;
pub use canton_ledger_api as ledger_api;
pub use canton_crypto as crypto;
pub use canton_transport as transport;
pub use canton_reliability as reliability;
pub use canton_observability as observability;
pub use canton_omnichain as omnichain;

pub mod prelude;

use std::sync::Arc;
use canton_core::{config::SdkConfig, error::SdkResult};
use canton_ledger_api::LedgerClient;
use canton_omnichain::OmniChainClient;

/// Main SDK client
pub struct CantonSdk {
    /// Configuration
    config: SdkConfig,
    /// Ledger client
    ledger: Arc<LedgerClient>,
    /// OmniChain client (optional)
    omnichain: Option<Arc<OmniChainClient>>,
    /// Shutdown coordinator
    shutdown: Arc<ShutdownCoordinator>,
}

impl CantonSdk {
    /// Create SDK builder
    pub fn builder() -> CantonSdkBuilder {
        CantonSdkBuilder::new()
    }
    
    /// Get ledger client
    pub fn ledger(&self) -> &LedgerClient {
        &self.ledger
    }
    
    /// Get OmniChain client
    pub fn omnichain(&self) -> Option<&OmniChainClient> {
        self.omnichain.as_deref()
    }
    
    /// Shutdown SDK gracefully
    pub async fn shutdown(&self, timeout: Duration) {
        self.shutdown.shutdown(timeout).await;
    }
}

/// SDK builder
pub struct CantonSdkBuilder {
    config: Option<SdkConfig>,
    config_path: Option<String>,
    omnichain_enabled: bool,
}

impl CantonSdkBuilder {
    pub fn new() -> Self {
        Self {
            config: None,
            config_path: None,
            omnichain_enabled: false,
        }
    }
    
    /// Set configuration
    pub fn config(mut self, config: SdkConfig) -> Self {
        self.config = Some(config);
        self
    }
    
    /// Load configuration from file
    pub fn config_file(mut self, path: impl Into<String>) -> Self {
        self.config_path = Some(path.into());
        self
    }
    
    /// Enable OmniChain integration
    pub fn with_omnichain(mut self) -> Self {
        self.omnichain_enabled = true;
        self
    }
    
    /// Build the SDK
    pub async fn build(self) -> SdkResult<CantonSdk> {
        // Load configuration
        let config = match (self.config, self.config_path) {
            (Some(config), _) => config,
            (None, Some(path)) => SdkConfig::load_from_file(&path)?,
            (None, None) => SdkConfig::load()?,
        };
        
        // Validate configuration
        config.validate()?;
        
        // Initialize observability
        observability::init(&config.observability)?;
        
        // Create ledger client
        let ledger = Arc::new(LedgerClient::connect(&config.canton).await?);
        
        // Create OmniChain client if enabled
        let omnichain = if self.omnichain_enabled {
            Some(Arc::new(OmniChainClient::new(&config.omnichain, ledger.clone()).await?))
        } else {
            None
        };
        
        // Create shutdown coordinator
        let shutdown = Arc::new(ShutdownCoordinator::new());
        
        Ok(CantonSdk {
            config,
            ledger,
            omnichain,
            shutdown,
        })
    }
}
```

### 4.2 Ledger Client

```rust
// canton-ledger-api/src/client.rs

use std::sync::Arc;
use tokio::sync::RwLock;
use canton_core::{
    config::CantonConfig,
    error::{SdkError, SdkResult},
    types::*,
};
use canton_transport::{ChannelBuilder, ConnectionPool};
use canton_reliability::{CircuitBreaker, RateLimiter, RetryPolicy};
use canton_observability::metrics::SdkMetrics;

/// Ledger API client
pub struct LedgerClient {
    /// gRPC channel
    channel: tonic::transport::Channel,
    /// Ledger ID
    ledger_id: String,
    /// Services
    services: LedgerServices,
    /// Reliability components
    reliability: ReliabilityComponents,
    /// Metrics
    metrics: Arc<SdkMetrics>,
    /// Configuration
    config: CantonConfig,
}

struct LedgerServices {
    command: CommandServiceClient<tonic::transport::Channel>,
    command_submission: CommandSubmissionServiceClient<tonic::transport::Channel>,
    command_completion: CommandCompletionServiceClient<tonic::transport::Channel>,
    transaction: TransactionServiceClient<tonic::transport::Channel>,
    active_contracts: ActiveContractsServiceClient<tonic::transport::Channel>,
    party_management: PartyManagementServiceClient<tonic::transport::Channel>,
    package: PackageServiceClient<tonic::transport::Channel>,
    ledger_identity: LedgerIdentityServiceClient<tonic::transport::Channel>,
}

struct ReliabilityComponents {
    circuit_breaker: Arc<CircuitBreaker>,
    rate_limiter: Arc<RateLimiter>,
    retry_policy: RetryPolicy,
}

impl LedgerClient {
    /// Connect to Canton ledger
    pub async fn connect(config: &CantonConfig) -> SdkResult<Self> {
        // Build channel
        let channel = ChannelBuilder::new(&config.endpoint)
            .tls(config.tls.clone())
            .connect_timeout(config.connect_timeout)
            .request_timeout(config.request_timeout)
            .keep_alive_interval(config.keep_alive_interval)
            .build()
            .await?;
        
        // Get ledger ID
        let ledger_id = Self::fetch_ledger_id(&channel).await?;
        
        // Create services
        let services = LedgerServices {
            command: CommandServiceClient::new(channel.clone()),
            command_submission: CommandSubmissionServiceClient::new(channel.clone()),
            command_completion: CommandCompletionServiceClient::new(channel.clone()),
            transaction: TransactionServiceClient::new(channel.clone()),
            active_contracts: ActiveContractsServiceClient::new(channel.clone()),
            party_management: PartyManagementServiceClient::new(channel.clone()),
            package: PackageServiceClient::new(channel.clone()),
            ledger_identity: LedgerIdentityServiceClient::new(channel.clone()),
        };
        
        // Create reliability components
        let reliability = ReliabilityComponents {
            circuit_breaker: Arc::new(CircuitBreaker::new(config.reliability.circuit_breaker.clone())),
            rate_limiter: Arc::new(RateLimiter::new(config.reliability.rate_limiter.clone())),
            retry_policy: config.reliability.retry.clone().into(),
        };
        
        // Create metrics
        let metrics = Arc::new(SdkMetrics::new());
        
        Ok(Self {
            channel,
            ledger_id,
            services,
            reliability,
            metrics,
            config: config.clone(),
        })
    }
    
    /// Get ledger ID
    pub fn ledger_id(&self) -> &str {
        &self.ledger_id
    }
    
    /// Submit command and wait for result
    #[tracing::instrument(skip(self, commands), fields(command_id = %commands.command_id))]
    pub async fn submit_and_wait(&self, commands: Commands) -> SdkResult<Transaction> {
        let start = std::time::Instant::now();
        
        // Check circuit breaker
        let guard = self.reliability.circuit_breaker.allow_request().await?;
        
        // Check rate limiter
        self.reliability.rate_limiter.acquire().await?;
        
        // Execute with retry
        let result = self.reliability.retry_policy
            .execute(|| self.do_submit_and_wait(&commands))
            .await;
        
        // Record metrics
        let duration = start.elapsed();
        match &result {
            Ok(_) => {
                guard.success().await;
                self.metrics.record_command("submit_and_wait", true, duration);
            }
            Err(e) => {
                guard.failure().await;
                self.metrics.record_command("submit_and_wait", false, duration);
                self.metrics.record_error(&e.error_type());
            }
        }
        
        result
    }
    
    async fn do_submit_and_wait(&self, commands: &Commands) -> SdkResult<Transaction> {
        let request = commands.to_proto(&self.ledger_id)?;
        
        let response = self.services.command
            .clone()
            .submit_and_wait_for_transaction(request)
            .await?;
        
        Transaction::from_proto(response.into_inner())
    }
    
    /// Submit command asynchronously
    pub async fn submit(&self, commands: Commands) -> SdkResult<()> {
        let request = commands.to_proto(&self.ledger_id)?;
        
        self.services.command_submission
            .clone()
            .submit(request)
            .await?;
        
        Ok(())
    }
    
    /// Get transaction stream
    pub fn get_transactions(
        &self,
        begin: LedgerOffset,
        end: Option<LedgerOffset>,
        filter: TransactionFilter,
    ) -> impl Stream<Item = SdkResult<Transaction>> + '_ {
        TransactionStream::new(
            self.services.transaction.clone(),
            &self.ledger_id,
            begin,
            end,
            filter,
        )
    }
    
    /// Get transaction trees stream
    pub fn get_transaction_trees(
        &self,
        begin: LedgerOffset,
        end: Option<LedgerOffset>,
        filter: TransactionFilter,
    ) -> impl Stream<Item = SdkResult<TransactionTree>> + '_ {
        TransactionTreeStream::new(
            self.services.transaction.clone(),
            &self.ledger_id,
            begin,
            end,
            filter,
        )
    }
    
    /// Get active contracts
    pub async fn get_active_contracts(
        &self,
        filter: TransactionFilter,
    ) -> SdkResult<Vec<CreatedEvent>> {
        let request = GetActiveContractsRequest {
            ledger_id: self.ledger_id.clone(),
            filter: Some(filter.to_proto()),
            verbose: true,
            active_at_offset: String::new(),
        };
        
        let mut stream = self.services.active_contracts
            .clone()
            .get_active_contracts(request)
            .await?
            .into_inner();
        
        let mut contracts = Vec::new();
        while let Some(response) = stream.message().await? {
            for contract in response.active_contracts {
                contracts.push(CreatedEvent::from_proto(contract)?);
            }
        }
        
        Ok(contracts)
    }
    
    /// Allocate party
    pub async fn allocate_party(
        &self,
        party_id_hint: Option<&str>,
        display_name: Option<&str>,
    ) -> SdkResult<PartyDetails> {
        let request = AllocatePartyRequest {
            party_id_hint: party_id_hint.unwrap_or_default().to_string(),
            display_name: display_name.unwrap_or_default().to_string(),
            local_metadata: String::new(),
        };
        
        let response = self.services.party_management
            .clone()
            .allocate_party(request)
            .await?;
        
        PartyDetails::from_proto(response.into_inner().party_details.unwrap())
    }
    
    /// List known parties
    pub async fn list_known_parties(&self) -> SdkResult<Vec<PartyDetails>> {
        let request = ListKnownPartiesRequest {};
        
        let response = self.services.party_management
            .clone()
            .list_known_parties(request)
            .await?;
        
        response.into_inner()
            .party_details
            .into_iter()
            .map(PartyDetails::from_proto)
            .collect()
    }
    
    /// Get ledger end
    pub async fn get_ledger_end(&self) -> SdkResult<LedgerOffset> {
        let request = GetLedgerEndRequest {
            ledger_id: self.ledger_id.clone(),
        };
        
        let response = self.services.transaction
            .clone()
            .get_ledger_end(request)
            .await?;
        
        LedgerOffset::from_proto(response.into_inner().offset.unwrap())
    }
    
    async fn fetch_ledger_id(channel: &tonic::transport::Channel) -> SdkResult<String> {
        let mut client = LedgerIdentityServiceClient::new(channel.clone());
        let response = client.get_ledger_identity(GetLedgerIdentityRequest {}).await?;
        Ok(response.into_inner().ledger_id)
    }
}
```

## 5. OmniChain Architecture

### 5.1 OmniChain Client

```rust
// canton-omnichain/src/client.rs

use std::sync::Arc;
use std::collections::HashMap;
use canton_core::{
    config::OmniChainConfig,
    error::SdkResult,
};
use canton_ledger_api::LedgerClient;

use crate::adapters::{ChainAdapter, ChainAdapterFactory};
use crate::router::MessageRouter;
use crate::bridge::Bridge;
use crate::types::*;

/// OmniChain client for cross-chain operations
pub struct OmniChainClient {
    /// Canton ledger client
    canton: Arc<LedgerClient>,
    /// Chain adapters
    adapters: HashMap<ChainId, Arc<dyn ChainAdapter>>,
    /// Message router
    router: MessageRouter,
    /// Bridge
    bridge: Bridge,
    /// Configuration
    config: OmniChainConfig,
}

impl OmniChainClient {
    /// Create new OmniChain client
    pub async fn new(
        config: &OmniChainConfig,
        canton: Arc<LedgerClient>,
    ) -> SdkResult<Self> {
        // Create adapters for enabled chains
        let mut adapters = HashMap::new();
        
        for chain_name in &config.enabled_chains {
            let chain_config = config.chains.get(chain_name)
                .ok_or_else(|| SdkError::Config(format!("Missing config for chain: {}", chain_name)))?;
            
            let adapter = ChainAdapterFactory::create(chain_name, chain_config).await?;
            adapters.insert(adapter.chain_id(), adapter);
        }
        
        // Create router
        let router = MessageRouter::new(adapters.clone());
        
        // Create bridge
        let bridge = Bridge::new(canton.clone(), adapters.clone(), &config.bridge)?;
        
        Ok(Self {
            canton,
            adapters,
            router,
            bridge,
            config: config.clone(),
        })
    }
    
    /// Get Canton client
    pub fn canton(&self) -> &LedgerClient {
        &self.canton
    }
    
    /// Get chain adapter
    pub fn adapter(&self, chain_id: &ChainId) -> Option<&dyn ChainAdapter> {
        self.adapters.get(chain_id).map(|a| a.as_ref())
    }
    
    /// Execute cross-chain transfer
    #[tracing::instrument(skip(self))]
    pub async fn transfer(
        &self,
        request: CrossChainTransferRequest,
    ) -> SdkResult<CrossChainTransferResult> {
        // Validate request
        request.validate()?;
        
        // Execute via bridge
        self.bridge.execute_transfer(request).await
    }
    
    /// Execute cross-chain contract call
    pub async fn call(
        &self,
        request: CrossChainCallRequest,
    ) -> SdkResult<CrossChainCallResult> {
        // Validate request
        request.validate()?;
        
        // Route to appropriate chain
        self.router.route_call(request).await
    }
    
    /// Query state across chains
    pub async fn query(
        &self,
        request: CrossChainQueryRequest,
    ) -> SdkResult<CrossChainQueryResult> {
        self.router.route_query(request).await
    }
    
    /// Subscribe to cross-chain events
    pub fn subscribe_events(
        &self,
        filter: CrossChainEventFilter,
    ) -> impl Stream<Item = SdkResult<CrossChainEvent>> + '_ {
        CrossChainEventStream::new(&self.adapters, filter)
    }
    
    /// Get bridge status
    pub async fn bridge_status(&self) -> SdkResult<BridgeStatus> {
        self.bridge.status().await
    }
}
```

### 5.2 Chain Adapter Trait

```rust
// canton-omnichain/src/adapters/mod.rs

use async_trait::async_trait;
use canton_core::error::SdkResult;
use crate::types::*;

/// Chain adapter trait
#[async_trait]
pub trait ChainAdapter: Send + Sync {
    /// Get chain ID
    fn chain_id(&self) -> ChainId;
    
    /// Get chain type
    fn chain_type(&self) -> ChainType;
    
    /// Check if connected
    async fn is_connected(&self) -> bool;
    
    /// Get current block height
    async fn block_height(&self) -> SdkResult<u64>;
    
    /// Submit transaction
    async fn submit_transaction(
        &self,
        tx: ChainTransaction,
    ) -> SdkResult<TransactionReceipt>;
    
    /// Query state
    async fn query_state(
        &self,
        query: StateQuery,
    ) -> SdkResult<StateQueryResult>;
    
    /// Get transaction by hash
    async fn get_transaction(
        &self,
        hash: &str,
    ) -> SdkResult<Option<ChainTransaction>>;
    
    /// Subscribe to events
    fn subscribe_events(
        &self,
        filter: EventFilter,
    ) -> Box<dyn Stream<Item = SdkResult<ChainEvent>> + Send + Unpin>;
    
    /// Verify proof
    async fn verify_proof(
        &self,
        proof: &ChainProof,
    ) -> SdkResult<bool>;
    
    /// Generate proof
    async fn generate_proof(
        &self,
        request: ProofRequest,
    ) -> SdkResult<ChainProof>;
}

/// Chain types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ChainType {
    Canton,
    Ethereum,
    Cosmos,
    Substrate,
    Custom,
}

/// Chain adapter factory
pub struct ChainAdapterFactory;

impl ChainAdapterFactory {
    pub async fn create(
        chain_name: &str,
        config: &ChainAdapterConfig,
    ) -> SdkResult<Arc<dyn ChainAdapter>> {
        match chain_name {
            "canton" => Ok(Arc::new(CantonAdapter::new(config).await?)),
            "ethereum" => Ok(Arc::new(EthereumAdapter::new(config).await?)),
            "cosmos" => Ok(Arc::new(CosmosAdapter::new(config).await?)),
            "substrate" => Ok(Arc::new(SubstrateAdapter::new(config).await?)),
            _ => Err(SdkError::Config(format!("Unknown chain: {}", chain_name))),
        }
    }
}
```

## 6. Error Handling Architecture

```rust
// canton-core/src/error.rs

use std::backtrace::Backtrace;
use thiserror::Error;

/// Main SDK error type
#[derive(Error, Debug)]
pub enum SdkError {
    // Connection errors
    #[error("Connection error: {message}")]
    Connection {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
        backtrace: Backtrace,
    },
    
    // Authentication errors
    #[error("Authentication failed: {reason}")]
    Authentication {
        reason: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },
    
    // Transaction errors
    #[error("Transaction error: {kind}")]
    Transaction {
        kind: TransactionErrorKind,
        transaction_id: Option<String>,
        details: HashMap<String, String>,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },
    
    // Validation errors
    #[error("Validation error: {field} - {message}")]
    Validation {
        field: String,
        message: String,
    },
    
    // Configuration errors
    #[error("Configuration error: {0}")]
    Config(String),
    
    // Serialization errors
    #[error("Serialization error: {0}")]
    Serialization(String),
    
    // Cryptographic errors
    #[error("Cryptographic error: {0}")]
    Crypto(String),
    
    // Timeout errors
    #[error("Operation timed out after {duration:?}: {operation}")]
    Timeout {
        duration: std::time::Duration,
        operation: String,
    },
    
    // Rate limit errors
    #[error("Rate limited, retry after {retry_after:?}")]
    RateLimited {
        retry_after: Option<std::time::Duration>,
    },
    
    // Circuit breaker errors
    #[error("Circuit breaker open")]
    CircuitOpen,
    
    // Cross-chain errors
    #[error("Cross-chain error: {message}")]
    CrossChain {
        message: String,
        source_chain: Option<ChainId>,
        target_chain: Option<ChainId>,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },
    
    // Internal errors
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
    Rejected,
    Unknown,
}

impl SdkError {
    /// Check if error is retryable
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            SdkError::Connection { .. }
                | SdkError::Timeout { .. }
                | SdkError::RateLimited { .. }
                | SdkError::Transaction { kind: TransactionErrorKind::Conflict, .. }
        )
    }
    
    /// Get error type for metrics
    pub fn error_type(&self) -> &'static str {
        match self {
            SdkError::Connection { .. } => "connection",
            SdkError::Authentication { .. } => "authentication",
            SdkError::Transaction { .. } => "transaction",
            SdkError::Validation { .. } => "validation",
            SdkError::Config(_) => "config",
            SdkError::Serialization(_) => "serialization",
            SdkError::Crypto(_) => "crypto",
            SdkError::Timeout { .. } => "timeout",
            SdkError::RateLimited { .. } => "rate_limited",
            SdkError::CircuitOpen => "circuit_open",
            SdkError::CrossChain { .. } => "cross_chain",
            SdkError::Internal { .. } => "internal",
        }
    }
}

/// Result type alias
pub type SdkResult<T> = Result<T, SdkError>;

/// Error conversion from tonic status
impl From<tonic::Status> for SdkError {
    fn from(status: tonic::Status) -> Self {
        match status.code() {
            tonic::Code::NotFound => SdkError::Transaction {
                kind: TransactionErrorKind::ContractNotFound,
                transaction_id: None,
                details: HashMap::new(),
                source: Some(Box::new(status)),
            },
            tonic::Code::PermissionDenied => SdkError::Authentication {
                reason: status.message().to_string(),
                source: Some(Box::new(status)),
            },
            tonic::Code::DeadlineExceeded => SdkError::Timeout {
                duration: std::time::Duration::from_secs(0),
                operation: status.message().to_string(),
            },
            tonic::Code::ResourceExhausted => SdkError::RateLimited {
                retry_after: None,
            },
            tonic::Code::Unavailable => SdkError::Connection {
                message: status.message().to_string(),
                source: Some(Box::new(status)),
                backtrace: Backtrace::capture(),
            },
            _ => SdkError::Internal {
                message: format!("gRPC error: {}", status.message()),
                backtrace: Backtrace::capture(),
            },
        }
    }
}
```

## 7. Feature Flags

```toml
# canton-sdk/Cargo.toml

[features]
default = ["tls", "compression", "tracing"]

# Core features
tls = ["canton-transport/tls"]
compression = ["canton-transport/compression"]

# Observability
tracing = ["canton-observability/tracing"]
metrics = ["canton-observability/metrics"]
full-observability = ["tracing", "metrics"]

# OmniChain adapters
omnichain = ["canton-omnichain"]
ethereum = ["omnichain", "canton-omnichain/ethereum"]
cosmos = ["omnichain", "canton-omnichain/cosmos"]
substrate = ["omnichain", "canton-omnichain/substrate"]
all-chains = ["ethereum", "cosmos", "substrate"]

# Key stores
hsm = ["canton-crypto/hsm"]
vault = ["canton-crypto/vault"]

# Development
dev = ["canton-testing"]
```

## 8. Public API Summary

```rust
// canton-sdk/src/prelude.rs

//! Common imports for Canton SDK users

// Core types
pub use canton_core::types::{
    DamlValue, DamlRecord, DamlVariant, DamlEnum, RecordField,
    Identifier, PartyId, ContractId,
    Command, Commands, CreateCommand, ExerciseCommand,
    Transaction, TransactionTree, Event, CreatedEvent, ArchivedEvent, ExercisedEvent,
    TransactionFilter, LedgerOffset,
};

// Error types
pub use canton_core::error::{SdkError, SdkResult, TransactionErrorKind};

// Configuration
pub use canton_core::config::{SdkConfig, CantonConfig, OmniChainConfig};

// Main client
pub use crate::{CantonSdk, CantonSdkBuilder};

// Ledger client
pub use canton_ledger_api::LedgerClient;

// OmniChain (when enabled)
#[cfg(feature = "omnichain")]
pub use canton_omnichain::{
    OmniChainClient,
    CrossChainTransferRequest, CrossChainTransferResult,
    CrossChainCallRequest, CrossChainCallResult,
    ChainId, ChainType,
};

// Builders
pub use canton_core::builders::{
    CommandBuilder, TransactionFilterBuilder, RecordBuilder,
};

// Macros
pub use canton_core::{daml_record, daml_list, daml_variant};

// Traits
pub use canton_core::traits::{
    ToProto, FromProto,
};
```

This architecture provides:

1. **Modularity**: Clear separation of concerns across crates
2. **Extensibility**: Easy to add new chain adapters and features
3. **Reliability**: Built-in circuit breakers, rate limiting, retries
4. **Observability**: Comprehensive logging, metrics, and tracing
5. **Type Safety**: Strong typing with ergonomic builders
6. **Performance**: Connection pooling, streaming, batching
7. **Security**: HSM support, secure key management
8. **Testability**: Mock implementations, test utilities
