// Canton Wallet SDK - Main Facade Crate
//
// This crate provides a unified interface to the Canton Wallet SDK functionality.
// It re-exports the most commonly used types and provides the main CantonWallet
// implementation.

#![warn(missing_docs)]
#![warn(rust_2018_idioms)]
#![warn(clippy::all)]
#![deny(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::missing_errors_doc)]

pub use canton_wallet_core::{
    error::{WalletError, WalletResult},
    traits::Wallet,
    types::{
        Address, ApprovalResponse, AuditLogEntry, Balance, ChainAddress, ChainId,
        Command, Commands, ContractId, ContractInfo, CreatedEvent, DamlRecord,
        DamlValue, Event, Identifier, KeyAlgorithm, KeyId, KeyInfo, KeyMetadata,
        KeyPurpose, LedgerOffset, PartyId, Signature, Transaction,
        TransactionFilter, WalletBalance, WalletId,
    },
};

pub use canton_wallet_crypto::{
    crypto::{CryptoOps, KeyPair},
    hd::{HDWallet, HDAccount},
    keystore::{EncryptedKeyStore, InMemoryKeyStore, KeyStore},
};

pub use canton_wallet_transactions::{
    builder::TransactionBuilder,
    validator::TransactionValidator,
};

pub use canton_wallet_contracts::manager::ContractManager;

pub use canton_wallet_events::stream::EventStream;

pub use canton_wallet_security::{
    approval::{ApprovalManager, UserApproval},
    audit::AuditLogger,
    rate_limit::RateLimiter,
};

pub use canton_wallet_recovery::{
    backup::WalletBackup,
    recovery::RecoveryManager,
};

pub use canton_wallet_omnichain::{
    client::MultiChainWallet,
    bridge::BridgeManager,
};

pub use canton_ledger_api::client::LedgerClient;

pub use canton_observability::{
    logging::{init_logging, LogConfig},
    metrics::{init_metrics, MetricsConfig},
    tracing::{init_tracing, TracingConfig},
};

pub use canton_reliability::{
    retry::{RetryConfig, RetryPolicy},
    circuit_breaker::{CircuitBreaker, CircuitBreakerConfig},
};

pub use canton_transport::{
    config::TransportConfig,
    tls::TlsConfig,
};

use async_trait::async_trait;
use canton_wallet_core::types::CreateCommand;
use canton_wallet_crypto::keystore::KeyStoreConfig;
use canton_wallet_security::approval::UserApproval;
use canton_wallet_security::audit::AuditLogger;
use canton_wallet_security::rate_limit::RateLimiter;
use canton_wallet_transactions::validator::TransactionValidator;
use canton_wallet_transactions::builder::TransactionBuilder;
use canton_wallet_contracts::manager::ContractManager;
use canton_wallet_events::stream::EventStream;
use canton_wallet_recovery::backup::WalletBackup;
use canton_wallet_recovery::recovery::RecoveryManager;
use canton_wallet_omnichain::client::MultiChainWallet;
use canton_wallet_omnichain::bridge::BridgeManager;
use canton_ledger_api::client::LedgerClient;
use canton_observability::logging::init_logging;
use canton_observability::metrics::init_metrics;
use canton_observability::tracing::init_tracing;
use canton_reliability::retry::RetryConfig;
use canton_reliability::circuit_breaker::CircuitBreakerConfig;
use canton_transport::config::TransportConfig;
use canton_transport::tls::TlsConfig;
use canton_wallet_core::error::WalletError;
use canton_wallet_core::traits::Wallet;
use canton_wallet_core::types::{
    Address, ApprovalResponse, AuditLogEntry, Balance, ChainAddress, ChainId,
    Command, Commands, ContractId, ContractInfo, CreatedEvent, DamlRecord,
    DamlValue, Event, Identifier, KeyAlgorithm, KeyId, KeyInfo, KeyMetadata,
    KeyPurpose, LedgerOffset, PartyId, Signature, Transaction,
    TransactionFilter, WalletBalance, WalletId,
};
use canton_wallet_crypto::crypto::{CryptoOps, KeyPair};
use canton_wallet_crypto::hd::{HDWallet, HDAccount};
use canton_wallet_crypto::keystore::{EncryptedKeyStore, InMemoryKeyStore, KeyStore};
use canton_wallet_transactions::builder::TransactionBuilder;
use canton_wallet_transactions::validator::TransactionValidator;
use canton_wallet_contracts::manager::ContractManager;
use canton_wallet_events::stream::EventStream;
use canton_wallet_security::approval::{ApprovalManager, UserApproval};
use canton_wallet_security::audit::AuditLogger;
use canton_wallet_security::rate_limit::RateLimiter;
use canton_wallet_recovery::backup::WalletBackup;
use canton_wallet_recovery::recovery::RecoveryManager;
use canton_wallet_omnichain::client::MultiChainWallet;
use canton_wallet_omnichain::bridge::BridgeManager;
use canton_ledger_api::client::LedgerClient;
use canton_observability::logging::{init_logging, LogConfig};
use canton_observability::metrics::{init_metrics, MetricsConfig};
use canton_observability::tracing::{init_tracing, TracingConfig};
use canton_reliability::retry::{RetryConfig, RetryPolicy};
use canton_reliability::circuit_breaker::{CircuitBreaker, CircuitBreakerConfig};
use canton_transport::config::TransportConfig;
use canton_transport::tls::TlsConfig;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, instrument, warn};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use futures::Stream;

