# Wallet SDK Best Practices Research

## 1. Overview

This document researches wallet SDK best practices from major blockchain ecosystems (Ethereum, Solana, Cosmos, Polkadot) to inform the design of a production-ready Canton Wallet SDK in Rust.

## 2. Ethereum Wallet SDK Analysis

### 2.1 Key Implementations

| SDK | Language | Status | Key Features |
|-----|----------|--------|--------------|
| ethers-rs | Rust | Production | Full Ethereum wallet, signing, transactions |
| alloy | Rust | Modern | Type-safe, async-first, modular |
| web3.js | JavaScript | Production | Web-focused, browser integration |
| viem | TypeScript | Modern | Type-safe, minimal dependencies |

### 2.2 Core Wallet Patterns

#### Pattern 1: Account Abstraction

```rust
// Ethereum-style account abstraction
use ethers::signers::{Signer, Wallet};

/// Wallet account abstraction
pub trait WalletAccount: Send + Sync {
    /// Get account address
    fn address(&self) -> Address;
    
    /// Sign transaction
    async fn sign_transaction(&self, tx: &Transaction) -> Result<Signature, SignerError>;
    
    /// Sign message
    async fn sign_message(&self, message: &[u8]) -> Result<Signature, SignerError>;
    
    /// Get chain ID
    fn chain_id(&self) -> u64;
}

/// HD wallet implementation
pub struct HDWallet {
    mnemonic: Mnemonic,
    derivation_path: DerivationPath,
    accounts: HashMap<u32, Wallet>,
}

impl HDWallet {
    pub fn from_mnemonic(mnemonic: &str, path: &str) -> Result<Self, WalletError> {
        let mnemonic = Mnemonic::from_phrase(mnemonic)?;
        let derivation_path = DerivationPath::from_str(path)?;
        
        Ok(Self {
            mnemonic,
            derivation_path,
            accounts: HashMap::new(),
        })
    }
    
    pub fn derive_account(&mut self, index: u32) -> Result<Address, WalletError> {
        if self.accounts.contains_key(&index) {
            return Ok(self.accounts[&index].address());
        }
        
        let wallet = Wallet::from_mnemonic_phrase_with_index(
            &self.mnemonic.phrase(),
            index,
        )?;
        
        let address = wallet.address();
        self.accounts.insert(index, wallet);
        
        Ok(address)
    }
}
```

#### Pattern 2: Transaction Builder

```rust
/// Transaction builder pattern
pub struct TransactionBuilder {
    from: Option<Address>,
    to: Option<Address>,
    value: Option<U256>,
    data: Option<Bytes>,
    gas_limit: Option<U256>,
    gas_price: Option<U256>,
    nonce: Option<U256>,
    chain_id: Option<u64>,
}

impl TransactionBuilder {
    pub fn new() -> Self {
        Self {
            from: None,
            to: None,
            value: None,
            data: None,
            gas_limit: None,
            gas_price: None,
            nonce: None,
            chain_id: None,
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
    
    pub fn value(mut self, value: U256) -> Self {
        self.value = Some(value);
        self
    }
    
    pub fn data(mut self, data: Bytes) -> Self {
        self.data = Some(data);
        self
    }
    
    pub fn gas_limit(mut self, limit: U256) -> Self {
        self.gas_limit = Some(limit);
        self
    }
    
    pub fn gas_price(mut self, price: U256) -> Self {
        self.gas_price = Some(price);
        self
    }
    
    pub fn nonce(mut self, nonce: U256) -> Self {
        self.nonce = Some(nonce);
        self
    }
    
    pub fn chain_id(mut self, id: u64) -> Self {
        self.chain_id = Some(id);
        self
    }
    
    pub async fn build<W: WalletAccount>(
        self,
        wallet: &W,
    ) -> Result<SignedTransaction, WalletError> {
        let tx = Transaction {
            from: self.from.ok_or(WalletError::MissingField("from"))?,
            to: self.to.ok_or(WalletError::MissingField("to"))?,
            value: self.value.unwrap_or_else(U256::zero),
            data: self.data.unwrap_or_else(Bytes::new),
            gas_limit: self.gas_limit.ok_or(WalletError::MissingField("gas_limit"))?,
            gas_price: self.gas_price.ok_or(WalletError::MissingField("gas_price"))?,
            nonce: self.nonce.ok_or(WalletError::MissingField("nonce"))?,
            chain_id: self.chain_id.ok_or(WalletError::MissingField("chain_id"))?,
        };
        
        let signature = wallet.sign_transaction(&tx).await?;
        
        Ok(SignedTransaction {
            transaction: tx,
            signature,
        })
    }
}
```

