# Advanced Wallet Patterns

## 1. Overview

This document covers advanced wallet patterns including key management, transaction signing, multi-chain support, and production-ready implementations.

## 2. Advanced Key Management

### 2.1 Hierarchical Deterministic (HD) Wallets

```rust
use bip39::{Mnemonic, MnemonicType, Language, Seed};
use bip32::{Mnemonic as Bip32Mnemonic, XPrv, XPub, DerivationPath, ChildNumber};

/// HD wallet implementation
pub struct HDWallet {
    mnemonic: Mnemonic,
    root_key: XPrv,
    accounts: HashMap<u32, HDAccount>,
}

impl HDWallet {
    /// Create new HD wallet
    pub fn new(word_count: MnemonicType) -> Result<Self, WalletError> {
        let mnemonic = Mnemonic::new(word_count, Language::English);
        let seed = Seed::new(&mnemonic, "");
        
        let root_key = XPrv::new(seed.as_bytes())
            .map_err(|e| WalletError::DerivationError(e.to_string()))?;
        
        Ok(Self {
            mnemonic,
            root_key,
            accounts: HashMap::new(),
        })
    }
    
    /// Restore from mnemonic
    pub fn from_mnemonic(mnemonic: &str) -> Result<Self, WalletError> {
        let mnemonic = Mnemonic::from_phrase(mnemonic, Language::English)
            .map_err(|e| WalletError::InvalidMnemonic(e.to_string()))?;
        
        let seed = Seed::new(&mnemonic, "");
        
        let root_key = XPrv::new(seed.as_bytes())
            .map_err(|e| WalletError::DerivationError(e.to_string()))?;
        
        Ok(Self {
            mnemonic,
            root_key,
            accounts: HashMap::new(),
        })
    }
    
    /// Derive account at index
    pub fn derive_account(&mut self, index: u32) -> Result<&HDAccount, WalletError> {
        if self.accounts.contains_key(&index) {
            return Ok(&self.accounts[&index]);
        }
        
        // BIP44 path: m/44'/coin_type'/account'/change/address_index
        let path = DerivationPath::from_str(&format!(
            "m/44'/118'/{}'/0/0",
            index
        ))
        .map_err(|e| WalletError::InvalidDerivationPath(e.to_string()))?;
        
        let account_key = self.root_key
            .derive_path(&path)
            .map_err(|e| WalletError::DerivationError(e.to_string()))?;
        
        let account = HDAccount {
            index,
            private_key: account_key,
            public_key: account_key.public_key(),
        };
        
        self.accounts.insert(index, account);
        Ok(&self.accounts[&index])
    }
    
    /// Get mnemonic phrase
    pub fn mnemonic_phrase(&self) -> &str {
        self.mnemonic.phrase()
    }
    
    /// Export seed (WARNING: Use with caution)
    pub fn export_seed(&self) -> Result<Vec<u8>, WalletError> {
        Ok(self.root_key.private_key().as_bytes().to_vec())
    }
}

/// HD account
pub struct HDAccount {
    index: u32,
    private_key: XPrv,
    public_key: XPub,
}

impl HDAccount {
    pub fn index(&self) -> u32 {
        self.index
    }
    
    pub fn private_key(&self) -> &XPrv {
        &self.private_key
    }
    
    pub fn public_key(&self) -> &XPub {
        &self.public_key
    }
    
    pub fn address(&self) -> String {
        // Convert public key to address (implementation depends on chain)
        self.public_key.to_string()
    }
}
```

### 2.2 Multi-Signature Wallets

