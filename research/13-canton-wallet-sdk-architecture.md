# Canton Wallet SDK Architecture

## 1. Executive Summary

This document presents a comprehensive, production-ready architecture for the Canton Wallet SDK in Rust. The architecture integrates best practices from major blockchain ecosystems (Ethereum, Solana, Cosmos, Polkadot) while addressing Canton Network's unique requirements for privacy, multi-domain support, and smart contract interactions.

## 2. Architecture Overview

### 2.1 High-Level Architecture

```
┌─────────────────────────────────────────────────────────────────────────────────┐
│                        Canton Wallet SDK                                     │
├─────────────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  ┌──────────────────────────────────────────────────────────────────────────┐ │
│  │                      Public API Layer                                  │ │
│  │  ┌──────────────┐ ┌──────────────┐ ┌──────────────┐ ┌──────────┐ │ │
│  │  │ CantonWallet │ │ HDWallet     │ │ MultiSig     │ │ Recovery │ │ │
│  │  │              │ │              │ │ Wallet       │ │ Manager  │ │ │
│  │  └──────────────┘ └──────────────┘ └──────────────┘ └──────────┘ │ │
│  └──────────────────────────────────────────────────────────────────────────┘ │
│                                    │                                     │
│  ┌──────────────────────────────────────────────────────────────────────────┐ │
│  │                      Core Wallet Layer                                │ │
│  │  ┌──────────────┐ ┌──────────────┐ ┌──────────────┐ ┌──────────┐ │ │
│  │  │ KeyManager   │ │ TxBuilder    │ │ TxValidator  │ │ Signer   │ │ │
│  │  │              │ │              │ │              │ │          │ │ │
│  │  └──────────────┘ └──────────────┘ └──────────────┘ └──────────┘ │ │
│  │  ┌──────────────┐ ┌──────────────┐ ┌──────────────┐ ┌──────────┐ │ │
│  │  │ PartyManager │ │ ContractMgr  │ │ EventStream  │ │ Balance  │ │ │
│  │  │              │ │              │ │              │ │ Tracker  │ │ │
│  │  └──────────────┘ └──────────────┘ └──────────────┘ └──────────┘ │ │
│  └──────────────────────────────────────────────────────────────────────────┘ │
│                                    │                                     │
│  ┌──────────────────────────────────────────────────────────────────────────┐ │
│  │                    Canton Integration Layer                            │ │
│  │  ┌──────────────┐ ┌──────────────┐ ┌──────────────┐ ┌──────────┐ │ │
│  │  │ LedgerClient │ │ PartyService │ │ PackageSvc   │ │ AdminSvc │ │ │
│  │  │              │ │              │ │              │ │          │ │ │
│  │  └──────────────┘ └──────────────┘ └──────────────┘ └──────────┘ │ │
│  └──────────────────────────────────────────────────────────────────────────┘ │
│                                    │                                     │
│  ┌──────────────────────────────────────────────────────────────────────────┐ │
│  │                    Security & Crypto Layer                             │ │
│  │  ┌──────────────┐ ┌──────────────┐ ┌──────────────┐ ┌──────────┐ │ │
│  │  │ KeyStore     │ │ CryptoOps    │ │ SecureStore  │ │ HSM      │ │ │
│  │  │              │ │              │ │              │ │ Support  │ │ │
│  │  └──────────────┘ └──────────────┘ └──────────────┘ └──────────┘ │ │
│  └──────────────────────────────────────────────────────────────────────────┘ │
│                                    │                                     │
│  ┌──────────────────────────────────────────────────────────────────────────┐ │
│  │                    Infrastructure Layer                                │ │
│  │  ┌──────────────┐ ┌──────────────┐ ┌──────────────┐ ┌──────────┐ │ │
│  │  │ Transport    │ │ Reliability  │ │ Observability │ │ Config   │ │ │
│  │  │ (gRPC/TLS)   │ │ (Retry/CB)   │ │ (Logs/Metrics)│ │          │ │ │
│  │  └──────────────┘ └──────────────┘ └──────────────┘ └──────────┘ │ │
│  └──────────────────────────────────────────────────────────────────────────┘ │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────────────┘
```

### 2.2 Crate Structure