### 2.3 Key Management Patterns

```rust
/// Key store interface
#[async_trait]
pub trait KeyStore: Send + Sync {
    /// Generate new key
    async fn generate_key(&self) -> Result<KeyId, KeyStoreError>;
    
    /// Import key
    async fn import_key(&self, key: PrivateKey) -> Result<KeyId, KeyStoreError>;
    
    /// Export public key
    async fn export_public_key(&self, key_id: &KeyId) -> Result<PublicKey, KeyStoreError>;
    
    /// Sign data
    async fn sign(&self, key_id: &KeyId, data: &[u8]) -> Result<Signature, KeyStoreError>;
    
    /// Delete key
    async fn delete_key(&self, key_id: &KeyId) -> Result<(), KeyStoreError>;
    
    /// List keys
    async fn list_keys(&self) -> Result<Vec<KeyInfo>, KeyStoreError>;
}

/// Encrypted key store
pub struct EncryptedKeyStore {
    encryption_key: EncryptionKey,
    storage: Box<dyn StorageBackend>,
}

impl EncryptedKeyStore {
    pub fn new(encryption_key: EncryptionKey, storage: Box<dyn StorageBackend>) -> Self {
        Self {
            encryption_key,
            storage,
        }
    }
    
    async fn encrypt_key(&self, key: &PrivateKey) -> Result<EncryptedKey, KeyStoreError> {
        let key_bytes = key.to_bytes();
        let encrypted = self.encryption_key.encrypt(&key_bytes)?;
        
        Ok(EncryptedKey {
            ciphertext: encrypted,
            nonce: self.encryption_key.generate_nonce(),
        })
    }
    
    async fn decrypt_key(&self, encrypted: &EncryptedKey) -> Result<PrivateKey, KeyStoreError> {
        let decrypted = self.encryption_key.decrypt(&encrypted.ciphertext, &encrypted.nonce)?;
        PrivateKey::from_bytes(&decrypted)
    }
}

#[async_trait]
impl KeyStore for EncryptedKeyStore {
    async fn generate_key(&self) -> Result<KeyId, KeyStoreError> {
        let key = PrivateKey::generate();
        let key_id = KeyId::new();
        
        let encrypted = self.encrypt_key(&key).await?;
        self.storage.store(&key_id, &encrypted).await?;
        
        Ok(key_id)
    }
    
    async fn sign(&self, key_id: &KeyId, data: &[u8]) -> Result<Signature, KeyStoreError> {
        let encrypted = self.storage.load(key_id).await?;
        let key = self.decrypt_key(&encrypted).await?;
        
        Ok(key.sign(data))
    }
    
    // ... other methods
}
```

## 3. Solana Wallet SDK Analysis

### 3.1 Key Implementations

| SDK | Language | Status | Key Features |
|-----|----------|--------|--------------|
| solana-sdk | Rust | Production | Core wallet functionality |
| solana-client | Rust | Production | RPC client integration |
| anchor-lang | Rust | Production | Framework for programs |

### 3.2 Solana-Specific Patterns

#### Pattern 1: Keypair Management

