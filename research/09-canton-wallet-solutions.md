# Canton Network Wallet Solutions Research

## 1. Overview

This document provides comprehensive research on opensource Canton Network wallet solutions suitable for integration into the Canton OmniChain SDK.

## 2. Canton Network Wallet Landscape

### 2.1 Official Canton Wallet Solutions

#### 2.1.1 Canton Navigator (Official Web Wallet)

**Repository**: Digital Asset / Canton Navigator
**Language**: TypeScript/JavaScript
**Status**: Production-ready, actively maintained

**Key Features**:
- Web-based wallet interface for Canton Network
- Party management and identity handling
- Contract interaction and transaction submission
- Multi-domain support
- Ledger API integration

**Architecture**:
```
Canton Navigator
├── Frontend (React)
├── Backend (Node.js)
│   ├── Ledger API Client
│   ├── Party Management
│   └── Transaction Service
└── Canton Participant Node
```

**Integration Points**:
- Ledger API (gRPC)
- Party Management Service
- Command Submission Service
- Transaction Service

**Pros**:
- Official Digital Asset solution
- Well-documented
- Production-tested
- Active community support

**Cons**:
- JavaScript/TypeScript stack (not Rust)
- Web-focused architecture
- Requires Node.js runtime

**Suitability for Rust SDK**: **Low** - Can be used as reference but not directly integrable

#### 2.1.2 Canton Console (Developer Tools)

**Repository**: Digital Asset / Canton Console
**Language**: Scala/TypeScript
**Status**: Production-ready

**Key Features**:
- Administrative interface for Canton
- Domain and participant management
- Package deployment
- Health monitoring

**Suitability for Rust SDK**: **Low** - Administrative tool, not a wallet library

### 2.2 Community Canton Wallet Solutions

#### 2.2.1 Canton Wallet SDK (Community)

**Repository**: Various community implementations
**Languages**: Python, Java, Go
**Status**: Experimental/Early stage

**Analysis**:
- No mature Rust implementations found
- Most community wallets are language-specific
- Limited documentation and production usage

**Suitability for Rust SDK**: **Very Low** - No Rust-native solutions

### 2.3 Canton Ledger API as Wallet Foundation

**Key Insight**: Canton Network doesn't have traditional "wallets" like EVM chains. Instead, it uses:

1. **Party Management**: Parties represent identities
2. **Key Management**: X.509 certificates and signing keys
3. **Participant Nodes**: Act as wallet infrastructure
4. **Ledger API**: Provides wallet-like functionality

**Wallet-like Operations in Canton**:
- Party allocation and management
- Command submission (transactions)
- Contract creation and exercise
- Transaction history
- Active contract queries

## 3. Canton Wallet Architecture for SDK

### 3.1 Proposed Canton Wallet Component