/// Wallet configuration
#[derive(Debug, Clone)]
pub struct WalletConfig {
    /// Ledger endpoint URL
    pub ledger_endpoint: String,

    /// Key store configuration
    pub key_store: KeyStoreConfig,

    /// Party ID (optional, will be generated if not provided)
    pub party_id: Option<PartyId>,

    /// Participant ID (optional, will be derived from party ID if not provided)
    pub participant_id: Option<String>,

    /// Application ID for transactions
    pub application_id: Option<String>,

    /// Transport configuration
    pub transport: Option<TransportConfig>,

    /// TLS configuration
    pub tls: Option<TlsConfig>,

    /// Retry configuration
    pub retry: Option<RetryConfig>,

    /// Circuit breaker configuration
    pub circuit_breaker: Option<CircuitBreakerConfig>,

    /// Logging configuration
    pub logging: Option<LogConfig>,

    /// Metrics configuration
    pub metrics: Option<MetricsConfig>,

    /// Tracing configuration
    pub tracing: Option<TracingConfig>,

    /// User approval callback
    pub user_approval: Option<Arc<dyn UserApproval>>,

    /// Rate limiter
    pub rate_limiter: Option<Arc<RateLimiter>>,

    /// Audit logger
    pub audit_logger: Option<Arc<AuditLogger>>,

    /// Transaction validator
    pub transaction_validator: Option<TransactionValidator>,

    /// Enable HD wallet features
    pub enable_hd_wallet: bool,

    /// Enable multi-chain features
    pub enable_multi_chain: bool,

    /// Enable recovery features
    pub enable_recovery: bool,
}

impl Default for WalletConfig {
    fn default() -> Self {
        Self {
            ledger_endpoint: "http://localhost:50051".to_string(),
            key_store: KeyStoreConfig::InMemory,
            party_id: None,
            participant_id: None,
            application_id: Some("canton-wallet-sdk".to_string()),
            transport: None,
            tls: None,
            retry: Some(RetryConfig::default()),
            circuit_breaker: Some(CircuitBreakerConfig::default()),
            logging: Some(LogConfig::default()),
            metrics: Some(MetricsConfig::default()),
            tracing: Some(TracingConfig::default()),
            user_approval: None,
            rate_limiter: None,
            audit_logger: None,
            transaction_validator: Some(TransactionValidator::default()),
            enable_hd_wallet: false,
            enable_multi_chain: false,
            enable_recovery: true,
        }
    }
}

/// Main Canton wallet implementation
///
/// This is the primary entry point for interacting with the Canton Network.
/// It provides a high-level API for wallet operations, contract management,
/// and transaction submission.
///
/// # Examples
///
/// ```no_run
/// use canton_wallet::{CantonWallet, WalletConfig};
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let config = WalletConfig::default();
/// let wallet = CantonWallet::new(config).await?;
///
/// let balance = wallet.balance().await?;
/// println!("Balance: {}", balance.total_amount);
/// # Ok(())
/// # }
/// ```
pub struct CantonWallet {
    /// Wallet ID
    wallet_id: WalletId,

    /// Party ID
    party_id: PartyId,

    /// Participant ID
    participant_id: String,

    /// Ledger client
    ledger_client: Arc<LedgerClient>,