```
canton-wallet-sdk/
├── Cargo.toml                          # Workspace root
├── rust-toolchain.toml
├── .cargo/config.toml
│
├── crates/
│   │
│   ├── canton-wallet/                  # Main wallet facade
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── wallet.rs
│   │       ├── hd_wallet.rs
│   │       ├── multisig.rs
│   │       └── recovery.rs
│   │
│   ├── canton-wallet-core/             # Core wallet types
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── types/
│   │       │   ├── mod.rs
│   │       │   ├── wallet.rs
│   │       │   ├── transaction.rs
│   │       │   ├── party.rs
│   │       │   ├── contract.rs
│   │       │   └── balance.rs
│   │       ├── traits/
│   │       │   ├── mod.rs
│   │       │   ├── wallet.rs
│   │       │   ├── signer.rs
│   │       │   └── key_manager.rs
│   │       └── error.rs
│   │
│   ├── canton-wallet-crypto/           # Cryptographic operations
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── keys/
│   │       │   ├── mod.rs
│   │       │   ├── ed25519.rs
│   │       │   ├── ecdsa.rs
│   │       │   └── derivation.rs
│   │       ├── keystore/
│   │       │   ├── mod.rs
│   │       │   ├── memory.rs
│   │       │   ├── encrypted.rs
│   │       │   ├── hsm.rs
│   │       │   └── traits.rs
│   │       ├── signing.rs
│   │       ├── encryption.rs
│   │       └── random.rs
│   │
│   ├── canton-wallet-ledger/           # Ledger API integration
│   │   ├── Cargo.toml
│   │   ├── build.rs
│   │   ├── proto/
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── client.rs
│   │       ├── services/
│   │       │   ├── mod.rs
│   │       │   ├── command.rs
│   │       │   ├── transaction.rs
│   │       │   ├── party.rs
│   │       │   └── package.rs
│   │       ├── streaming.rs
│   │       └── conversion.rs
│   │
│   ├── canton-wallet-transactions/      # Transaction management
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── builder.rs
│   │       ├── validator.rs
│   │       ├── estimator.rs
│   │       ├── batch.rs
│   │       └── signer.rs
│   │
│   ├── canton-wallet-contracts/        # Contract management
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── manager.rs
│   │       ├── template.rs
│   │       ├── choice.rs
│   │       └── query.rs
│   │
│   ├── canton-wallet-events/           # Event streaming
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── stream.rs
│   │       ├── filter.rs
│   │       ├── processor.rs
│   │       └── subscription.rs
│   │
│   ├── canton-wallet-security/         # Security features
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── validation.rs
│   │       ├── approval.rs
│   │       ├── audit.rs
│   │       └── rate_limit.rs
│   │
│   ├── canton-wallet-recovery/         # Recovery mechanisms
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── backup.rs
│   │       ├── restore.rs
│   │       ├── social.rs
│   │       └── rotation.rs
│   │
│   ├── canton-wallet-omnichain/        # OmniChain integration
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── client.rs
│   │       ├── bridge.rs
│   │       ├── adapters/
│   │       │   ├── mod.rs
│   │       │   ├── canton.rs
│   │       │   ├── ethereum.rs
│   │       │   ├── cosmos.rs
│   │       │   └── substrate.rs
│   │       └── swap.rs
│   │
│   └── canton-wallet-testing/         # Testing utilities
│       ├── Cargo.toml
│       └── src/
│           ├── lib.rs
│           ├── fixtures.rs
│           ├── mocks.rs
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
│   ├── basic_wallet/
│   ├── hd_wallet/
│   ├── multisig/
│   ├── contracts/
│   └── omnichain/
│
└── docs/                               # Documentation
    ├── architecture.md
    ├── getting_started.md
    ├── api/
    └── security.md
```

## 3. Core Components

### 3.1 Wallet Interface

```rust
// canton-wallet-core/src/traits/wallet.rs

use async_trait::async_trait;
use std::future::Future;

/// Core wallet trait
#[async_trait]
pub trait Wallet: Send + Sync {
    /// Get wallet ID
    fn wallet_id(&self) -> &WalletId;
    
    /// Get party ID
    fn party_id(&self) -> &PartyId;
    
    /// Get participant ID
    fn participant_id(&self) -> &ParticipantId;
    
    /// Get wallet address
    async fn address(&self) -> Result<String, WalletError>;
    
    /// Get balance
    async fn balance(&self) -> Result<WalletBalance, WalletError>;
    
    /// Submit command
    async fn submit_command(
        &self,
        command: Command,
    ) -> Result<Transaction, WalletError>;
    
    /// Submit and wait for transaction
    async fn submit_and_wait(
        &self,
        command: Command,
    ) -> Result<Transaction, WalletError>;
    
    /// Get active contracts
    async fn active_contracts(
        &self,
        filter: Option<TransactionFilter>,
    ) -> Result<Vec<CreatedEvent>, WalletError>;
    
    /// Exercise choice on contract
    async fn exercise_choice(
        &self,
        contract_id: ContractId,
        choice: &str,
        argument: DamlValue,
    ) -> Result<Transaction, WalletError>;
    
    /// Create contract
    async fn create_contract(
        &self,
        template_id: Identifier,
        arguments: DamlRecord,
    ) -> Result<CreatedEvent, WalletError>;
    
    /// Get transaction history
    fn transactions(
        &self,
        begin: LedgerOffset,
        end: Option<LedgerOffset>,
        filter: TransactionFilter,
    ) -> impl Stream<Item = Result<Transaction, WalletError>> + Send;
    
    /// Sign data
    async fn sign(&self, data: &[u8]) -> Result<Signature, WalletError>;
    
    /// Verify signature
    async fn verify(
        &self,
        data: &[u8],
        signature: &Signature,
    ) -> Result<bool, WalletError>;
}

/// HD wallet trait
#[async_trait]
pub trait HDWallet: Wallet {
    /// Get mnemonic phrase
    fn mnemonic(&self) -> &str;
    
    /// Derive account at index
    async fn derive_account(&mut self, index: u32) -> Result<PartyId, WalletError>;
    
    /// Get derived accounts
    fn accounts(&self) -> &[PartyId];
}

/// Multi-signature wallet trait
#[async_trait]
pub trait MultiSigWallet: Wallet {
    /// Get signers
    fn signers(&self) -> &[PartyId];
    
    /// Get threshold
    fn threshold(&self) -> usize;
    
    /// Add signature
    async fn add_signature(
        &mut self,
        tx_id: &TxId,
        signer: PartyId,
        signature: Signature,
    ) -> Result<(), WalletError>;
    
    /// Check if transaction is ready
    fn is_ready(&self, tx_id: &TxId) -> bool;
    
    /// Execute transaction
    async fn execute(&mut self, tx_id: &TxId) -> Result<Transaction, WalletError>;
}
```