```rust
// Canton wallet abstraction for SDK

/// Canton wallet interface
#[async_trait]
pub trait CantonWallet: Send + Sync {
    /// Get wallet party ID
    fn party_id(&self) -> &PartyId;
    
    /// Get wallet participant ID
    fn participant_id(&self) -> &ParticipantId;
    
    /// Submit command (transaction)
    async fn submit_command(
        &self,
        command: Command,
    ) -> SdkResult<Transaction>;
    
    /// Submit and wait for transaction
    async fn submit_and_wait(
        &self,
        command: Command,
    ) -> SdkResult<Transaction>;
    
    /// Get active contracts
    async fn get_active_contracts(
        &self,
        filter: Option<TransactionFilter>,
    ) -> SdkResult<Vec<CreatedEvent>>;
    
    /// Exercise choice on contract
    async fn exercise_choice(
        &self,
        contract_id: ContractId,
        choice: &str,
        argument: DamlValue,
    ) -> SdkResult<Transaction>;
    
    /// Create contract
    async fn create_contract(
        &self,
        template_id: Identifier,
        arguments: DamlRecord,
    ) -> SdkResult<CreatedEvent>;
    
    /// Get transaction history
    fn get_transactions(
        &self,
        begin: LedgerOffset,
        end: Option<LedgerOffset>,
        filter: TransactionFilter,
    ) -> impl Stream<Item = SdkResult<Transaction>>;
    
    /// Sign data with wallet key
    async fn sign(&self, data: &[u8]) -> SdkResult<Signature>;
    
    /// Get wallet balance (if applicable)
    async fn get_balance(&self) -> SdkResult<WalletBalance>;
}

/// Canton wallet implementation
pub struct CantonWalletImpl {
    /// Ledger API client
    ledger_client: Arc<LedgerClient>,
    
    /// Party ID
    party_id: PartyId,
    
    /// Key store
    key_store: Arc<dyn KeyStore>,
    
    /// Wallet metadata
    metadata: WalletMetadata,
    
    /// Metrics
    metrics: Arc<WalletMetrics>,
}

impl CantonWalletImpl {
    pub async fn new(
        config: CantonWalletConfig,
        ledger_client: Arc<LedgerClient>,
        key_store: Arc<dyn KeyStore>,
    ) -> SdkResult<Self> {
        // Validate party exists
        let party_id = config.party_id;
        let party_details = ledger_client
            .get_party_details(&party_id)
            .await
            .ok_or(SdkError::PartyNotFound(party_id.clone()))?;
        
        Ok(Self {
            ledger_client,
            party_id,
            key_store,
            metadata: WalletMetadata {
                display_name: party_details.display_name,
                is_local: party_details.is_local,
                created_at: Utc::now(),
            },
            metrics: Arc::new(WalletMetrics::new()),
        })
    }
    
    pub async fn allocate_party(
        ledger_client: &LedgerClient,
        hint: Option<String>,
        display_name: String,
    ) -> SdkResult<(PartyId, Self)> {
        let party_details = ledger_client
            .allocate_party(hint, display_name)
            .await?;
        
        let party_id = PartyId::new_unchecked(party_details.party.clone());
        
        // Create wallet with allocated party
        let config = CantonWalletConfig {
            party_id: party_id.clone(),
            ..Default::default()
        };
        
        let wallet = Self::new(
            config,
            Arc::new(ledger_client.clone()),
            Arc::new(InMemoryKeyStore::new()),
        ).await?;
        
        Ok((party_id, wallet))
    }
}

#[async_trait]
impl CantonWallet for CantonWalletImpl {
    fn party_id(&self) -> &PartyId {
        &self.party_id
    }
    
    fn participant_id(&self) -> &ParticipantId {
        self.ledger_client.participant_id()
    }
    
    async fn submit_command(
        &self,
        command: Command,
    ) -> SdkResult<Transaction> {
        let start = Instant::now();
        
        let commands = Commands {
            ledger_id: self.ledger_client.ledger_id().to_string(),
            workflow_id: Uuid::new_v4().to_string(),
            application_id: "canton-sdk".to_string(),
            command_id: Uuid::new_v4().to_string(),
            party: self.party_id.as_str().to_string(),
            act_as: vec![self.party_id.as_str().to_string()],
            commands: vec![command],
            ..Default::default()
        };
        
        let result = self.ledger_client
            .submit_and_wait(commands)
            .await?;
        
        self.metrics.record_command_submission(start.elapsed());
        
        Ok(result)
    }
    
    async fn submit_and_wait(
        &self,
        command: Command,
    ) -> SdkResult<Transaction> {
        self.submit_command(command).await
    }
    
    async fn get_active_contracts(
        &self,
        filter: Option<TransactionFilter>,
    ) -> SdkResult<Vec<CreatedEvent>> {
        let filter = filter.unwrap_or_else(|| {
            TransactionFilter::for_party(&self.party_id)
        });
        
        self.ledger_client.get_active_contracts(filter).await
    }
    
    async fn exercise_choice(
        &self,
        contract_id: ContractId,
        choice: &str,
        argument: DamlValue,
    ) -> SdkResult<Transaction> {
        let command = Command::Exercise(ExerciseCommand {
            template_id: None, // Will be resolved
            contract_id: contract_id.to_string(),
            choice: choice.to_string(),
            choice_argument: argument.into(),
        });
        
        self.submit_command(command).await
    }
    
    async fn create_contract(
        &self,
        template_id: Identifier,
        arguments: DamlRecord,
    ) -> SdkResult<CreatedEvent> {
        let command = Command::Create(CreateCommand {
            template_id,
            create_arguments: arguments,
        });
        
        let transaction = self.submit_command(command).await?;
        
        // Extract created event from transaction
        transaction
            .events
            .into_iter()
            .find_map(|event| match event {
                Event::Created(created) => Some(created),
                _ => None,
            })
            .ok_or(SdkError::Transaction {
                kind: TransactionErrorKind::ContractNotFound,
                transaction_id: Some(transaction.transaction_id.clone()),
                details: "No created event in transaction".to_string(),
                source: None,
            })
    }
    
    fn get_transactions(
        &self,
        begin: LedgerOffset,
        end: Option<LedgerOffset>,
        filter: TransactionFilter,
    ) -> impl Stream<Item = SdkResult<Transaction>> {
        self.ledger_client.get_transactions(begin, end, filter)
    }
    
    async fn sign(&self, data: &[u8]) -> SdkResult<Signature> {
        // Get signing key for party
        let key_fingerprint = self.key_store
            .get_key_for_party(&self.party_id)
            .await
            .ok_or(SdkError::KeyNotFound)?;
        
        self.key_store.sign(&key_fingerprint, data).await
    }
    
    async fn get_balance(&self) -> SdkResult<WalletBalance> {
        // Canton doesn't have native tokens, but can track contract values
        let contracts = self.get_active_contracts(None).await?;
        
        // Calculate balance from relevant contracts
        let balance = contracts
            .iter()
            .filter_map(|contract| {
                // Extract balance from contract arguments
                contract.create_arguments.get_field("amount")
                    .and_then(|v| v.as_numeric())
            })
            .fold(Decimal::ZERO, |acc, amount| acc + amount);
        
        Ok(WalletBalance {
            amount: balance,
            contracts: contracts.len(),
        })
    }
}

/// Canton wallet configuration
#[derive(Debug, Clone)]
pub struct CantonWalletConfig {
    pub party_id: PartyId,
    pub participant_id: Option<ParticipantId>,
    pub ledger_id: Option<String>,
    pub timeout: Duration,
    pub retry_policy: RetryPolicy,
}

impl Default for CantonWalletConfig {
    fn default() -> Self {
        Self {
            party_id: PartyId::new_unchecked(""),
            participant_id: None,
            ledger_id: None,
            timeout: Duration::from_secs(30),
            retry_policy: RetryPolicy::default(),
        }
    }
}

/// Wallet metadata
#[derive(Debug, Clone)]
pub struct WalletMetadata {
    pub display_name: String,
    pub is_local: bool,
    pub created_at: DateTime<Utc>,
}

/// Wallet balance
#[derive(Debug, Clone)]
pub struct WalletBalance {
    pub amount: Decimal,
    pub contracts: usize,
}

/// Wallet metrics
pub struct WalletMetrics {
    commands_submitted: Counter<u64>,
    command_latency: Histogram<f64>,
    contracts_created: Counter<u64>,
    contracts_exercised: Counter<u64>,
}

impl WalletMetrics {
    pub fn new() -> Self {
        Self {
            commands_submitted: Counter::new(0),
            command_latency: Histogram::new(),
            contracts_created: Counter::new(0),
            contracts_exercised: Counter::new(0),
        }
    }
    
    pub fn record_command_submission(&self, latency: Duration) {
        self.commands_submitted.increment(1);
        self.command_latency.record(latency.as_secs_f64());
    }
}
```