```rust
use solana_sdk::{
    pubkey::Pubkey,
    signature::{Keypair, Signer, Signature},
    transaction::Transaction,
};

/// Solana wallet wrapper
pub struct SolanaWallet {
    keypair: Keypair,
    pubkey: Pubkey,
}

impl SolanaWallet {
    pub fn new() -> Self {
        let keypair = Keypair::new();
        let pubkey = keypair.pubkey();
        
        Self { keypair, pubkey }
    }
    
    pub fn from_seed(seed: &[u8]) -> Result<Self, WalletError> {
        let keypair = Keypair::from_seed(seed)
            .map_err(|e| WalletError::InvalidSeed(e.to_string()))?;
        let pubkey = keypair.pubkey();
        
        Ok(Self { keypair, pubkey })
    }
    
    pub fn pubkey(&self) -> &Pubkey {
        &self.pubkey
    }
    
    pub fn sign_transaction(&self, tx: &mut Transaction) -> Result<Signature, WalletError> {
        Ok(self.keypair.try_sign_message(tx.message_data())?)
    }
    
    pub fn sign_message(&self, message: &[u8]) -> Result<Signature, WalletError> {
        Ok(self.keypair.sign_message(message))
    }
}

/// Multi-signature wallet
pub struct MultiSigWallet {
    signers: Vec<Keypair>,
    threshold: usize,
}

impl MultiSigWallet {
    pub fn new(signers: Vec<Keypair>, threshold: usize) -> Self {
        Self { signers, threshold }
    }
    
    pub fn sign_transaction(&self, tx: &mut Transaction) -> Result<Vec<Signature>, WalletError> {
        if self.signers.len() < self.threshold {
            return Err(WalletError::InsufficientSigners);
        }
        
        let mut signatures = Vec::new();
        for signer in self.signers.iter().take(self.threshold) {
            let sig = signer.try_sign_message(tx.message_data())?;
            signatures.push(sig);
        }
        
        Ok(signatures)
    }
}
```

#### Pattern 2: Program Interaction

```rust
/// Program client builder
pub struct ProgramClientBuilder {
    rpc_url: Option<String>,
    commitment: Option<CommitmentConfig>,
    payer: Option<Keypair>,
}

impl ProgramClientBuilder {
    pub fn new() -> Self {
        Self {
            rpc_url: None,
            commitment: None,
            payer: None,
        }
    }
    
    pub fn rpc_url(mut self, url: impl Into<String>) -> Self {
        self.rpc_url = Some(url.into());
        self
    }
    
    pub fn commitment(mut self, commitment: CommitmentConfig) -> Self {
        self.commitment = Some(commitment);
        self
    }
    
    pub fn payer(mut self, keypair: Keypair) -> Self {
        self.payer = Some(keypair);
        self
    }
    
    pub async fn build(self) -> Result<ProgramClient, WalletError> {
        let rpc_url = self.rpc_url.ok_or(WalletError::MissingRpcUrl)?;
        let payer = self.payer.ok_or(WalletError::MissingPayer)?;
        let commitment = self.commitment.unwrap_or(CommitmentConfig::confirmed());
        
        let rpc_client = RpcClient::new_with_commitment(rpc_url, commitment);
        
        Ok(ProgramClient {
            rpc_client,
            payer,
        })
    }
}

/// Program client
pub struct ProgramClient {
    rpc_client: RpcClient,
    payer: Keypair,
}

impl ProgramClient {
    pub async fn send_transaction(
        &self,
        instructions: Vec<Instruction>,
    ) -> Result<Signature, WalletError> {
        let recent_blockhash = self.rpc_client
            .get_latest_blockhash()
            .await
            .map_err(|e| WalletError::RpcError(e.to_string()))?;
        
        let mut transaction = Transaction::new_with_payer(
            &instructions,
            Some(&self.payer.pubkey()),
        );
        
        transaction.sign(&[&self.payer], recent_blockhash);
        
        let signature = self.rpc_client
            .send_transaction_with_config(
                &transaction,
                RpcSendTransactionConfig {
                    skip_preflight: false,
                    preflight_commitment: Some(CommitmentLevel::Confirmed),
                    ..Default::default()
                },
            )
            .await
            .map_err(|e| WalletError::TransactionError(e.to_string()))?;
        
        Ok(signature)
    }
}
```