### 3.2 Key Management

```rust
// canton-wallet-crypto/src/keystore/traits.rs

use async_trait::async_trait;

/// Key store trait
#[async_trait]
pub trait KeyStore: Send + Sync {
    /// Generate new key
    async fn generate_key(
        &self,
        algorithm: KeyAlgorithm,
        purpose: KeyPurpose,
        metadata: KeyMetadata,
    ) -> Result<KeyId, KeyStoreError>;
    
    /// Import existing key
    async fn import_key(
        &self,
        key_bytes: &[u8],
        algorithm: KeyAlgorithm,
        purpose: KeyPurpose,
        metadata: KeyMetadata,
    ) -> Result<KeyId, KeyStoreError>;
    
    /// Export public key
    async fn export_public_key(
        &self,
        key_id: &KeyId,
    ) -> Result<PublicKey, KeyStoreError>;
    
    /// Sign data
    async fn sign(
        &self,
        key_id: &KeyId,
        data: &[u8],
    ) -> Result<Signature, KeyStoreError>;
    
    /// Verify signature
    async fn verify(
        &self,
        key_id: &KeyId,
        data: &[u8],
        signature: &Signature,
    ) -> Result<bool, KeyStoreError>;
    
    /// Delete key
    async fn delete_key(
        &self,
        key_id: &KeyId,
    ) -> Result<(), KeyStoreError>;
    
    /// List all keys
    async fn list_keys(&self) -> Result<Vec<KeyInfo>, KeyStoreError>;
    
    /// Get key info
    async fn get_key_info(
        &self,
        key_id: &KeyId,
    ) -> Result<KeyInfo, KeyStoreError>;
    
    /// Rotate key
    async fn rotate_key(
        &self,
        old_key_id: &KeyId,
        new_algorithm: KeyAlgorithm,
    ) -> Result<KeyId, KeyStoreError>;
}

/// Key manager trait
#[async_trait]
pub trait KeyManager: Send + Sync {
    /// Get key store
    fn key_store(&self) -> &dyn KeyStore;
    
    /// Get key for party
    async fn get_key_for_party(
        &self,
        party_id: &PartyId,
    ) -> Result<KeyId, KeyManagerError>;
    
    /// Associate key with party
    async fn associate_key(
        &mut self,
        party_id: PartyId,
        key_id: KeyId,
    ) -> Result<(), KeyManagerError>;
    
    /// Generate key for party
    async fn generate_key_for_party(
        &mut self,
        party_id: PartyId,
        algorithm: KeyAlgorithm,
    ) -> Result<KeyId, KeyManagerError>;
}
```

### 3.3 Transaction Management