```rust
/// Multi-signature wallet
pub struct MultiSigWallet {
    signers: Vec<Box<dyn Signer>>,
    threshold: usize,
    pending_signatures: HashMap<TxId, Vec<Signature>>,
}

impl MultiSigWallet {
    pub fn new(signers: Vec<Box<dyn Signer>>, threshold: usize) -> Self {
        Self {
            signers,
            threshold,
            pending_signatures: HashMap::new(),
        }
    }
    
    /// Submit transaction for signing
    pub async fn submit_transaction(
        &mut self,
        tx: Transaction,
    ) -> Result<TxId, WalletError> {
        let tx_id = tx.id();
        
        // Validate transaction
        self.validate_transaction(&tx)?;
        
        // Initialize pending signatures
        self.pending_signatures.insert(tx_id.clone(), Vec::new());
        
        Ok(tx_id)
    }
    
    /// Add signature to transaction
    pub async fn add_signature(
        &mut self,
        tx_id: &TxId,
        signer_index: usize,
        signature: Signature,
    ) -> Result<(), WalletError> {
        let signatures = self.pending_signatures
            .get_mut(tx_id)
            .ok_or(WalletError::TransactionNotFound(tx_id.clone()))?;
        
        // Verify signature
        let signer = self.signers.get(signer_index)
            .ok_or(WalletError::InvalidSignerIndex(signer_index))?;
        
        if !signer.verify_signature(&signature)? {
            return Err(WalletError::InvalidSignature);
        }
        
        signatures.push(signature);
        
        Ok(())
    }
    
    /// Check if transaction has enough signatures
    pub fn is_ready(&self, tx_id: &TxId) -> bool {
        self.pending_signatures
            .get(tx_id)
            .map(|sigs| sigs.len() >= self.threshold)
            .unwrap_or(false)
    }
    
    /// Execute transaction when ready
    pub async fn execute_transaction(
        &mut self,
        tx_id: &TxId,
    ) -> Result<ExecutedTransaction, WalletError> {
        let signatures = self.pending_signatures
            .get(tx_id)
            .ok_or(WalletError::TransactionNotFound(tx_id.clone()))?;
        
        if signatures.len() < self.threshold {
            return Err(WalletError::InsufficientSignatures {
                have: signatures.len(),
                need: self.threshold,
            });
        }
        
        // Combine signatures
        let combined_signature = self.combine_signatures(signatures)?;
        
        // Execute transaction
        let executed = self.execute_with_signature(tx_id, &combined_signature).await?;
        
        // Clean up
        self.pending_signatures.remove(tx_id);
        
        Ok(executed)
    }
    
    fn validate_transaction(&self, tx: &Transaction) -> Result<(), WalletError> {
        // Validate transaction structure
        if tx.inputs.is_empty() {
            return Err(WalletError::InvalidTransaction("No inputs".to_string()));
        }
        
        if tx.outputs.is_empty() {
            return Err(WalletError::InvalidTransaction("No outputs".to_string()));
        }
        
        // Validate amounts
        let total_in: u64 = tx.inputs.iter().map(|i| i.amount).sum();
        let total_out: u64 = tx.outputs.iter().map(|o| o.amount).sum();
        
        if total_out > total_in {
            return Err(WalletError::InsufficientFunds {
                required: total_out,
                available: total_in,
            });
        }
        
        Ok(())
    }
    
    fn combine_signatures(&self, signatures: &[Signature]) -> Result<Signature, WalletError> {
        // Combine multiple signatures into one
        // Implementation depends on signature scheme
        Ok(signatures[0].clone())
    }
    
    async fn execute_with_signature(
        &self,
        tx_id: &TxId,
        signature: &Signature,
    ) -> Result<ExecutedTransaction, WalletError> {
        // Execute transaction with combined signature
        Ok(ExecutedTransaction {
            tx_id: tx_id.clone(),
            signature: signature.clone(),
            timestamp: Utc::now(),
        })
    }
}

/// Signer trait
#[async_trait]
pub trait Signer: Send + Sync {
    fn public_key(&self) -> &PublicKey;
    async fn sign(&self, data: &[u8]) -> Result<Signature, WalletError>;
    fn verify_signature(&self, signature: &Signature) -> Result<bool, WalletError>;
}
```

### 2.3 Key Rotation