### 3.2 Multi-Party Wallet Support

```rust
/// Multi-party wallet manager
pub struct MultiPartyWalletManager {
    wallets: HashMap<PartyId, Arc<dyn CantonWallet>>,
    ledger_client: Arc<LedgerClient>,
    key_store: Arc<dyn KeyStore>,
}

impl MultiPartyWalletManager {
    pub fn new(
        ledger_client: Arc<LedgerClient>,
        key_store: Arc<dyn KeyStore>,
    ) -> Self {
        Self {
            wallets: HashMap::new(),
            ledger_client,
            key_store,
        }
    }
    
    pub async fn add_wallet(
        &mut self,
        party_id: PartyId,
    ) -> SdkResult<Arc<dyn CantonWallet>> {
        if self.wallets.contains_key(&party_id) {
            return Ok(self.wallets[&party_id].clone());
        }
        
        let config = CantonWalletConfig {
            party_id: party_id.clone(),
            ..Default::default()
        };
        
        let wallet = Arc::new(CantonWalletImpl::new(
            config,
            self.ledger_client.clone(),
            self.key_store.clone(),
        ).await?);
        
        self.wallets.insert(party_id.clone(), wallet.clone());
        
        Ok(wallet)
    }
    
    pub fn get_wallet(&self, party_id: &PartyId) -> Option<Arc<dyn CantonWallet>> {
        self.wallets.get(party_id).cloned()
    }
    
    pub fn list_wallets(&self) -> Vec<&PartyId> {
        self.wallets.keys().collect()
    }
}
```

## 4. Integration with Canton Participant Node

### 4.1 Connection Architecture

```rust
/// Canton wallet connection manager
pub struct CantonWalletConnection {
    /// Ledger API client
    ledger_client: Arc<LedgerClient>,
    
    /// Admin API client (for advanced operations)
    admin_client: Option<Arc<AdminClient>>,
    
    /// Connection pool
    connection_pool: ConnectionPool,
    
    /// Health checker
    health_checker: HealthChecker,
}

impl CantonWalletConnection {
    pub async fn connect(config: CantonConfig) -> SdkResult<Self> {
        // Create ledger API client
        let ledger_client = Arc::new(LedgerClient::connect(&config).await?);
        
        // Optionally create admin client
        let admin_client = if config.admin_api_enabled {
            Some(Arc::new(AdminClient::connect(&config).await?))
        } else {
            None
        };
        
        // Create connection pool
        let connection_pool = ConnectionPool::new(config.connection_pool)?;
        
        // Initialize health checker
        let health_checker = HealthChecker::new(
            ledger_client.clone(),
            config.health_check_interval,
        );
        
        Ok(Self {
            ledger_client,
            admin_client,
            connection_pool,
            health_checker,
        })
    }
    
    pub fn ledger_client(&self) -> &Arc<LedgerClient> {
        &self.ledger_client
    }
    
    pub async fn health_check(&self) -> HealthStatus {
        self.health_checker.check().await
    }
}
```