    /// Key store
    key_store: Arc<dyn KeyStore>,

    /// Signing key ID
    signing_key_id: KeyId,

    /// Contract manager
    contract_manager: Arc<ContractManager>,

    /// Event stream
    event_stream: Arc<EventStream>,

    /// Approval manager
    approval_manager: Option<Arc<ApprovalManager>>,

    /// Rate limiter
    rate_limiter: Option<Arc<RateLimiter>>,

    /// Audit logger
    audit_logger: Option<Arc<AuditLogger>>,

    /// Transaction validator
    transaction_validator: TransactionValidator,

    /// HD wallet (if enabled)
    hd_wallet: Option<Arc<HDWallet>>,

    /// Multi-chain wallet (if enabled)
    multi_chain_wallet: Option<Arc<MultiChainWallet>>,

    /// Recovery manager (if enabled)
    recovery_manager: Option<Arc<RecoveryManager>>,

    /// Application ID
    application_id: String,

    /// Circuit breaker
    circuit_breaker: Option<Arc<CircuitBreaker>>,

    /// Cache for balances
    balance_cache: Arc<RwLock<HashMap<String, WalletBalance>>>,

    /// Cache for contracts
    contract_cache: Arc<RwLock<HashMap<ContractId, ContractInfo>>>,
}