```rust
/// Key rotation manager
pub struct KeyRotationManager {
    current_key: KeyId,
    previous_keys: VecDeque<KeyId>,
    key_store: Arc<dyn KeyStore>,
    rotation_interval: Duration,
    last_rotation: DateTime<Utc>,
}

impl KeyRotationManager {
    pub fn new(
        initial_key: KeyId,
        key_store: Arc<dyn KeyStore>,
        rotation_interval: Duration,
    ) -> Self {
        Self {
            current_key: initial_key,
            previous_keys: VecDeque::new(),
            key_store,
            rotation_interval,
            last_rotation: Utc::now(),
        }
    }
    
    /// Check if rotation is needed
    pub fn needs_rotation(&self) -> bool {
        Utc::now() - self.last_rotation > self.rotation_interval
    }
    
    /// Rotate to new key
    pub async fn rotate_key(&mut self) -> Result<KeyId, WalletError> {
        // Generate new key
        let new_key = self.key_store
            .generate_key(KeyAlgorithm::Ed25519, KeyPurpose::Signing, KeyMetadata::default())
            .await?;
        
        // Move current key to previous
        self.previous_keys.push_back(self.current_key.clone());
        
        // Keep only last N previous keys
        if self.previous_keys.len() > 10 {
            self.previous_keys.pop_front();
        }
        
        // Set new key as current
        self.current_key = new_key.clone();
        self.last_rotation = Utc::now();
        
        Ok(new_key)
    }
    
    /// Get current key
    pub fn current_key(&self) -> &KeyId {
        &self.current_key
    }
    
    /// Get previous keys
    pub fn previous_keys(&self) -> &[KeyId] {
        &self.previous_keys
    }
    
    /// Verify signature with any key (current or previous)
    pub async fn verify_with_any_key(
        &self,
        data: &[u8],
        signature: &Signature,
    ) -> Result<bool, WalletError> {
        // Try current key
        if self.key_store.verify(&self.current_key, data, signature).await? {
            return Ok(true);
        }
        
        // Try previous keys
        for key_id in &self.previous_keys {
            if self.key_store.verify(key_id, data, signature).await? {
                return Ok(true);
            }
        }
        
        Ok(false)
    }
}
```

## 3. Advanced Transaction Patterns

### 3.1 Transaction Builder with Validation

```rust
/// Transaction builder with comprehensive validation
pub struct TransactionBuilder {
    from: Option<Address>,
    to: Option<Address>,
    value: Option<u64>,
    fee: Option<u64>,
    gas_limit: Option<u64>,
    gas_price: Option<u64>,
    nonce: Option<u64>,
    data: Option<Vec<u8>>,
    chain_id: Option<u64>,
    validator: TransactionValidator,
}

impl TransactionBuilder {
    pub fn new() -> Self {
        Self {
            from: None,
            to: None,
            value: None,
            fee: None,
            gas_limit: None,
            gas_price: None,
            nonce: None,
            data: None,
            chain_id: None,
            validator: TransactionValidator::default(),
        }
    }
    
    pub fn from(mut self, address: Address) -> Self {
        self.from = Some(address);
        self
    }
    
    pub fn to(mut self, address: Address) -> Self {
        self.to = Some(address);
        self
    }
    
    pub fn value(mut self, value: u64) -> Self {
        self.value = Some(value);
        self
    }
    
    pub fn fee(mut self, fee: u64) -> Self {
        self.fee = Some(fee);
        self
    }
    
    pub fn gas_limit(mut self, limit: u64) -> Self {
        self.gas_limit = Some(limit);
        self
    }
    
    pub fn gas_price(mut self, price: u64) -> Self {
        self.gas_price = Some(price);
        self
    }
    
    pub fn nonce(mut self, nonce: u64) -> Self {
        self.nonce = Some(nonce);
        self
    }
    
    pub fn data(mut self, data: Vec<u8>) -> Self {
        self.data = Some(data);
        self
    }
    
    pub fn chain_id(mut self, id: u64) -> Self {
        self.chain_id = Some(id);
        self
    }
    
    pub fn with_validator(mut self, validator: TransactionValidator) -> Self {
        self.validator = validator;
        self
    }
    
    /// Build and validate transaction
    pub fn build(self) -> Result<Transaction, WalletError> {
        let tx = Transaction {
            from: self.from.ok_or(WalletError::MissingField("from"))?,
            to: self.to.ok_or(WalletError::MissingField("to"))?,
            value: self.value.unwrap_or(0),
            fee: self.fee.ok_or(WalletError::MissingField("fee"))?,
            gas_limit: self.gas_limit.ok_or(WalletError::MissingField("gas_limit"))?,
            gas_price: self.gas_price.ok_or(WalletError::MissingField("gas_price"))?,
            nonce: self.nonce.ok_or(WalletError::MissingField("nonce"))?,
            data: self.data.unwrap_or_default(),
            chain_id: self.chain_id.ok_or(WalletError::MissingField("chain_id"))?,
            signature: None,
        };
        
        // Validate transaction
        self.validator.validate(&tx)?;
        
        Ok(tx)
    }
}

/// Transaction validator
pub struct TransactionValidator {
    max_fee: u64,
    max_gas_limit: u64,
    min_gas_price: u64,
    max_data_size: usize,
}

impl Default for TransactionValidator {
    fn default() -> Self {
        Self {
            max_fee: 1_000_000_000,
            max_gas_limit: 10_000_000,
            min_gas_price: 1_000_000_000,
            max_data_size: 1_000_000,
        }
    }
}

impl TransactionValidator {
    pub fn validate(&self, tx: &Transaction) -> Result<(), WalletError> {
        // Validate fee
        if tx.fee > self.max_fee {
            return Err(WalletError::FeeTooHigh {
                actual: tx.fee,
                max: self.max_fee,
            });
        }
        
        // Validate gas limit
        if tx.gas_limit > self.max_gas_limit {
            return Err(WalletError::GasLimitTooHigh {
                actual: tx.gas_limit,
                max: self.max_gas_limit,
            });
        }
        
        // Validate gas price
        if tx.gas_price < self.min_gas_price {
            return Err(WalletError::GasPriceTooLow {
                actual: tx.gas_price,
                min: self.min_gas_price,
            });
        }
        
        // Validate data size
        if tx.data.len() > self.max_data_size {
            return Err(WalletError::DataTooLarge {
                actual: tx.data.len(),
                max: self.max_data_size,
            });
        }
        
        // Validate addresses
        if tx.from == tx.to {
            return Err(WalletError::InvalidTransaction("From and to addresses are the same".to_string()));
        }
        
        Ok(())
    }
}
```