```rust
// canton-wallet-transactions/src/builder.rs

/// Transaction builder
pub struct TransactionBuilder {
    party_id: Option<PartyId>,
    commands: Vec<Command>,
    workflow_id: Option<String>,
    application_id: Option<String>,
    command_id: Option<String>,
    act_as: Vec<PartyId>,
    read_as: Vec<PartyId>,
    min_ledger_time: Option<DateTime<Utc>>,
    deduplication_period: Option<Duration>,
    validator: TransactionValidator,
}

impl TransactionBuilder {
    pub fn new() -> Self {
        Self {
            party_id: None,
            commands: Vec::new(),
            workflow_id: None,
            application_id: None,
            command_id: None,
            act_as: Vec::new(),
            read_as: Vec::new(),
            min_ledger_time: None,
            deduplication_period: None,
            validator: TransactionValidator::default(),
        }
    }
    
    pub fn party_id(mut self, party_id: PartyId) -> Self {
        self.party_id = Some(party_id);
        self
    }
    
    pub fn add_command(mut self, command: Command) -> Self {
        self.commands.push(command);
        self
    }
    
    pub fn workflow_id(mut self, id: impl Into<String>) -> Self {
        self.workflow_id = Some(id.into());
        self
    }
    
    pub fn application_id(mut self, id: impl Into<String>) -> Self {
        self.application_id = Some(id.into());
        self
    }
    
    pub fn act_as(mut self, party_id: PartyId) -> Self {
        self.act_as.push(party_id);
        self
    }
    
    pub fn read_as(mut self, party_id: PartyId) -> Self {
        self.read_as.push(party_id);
        self
    }
    
    pub fn min_ledger_time(mut self, time: DateTime<Utc>) -> Self {
        self.min_ledger_time = Some(time);
        self
    }
    
    pub fn deduplication_period(mut self, period: Duration) -> Self {
        self.deduplication_period = Some(period);
        self
    }
    
    pub fn with_validator(mut self, validator: TransactionValidator) -> Self {
        self.validator = validator;
        self
    }
    
    /// Build transaction
    pub fn build(self) -> Result<Commands, WalletError> {
        let party_id = self.party_id.ok_or(WalletError::MissingPartyId)?;
        
        let commands = Commands {
            ledger_id: String::new(), // Will be set by client
            workflow_id: self.workflow_id.unwrap_or_else(|| Uuid::new_v4().to_string()),
            application_id: self.application_id.unwrap_or_else(|| "canton-wallet-sdk".to_string()),
            command_id: self.command_id.unwrap_or_else(|| Uuid::new_v4().to_string()),
            party: party_id.to_string(),
            commands: self.commands,
            act_as: if self.act_as.is_empty() {
                vec![party_id.to_string()]
            } else {
                self.act_as.iter().map(|p| p.to_string()).collect()
            },
            read_as: self.read_as.iter().map(|p| p.to_string()).collect(),
            min_ledger_time_abs: self.min_ledger_time.map(|t| t.into()),
            min_ledger_time_rel: self.deduplication_period.map(|d| d.into()),
            ..Default::default()
        };
        
        // Validate transaction
        self.validator.validate(&commands)?;
        
        Ok(commands)
    }
}

/// Transaction validator
pub struct TransactionValidator {
    max_commands: usize,
    max_act_as: usize,
    max_read_as: usize,
    allowed_templates: Option<HashSet<Identifier>>,
}

impl Default for TransactionValidator {
    fn default() -> Self {
        Self {
            max_commands: 100,
            max_act_as: 10,
            max_read_as: 10,
            allowed_templates: None,
        }
    }
}

impl TransactionValidator {
    pub fn validate(&self, commands: &Commands) -> Result<(), WalletError> {
        // Validate command count
        if commands.commands.len() > self.max_commands {
            return Err(WalletError::TooManyCommands {
                actual: commands.commands.len(),
                max: self.max_commands,
            });
        }
        
        // Validate act_as count
        if commands.act_as.len() > self.max_act_as {
            return Err(WalletError::TooManyActAs {
                actual: commands.act_as.len(),
                max: self.max_act_as,
            });
        }
        
        // Validate read_as count
        if commands.read_as.len() > self.max_read_as {
            return Err(WalletError::TooManyReadAs {
                actual: commands.read_as.len(),
                max: self.max_read_as,
            });
        }
        
        // Validate templates
        if let Some(allowed) = &self.allowed_templates {
            for command in &commands.commands {
                let template_id = command.template_id()?;
                if !allowed.contains(&template_id) {
                    return Err(WalletError::UnauthorizedTemplate(template_id));
                }
            }
        }
        
        Ok(())
    }
}
```

### 3.4 Contract Management