impl CantonWallet {
    /// Create a new wallet with the given configuration
    ///
    /// # Arguments
    ///
    /// * `config` - Wallet configuration
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing the created `CantonWallet` or a `WalletError`
    ///
    /// # Errors
    ///
    /// Returns `WalletError::ConnectionFailed` if unable to connect to the ledger
    /// Returns `WalletError::KeyGenerationFailed` if unable to generate keys
    /// Returns `WalletError::PartyNotFound` if the specified party doesn't exist
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use canton_wallet::{CantonWallet, WalletConfig};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let config = WalletConfig::default();
    /// let wallet = CantonWallet::new(config).await?;
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip(config))]
    pub async fn new(config: WalletConfig) -> Result<Self, WalletError> {
        info!("Creating new Canton wallet");

        // Initialize logging
        if let Some(logging_config) = &config.logging {
            init_logging(logging_config)?;
        }

        // Initialize metrics
        if let Some(metrics_config) = &config.metrics {
            init_metrics(metrics_config)?;
        }

        // Initialize tracing
        if let Some(tracing_config) = &config.tracing {
            init_tracing(tracing_config)?;
        }

        // Create ledger client
        let ledger_client = Arc::new(
            LedgerClient::connect(
                &config.ledger_endpoint,
                config.transport.clone(),
                config.tls.clone(),
                config.retry.clone(),
            )
            .await?,
        );

        // Create key store
        let key_store: Arc<dyn KeyStore> = match &config.key_store {
            KeyStoreConfig::InMemory => Arc::new(InMemoryKeyStore::new()),
            KeyStoreConfig::EncryptedFile { path, encryption_key } => {
                Arc::new(EncryptedKeyStore::new(path, encryption_key)?)
            }
        };

        // Generate signing key
        let signing_key_id = key_store
            .generate_key(
                KeyAlgorithm::Ed25519,
                KeyPurpose::Signing,
                KeyMetadata::default(),
            )
            .await?;

        // Get or generate party ID
        let party_id = if let Some(party_id) = config.party_id {
            party_id
        } else {
            // Generate party ID from public key
            let public_key = key_store.export_public_key(&signing_key_id).await?;
            PartyId::from_public_key(&public_key)?
        };

        // Derive participant ID
        let participant_id = config
            .participant_id
            .unwrap_or_else(|| party_id.to_string());

        // Create contract manager
        let contract_manager = Arc::new(ContractManager::new(
            ledger_client.clone(),
            party_id.clone(),
        ));

        // Create event stream
        let event_stream = Arc::new(EventStream::new(
            ledger_client.clone(),
            party_id.clone(),
            TransactionFilter::for_party(&party_id),
        ));

        // Create approval manager
        let approval_manager = if let Some(user_approval) = config.user_approval {
            let audit_logger = config.audit_logger.clone().unwrap_or_else(|| {
                Arc::new(AuditLogger::new_in_memory())
            });
            Some(Arc::new(ApprovalManager::new(
                user_approval,
                config.transaction_validator.clone().unwrap_or_default(),
                audit_logger,
            )))
        } else {
            None
        };

        // Create circuit breaker
        let circuit_breaker = config
            .circuit_breaker
            .map(|config| Arc::new(CircuitBreaker::new(config)));

        // Create HD wallet if enabled
        let hd_wallet = if config.enable_hd_wallet {
            Some(Arc::new(HDWallet::new(bip39::MnemonicType::Words12)?))
        } else {
            None
        };

        // Create multi-chain wallet if enabled
        let multi_chain_wallet = if config.enable_multi_chain {
            let bridge_manager = Arc::new(BridgeManager::new(ledger_client.clone()));
            Some(Arc::new(MultiChainWallet::new(
                Arc::new(Self::create_wallet_for_multichain(
                    ledger_client.clone(),
                    key_store.clone(),
                    party_id.clone(),
                    participant_id.clone(),
                )?),
                bridge_manager,
            )))
        } else {
            None
        };

        // Create recovery manager if enabled
        let recovery_manager = if config.enable_recovery {
            Some(Arc::new(RecoveryManager::new(
                key_store.clone(),
                config.audit_logger.clone().unwrap_or_else(|| {
                    Arc::new(AuditLogger::new_in_memory())
                }),
            )))
        } else {
            None
        };

        // Generate wallet ID
        let wallet_id = WalletId::new();

        // Get application ID
        let application_id = config
            .application_id
            .unwrap_or_else(|| "canton-wallet-sdk".to_string());

        info!("Created wallet with ID: {}", wallet_id);

        Ok(Self {
            wallet_id,
            party_id,
            participant_id,
            ledger_client,
            key_store,
            signing_key_id,
            contract_manager,
            event_stream,
            approval_manager,
            rate_limiter: config.rate_limiter,
            audit_logger: config.audit_logger,
            transaction_validator: config.transaction_validator.unwrap_or_default(),
            hd_wallet,
            multi_chain_wallet,
            recovery_manager,
            application_id,
            circuit_breaker,
            balance_cache: Arc::new(RwLock::new(HashMap::new())),
            contract_cache: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    /// Create a wallet for multi-chain use
    fn create_wallet_for_multichain(
        ledger_client: Arc<LedgerClient>,
        key_store: Arc<dyn KeyStore>,
        party_id: PartyId,
        participant_id: String,
    ) -> Result<Self, WalletError> {
        Ok(Self {
            wallet_id: WalletId::new(),
            party_id: party_id.clone(),
            participant_id,
            ledger_client,
            key_store,
            signing_key_id: KeyId::new(),
            contract_manager: Arc::new(ContractManager::new(
                Arc::new(LedgerClient::connect(
                    "http://localhost:50051",
                    None,
                    None,
                    None,
                )?),
                party_id,
            )),
            event_stream: Arc::new(EventStream::new(
                Arc::new(LedgerClient::connect(
                    "http://localhost:50051",
                    None,
                    None,
                    None,
                )?),
                party_id,
                TransactionFilter::default(),
            )),
            approval_manager: None,
            rate_limiter: None,
            audit_logger: None,
            transaction_validator: TransactionValidator::default(),
            hd_wallet: None,
            multi_chain_wallet: None,
            recovery_manager: None,
            application_id: "canton-wallet-sdk".to_string(),
            circuit_breaker: None,
            balance_cache: Arc::new(RwLock::new(HashMap::new())),
            contract_cache: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    /// Get the HD wallet if enabled
    pub fn hd_wallet(&self) -> Option<&Arc<HDWallet>> {
        self.hd_wallet.as_ref()
    }

    /// Get the multi-chain wallet if enabled
    pub fn multi_chain_wallet(&self) -> Option<&Arc<MultiChainWallet>> {
        self.multi_chain_wallet.as_ref()
    }

    /// Get the recovery manager if enabled
    pub fn recovery_manager(&self) -> Option<&Arc<RecoveryManager>> {
        self.recovery_manager.as_ref()
    }

    /// Get the ledger client
    pub fn ledger_client(&self) -> &Arc<LedgerClient> {
        &self.ledger_client
    }

    /// Get the key store
    pub fn key_store(&self) -> &Arc<dyn KeyStore> {
        &self.key_store
    }

    /// Get the contract manager
    pub fn contract_manager(&self) -> &Arc<ContractManager> {
        &self.contract_manager
    }

    /// Get the event stream
    pub fn event_stream(&self) -> &Arc<EventStream> {
        &self.event_stream
    }

    /// Get the approval manager
    pub fn approval_manager(&self) -> Option<&Arc<ApprovalManager>> {
        self.approval_manager.as_ref()
    }

    /// Get the audit logger
    pub fn audit_logger(&self) -> Option<&Arc<AuditLogger>> {
        self.audit_logger.as_ref()
    }

    /// Get the rate limiter
    pub fn rate_limiter(&self) -> Option<&Arc<RateLimiter>> {
        self.rate_limiter.as_ref()
    }

    /// Create a transaction builder
    pub fn transaction_builder(&self) -> TransactionBuilder {
        TransactionBuilder::new()
            .party_id(self.party_id.clone())
            .application_id(self.application_id.clone())
            .with_validator(self.transaction_validator.clone())
    }

    /// Clear the balance cache
    pub async fn clear_balance_cache(&self) {
        let mut cache = self.balance_cache.write().await;
        cache.clear();
        debug!("Cleared balance cache");
    }

    /// Clear the contract cache
    pub async fn clear_contract_cache(&self) {
        let mut cache = self.contract_cache.write().await;
        cache.clear();
        debug!("Cleared contract cache");
    }

    /// Clear all caches
    pub async fn clear_all_caches(&self) {
        self.clear_balance_cache().await;
        self.clear_contract_cache().await;
        debug!("Cleared all caches");
    }
}

#[async_trait]
impl Wallet for CantonWallet {
    fn wallet_id(&self) -> &WalletId {
        &self.wallet_id
    }

    fn party_id(&self) -> &PartyId {
        &self.party_id
    }

    fn participant_id(&self) -> &str {
        &self.participant_id
    }

    async fn address(&self) -> Result<String, WalletError> {
        let public_key = self.key_store.export_public_key(&self.signing_key_id).await?;
        Ok(Address::from_public_key(&public_key)?.to_string())
    }

    async fn balance(&self) -> Result<WalletBalance, WalletError> {
        // Check cache first
        let cache_key = self.party_id.to_string();
        {
            let cache = self.balance_cache.read().await;
            if let Some(balance) = cache.get(&cache_key) {
                debug!("Balance cache hit for party: {}", self.party_id);
                return Ok(balance.clone());
            }
        }

        // Fetch from ledger
        let contracts = self
            .contract_manager
            .active_contracts(None)
            .await?;

        let mut total_amount = rust_decimal::Decimal::ZERO;
        let mut assets: HashMap<String, rust_decimal::Decimal> = HashMap::new();

        for contract in contracts {
            // Extract balance information from contract
            if let Some(amount) = contract.extract_balance() {
                total_amount += amount;
                let asset_id = contract.template_id.to_string();
                *assets.entry(asset_id).or_insert_with(rust_decimal::Decimal::zero) += amount;
            }
        }

        let balance = WalletBalance {
            total_amount,
            assets,
            timestamp: Utc::now(),
        };

        // Update cache
        {
            let mut cache = self.balance_cache.write().await;
            cache.insert(cache_key, balance.clone());
        }

        Ok(balance)
    }

    async fn submit_command(&self, command: Command) -> Result<Transaction, WalletError> {
        // Check rate limiter
        if let Some(rate_limiter) = &self.rate_limiter {
            rate_limiter.check_limit(&self.party_id.to_string()).await?;
        }

        // Check circuit breaker
        if let Some(circuit_breaker) = &self.circuit_breaker {
            if !circuit_breaker.allow_request().await {
                return Err(WalletError::CircuitBreakerOpen);
            }
        }

        // Build commands
        let commands = self
            .transaction_builder()
            .add_command(command)
            .build()?;

        // Request approval if configured
        if let Some(approval_manager) = &self.approval_manager {
            let tx = Transaction::from_commands(&commands);
            approval_manager.request_approval(&tx).await?;
        }

        // Submit to ledger
        let result = self.ledger_client.submit_commands(commands).await;

        // Update circuit breaker state
        if let Some(circuit_breaker) = &self.circuit_breaker {
            if result.is_ok() {
                circuit_breaker.record_success().await;
            } else {
                circuit_breaker.record_failure().await;
            }
        }

        // Log audit entry
        if let Some(audit_logger) = &self.audit_logger {
            audit_logger
                .log(AuditLogEntry {
                    timestamp: Utc::now(),
                    operation: "submit_command".to_string(),
                    details: serde_json::json!({
                        "party_id": self.party_id.to_string(),
                        "success": result.is_ok(),
                    }),
                })
                .await?;
        }

        result
    }

    async fn submit_and_wait(&self, command: Command) -> Result<Transaction, WalletError> {
        // Check rate limiter
        if let Some(rate_limiter) = &self.rate_limiter {
            rate_limiter.check_limit(&self.party_id.to_string()).await?;
        }

        // Check circuit breaker
        if let Some(circuit_breaker) = &self.circuit_breaker {
            if !circuit_breaker.allow_request().await {
                return Err(WalletError::CircuitBreakerOpen);
            }
        }

        // Build commands
        let commands = self
            .transaction_builder()
            .add_command(command)
            .build()?;

        // Request approval if configured
        if let Some(approval_manager) = &self.approval_manager {
            let tx = Transaction::from_commands(&commands);
            approval_manager.request_approval(&tx).await?;
        }

        // Submit and wait
        let result = self
            .ledger_client
            .submit_and_wait_for_party(&self.party_id, commands)
            .await;

        // Update circuit breaker state
        if let Some(circuit_breaker) = &self.circuit_breaker {
            if result.is_ok() {
                circuit_breaker.record_success().await;
            } else {
                circuit_breaker.record_failure().await;
            }
        }

        // Log audit entry
        if let Some(audit_logger) = &self.audit_logger {
            audit_logger
                .log(AuditLogEntry {
                    timestamp: Utc::now(),
                    operation: "submit_and_wait".to_string(),
                    details: serde_json::json!({
                        "party_id": self.party_id.to_string(),
                        "success": result.is_ok(),
                    }),
                })
                .await?;
        }

        result
    }

    async fn active_contracts(
        &self,
        filter: Option<TransactionFilter>,
    ) -> Result<Vec<CreatedEvent>, WalletError> {
        self.contract_manager.active_contracts(filter).await
    }

    async fn exercise_choice(
        &self,
        contract_id: ContractId,
        choice: &str,
        argument: DamlValue,
    ) -> Result<Transaction, WalletError> {
        self.contract_manager
            .exercise(contract_id, choice, argument)
            .await
    }

    async fn create_contract(
        &self,
        template_id: Identifier,
        arguments: DamlRecord,
    ) -> Result<CreatedEvent, WalletError> {
        self.contract_manager.create(template_id, arguments).await
    }

    fn transactions(
        &self,
        begin: LedgerOffset,
        end: Option<LedgerOffset>,
        filter: TransactionFilter,
    ) -> impl Stream<Item = Result<Transaction, WalletError>> + Send {
        self.event_stream
            .with_offset(begin)
            .subscribe()
    }

    async fn sign(&self, data: &[u8]) -> Result<Signature, WalletError> {
        self.key_store.sign(&self.signing_key_id, data).await
    }

    async fn verify(&self, data: &[u8], signature: &Signature) -> Result<bool, WalletError> {
        self.key_store
            .verify(&self.signing_key_id, data, signature)
            .await
    }
}

impl std::fmt::Debug for CantonWallet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CantonWallet")
            .field("wallet_id", &self.wallet_id)
            .field("party_id", &self.party_id)
            .field("participant_id", &self.participant_id)
            .field("application_id", &self.application_id)
            .field("enable_hd_wallet", &self.hd_wallet.is_some())
            .field("enable_multi_chain", &self.multi_chain_wallet.is_some())
            .field("enable_recovery", &self.recovery_manager.is_some())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wallet_config_default() {
        let config = WalletConfig::default();
        assert_eq!(config.ledger_endpoint, "http://localhost:50051");
        assert!(config.party_id.is_none());
        assert!(config.enable_hd_wallet == false);
        assert!(config.enable_multi_chain == false);
        assert!(config.enable_recovery == true);
    }

    #[test]
    fn test_wallet_id_generation() {
        let id1 = WalletId::new();
        let id2 = WalletId::new();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_party_id_from_public_key() {
        let key_pair = KeyPair::generate(KeyAlgorithm::Ed25519).unwrap();
        let party_id = PartyId::from_public_key(&key_pair.public_key).unwrap();
        assert!(!party_id.to_string().is_empty());
    }
}