## 4. Cosmos Wallet SDK Analysis

### 4.1 Key Implementations

| SDK | Language | Status | Key Features |
|-----|----------|--------|--------------|
| cosmrs | Rust | Production | Tendermint RPC, Amino, Protobuf |
| cosmos-sdk | Go | Production | Official SDK |
| cosmwasm | Rust | Production | Smart contracts |

### 4.2 Cosmos-Specific Patterns

#### Pattern 1: HD Wallet with BIP39/44

```rust
use bip39::{Mnemonic, MnemonicType, Language, Seed};
use bip32::{Mnemonic as Bip32Mnemonic, XPrv, XPub, DerivationPath};
use cosmrs::{
    tx::{Msg, SignDoc, SignerInfo},
    crypto::{secp256k1::SigningKey, PublicKey},
};

/// Cosmos HD wallet
pub struct CosmosHDWallet {
    mnemonic: Mnemonic,
    accounts: HashMap<u32, CosmosAccount>,
}

impl CosmosHDWallet {
    pub fn new(mnemonic: &str) -> Result<Self, WalletError> {
        let mnemonic = Mnemonic::from_phrase(mnemonic, Language::English)
            .map_err(|e| WalletError::InvalidMnemonic(e.to_string()))?;
        
        Ok(Self {
            mnemonic,
            accounts: HashMap::new(),
        })
    }
    
    pub fn generate() -> Result<Self, WalletError> {
        let mnemonic = Mnemonic::new(MnemonicType::Words12, Language::English);
        
        Ok(Self {
            mnemonic,
            accounts: HashMap::new(),
        })
    }
    
    pub fn derive_account(&mut self, index: u32) -> Result<&CosmosAccount, WalletError> {
        if self.accounts.contains_key(&index) {
            return Ok(&self.accounts[&index]);
        }
        
        let seed = Seed::new(&self.mnemonic, "");
        let path = format!("m/44'/118'/0'/0/{}", index);
        let derivation_path = DerivationPath::from_str(&path)
            .map_err(|e| WalletError::InvalidDerivationPath(e.to_string()))?;
        
        let xprv = XPrv::derive_from_path(
            Bip32Mnemonic::from_phrase(&self.mnemonic.phrase(), Language::English)
                .map_err(|e| WalletError::InvalidMnemonic(e.to_string()))?,
            &derivation_path,
        )
        .map_err(|e| WalletError::DerivationError(e.to_string()))?;
        
        let signing_key = SigningKey::from_slice(xprv.private_key().as_ref())
            .map_err(|e| WalletError::KeyError(e.to_string()))?;
        
        let public_key = PublicKey::from(&signing_key);
        let address = public_key.account_id("cosmos")?;
        
        let account = CosmosAccount {
            index,
            signing_key,
            public_key,
            address,
        };
        
        self.accounts.insert(index, account);
        Ok(&self.accounts[&index])
    }
}

/// Cosmos account
pub struct CosmosAccount {
    index: u32,
    signing_key: SigningKey,
    public_key: PublicKey,
    address: cosmrs::AccountId,
}

impl CosmosAccount {
    pub fn address(&self) -> &cosmrs::AccountId {
        &self.address
    }
    
    pub fn public_key(&self) -> &PublicKey {
        &self.public_key
    }
    
    pub fn sign(&self, sign_doc: &SignDoc) -> Result<cosmrs::crypto::secp256k1::Signature, WalletError> {
        use k256::ecdsa::signature::Signer;
        
        let signature = self.signing_key.sign(sign_doc.sign_bytes());
        Ok(signature)
    }
}
```

#### Pattern 2: Transaction Builder

