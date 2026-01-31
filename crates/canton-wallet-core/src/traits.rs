// Core traits for Canton Wallet SDK

use crate::types::*;
use crate::{WalletError, WalletResult};
use async_trait::async_trait;
use futures::Stream;

/// Core wallet trait - implement this for different wallet types
#[async_trait]
pub trait Wallet: Send + Sync {
    /// Get wallet ID
    fn wallet_id(&self) -> &WalletId;

    /// Get party ID
    fn party_id(&self) -> &PartyId;

    /// Get participant ID
    fn participant_id(&self) -> &ParticipantId;

    /// Get wallet address
    async fn address(&self) -> WalletResult<String>;

    /// Get balance
    async fn balance(&self) -> WalletResult<WalletBalance>;

    /// Submit command
    async fn submit_command(&self, command: Command) -> WalletResult<Transaction>;

    /// Submit and wait for transaction
    async fn submit_and_wait(&self, command: Command) -> WalletResult<Transaction>;

    /// Get active contracts
    async fn active_contracts(
        &self,
        filter: Option<TransactionFilter>,
    ) -> WalletResult<Vec<CreatedEvent>>;

    /// Exercise choice on contract
    async fn exercise_choice(
        &self,
        contract_id: ContractId,
        choice: &str,
        argument: DamlValue,
    ) -> WalletResult<Transaction>;

    /// Create contract
    async fn create_contract(
        &self,
        template_id: TemplateId,
        arguments: DamlRecord,
    ) -> WalletResult<CreatedEvent>;

    /// Get transaction history
    fn transactions(
        &self,
        begin: LedgerOffset,
        end: Option<LedgerOffset>,
        filter: TransactionFilter,
    ) -> impl Stream<Item = WalletResult<Transaction>> + Send;

    /// Sign data
    async fn sign(&self, data: &[u8]) -> WalletResult<Signature>;

    /// Verify signature
    async fn verify(&self, data: &[u8], signature: &Signature) -> WalletResult<bool>;
}

/// Key store trait - implement for different storage backends
#[async_trait]
pub trait KeyStore: Send + Sync {
    /// Generate new key
    async fn generate_key(
        &self,
        algorithm: KeyAlgorithm,
        purpose: KeyPurpose,
        metadata: KeyMetadata,
    ) -> WalletResult<KeyId>;

    /// Import existing key
    async fn import_key(
        &self,
        key_bytes: &[u8],
        algorithm: KeyAlgorithm,
        purpose: KeyPurpose,
        metadata: KeyMetadata,
    ) -> WalletResult<KeyId>;

    /// Export public key
    async fn export_public_key(&self, key_id: &KeyId) -> WalletResult<PublicKey>;

    /// Sign data
    async fn sign(&self, key_id: &KeyId, data: &[u8]) -> WalletResult<Signature>;

    /// Verify signature
    async fn verify(
        &self,
        key_id: &KeyId,
        data: &[u8],
        signature: &Signature,
    ) -> WalletResult<bool>;

    /// Delete key
    async fn delete_key(&self, key_id: &KeyId) -> WalletResult<()>;

    /// List all keys
    async fn list_keys(&self) -> WalletResult<Vec<KeyInfo>>;

    /// Get key info
    async fn get_key_info(&self, key_id: &KeyId) -> WalletResult<KeyInfo>;

    /// Rotate key
    async fn rotate_key(
        &self,
        old_key_id: &KeyId,
        new_algorithm: KeyAlgorithm,
    ) -> WalletResult<KeyId>;
}

/// Ledger client trait - implement for different ledger backends
#[async_trait]
pub trait LedgerClient: Send + Sync {
    /// Submit command to ledger
    async fn submit_command(&self, commands: Commands) -> WalletResult<TransactionId>;

    /// Submit and wait for transaction
    async fn submit_and_wait(&self, commands: Commands) -> WalletResult<Transaction>;

    /// Get active contracts
    async fn get_active_contracts(
        &self,
        filter: TransactionFilter,
    ) -> WalletResult<Vec<CreatedEvent>>;

    /// Get transactions
    async fn get_transactions(
        &self,
        begin: LedgerOffset,
        end: Option<LedgerOffset>,
        filter: TransactionFilter,
    ) -> WalletResult<Vec<Transaction>>;

    /// Subscribe to transactions
    fn subscribe_transactions(
        &self,
        begin: LedgerOffset,
        filter: TransactionFilter,
    ) -> impl Stream<Item = WalletResult<Transaction>> + Send;

    /// Get current ledger offset
    async fn get_ledger_end(&self) -> WalletResult<LedgerOffset>;