### 3.2 Batch Transactions

```rust
/// Batch transaction manager
pub struct BatchTransactionManager {
    transactions: Vec<Transaction>,
    max_batch_size: usize,
    max_total_fee: u64,
}

impl BatchTransactionManager {
    pub fn new(max_batch_size: usize, max_total_fee: u64) -> Self {
        Self {
            transactions: Vec::new(),
            max_batch_size,
            max_total_fee,
        }
    }
    
    /// Add transaction to batch
    pub fn add_transaction(&mut self, tx: Transaction) -> Result<(), WalletError> {
        // Check batch size
        if self.transactions.len() >= self.max_batch_size {
            return Err(WalletError::BatchFull);
        }
        
        // Check total fee
        let current_total_fee: u64 = self.transactions.iter().map(|t| t.fee).sum();
        if current_total_fee + tx.fee > self.max_total_fee {
            return Err(WalletError::TotalFeeTooHigh {
                actual: current_total_fee + tx.fee,
                max: self.max_total_fee,
            });
        }
        
        self.transactions.push(tx);
        Ok(())
    }
    
    /// Execute all transactions in batch
    pub async fn execute_batch<W: Wallet>(
        &mut self,
        wallet: &W,
    ) -> Result<Vec<TransactionResult>, WalletError> {
        let mut results = Vec::new();
        
        for tx in self.transactions.drain(..) {
            let result = wallet.sign_and_send_transaction(&tx).await?;
            results.push(result);
        }
        
        Ok(results)
    }
    
    /// Clear batch
    pub fn clear(&mut self) {
        self.transactions.clear();
    }
    
    /// Get batch size
    pub fn size(&self) -> usize {
        self.transactions.len()
    }
    
    /// Get total fee
    pub fn total_fee(&self) -> u64 {
        self.transactions.iter().map(|t| t.fee).sum()
    }
}
```

### 3.3 Transaction Estimation