```rust
/// Cosmos transaction builder
pub struct CosmosTxBuilder {
    chain_id: String,
    account_number: u64,
    sequence: u64,
    fee: Option<Fee>,
    memo: Option<String>,
    timeout_height: Option<u64>,
    messages: Vec<Box<dyn Msg>>,
}

impl CosmosTxBuilder {
    pub fn new(chain_id: impl Into<String>) -> Self {
        Self {
            chain_id: chain_id.into(),
            account_number: 0,
            sequence: 0,
            fee: None,
            memo: None,
            timeout_height: None,
            messages: Vec::new(),
        }
    }
    
    pub fn account_number(mut self, number: u64) -> Self {
        self.account_number = number;
        self
    }
    
    pub fn sequence(mut self, seq: u64) -> Self {
        self.sequence = seq;
        self
    }
    
    pub fn fee(mut self, fee: Fee) -> Self {
        self.fee = Some(fee);
        self
    }
    
    pub fn memo(mut self, memo: impl Into<String>) -> Self {
        self.memo = Some(memo.into());
        self
    }
    
    pub fn message(mut self, msg: Box<dyn Msg>) -> Self {
        self.messages.push(msg);
        self
    }
    
    pub fn build(&self) -> Result<cosmrs::tx::TxBody, WalletError> {
        Ok(cosmrs::tx::TxBody::new(
            self.messages.iter().map(|msg| msg.as_any()).collect(),
            self.memo.clone(),
            self.timeout_height,
        ))
    }
}
```

## 5. Polkadot Wallet SDK Analysis

### 5.1 Key Implementations

| SDK | Language | Status | Key Features |
|-----|----------|--------|--------------|
| subxt | Rust | Production | Substrate RPC client |
| polkadot-js | TypeScript | Production | Web-focused |
| sp-core | Rust | Production | Core primitives |

### 5.2 Polkadot-Specific Patterns

#### Pattern 1: Sr25519 Key Management

```rust
use sp_core::{
    crypto::{Ss58Codec, Ss58AddressFormat},
    sr25519::{Pair, Public, Signature},
    Pair as PairTrait,
};

/// Polkadot wallet
pub struct PolkadotWallet {
    pair: Pair,
    public: Public,
}

impl PolkadotWallet {
    pub fn new() -> Self {
        let (pair, public) = Pair::generate();
        
        Self { pair, public }
    }
    
    pub fn from_mnemonic(mnemonic: &str, password: Option<&str>) -> Result<Self, WalletError> {
        let pair = Pair::from_phrase(mnemonic, password.unwrap_or(""))
            .map_err(|e| WalletError::InvalidMnemonic(e.to_string()))?
            .0;
        
        let public = pair.public();
        
        Ok(Self { pair, public })
    }
    
    pub fn from_seed(seed: &str) -> Result<Self, WalletError> {
        let pair = Pair::from_string(seed, None)
            .map_err(|e| WalletError::InvalidSeed(e.to_string()))?;
        
        let public = pair.public();
        
        Ok(Self { pair, public })
    }
    
    pub fn address(&self, format: Ss58AddressFormat) -> String {
        self.public.to_ss58check_with_version(format)
    }
    
    pub fn public_key(&self) -> &Public {
        &self.public
    }
    
    pub fn sign(&self, message: &[u8]) -> Signature {
        self.pair.sign(message)
    }
    
    pub fn verify(&self, message: &[u8], signature: &Signature) -> bool {
        self.pair.verify(message, signature).is_ok()
    }
}
```

## 6. Cross-Chain Wallet Patterns

### 6.1 Unified Wallet Interface