    /// Get party allocation
    async fn get_party_allocation(&self, party_id: &PartyId) -> WalletResult<ParticipantId>;
}

/// Transaction validator trait
pub trait TransactionValidator: Send + Sync {
    /// Validate transaction
    fn validate(&self, commands: &Commands) -> WalletResult<()>;

    /// Validate command
    fn validate_command(&self, command: &Command) -> WalletResult<()>;

    /// Validate party ID
    fn validate_party_id(&self, party_id: &PartyId) -> WalletResult<()>;

    /// Validate contract ID
    fn validate_contract_id(&self, contract_id: &ContractId) -> WalletResult<()>;
}

/// User approval trait
#[async_trait]
pub trait UserApproval: Send + Sync {
    /// Request approval for transaction
    async fn request_approval(&self, tx: &Transaction) -> WalletResult<ApprovalResponse>;

    /// Request approval for key operation
    async fn request_key_approval(&self, operation: &str) -> WalletResult<bool>;
}

/// Audit logger trait
#[async_trait]
pub trait AuditLogger: Send + Sync {
    /// Log an audit entry
    async fn log(&self, entry: AuditLogEntry) -> WalletResult<()>;

    /// Get audit logs
    async fn get_logs(
        &self,
        start: DateTime<Utc>,
        end: Option<DateTime<Utc>>,
    ) -> WalletResult<Vec<AuditLogEntry>>;
}

/// Audit log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogEntry {
    pub timestamp: DateTime<Utc>,
    pub operation: String,
    pub details: serde_json::Value,
}

impl AuditLogEntry {
    /// Create a new audit log entry
    pub fn new(operation: impl Into<String>, details: serde_json::Value) -> Self {
        Self {
            timestamp: Utc::now(),
            operation: operation.into(),
            details,
        }
    }
}

/// Contract manager trait
#[async_trait]
pub trait ContractManager: Send + Sync {
    /// Create contract
    async fn create(
        &self,
        template_id: TemplateId,
        arguments: DamlRecord,
    ) -> WalletResult<CreatedEvent>;

    /// Exercise choice on contract
    async fn exercise(
        &self,
        contract_id: ContractId,
        choice: &str,
        argument: DamlValue,
    ) -> WalletResult<Transaction>;

    /// Exercise choice by key
    async fn exercise_by_key(
        &self,
        template_id: TemplateId,
        contract_key: DamlValue,
        choice: &str,
        argument: DamlValue,
    ) -> WalletResult<Transaction>;

    /// Create and exercise
    async fn create_and_exercise(
        &self,
        template_id: TemplateId,
        create_arguments: DamlRecord,
        choice: &str,
        choice_argument: DamlValue,
    ) -> WalletResult<Transaction>;

    /// Get active contracts
    async fn active_contracts(
        &self,
        filter: Option<TransactionFilter>,
    ) -> WalletResult<Vec<CreatedEvent>>;

    /// Get contract by ID
    async fn get_contract(&self, contract_id: ContractId) -> WalletResult<ContractInfo>;

    /// Query contracts by template
    async fn query_by_template(
        &self,
        template_id: TemplateId,
    ) -> WalletResult<Vec<CreatedEvent>>;

    /// Archive contract
    async fn archive(&self, contract_id: ContractId) -> WalletResult<Transaction>;
}

/// Event stream trait
pub trait EventStream: Send + Sync {
    /// Subscribe to events
    fn subscribe(&self) -> impl Stream<Item = WalletResult<Transaction>> + Send;

    /// Subscribe with offset
    fn subscribe_with_offset(
        &self,
        offset: LedgerOffset,
    ) -> impl Stream<Item = WalletResult<Transaction>> + Send;

    /// Get current offset
    fn current_offset(&self) -> LedgerOffset;
}

/// Recovery manager trait
#[async_trait]
pub trait RecoveryManager: Send + Sync {
    /// Create backup
    async fn create_backup(&self, wallet_id: &WalletId) -> WalletResult<Vec<u8>>;

    /// Restore from backup
    async fn restore_backup(&self, backup: &[u8]) -> WalletResult<WalletId>;

    /// Initiate social recovery
    async fn initiate_social_recovery(
        &self,
        wallet_id: &WalletId,
        guardians: Vec<PartyId>,
    ) -> WalletResult<String>;

    /// Complete social recovery
    async fn complete_social_recovery(
        &self,
        recovery_id: &str,
        signatures: Vec<Signature>,
    ) -> WalletResult<WalletId>;