```rust
/// Transaction estimator
pub struct TransactionEstimator {
    gas_price_oracle: Arc<dyn GasPriceOracle>,
    gas_estimator: Arc<dyn GasEstimator>,
}

impl TransactionEstimator {
    pub fn new(
        gas_price_oracle: Arc<dyn GasPriceOracle>,
        gas_estimator: Arc<dyn GasEstimator>,
    ) -> Self {
        Self {
            gas_price_oracle,
            gas_estimator,
        }
    }
    
    /// Estimate gas for transaction
    pub async fn estimate_gas(
        &self,
        tx: &Transaction,
    ) -> Result<u64, WalletError> {
        self.gas_estimator.estimate_gas(tx).await
    }
    
    /// Estimate gas price
    pub async fn estimate_gas_price(&self) -> Result<u64, WalletError> {
        self.gas_price_oracle.get_gas_price().await
    }
    
    /// Estimate total fee
    pub async fn estimate_fee(
        &self,
        tx: &Transaction,
    ) -> Result<u64, WalletError> {
        let gas_limit = self.estimate_gas(tx).await?;
        let gas_price = self.estimate_gas_price().await?;
        
        Ok(gas_limit * gas_price)
    }
    
    /// Populate transaction with estimates
    pub async fn populate_transaction(
        &self,
        mut tx: Transaction,
    ) -> Result<Transaction, WalletError> {
        if tx.gas_limit == 0 {
            tx.gas_limit = self.estimate_gas(&tx).await?;
        }
        
        if tx.gas_price == 0 {
            tx.gas_price = self.estimate_gas_price().await?;
        }
        
        tx.fee = tx.gas_limit * tx.gas_price;
        
        Ok(tx)
    }
}

/// Gas price oracle trait
#[async_trait]
pub trait GasPriceOracle: Send + Sync {
    async fn get_gas_price(&self) -> Result<u64, WalletError>;
}

/// Gas estimator trait
#[async_trait]
pub trait GasEstimator: Send + Sync {
    async fn estimate_gas(&self, tx: &Transaction) -> Result<u64, WalletError>;
}
```

## 4. Multi-Chain Wallet Patterns

### 4.1 Unified Multi-Chain Wallet

```rust
/// Unified multi-chain wallet
pub struct UnifiedWallet {
    wallet_id: WalletId,
    chains: HashMap<ChainId, Box<dyn ChainWallet>>,
    shared_key_store: Arc<dyn KeyStore>,
}

impl UnifiedWallet {
    pub fn new(wallet_id: WalletId, shared_key_store: Arc<dyn KeyStore>) -> Self {
        Self {
            wallet_id,
            chains: HashMap::new(),
            shared_key_store,
        }
    }
    
    /// Add chain support
    pub fn add_chain(&mut self, chain_id: ChainId, wallet: Box<dyn ChainWallet>) {
        self.chains.insert(chain_id, wallet);
    }
    
    /// Get address for chain
    pub async fn get_address(&self, chain_id: ChainId) -> Result<String, WalletError> {
        let wallet = self.chains.get(&chain_id)
            .ok_or(WalletError::UnsupportedChain(chain_id))?;
        wallet.get_address().await
    }
    
    /// Get balance for chain
    pub async fn get_balance(&self, chain_id: ChainId) -> Result<Balance, WalletError> {
        let wallet = self.chains.get(&chain_id)
            .ok_or(WalletError::UnsupportedChain(chain_id))?;
        wallet.get_balance().await
    }
    
    /// Send transaction on chain
    pub async fn send_transaction(
        &self,
        chain_id: ChainId,
        tx: &[u8],
    ) -> Result<TxHash, WalletError> {
        let wallet = self.chains.get(&chain_id)
            .ok_or(WalletError::UnsupportedChain(chain_id))?;
        wallet.send_transaction(tx).await
    }
    
    /// Get all balances
    pub async fn get_all_balances(&self) -> Result<HashMap<ChainId, Balance>, WalletError> {
        let mut balances = HashMap::new();
        
        for (chain_id, wallet) in &self.chains {
            let balance = wallet.get_balance().await?;
            balances.insert(*chain_id, balance);
        }
        
        Ok(balances)
    }
    
    /// Get supported chains
    pub fn supported_chains(&self) -> Vec<ChainId> {
        self.chains.keys().cloned().collect()
    }
}

/// Chain wallet trait
#[async_trait]
pub trait ChainWallet: Send + Sync {
    async fn get_address(&self) -> Result<String, WalletError>;
    async fn get_balance(&self) -> Result<Balance, WalletError>;
    async fn send_transaction(&self, tx: &[u8]) -> Result<TxHash, WalletError>;
    async fn sign_transaction(&self, tx: &[u8]) -> Result<Signature, WalletError>;
}
```

### 4.2 Cross-Chain Atomic Swaps