```rust
/// Unified wallet interface for multiple chains
#[async_trait]
pub trait UnifiedWallet: Send + Sync {
    /// Get wallet identifier
    fn wallet_id(&self) -> &WalletId;
    
    /// Get supported chains
    fn supported_chains(&self) -> Vec<ChainId>;
    
    /// Get address for chain
    async fn get_address(&self, chain: ChainId) -> Result<String, WalletError>;
    
    /// Sign transaction
    async fn sign_transaction(
        &self,
        chain: ChainId,
        transaction: &[u8],
    ) -> Result<Signature, WalletError>;
    
    /// Sign message
    async fn sign_message(
        &self,
        chain: ChainId,
        message: &[u8],
    ) -> Result<Signature, WalletError>;
    
    /// Get balance
    async fn get_balance(&self, chain: ChainId) -> Result<Balance, WalletError>;
    
    /// Send transaction
    async fn send_transaction(
        &self,
        chain: ChainId,
        transaction: &[u8],
    ) -> Result<TxHash, WalletError>;
}

/// Multi-chain wallet implementation
pub struct MultiChainWallet {
    wallet_id: WalletId,
    chains: HashMap<ChainId, Box<dyn ChainWallet>>,
}

impl MultiChainWallet {
    pub fn new(wallet_id: WalletId) -> Self {
        Self {
            wallet_id,
            chains: HashMap::new(),
        }
    }
    
    pub fn add_chain(&mut self, chain: ChainId, wallet: Box<dyn ChainWallet>) {
        self.chains.insert(chain, wallet);
    }
}

#[async_trait]
impl UnifiedWallet for MultiChainWallet {
    fn wallet_id(&self) -> &WalletId {
        &self.wallet_id
    }
    
    fn supported_chains(&self) -> Vec<ChainId> {
        self.chains.keys().cloned().collect()
    }
    
    async fn get_address(&self, chain: ChainId) -> Result<String, WalletError> {
        let wallet = self.chains.get(&chain)
            .ok_or(WalletError::UnsupportedChain(chain))?;
        wallet.get_address().await
    }
    
    async fn sign_transaction(
        &self,
        chain: ChainId,
        transaction: &[u8],
    ) -> Result<Signature, WalletError> {
        let wallet = self.chains.get(&chain)
            .ok_or(WalletError::UnsupportedChain(chain))?;
        wallet.sign_transaction(transaction).await
    }
    
    async fn sign_message(
        &self,
        chain: ChainId,
        message: &[u8],
    ) -> Result<Signature, WalletError> {
        let wallet = self.chains.get(&chain)
            .ok_or(WalletError::UnsupportedChain(chain))?;
        wallet.sign_message(message).await
    }
    
    async fn get_balance(&self, chain: ChainId) -> Result<Balance, WalletError> {
        let wallet = self.chains.get(&chain)
            .ok_or(WalletError::UnsupportedChain(chain))?;
        wallet.get_balance().await
    }
    
    async fn send_transaction(
        &self,
        chain: ChainId,
        transaction: &[u8],
    ) -> Result<TxHash, WalletError> {
        let wallet = self.chains.get(&chain)
            .ok_or(WalletError::UnsupportedChain(chain))?;
        wallet.send_transaction(transaction).await
    }
}

/// Chain-specific wallet trait
#[async_trait]
pub trait ChainWallet: Send + Sync {
    async fn get_address(&self) -> Result<String, WalletError>;
    async fn sign_transaction(&self, transaction: &[u8]) -> Result<Signature, WalletError>;
    async fn sign_message(&self, message: &[u8]) -> Result<Signature, WalletError>;
    async fn get_balance(&self) -> Result<Balance, WalletError>;
    async fn send_transaction(&self, transaction: &[u8]) -> Result<TxHash, WalletError>;
}
```

## 7. Security Best Practices

### 7.1 Key Security