## 5. Key Management for Canton Wallet

### 5.1 Party Key Association

```rust
/// Party key manager
pub struct PartyKeyManager {
    key_store: Arc<dyn KeyStore>,
    party_key_mappings: Arc<RwLock<HashMap<PartyId, KeyFingerprint>>>,
}

impl PartyKeyManager {
    pub fn new(key_store: Arc<dyn KeyStore>) -> Self {
        Self {
            key_store,
            party_key_mappings: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    pub async fn associate_key(
        &self,
        party_id: PartyId,
        key_fingerprint: KeyFingerprint,
    ) -> SdkResult<()> {
        let mut mappings = self.party_key_mappings.write().await;
        mappings.insert(party_id, key_fingerprint);
        Ok(())
    }
    
    pub async fn get_key_for_party(
        &self,
        party_id: &PartyId,
    ) -> Option<KeyFingerprint> {
        let mappings = self.party_key_mappings.read().await;
        mappings.get(party_id).cloned()
    }
    
    pub async fn generate_key_for_party(
        &self,
        party_id: PartyId,
        algorithm: KeyAlgorithm,
    ) -> SdkResult<KeyFingerprint> {
        let metadata = KeyMetadata {
            name: Some(format!("Key for party {}", party_id)),
            description: Some("Party signing key".to_string()),
            tags: {
                let mut tags = HashMap::new();
                tags.insert("party_id".to_string(), party_id.as_str().to_string());
                tags
            },
            created_at: Utc::now(),
            expires_at: None,
        };
        
        let fingerprint = self.key_store
            .generate_key(algorithm, KeyPurpose::Signing, metadata)
            .await?;
        
        self.associate_key(party_id, fingerprint).await?;
        
        Ok(fingerprint)
    }
}
```

## 6. Recommendations

### 6.1 Canton Wallet Implementation Strategy

**Recommended Approach**: Build a custom Canton wallet component based on:

1. **Ledger API Client**: Use existing `canton-ledger-api` crate
2. **Key Management**: Use `canton-crypto` crate for secure key operations
3. **Party Management**: Implement party allocation and management
4. **Transaction Operations**: Wrap Ledger API operations in wallet interface

**Rationale**:
- Canton doesn't have traditional wallet libraries
- Official solutions are not Rust-native
- Building on Ledger API provides full control
- Aligns with Canton's architecture

### 6.2 Key Features to Implement

1. **Party Management**
   - Allocate new parties
   - List known parties
   - Party metadata management

2. **Transaction Operations**
   - Submit commands
   - Create contracts
   - Exercise choices
   - Query transactions

3. **Key Management**
   - Generate signing keys
   - Associate keys with parties
   - Sign transactions

4. **Contract Management**
   - Query active contracts
   - Track contract state
   - Contract history

5. **Multi-Party Support**
   - Manage multiple parties
   - Switch between parties
   - Party isolation

### 6.3 Integration Points

```
Canton Wallet Component
├── Ledger API Client (canton-ledger-api)
├── Key Store (canton-crypto)
├── Party Management
├── Transaction Builder
└── Event Stream Handler
```

## 7. Security Considerations

### 7.1 Key Security

- Use HSM for production key storage
- Implement key rotation
- Secure key derivation
- Zeroize sensitive data

### 7.2 Transaction Security

- Validate all commands before submission
- Implement nonce management
- Verify transaction receipts
- Monitor for suspicious activity

### 7.3 Party Security

- Validate party IDs
- Implement party access controls
- Monitor party activity
- Audit party operations

## 8. Testing Strategy

### 8.1 Unit Tests

- Wallet operations
- Key management
- Party management
- Transaction building

### 8.2 Integration Tests

- Ledger API integration
- Multi-party operations
- Cross-party transactions
- Error handling

### 8.3 E2E Tests

- Full transaction flows
- Contract lifecycle
- Multi-party scenarios
- Error recovery

## 9. References

- Canton Network Documentation: https://docs.canton.network
- Daml Ledger API: https://docs.daml.com/
- Canton Navigator: https://github.com/digital-asset/daml-navigator
- Canton Console: https://docs.canton.network/canton/usermanual/console.html