```rust
/// Cross-chain atomic swap
pub struct AtomicSwap {
    swap_id: SwapId,
    initiator: ChainId,
    counterparty: ChainId,
    initiator_address: Address,
    counterparty_address: Address,
    amount: u64,
    secret_hash: [u8; 32],
    timeout: Duration,
    status: SwapStatus,
}

impl AtomicSwap {
    pub fn new(
        initiator: ChainId,
        counterparty: ChainId,
        initiator_address: Address,
        counterparty_address: Address,
        amount: u64,
        secret_hash: [u8; 32],
        timeout: Duration,
    ) -> Self {
        Self {
            swap_id: SwapId::new(),
            initiator,
            counterparty,
            initiator_address,
            counterparty_address,
            amount,
            secret_hash,
            timeout,
            status: SwapStatus::Initiated,
        }
    }
    
    /// Initiate swap on initiator chain
    pub async fn initiate<W: ChainWallet>(
        &self,
        wallet: &W,
    ) -> Result<TxHash, WalletError> {
        // Create lock transaction on initiator chain
        let lock_tx = self.create_lock_transaction()?;
        
        wallet.send_transaction(&lock_tx).await
    }
    
    /// Participate in swap on counterparty chain
    pub async fn participate<W: ChainWallet>(
        &self,
        wallet: &W,
    ) -> Result<TxHash, WalletError> {
        // Create lock transaction on counterparty chain
        let lock_tx = self.create_lock_transaction()?;
        
        wallet.send_transaction(&lock_tx).await
    }
    
    /// Claim swap with secret
    pub async fn claim<W: ChainWallet>(
        &self,
        wallet: &W,
        secret: &[u8; 32],
    ) -> Result<TxHash, WalletError> {
        // Verify secret hash
        let computed_hash = self.compute_secret_hash(secret);
        if computed_hash != self.secret_hash {
            return Err(WalletError::InvalidSecret);
        }
        
        // Create claim transaction
        let claim_tx = self.create_claim_transaction(secret)?;
        
        wallet.send_transaction(&claim_tx).await
    }
    
    /// Refund swap after timeout
    pub async fn refund<W: ChainWallet>(
        &self,
        wallet: &W,
    ) -> Result<TxHash, WalletError> {
        // Check if timeout has passed
        if !self.is_timeout_reached() {
            return Err(WalletError::TimeoutNotReached);
        }
        
        // Create refund transaction
        let refund_tx = self.create_refund_transaction()?;
        
        wallet.send_transaction(&refund_tx).await
    }
    
    fn compute_secret_hash(&self, secret: &[u8; 32]) -> [u8; 32] {
        use sha2::{Sha256, Digest};
        
        let mut hasher = Sha256::new();
        hasher.update(secret);
        let result = hasher.finalize();
        
        let mut hash = [0u8; 32];
        hash.copy_from_slice(&result);
        hash
    }
    
    fn create_lock_transaction(&self) -> Result<Vec<u8>, WalletError> {
        // Create lock transaction (implementation depends on chain)
        Ok(vec![])
    }
    
    fn create_claim_transaction(&self, secret: &[u8; 32]) -> Result<Vec<u8>, WalletError> {
        // Create claim transaction (implementation depends on chain)
        Ok(vec![])
    }
    
    fn create_refund_transaction(&self) -> Result<Vec<u8>, WalletError> {
        // Create refund transaction (implementation depends on chain)
        Ok(vec![])
    }
    
    fn is_timeout_reached(&self) -> bool {
        // Check if timeout has passed
        true
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum SwapStatus {
    Initiated,
    Locked,
    Claimed,
    Refunded,
    Expired,
}
```

## 5. Wallet Recovery

### 5.1 Backup and Restore