```rust
use zeroize::{Zeroize, ZeroizeOnDrop};

/// Secure private key wrapper
#[derive(ZeroizeOnDrop)]
pub struct SecurePrivateKey {
    #[zeroize(skip)]
    algorithm: KeyAlgorithm,
    key_bytes: Vec<u8>,
}

impl SecurePrivateKey {
    pub fn new(algorithm: KeyAlgorithm, key_bytes: Vec<u8>) -> Self {
        Self {
            algorithm,
            key_bytes,
        }
    }
    
    pub fn as_bytes(&self) -> &[u8] {
        &self.key_bytes
    }
    
    pub fn algorithm(&self) -> KeyAlgorithm {
        self.algorithm
    }
}

impl Drop for SecurePrivateKey {
    fn drop(&mut self) {
        self.key_bytes.zeroize();
    }
}

/// Memory-locked key storage
pub struct MemoryLockedKey {
    #[cfg(target_os = "linux")]
    key: mlock::MlockedVec<u8>,
    
    #[cfg(not(target_os = "linux"))]
    key: Vec<u8>,
}

impl MemoryLockedKey {
    pub fn new(key_bytes: Vec<u8>) -> Result<Self, WalletError> {
        #[cfg(target_os = "linux")]
        let key = mlock::MlockedVec::new(key_bytes)
            .map_err(|e| WalletError::MemoryLockError(e.to_string()))?;
        
        #[cfg(not(target_os = "linux"))]
        let key = key_bytes;
        
        Ok(Self { key })
    }
    
    pub fn as_bytes(&self) -> &[u8] {
        #[cfg(target_os = "linux")]
        return &self.key;
        
        #[cfg(not(target_os = "linux"))]
        return &self.key;
    }
}
```

### 7.2 Secure Random Generation

```rust
use rand::{rngs::OsRng, RngCore};

/// Secure random generator
pub struct SecureRandom;

impl SecureRandom {
    /// Generate random bytes
    pub fn bytes(len: usize) -> Vec<u8> {
        let mut bytes = vec![0u8; len];
        OsRng.fill_bytes(&mut bytes);
        bytes
    }
    
    /// Generate random u32
    pub fn u32() -> u32 {
        OsRng.next_u32()
    }
    
    /// Generate random u64
    pub fn u64() -> u64 {
        OsRng.next_u64()
    }
    
    /// Generate random mnemonic
    pub fn mnemonic(word_count: usize) -> Result<String, WalletError> {
        let entropy_len = word_count * 11 / 3 * 4;
        let entropy = Self::bytes(entropy_len);
        
        let mnemonic = Mnemonic::from_entropy(&entropy)
            .map_err(|e| WalletError::MnemonicGenerationError(e.to_string()))?;
        
        Ok(mnemonic.phrase().to_string())
    }
}
```

### 7.3 Transaction Validation

```rust
/// Transaction validator
pub struct TransactionValidator {
    max_fee: u64,
    max_gas_limit: u64,
    min_gas_price: u64,
}

impl TransactionValidator {
    pub fn new(max_fee: u64, max_gas_limit: u64, min_gas_price: u64) -> Self {
        Self {
            max_fee,
            max_gas_limit,
            min_gas_price,
        }
    }
    
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
        
        // Validate nonce
        if tx.nonce == 0 {
            return Err(WalletError::InvalidNonce(tx.nonce));
        }
        
        // Validate signature
        if !self.verify_signature(tx)? {
            return Err(WalletError::InvalidSignature);
        }
        
        Ok(())
    }
    
    fn verify_signature(&self, tx: &Transaction) -> Result<bool, WalletError> {
        // Implementation depends on chain
        Ok(true)
    }
}
```

## 8. Error Handling Patterns