```rust
// canton-wallet-contracts/src/manager.rs

/// Contract manager
pub struct ContractManager {
    ledger_client: Arc<LedgerClient>,
    party_id: PartyId,
    cache: Arc<RwLock<HashMap<ContractId, ContractInfo>>>,
}

impl ContractManager {
    pub fn new(ledger_client: Arc<LedgerClient>, party_id: PartyId) -> Self {
        Self {
            ledger_client,
            party_id,
            cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Create contract
    pub async fn create(
        &self,
        template_id: Identifier,
        arguments: DamlRecord,
    ) -> Result<CreatedEvent, WalletError> {
        let command = Command::Create(CreateCommand {
            template_id,
            create_arguments: arguments,
        });
        
        let transaction = self.ledger_client
            .submit_and_wait_for_party(&self.party_id, command)
            .await?;
        
        // Extract created event
        transaction
            .events
            .into_iter()
            .find_map(|event| match event {
                Event::Created(created) => Some(created),
                _ => None,
            })
            .ok_or(WalletError::ContractCreationFailed)
    }
    
    /// Exercise choice on contract
    pub async fn exercise(
        &self,
        contract_id: ContractId,
        choice: &str,
        argument: DamlValue,
    ) -> Result<Transaction, WalletError> {
        let command = Command::Exercise(ExerciseCommand {
            template_id: None,
            contract_id: contract_id.to_string(),
            choice: choice.to_string(),
            choice_argument: argument.into(),
        });
        
        self.ledger_client
            .submit_and_wait_for_party(&self.party_id, command)
            .await
    }
    
    /// Exercise choice by key
    pub async fn exercise_by_key(
        &self,
        template_id: Identifier,
        contract_key: DamlValue,
        choice: &str,
        argument: DamlValue,
    ) -> Result<Transaction, WalletError> {
        let command = Command::ExerciseByKey(ExerciseByKeyCommand {
            template_id,
            contract_key: contract_key.into(),
            choice: choice.to_string(),
            choice_argument: argument.into(),
        });
        
        self.ledger_client
            .submit_and_wait_for_party(&self.party_id, command)
            .await
    }
    
    /// Create and exercise
    pub async fn create_and_exercise(
        &self,
        template_id: Identifier,
        create_arguments: DamlRecord,
        choice: &str,
        choice_argument: DamlValue,
    ) -> Result<Transaction, WalletError> {
        let command = Command::CreateAndExercise(CreateAndExerciseCommand {
            template_id,
            create_arguments,
            choice: choice.to_string(),
            choice_argument: choice_argument.into(),
        });
        
        self.ledger_client
            .submit_and_wait_for_party(&self.party_id, command)
            .await
    }
    
    /// Get active contracts
    pub async fn active_contracts(
        &self,
        filter: Option<TransactionFilter>,
    ) -> Result<Vec<CreatedEvent>, WalletError> {
        let filter = filter.unwrap_or_else(|| {
            TransactionFilter::for_party(&self.party_id)
        });
        
        self.ledger_client.get_active_contracts(filter).await
    }
    
    /// Get contract by ID
    pub async fn get_contract(
        &self,
        contract_id: ContractId,
    ) -> Result<ContractInfo, WalletError> {
        // Check cache first
        {
            let cache = self.cache.read().await;
            if let Some(info) = cache.get(&contract_id) {
                return Ok(info.clone());
            }
        }
        
        // Fetch from ledger
        let contracts = self.active_contracts(None).await?;
        let contract = contracts
            .into_iter()
            .find(|c| c.contract_id == contract_id.to_string())
            .ok_or(WalletError::ContractNotFound(contract_id))?;
        
        let info = ContractInfo::from_created_event(&contract);
        
        // Update cache
        {
            let mut cache = self.cache.write().await;
            cache.insert(contract_id, info.clone());
        }
        
        Ok(info)
    }
    
    /// Query contracts by template
    pub async fn query_by_template(
        &self,
        template_id: Identifier,
    ) -> Result<Vec<CreatedEvent>, WalletError> {
        let filter = TransactionFilter::for_template(&self.party_id, template_id);
        self.active_contracts(Some(filter)).await
    }
    
    /// Archive contract
    pub async fn archive(
        &self,
        contract_id: ContractId,
    ) -> Result<Transaction, WalletError> {
        // Archive choice is typically "Archive"
        self.exercise(contract_id, "Archive", DamlValue::unit()).await
    }
}
```

### 3.5 Event Streaming