```rust
/// Wallet backup
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletBackup {
    pub version: u32,
    pub wallet_id: WalletId,
    pub encrypted_mnemonic: EncryptedData,
    pub encrypted_keys: Vec<EncryptedKey>,
    pub metadata: WalletMetadata,
    pub timestamp: DateTime<Utc>,
}

/// Wallet backup manager
pub struct WalletBackupManager {
    encryption_key: [u8; 32],
    storage: Box<dyn BackupStorage>,
}

impl WalletBackupManager {
    pub fn new(encryption_key: [u8; 32], storage: Box<dyn BackupStorage>) -> Self {
        Self {
            encryption_key,
            storage,
        }
    }
    
    /// Create backup
    pub async fn create_backup(
        &self,
        wallet: &HDWallet,
    ) -> Result<WalletBackup, WalletError> {
        // Encrypt mnemonic
        let encrypted_mnemonic = self.encrypt_data(wallet.mnemonic_phrase().as_bytes())?;
        
        // Encrypt keys
        let mut encrypted_keys = Vec::new();
        for (index, account) in wallet.accounts.iter() {
            let key_bytes = account.private_key().private_key().as_bytes().to_vec();
            let encrypted_key = self.encrypt_data(&key_bytes)?;
            
            encrypted_keys.push(EncryptedKey {
                account_index: *index,
                data: encrypted_key,
            });
        }
        
        let backup = WalletBackup {
            version: 1,
            wallet_id: WalletId::new(),
            encrypted_mnemonic,
            encrypted_keys,
            metadata: WalletMetadata::default(),
            timestamp: Utc::now(),
        };
        
        Ok(backup)
    }
    
    /// Restore from backup
    pub async fn restore_backup(
        &self,
        backup: &WalletBackup,
    ) -> Result<HDWallet, WalletError> {
        // Decrypt mnemonic
        let mnemonic_bytes = self.decrypt_data(&backup.encrypted_mnemonic)?;
        let mnemonic = String::from_utf8(mnemonic_bytes)
            .map_err(|e| WalletError::InvalidBackup(e.to_string()))?;
        
        // Restore wallet
        let mut wallet = HDWallet::from_mnemonic(&mnemonic)?;
        
        // Restore accounts
        for encrypted_key in &backup.encrypted_keys {
            let key_bytes = self.decrypt_data(&encrypted_key.data)?;
            // Restore account from key bytes
            // ...
        }
        
        Ok(wallet)
    }
    
    /// Save backup to storage
    pub async fn save_backup(&self, backup: &WalletBackup) -> Result<(), WalletError> {
        let backup_data = serde_json::to_vec(backup)
            .map_err(|e| WalletError::SerializationError(e.to_string()))?;
        
        self.storage.store(&backup.wallet_id, &backup_data).await
    }
    
    /// Load backup from storage
    pub async fn load_backup(&self, wallet_id: &WalletId) -> Result<WalletBackup, WalletError> {
        let backup_data = self.storage.load(wallet_id).await?;
        
        serde_json::from_slice(&backup_data)
            .map_err(|e| WalletError::InvalidBackup(e.to_string()))
    }
    
    fn encrypt_data(&self, data: &[u8]) -> Result<EncryptedData, WalletError> {
        let cipher = Aes256Gcm::new(&self.encryption_key.into());
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
        
        let ciphertext = cipher
            .encrypt(&nonce, data)
            .map_err(|e| WalletError::EncryptionError(e.to_string()))?;
        
        Ok(EncryptedData {
            nonce: nonce.to_vec(),
            ciphertext,
        })
    }
    
    fn decrypt_data(&self, encrypted: &EncryptedData) -> Result<Vec<u8>, WalletError> {
        let cipher = Aes256Gcm::new(&self.encryption_key.into());
        let nonce = Nonce::from_slice(&encrypted.nonce);
        
        cipher
            .decrypt(nonce, encrypted.ciphertext.as_ref())
            .map_err(|e| WalletError::DecryptionError(e.to_string()))
    }
}

/// Backup storage trait
#[async_trait]
pub trait BackupStorage: Send + Sync {
    async fn store(&self, wallet_id: &WalletId, data: &[u8]) -> Result<(), WalletError>;
    async fn load(&self, wallet_id: &WalletId) -> Result<Vec<u8>, WalletError>;
    async fn delete(&self, wallet_id: &WalletId) -> Result<(), WalletError>;
}
```

### 5.2 Social Recovery