    /// Verify recovery
    async fn verify_recovery(&self, recovery_id: &str) -> WalletResult<bool>;
}

/// Chain wallet trait for multi-chain support
#[async_trait]
pub trait ChainWallet: Send + Sync {
    /// Get chain ID
    fn chain_id(&self) -> String;

    /// Get address
    async fn address(&self) -> WalletResult<String>;

    /// Get balance
    async fn balance(&self) -> WalletResult<String>;

    /// Transfer to address
    async fn transfer(
        &self,
        to: String,
        amount: String,
    ) -> WalletResult<String>;

    /// Sign transaction
    async fn sign_transaction(&self, tx: &[u8]) -> WalletResult<Signature>;
}

/// Bridge manager trait for cross-chain transfers
#[async_trait]
pub trait BridgeManager: Send + Sync {
    /// Lock asset on Canton
    async fn lock_on_canton(
        &self,
        wallet: &dyn Wallet,
        asset: CantonAsset,
        target_chain: String,
        recipient: String,
    ) -> WalletResult<LockReceipt>;

    /// Lock asset on chain
    async fn lock_on_chain(
        &self,
        wallet: &dyn ChainWallet,
        asset: ChainAsset,
        target_chain: String,
        recipient: String,
    ) -> WalletResult<LockReceipt>;

    /// Generate lock proof
    async fn generate_lock_proof(&self, receipt: &LockReceipt) -> WalletResult<Proof>;

    /// Release on chain
    async fn release_on_chain(
        &self,
        wallet: &dyn ChainWallet,
        proof: Proof,
        recipient: String,
    ) -> WalletResult<ReleaseReceipt>;

    /// Release on Canton
    async fn release_on_canton(
        &self,
        wallet: &dyn Wallet,
        proof: Proof,
        recipient: PartyId,
    ) -> WalletResult<ReleaseReceipt>;
}

/// Canton asset
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CantonAsset {
    pub contract_id: ContractId,
    pub amount: String,
    pub template_id: TemplateId,
}

/// Chain asset
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainAsset {
    pub token_address: String,
    pub amount: String,
    pub chain_id: String,
}

/// Lock receipt
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockReceipt {
    pub tx_id: String,
    pub asset: String,
    pub amount: String,
    pub timestamp: DateTime<Utc>,
}

/// Proof
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Proof {
    pub lock_receipt: LockReceipt,
    pub signature: Signature,
    pub merkle_proof: Vec<String>,
}

/// Release receipt
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReleaseReceipt {
    pub tx_id: String,
    pub asset: String,
    pub amount: String,
    pub timestamp: DateTime<Utc>,
}

/// Cross-chain transaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossChainTx {
    pub canton_tx_id: String,
    pub target_tx_id: String,
    pub asset: CantonAsset,
    pub source_chain: String,
    pub target_chain: String,
    pub timestamp: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audit_log_entry() {
        let entry = AuditLogEntry::new("test_operation", serde_json::json!({"key": "value"}));
        assert_eq!(entry.operation, "test_operation");
    }

    #[test]
    fn test_canton_asset() {
        let asset = CantonAsset {
            contract_id: ContractId::new_unchecked("test-contract"),
            amount: "100".to_string(),
            template_id: TemplateId::new("pkg", "mod", "tpl"),
        };
        assert_eq!(asset.amount, "100");
    }

    #[test]
    fn test_chain_asset() {
        let asset = ChainAsset {
            token_address: "0x123".to_string(),
            amount: "100".to_string(),
            chain_id: "ethereum".to_string(),
        };
        assert_eq!(asset.token_address, "0x123");
    }

    #[test]
    fn test_lock_receipt() {
        let receipt = LockReceipt {
            tx_id: "tx-123".to_string(),
            asset: "token".to_string(),
            amount: "100".to_string(),
            timestamp: Utc::now(),
        };
        assert_eq!(receipt.tx_id, "tx-123");
    }

    #[test]
    fn test_cross_chain_tx() {
        let tx = CrossChainTx {
            canton_tx_id: "canton-tx".to_string(),
            target_tx_id: "target-tx".to_string(),
            asset: CantonAsset {
                contract_id: ContractId::new_unchecked("test-contract"),
                amount: "100".to_string(),
                template_id: TemplateId::new("pkg", "mod", "tpl"),
            },
            source_chain: "canton".to_string(),
            target_chain: "ethereum".to_string(),
            timestamp: Utc::now(),
        };
        assert_eq!(tx.source_chain, "canton");
        assert_eq!(tx.target_chain, "ethereum");
    }
}