```rust
// canton-wallet-events/src/stream.rs

/// Event stream
pub struct EventStream {
    ledger_client: Arc<LedgerClient>,
    party_id: PartyId,
    filter: TransactionFilter,
    offset: LedgerOffset,
    buffer_size: usize,
}

impl EventStream {
    pub fn new(
        ledger_client: Arc<LedgerClient>,
        party_id: PartyId,
        filter: TransactionFilter,
    ) -> Self {
        Self {
            ledger_client,
            party_id,
            filter,
            offset: LedgerOffset::Begin,
            buffer_size: 100,
        }
    }
    
    pub fn with_offset(mut self, offset: LedgerOffset) -> Self {
        self.offset = offset;
        self
    }
    
    pub fn with_buffer_size(mut self, size: usize) -> Self {
        self.buffer_size = size;
        self
    }
    
    /// Subscribe to events
    pub fn subscribe(&self) -> impl Stream<Item = Result<Transaction, WalletError>> + Send {
        let client = self.ledger_client.clone();
        let party_id = self.party_id.clone();
        let filter = self.filter.clone();
        let offset = self.offset.clone();
        
        async_stream::try_stream! {
            let mut current_offset = offset;
            
            loop {
                let transactions = client
                    .get_transactions(current_offset.clone(), None, filter.clone())
                    .await?;
                
                for tx in transactions {
                    current_offset = LedgerOffset::Absolute(tx.offset.clone());
                    yield tx;
                }
                
                // Wait before polling again
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
        }
    }
    
    /// Subscribe to events with callback
    pub async fn subscribe_with_callback<F, Fut>(
        &self,
        callback: F,
    ) -> Result<(), WalletError>
    where
        F: Fn(Transaction) -> Fut + Send + Sync,
        Fut: Future<Output = Result<(), WalletError>> + Send,
    {
        let mut stream = self.subscribe();
        
        while let Some(result) = stream.next().await {
            match result {
                Ok(tx) => {
                    if let Err(e) = callback(tx).await {
                        tracing::error!("Callback error: {}", e);
                    }
                }
                Err(e) => {
                    tracing::error!("Stream error: {}", e);
                }
            }
        }
        
        Ok(())
    }
}

/// Event processor
pub struct EventProcessor {
    handlers: HashMap<String, Box<dyn EventHandler>>,
}

impl EventProcessor {
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
        }
    }
    
    pub fn register_handler(&mut self, template_id: Identifier, handler: Box<dyn EventHandler>) {
        self.handlers.insert(template_id.qualified_name(), handler);
    }
    
    pub async fn process_transaction(&self, tx: Transaction) -> Result<(), WalletError> {
        for event in tx.events {
            match event {
                Event::Created(created) => {
                    if let Some(handler) = self.handlers.get(&created.template_id.qualified_name()) {
                        handler.on_created(created).await?;
                    }
                }
                Event::Exercised(exercised) => {
                    if let Some(handler) = self.handlers.get(&exercised.template_id.qualified_name()) {
                        handler.on_exercised(exercised).await?;
                    }
                }
                Event::Archived(archived) => {
                    if let Some(handler) = self.handlers.get(&archived.template_id.qualified_name()) {
                        handler.on_archived(archived).await?;
                    }
                }
            }
        }
        
        Ok(())
    }
}

/// Event handler trait
#[async_trait]
pub trait EventHandler: Send + Sync {
    async fn on_created(&self, event: CreatedEvent) -> Result<(), WalletError>;
    async fn on_exercised(&self, event: ExercisedEvent) -> Result<(), WalletError>;
    async fn on_archived(&self, event: ArchivedEvent) -> Result<(), WalletError>;
}
```

## 4. Security Architecture

### 4.1 Secure Key Storage

```rust
// canton-wallet-crypto/src/keystore/encrypted.rs

/// Encrypted key store
pub struct EncryptedKeyStore {
    encryption_key: [u8; 32],
    storage: Box<dyn StorageBackend>,
    key_cache: Arc<RwLock<HashMap<KeyId, CachedKey>>>,
}

struct CachedKey {
    public_key: PublicKey,
    algorithm: KeyAlgorithm,
    purpose: KeyPurpose,
    metadata: KeyMetadata,
}

impl EncryptedKeyStore {
    pub fn new(encryption_key: [u8; 32], storage: Box<dyn StorageBackend>) -> Self {
        Self {
            encryption_key,
            storage,
            key_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    async fn encrypt_key(&self, key: &SecurePrivateKey) -> Result<EncryptedKey, KeyStoreError> {
        let key_bytes = key.as_bytes();
        
        // Use AES-256-GCM for encryption
        let cipher = Aes256Gcm::new(&self.encryption_key.into());
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
        
        let ciphertext = cipher
            .encrypt(&nonce, key_bytes)
            .map_err(|e| KeyStoreError::EncryptionError(e.to_string()))?;
        
        Ok(EncryptedKey {
            nonce: nonce.to_vec(),
            ciphertext,
            algorithm: key.algorithm(),
            purpose: key.purpose(),
        })
    }
    
    async fn decrypt_key(&self, encrypted: &EncryptedKey) -> Result<SecurePrivateKey, KeyStoreError> {
        let cipher = Aes256Gcm::new(&self.encryption_key.into());
        let nonce = Nonce::from_slice(&encrypted.nonce);
        
        let plaintext = cipher
            .decrypt(nonce, encrypted.ciphertext.as_ref())
            .map_err(|e| KeyStoreError::DecryptionError(e.to_string()))?;
        
        Ok(SecurePrivateKey::new(encrypted.algorithm, plaintext))
    }
}

#[async_trait]
impl KeyStore for EncryptedKeyStore {
    async fn generate_key(
        &self,
        algorithm: KeyAlgorithm,
        purpose: KeyPurpose,
        metadata: KeyMetadata,
    ) -> Result<KeyId, KeyStoreError> {
        let key = SecurePrivateKey::generate(algorithm)?;
        let key_id = KeyId::new();
        
        let encrypted = self.encrypt_key(&key).await?;
        self.storage.store(&key_id, &encrypted).await?;
        
        // Cache public key
        let public_key = PublicKey::from_private_key(&key);
        let cached = CachedKey {
            public_key,
            algorithm,
            purpose,
            metadata,
        };
        
        self.key_cache.write().await.insert(key_id.clone(), cached);
        
        Ok(key_id)
    }
    
    async fn sign(
        &self,
        key_id: &KeyId,
        data: &[u8],
    ) -> Result<Signature, KeyStoreError> {
        let encrypted = self.storage.load(key_id).await?;
        let key = self.decrypt_key(&encrypted).await?;
        
        Ok(key.sign(data))
    }
    
    // ... other methods
}
```