```rust
/// Social recovery setup
pub struct SocialRecoverySetup {
    wallet_id: WalletId,
    guardians: Vec<Guardian>,
    threshold: usize,
    recovery_hash: [u8; 32],
}

impl SocialRecoverySetup {
    pub fn new(
        wallet_id: WalletId,
        guardians: Vec<Guardian>,
        threshold: usize,
    ) -> Self {
        Self {
            wallet_id,
            guardians,
            threshold,
            recovery_hash: [0u8; 32],
        }
    }
    
    /// Setup social recovery
    pub async fn setup<W: Wallet>(
        &self,
        wallet: &W,
    ) -> Result<(), WalletError> {
        // Generate recovery secret
        let recovery_secret = self.generate_recovery_secret()?;
        
        // Compute hash
        self.recovery_hash = self.compute_hash(&recovery_secret);
        
        // Split secret among guardians
        let shares = self.split_secret(&recovery_secret)?;
        
        // Distribute shares to guardians
        for (guardian, share) in self.guardians.iter().zip(shares.iter()) {
            self.distribute_share(guardian, share).await?;
        }
        
        // Store recovery hash on-chain
        self.store_recovery_hash(wallet).await?;
        
        Ok(())
    }
    
    /// Initiate recovery
    pub async fn initiate_recovery<W: Wallet>(
        &self,
        wallet: &W,
        recovery_secret: &[u8; 32],
    ) -> Result<(), WalletError> {
        // Verify recovery secret
        let computed_hash = self.compute_hash(recovery_secret);
        if computed_hash != self.recovery_hash {
            return Err(WalletError::InvalidRecoverySecret);
        }
        
        // Collect shares from guardians
        let mut shares = Vec::new();
        for guardian in &self.guardians {
            let share = self.collect_share(guardian).await?;
            shares.push(share);
        }
        
        // Reconstruct secret
        let reconstructed = self.reconstruct_secret(&shares)?;
        
        // Verify reconstructed secret
        if reconstructed != *recovery_secret {
            return Err(WalletError::ReconstructionFailed);
        }
        
        // Recover wallet
        self.recover_wallet(wallet, &reconstructed).await?;
        
        Ok(())
    }
    
    fn generate_recovery_secret(&self) -> Result<[u8; 32], WalletError> {
        let mut secret = [0u8; 32];
        OsRng.fill_bytes(&mut secret);
        Ok(secret)
    }
    
    fn compute_hash(&self, secret: &[u8; 32]) -> [u8; 32] {
        use sha2::{Sha256, Digest};
        
        let mut hasher = Sha256::new();
        hasher.update(secret);
        let result = hasher.finalize();
        
        let mut hash = [0u8; 32];
        hash.copy_from_slice(&result);
        hash
    }
    
    fn split_secret(&self, secret: &[u8; 32]) -> Result<Vec<Vec<u8>>, WalletError> {
        // Use Shamir's Secret Sharing
        // Implementation depends on library
        Ok(vec![])
    }
    
    fn reconstruct_secret(&self, shares: &[Vec<u8>]) -> Result<[u8; 32], WalletError> {
        // Reconstruct secret from shares
        // Implementation depends on library
        Ok([0u8; 32])
    }
    
    async fn distribute_share(&self, guardian: &Guardian, share: &[u8]) -> Result<(), WalletError> {
        // Send share to guardian
        Ok(())
    }
    
    async fn collect_share(&self, guardian: &Guardian) -> Result<Vec<u8>, WalletError> {
        // Collect share from guardian
        Ok(vec![])
    }
    
    async fn store_recovery_hash<W: Wallet>(&self, wallet: &W) -> Result<(), WalletError> {
        // Store recovery hash on-chain
        Ok(())
    }
    
    async fn recover_wallet<W: Wallet>(
        &self,
        wallet: &W,
        secret: &[u8; 32],
    ) -> Result<(), WalletError> {
        // Recover wallet using secret
        Ok(())
    }
}

/// Guardian information
#[derive(Debug, Clone)]
pub struct Guardian {
    pub id: GuardianId,
    pub address: Address,
    pub contact_info: String,
}
```

## 6. Summary

### 6.1 Key Management
- HD wallets with BIP39/44
- Multi-signature support
- Key rotation
- Secure key storage

### 6.2 Transaction Handling
- Transaction builders with validation
- Batch transactions
- Transaction estimation
- Gas price oracles

### 6.3 Multi-Chain Support
- Unified wallet interface
- Cross-chain atomic swaps
- Chain-specific adapters

### 6.4 Recovery
- Backup and restore
- Social recovery
- Secret sharing