```rust
use thiserror::Error;

/// Wallet error types
#[derive(Error, Debug)]
pub enum WalletError {
    #[error("Key not found: {0}")]
    KeyNotFound(String),
    
    #[error("Invalid mnemonic: {0}")]
    InvalidMnemonic(String),
    
    #[error("Invalid seed: {0}")]
    InvalidSeed(String),
    
    #[error("Invalid derivation path: {0}")]
    InvalidDerivationPath(String),
    
    #[error("Derivation error: {0}")]
    DerivationError(String),
    
    #[error("Key error: {0}")]
    KeyError(String),
    
    #[error("Invalid signature")]
    InvalidSignature,
    
    #[error("Transaction error: {0}")]
    TransactionError(String),
    
    #[error("Fee too high: {actual} > {max}")]
    FeeTooHigh { actual: u64, max: u64 },
    
    #[error("Gas limit too high: {actual} > {max}")]
    GasLimitTooHigh { actual: u64, max: u64 },
    
    #[error("Gas price too low: {actual} < {min}")]
    GasPriceTooLow { actual: u64, min: u64 },
    
    #[error("Invalid nonce: {0}")]
    InvalidNonce(u64),
    
    #[error("Unsupported chain: {0:?}")]
    UnsupportedChain(ChainId),
    
    #[error("RPC error: {0}")]
    RpcError(String),
    
    #[error("Network error: {0}")]
    NetworkError(String),
    
    #[error("Serialization error: {0}")]
    SerializationError(String),
    
    #[error("Memory lock error: {0}")]
    MemoryLockError(String),
    
    #[error("Mnemonic generation error: {0}")]
    MnemonicGenerationError(String),
    
    #[error("Missing field: {0}")]
    MissingField(&'static str),
    
    #[error("Missing RPC URL")]
    MissingRpcUrl,
    
    #[error("Missing payer")]
    MissingPayer,
    
    #[error("Insufficient signers")]
    InsufficientSigners,
    
    #[error("Internal error: {0}")]
    Internal(String),
}

/// Result type alias
pub type WalletResult<T> = Result<T, WalletError>;
```

## 9. Testing Patterns

```rust
/// Test wallet for testing
pub struct TestWallet {
    private_key: SecurePrivateKey,
    public_key: PublicKey,
}

impl TestWallet {
    pub fn new() -> Self {
        let key_bytes = SecureRandom::bytes(32);
        let private_key = SecurePrivateKey::new(KeyAlgorithm::Ed25519, key_bytes);
        let public_key = PublicKey::from_private_key(&private_key);
        
        Self {
            private_key,
            public_key,
        }
    }
    
    pub fn from_seed(seed: &[u8]) -> Self {
        use sha2::{Sha256, Digest};
        
        let mut hasher = Sha256::new();
        hasher.update(seed);
        let result = hasher.finalize();
        
        let private_key = SecurePrivateKey::new(KeyAlgorithm::Ed25519, result.to_vec());
        let public_key = PublicKey::from_private_key(&private_key);
        
        Self {
            private_key,
            public_key,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_wallet_sign_and_verify() {
        let wallet = TestWallet::new();
        let message = b"test message";
        
        let signature = wallet.sign(message).await.unwrap();
        let verified = wallet.verify(message, &signature).await.unwrap();
        
        assert!(verified);
    }
    
    #[tokio::test]
    async fn test_hd_wallet_derivation() {
        let mut hd_wallet = CosmosHDWallet::generate().unwrap();
        
        let account0 = hd_wallet.derive_account(0).unwrap();
        let account1 = hd_wallet.derive_account(1).unwrap();
        
        assert_ne!(account0.address(), account1.address());
    }
}
```

## 10. Summary of Best Practices

### 10.1 Key Management
- Use secure key storage with encryption
- Implement zeroization for sensitive data
- Support HD wallets with BIP39/44
- Provide multiple key storage backends (memory, file, HSM)

### 10.2 Transaction Handling
- Use builder pattern for transaction construction
- Implement transaction validation
- Support async signing
- Provide clear error messages

### 10.3 Security
- Use secure random generation
- Implement memory locking where possible
- Validate all inputs
- Use constant-time comparisons for sensitive data

### 10.4 API Design
- Provide ergonomic, type-safe APIs
- Support async/await
- Use traits for abstraction
- Provide clear documentation

### 10.5 Testing
- Provide test utilities
- Use property-based testing
- Test edge cases
- Mock external dependencies