### 4.2 Transaction Approval

```rust
// canton-wallet-security/src/approval.rs

/// Transaction approval manager
pub struct ApprovalManager {
    user_approval: Arc<dyn UserApproval>,
    validator: TransactionValidator,
    audit_logger: Arc<AuditLogger>,
}

impl ApprovalManager {
    pub fn new(
        user_approval: Arc<dyn UserApproval>,
        validator: TransactionValidator,
        audit_logger: Arc<AuditLogger>,
    ) -> Self {
        Self {
            user_approval,
            validator,
            audit_logger,
        }
    }
    
    /// Request approval for transaction
    pub async fn request_approval(
        &self,
        tx: &Transaction,
    ) -> Result<ApprovalResponse, WalletError> {
        // Validate transaction
        self.validator.validate(tx)?;
        
        // Log approval request
        self.audit_logger.log(AuditLogEntry {
            timestamp: Utc::now(),
            operation: "transaction_approval_request".to_string(),
            details: serde_json::to_value(tx).unwrap_or_default(),
        }).await?;
        
        // Request user approval
        let response = self.user_approval.request_approval(tx).await?;
        
        // Log approval response
        self.audit_logger.log(AuditLogEntry {
            timestamp: Utc::now(),
            operation: "transaction_approval_response".to_string(),
            details: serde_json::json!({
                "approved": response.approved,
                "timestamp": response.timestamp,
            }),
        }).await?;
        
        if !response.approved {
            return Err(WalletError::UserRejected);
        }
        
        Ok(response)
    }
}

/// User approval trait
#[async_trait]
pub trait UserApproval: Send + Sync {
    async fn request_approval(&self, tx: &Transaction) -> Result<ApprovalResponse, WalletError>;
}

/// Approval response
#[derive(Debug, Clone)]
pub struct ApprovalResponse {
    pub approved: bool,
    pub timestamp: DateTime<Utc>,
    pub metadata: HashMap<String, String>,
}
```

## 5. OmniChain Integration

### 5.1 Multi-Chain Wallet

```rust
// canton-wallet-omnichain/src/client.rs

/// Multi-chain wallet
pub struct MultiChainWallet {
    canton_wallet: Arc<dyn CantonWallet>,
    chain_wallets: HashMap<ChainId, Box<dyn ChainWallet>>,
    bridge_manager: Arc<BridgeManager>,
}

impl MultiChainWallet {
    pub fn new(
        canton_wallet: Arc<dyn CantonWallet>,
        bridge_manager: Arc<BridgeManager>,
    ) -> Self {
        Self {
            canton_wallet,
            chain_wallets: HashMap::new(),
            bridge_manager,
        }
    }
    
    pub fn add_chain(&mut self, chain_id: ChainId, wallet: Box<dyn ChainWallet>) {
        self.chain_wallets.insert(chain_id, wallet);
    }
    
    /// Transfer asset from Canton to another chain
    pub async fn transfer_to_chain(
        &self,
        asset: CantonAsset,
        target_chain: ChainId,
        recipient: ChainAddress,
    ) -> Result<CrossChainTx, WalletError> {
        // Lock asset on Canton
        let lock_receipt = self.bridge_manager
            .lock_on_canton(&self.canton_wallet, asset.clone(), target_chain, recipient.clone())
            .await?;
        
        // Generate proof
        let proof = self.bridge_manager
            .generate_lock_proof(&lock_receipt)
            .await?;
        
        // Release on target chain
        let target_wallet = self.chain_wallets.get(&target_chain)
            .ok_or(WalletError::UnsupportedChain(target_chain))?;
        
        let release_receipt = self.bridge_manager
            .release_on_chain(target_wallet, proof, recipient)
            .await?;
        
        Ok(CrossChainTx {
            canton_tx_id: lock_receipt.tx_id,
            target_tx_id: release_receipt.tx_id,
            asset,
            source_chain: ChainId::Canton,
            target_chain,
            timestamp: Utc::now(),
        })
    }
    
    /// Transfer asset from another chain to Canton
    pub async fn transfer_from_chain(
        &self,
        asset: ChainAsset,
        source_chain: ChainId,
        recipient: PartyId,
    ) -> Result<CrossChainTx, WalletError> {
        // Lock on source chain
        let source_wallet = self.chain_wallets.get(&source_chain)
            .ok_or(WalletError::UnsupportedChain(source_chain))?;
        
        let lock_receipt = self.bridge_manager
            .lock_on_chain(source_wallet, asset.clone(), ChainId::Canton, recipient.clone())
            .await?;
        
        // Generate proof
        let proof = self.bridge_manager
            .generate_lock_proof(&lock_receipt)
            .await?;
        
        // Release on Canton
        let release_receipt = self.bridge_manager
            .release_on_canton(&self.canton_wallet, proof, recipient)
            .await?;
        
        Ok(CrossChainTx {
            canton_tx_id: release_receipt.tx_id,
            target_tx_id: lock_receipt.tx_id,
            asset: CantonAsset::from_chain_asset(asset),
            source_chain,
            target_chain: ChainId::Canton,
            timestamp: Utc::now(),
        })
    }
}
```

## 6. Error Handling

```rust
// canton-wallet-core/src/error.rs

use thiserror::Error;

/// Wallet error types
#[derive(Error, Debug)]
pub enum WalletError {
    #[error("Key not found: {0}")]
    KeyNotFound(String),
    
    #[error("Invalid key format")]
    InvalidKeyFormat,
    
    #[error("Party not found: {0}")]
    PartyNotFound(PartyId),
    
    #[error("Contract not found: {0}")]
    ContractNotFound(ContractId),
    
    #[error("Contract creation failed")]
    ContractCreationFailed,
    
    #[error("Transaction failed: {0}")]
    TransactionFailed(String),
    
    #[error("Invalid transaction: {0}")]
    InvalidTransaction(String),
    
    #[error("Too many commands: {actual} > {max}")]
    TooManyCommands { actual: usize, max: usize },
    
    #[error("Too many act_as: {actual} > {max}")]
    TooManyActAs { actual: usize, max: usize },
    
    #[error("Too many read_as: {actual} > {max}")]
    TooManyReadAs { actual: usize, max: usize },
    
    #[error("Unauthorized template: {0}")]
    UnauthorizedTemplate(Identifier),
    
    #[error("Missing party ID")]
    MissingPartyId,
    
    #[error("User rejected operation")]
    UserRejected,
    
    #[error("Insufficient funds: required {required}, available {available}")]
    InsufficientFunds { required: u64, available: u64 },
    
    #[error("Unsupported chain: {0:?}")]
    UnsupportedChain(ChainId),
    
    #[error("Bridge error: {0}")]
    BridgeError(String),
    
    #[error("Network error: {0}")]
    NetworkError(String),
    
    #[error("Serialization error: {0}")]
    SerializationError(String),
    
    #[error("Internal error: {0}")]
    Internal(String),
}

/// Result type alias
pub type WalletResult<T> = Result<T, WalletError>;
```

## 7. Configuration

```rust
// canton-wallet/src/config.rs

/// Wallet configuration
#[derive(Debug, Clone, Deserialize)]
pub struct WalletConfig {
    /// Ledger API endpoint
    pub ledger_endpoint: String,
    
    /// Admin API endpoint (optional)
    pub admin_endpoint: Option<String>,
    
    /// TLS configuration
    pub tls: Option<TlsConfig>,
    
    /// Key store configuration
    pub key_store: KeyStoreConfig,
    
    /// Transaction configuration
    pub transaction: TransactionConfig,
    
    /// Security configuration
    pub security: SecurityConfig,
    
    /// Observability configuration
    pub observability: ObservabilityConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TlsConfig {
    pub ca_cert_path: Option<String>,
    pub client_cert_path: Option<String>,
    pub client_key_path: Option<String>,
    pub verify_hostname: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct KeyStoreConfig {
    pub backend: KeyStoreBackend,
    pub encryption_key: Option<String>,
    pub storage_path: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub enum KeyStoreBackend {
    Memory,
    EncryptedFile,
    Hsm,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TransactionConfig {
    pub max_commands: usize,
    pub max_act_as: usize,
    pub max_read_as: usize,
    pub timeout: Duration,
    pub retry_policy: RetryPolicy,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SecurityConfig {
    pub require_approval: bool,
    pub rate_limit: Option<RateLimitConfig>,
    pub allowed_templates: Option<Vec<String>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ObservabilityConfig {
    pub logging: LoggingConfig,
    pub metrics: MetricsConfig,
    pub tracing: TracingConfig,
}
```

## 8. Summary

### 8.1 Key Features
- **HD Wallet Support**: BIP39/44 compatible hierarchical deterministic wallets
- **Multi-Signature**: Threshold-based multi-signature wallets
- **Secure Key Management**: Encrypted key storage with HSM support
- **Transaction Management**: Builder pattern with validation and estimation
- **Contract Management**: Create, exercise, query, and archive contracts
- **Event Streaming**: Real-time event subscription and processing
- **Security**: Transaction approval, audit logging, rate limiting
- **Recovery**: Backup/restore and social recovery mechanisms
- **OmniChain**: Cross-chain asset transfers and atomic swaps

### 8.2 Design Principles
- **Type Safety**: Leverage Rust's type system for compile-time guarantees
- **Async-First**: Full async/await support for non-blocking operations
- **Modularity**: Clear separation of concerns with well-defined interfaces
- **Extensibility**: Trait-based design for easy customization
- **Security-First**: Zero-trust architecture with defense in depth
- **Production-Ready**: Comprehensive error handling, logging, and metrics
