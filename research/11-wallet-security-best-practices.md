# Wallet Security Best Practices

## 1. Overview

This document covers comprehensive security best practices for wallet SDKs, focusing on production-ready implementations that protect user assets and private keys.

## 2. Key Security Principles

### 2.1 Defense in Depth

```rust
/// Multi-layered security approach
pub struct SecurityLayers {
    /// Application-level security
    app_layer: AppSecurity,
    /// Transport-level security
    transport_layer: TransportSecurity,
    /// Storage-level security
    storage_layer: StorageSecurity,
    /// Hardware-level security (HSM/TEE)
    hardware_layer: Option<HardwareSecurity>,
}

impl SecurityLayers {
    pub fn new() -> Self {
        Self {
            app_layer: AppSecurity::new(),
            transport_layer: TransportSecurity::new(),
            storage_layer: StorageSecurity::new(),
            hardware_layer: None,
        }
    }
    
    pub fn with_hsm(mut self, hsm: HardwareSecurity) -> Self {
        self.hardware_layer = Some(hsm);
        self
    }
    
    /// Validate through all security layers
    pub async fn validate_operation(
        &self,
        operation: &SensitiveOperation,
    ) -> Result<(), SecurityError> {
        self.app_layer.validate(operation)?;
        self.transport_layer.validate(operation)?;
        self.storage_layer.validate(operation)?;
        
        if let Some(hardware) = &self.hardware_layer {
            hardware.validate(operation).await?;
        }
        
        Ok(())
    }
}
```

### 2.2 Zero Trust Architecture

```rust
/// Zero trust validation
pub struct ZeroTrustValidator {
    /// Require authentication for all operations
    require_auth: bool,
    /// Validate all inputs
    validate_inputs: bool,
    /// Audit all operations
    audit_operations: bool,
    /// Rate limiting
    rate_limiter: RateLimiter,
}

impl ZeroTrustValidator {
    pub fn new() -> Self {
        Self {
            require_auth: true,
            validate_inputs: true,
            audit_operations: true,
            rate_limiter: RateLimiter::new(RateLimitConfig::default()),
        }
    }
    
    pub async fn validate_request<T>(
        &self,
        request: &WalletRequest<T>,
    ) -> Result<(), SecurityError> {
        // Check rate limit
        self.rate_limiter.check_limit(&request.client_id).await?;
        
        // Validate authentication
        if self.require_auth {
            self.validate_auth(&request.auth).await?;
        }
        
        // Validate inputs
        if self.validate_inputs {
            self.validate_inputs_internal(&request.payload)?;
        }
        
        // Audit operation
        if self.audit_operations {
            self.audit_operation(request).await;
        }
        
        Ok(())
    }
    
    async fn validate_auth(&self, auth: &AuthContext) -> Result<(), SecurityError> {
        if !auth.is_valid() {
            return Err(SecurityError::Unauthorized);
        }
        
        // Check token expiration
        if auth.is_expired() {
            return Err(SecurityError::TokenExpired);
        }
        
        // Check permissions
        if !auth.has_permission(&auth.required_permission) {
            return Err(SecurityError::InsufficientPermissions);
        }
        
        Ok(())
    }
    
    fn validate_inputs_internal<T>(&self, payload: &T) -> Result<(), SecurityError> {
        // Validate payload size
        let payload_size = std::mem::size_of_val(payload);
        if payload_size > MAX_PAYLOAD_SIZE {
            return Err(SecurityError::PayloadTooLarge);
        }
        
        // Validate payload structure
        // ... implementation depends on T
        
        Ok(())
    }
    
    async fn audit_operation<T>(&self, request: &WalletRequest<T>) {
        // Log operation to audit trail
        tracing::info!(
            operation = %request.operation_type,
            client_id = %request.client_id,
            timestamp = %Utc::now(),
            "Wallet operation audited"
        );
    }
}
```

## 3. Secure Key Management

### 3.1 Key Derivation

```rust
use hkdf::Hkdf;
use sha2::Sha256;

/// Secure key derivation
pub struct KeyDerivation {
    salt: Vec<u8>,
    info: Vec<u8>,
}

impl KeyDerivation {
    pub fn new(salt: &[u8], info: &[u8]) -> Self {
        Self {
            salt: salt.to_vec(),
            info: info.to_vec(),
        }
    }
    
    /// Derive key using HKDF-SHA256
    pub fn derive_key(
        &self,
        ikm: &[u8], // Input key material
        length: usize,
    ) -> Result<Vec<u8>, CryptoError> {
        let hk = Hkdf::<Sha256>::new(Some(&self.salt), ikm);
        let mut okm = vec![0u8; length];
        
        hk.expand(&self.info, &mut okm)
            .map_err(|e| CryptoError::DerivationError(e.to_string()))?;
        
        Ok(okm)
    }
    
    /// Derive wallet key from mnemonic
    pub fn derive_wallet_key(
        &self,
        mnemonic: &str,
        password: Option<&str>,
    ) -> Result<[u8; 32], CryptoError> {
        use pbkdf2::{
            password_hash::{
                PasswordHasher, SaltString
            },
            Pbkdf2,
            Params,
        };
        
        let password_bytes = password.unwrap_or("").as_bytes();
        let salt = SaltString::generate(&mut OsRng);
        
        let hash = Pbkdf2
            .hash_password_customized(
                mnemonic.as_bytes(),
                None,
                None,
                Params {
                    rounds: 100_000,
                    output_length: 32,
                },
                &salt,
            )
            .map_err(|e| CryptoError::DerivationError(e.to_string()))?;
        
        let hash_bytes = hash.hash.unwrap().as_bytes();
        let mut key = [0u8; 32];
        key.copy_from_slice(&hash_bytes[..32]);
        
        Ok(key)
    }
}
```

### 3.2 Secure Key Storage

```rust
use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};

/// Encrypted key storage
pub struct EncryptedKeyStorage {
    encryption_key: [u8; 32],
    storage: Box<dyn StorageBackend>,
}

impl EncryptedKeyStorage {
    pub fn new(encryption_key: [u8; 32], storage: Box<dyn StorageBackend>) -> Self {
        Self {
            encryption_key,
            storage,
        }
    }
    
    /// Store key encrypted
    pub async fn store_key(
        &self,
        key_id: &KeyId,
        key: &SecurePrivateKey,
    ) -> Result<(), StorageError> {
        let key_bytes = key.as_bytes();
        
        // Encrypt key
        let cipher = Aes256Gcm::new(&self.encryption_key.into());
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
        
        let ciphertext = cipher
            .encrypt(&nonce, key_bytes)
            .map_err(|e| StorageError::EncryptionError(e.to_string()))?;
        
        // Store encrypted key
        let encrypted_key = EncryptedKey {
            nonce: nonce.to_vec(),
            ciphertext,
            algorithm: key.algorithm(),
        };
        
        self.storage.store(key_id, &encrypted_key).await?;
        
        Ok(())
    }
    
    /// Load and decrypt key
    pub async fn load_key(
        &self,
        key_id: &KeyId,
    ) -> Result<SecurePrivateKey, StorageError> {
        // Load encrypted key
        let encrypted_key = self.storage.load(key_id).await?;
        
        // Decrypt key
        let cipher = Aes256Gcm::new(&self.encryption_key.into());
        let nonce = Nonce::from_slice(&encrypted_key.nonce);
        
        let plaintext = cipher
            .decrypt(nonce, encrypted_key.ciphertext.as_ref())
            .map_err(|e| StorageError::DecryptionError(e.to_string()))?;
        
        Ok(SecurePrivateKey::new(encrypted_key.algorithm, plaintext))
    }
    
    /// Delete key
    pub async fn delete_key(&self, key_id: &KeyId) -> Result<(), StorageError> {
        self.storage.delete(key_id).await
    }
}

/// Encrypted key representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedKey {
    pub nonce: Vec<u8>,
    pub ciphertext: Vec<u8>,
    pub algorithm: KeyAlgorithm,
}
```

### 3.3 Hardware Security Module Integration

```rust
/// HSM key store
pub struct HsmKeyStore {
    hsm_client: Box<dyn HsmClient>,
    key_cache: Arc<RwLock<HashMap<KeyId, HsmKeyHandle>>>,
}

impl HsmKeyStore {
    pub fn new(hsm_client: Box<dyn HsmClient>) -> Self {
        Self {
            hsm_client,
            key_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Generate key in HSM
    pub async fn generate_key(
        &self,
        algorithm: KeyAlgorithm,
    ) -> Result<KeyId, HsmError> {
        let key_id = KeyId::new();
        
        let handle = self.hsm_client
            .generate_key(algorithm, &key_id)
            .await?;
        
        self.key_cache.write().await.insert(key_id.clone(), handle);
        
        Ok(key_id)
    }
    
    /// Sign with HSM key
    pub async fn sign(
        &self,
        key_id: &KeyId,
        data: &[u8],
    ) -> Result<Signature, HsmError> {
        let handle = self.get_key_handle(key_id).await?;
        
        self.hsm_client.sign(&handle, data).await
    }
    
    async fn get_key_handle(&self, key_id: &KeyId) -> Result<HsmKeyHandle, HsmError> {
        let cache = self.key_cache.read().await;
        
        cache.get(key_id)
            .cloned()
            .ok_or(HsmError::KeyNotFound(key_id.clone()))
    }
}

/// HSM client trait
#[async_trait]
pub trait HsmClient: Send + Sync {
    async fn generate_key(
        &self,
        algorithm: KeyAlgorithm,
        key_id: &KeyId,
    ) -> Result<HsmKeyHandle, HsmError>;
    
    async fn sign(
        &self,
        handle: &HsmKeyHandle,
        data: &[u8],
    ) -> Result<Signature, HsmError>;
    
    async fn delete_key(
        &self,
        handle: &HsmKeyHandle,
    ) -> Result<(), HsmError>;
}
```

## 4. Secure Transaction Signing

### 4.1 Transaction Validation

```rust
/// Transaction validator
pub struct SecureTransactionValidator {
    max_fee: u64,
    max_gas_limit: u64,
    min_gas_price: u64,
    allowed_contracts: HashSet<ContractId>,
    blacklist: HashSet<Address>,
}

impl SecureTransactionValidator {
    pub fn new() -> Self {
        Self {
            max_fee: 1_000_000_000, // 1 ETH equivalent
            max_gas_limit: 10_000_000,
            min_gas_price: 1_000_000_000, // 1 Gwei
            allowed_contracts: HashSet::new(),
            blacklist: HashSet::new(),
        }
    }
    
    pub fn with_allowed_contracts(mut self, contracts: Vec<ContractId>) -> Self {
        self.allowed_contracts = contracts.into_iter().collect();
        self
    }
    
    pub fn with_blacklist(mut self, addresses: Vec<Address>) -> Self {
        self.blacklist = addresses.into_iter().collect();
        self
    }
    
    pub fn validate(&self, tx: &Transaction) -> Result<(), SecurityError> {
        // Validate fee
        if tx.fee > self.max_fee {
            return Err(SecurityError::FeeTooHigh {
                actual: tx.fee,
                max: self.max_fee,
            });
        }
        
        // Validate gas limit
        if tx.gas_limit > self.max_gas_limit {
            return Err(SecurityError::GasLimitTooHigh {
                actual: tx.gas_limit,
                max: self.max_gas_limit,
            });
        }
        
        // Validate gas price
        if tx.gas_price < self.min_gas_price {
            return Err(SecurityError::GasPriceTooLow {
                actual: tx.gas_price,
                min: self.min_gas_price,
            });
        }
        
        // Check blacklist
        if self.blacklist.contains(&tx.to) {
            return Err(SecurityError::BlacklistedAddress(tx.to));
        }
        
        // Validate contract calls
        if let Some(contract_id) = tx.contract_id {
            if !self.allowed_contracts.is_empty() 
                && !self.allowed_contracts.contains(&contract_id) {
                return Err(SecurityError::UnauthorizedContract(contract_id));
            }
        }
        
        // Validate nonce
        if tx.nonce == 0 {
            return Err(SecurityError::InvalidNonce(tx.nonce));
        }
        
        // Validate signature
        if !self.verify_signature(tx)? {
            return Err(SecurityError::InvalidSignature);
        }
        
        Ok(())
    }
    
    fn verify_signature(&self, tx: &Transaction) -> Result<bool, SecurityError> {
        // Implementation depends on chain
        Ok(true)
    }
}
```

### 4.2 Secure Signing Flow

```rust
/// Secure signing flow
pub struct SecureSigningFlow {
    validator: SecureTransactionValidator,
    key_store: Arc<dyn KeyStore>,
    user_approval: Arc<dyn UserApproval>,
}

impl SecureSigningFlow {
    pub fn new(
        validator: SecureTransactionValidator,
        key_store: Arc<dyn KeyStore>,
        user_approval: Arc<dyn UserApproval>,
    ) -> Self {
        Self {
            validator,
            key_store,
            user_approval,
        }
    }
    
    /// Sign transaction with security checks
    pub async fn sign_transaction(
        &self,
        key_id: &KeyId,
        tx: &Transaction,
    ) -> Result<SignedTransaction, SecurityError> {
        // Step 1: Validate transaction
        self.validator.validate(tx)?;
        
        // Step 2: Get user approval
        let approval = self.user_approval
            .request_approval(tx)
            .await?;
        
        if !approval.approved {
            return Err(SecurityError::UserRejected);
        }
        
        // Step 3: Sign transaction
        let signature = self.key_store
            .sign(key_id, &tx.serialize())
            .await
            .map_err(|e| SecurityError::SigningError(e.to_string()))?;
        
        // Step 4: Verify signature
        if !self.verify_signature(tx, &signature)? {
            return Err(SecurityError::SignatureVerificationFailed);
        }
        
        Ok(SignedTransaction {
            transaction: tx.clone(),
            signature,
        })
    }
    
    fn verify_signature(
        &self,
        tx: &Transaction,
        signature: &Signature,
    ) -> Result<bool, SecurityError> {
        // Implementation depends on chain
        Ok(true)
    }
}

/// User approval trait
#[async_trait]
pub trait UserApproval: Send + Sync {
    async fn request_approval(
        &self,
        tx: &Transaction,
    ) -> Result<ApprovalResponse, SecurityError>;
}

/// Approval response
#[derive(Debug, Clone)]
pub struct ApprovalResponse {
    pub approved: bool,
    pub timestamp: DateTime<Utc>,
}
```

## 5. Secure Communication

### 5.1 TLS Configuration

```rust
use rustls::{
    ClientConfig, ServerConfig,
    pki_types::{CertificateDer, PrivateKeyDer, ServerName},
};

/// Secure TLS configuration
pub struct SecureTlsConfig {
    min_version: rustls::Version,
    cipher_suites: Vec<rustls::SupportedCipherSuite>,
    certificates: Vec<CertificateDer<'static>>,
    private_key: Option<PrivateKeyDer<'static>>,
}

impl SecureTlsConfig {
    pub fn new() -> Self {
        Self {
            min_version: rustls::Version::TLSv1_3,
            cipher_suites: rustls::ALL_CIPHER_SUITES.to_vec(),
            certificates: Vec::new(),
            private_key: None,
        }
    }
    
    pub fn with_certificates(
        mut self,
        certs: Vec<CertificateDer<'static>>,
        key: PrivateKeyDer<'static>,
    ) -> Self {
        self.certificates = certs;
        self.private_key = Some(key);
        self
    }
    
    pub fn build_client_config(&self) -> Result<ClientConfig, TlsError> {
        let config = ClientConfig::builder()
            .with_protocol_versions(&[&rustls::version::TLS13])
            .map_err(|e| TlsError::ConfigError(e.to_string()))?
            .with_root_certificates(rustls::RootCertStore::empty())
            .with_no_client_auth();
        
        Ok(config)
    }
    
    pub fn build_server_config(&self) -> Result<ServerConfig, TlsError> {
        let private_key = self.private_key
            .as_ref()
            .ok_or(TlsError::MissingPrivateKey)?;
        
        let config = ServerConfig::builder()
            .with_protocol_versions(&[&rustls::version::TLS13])
            .map_err(|e| TlsError::ConfigError(e.to_string()))?
            .with_no_client_auth()
            .with_single_cert(self.certificates.clone(), private_key.clone())
            .map_err(|e| TlsError::ConfigError(e.to_string()))?;
        
        Ok(config)
    }
}
```

### 5.2 Message Authentication

```rust
use hmac::{Hmac, Mac};
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

/// Message authentication
pub struct MessageAuth {
    key: [u8; 32],
}

impl MessageAuth {
    pub fn new(key: [u8; 32]) -> Self {
        Self { key }
    }
    
    /// Generate HMAC
    pub fn authenticate(&self, message: &[u8]) -> Vec<u8> {
        let mut mac = HmacSha256::new_from_slice(&self.key)
            .expect("HMAC can take key of any size");
        
        mac.update(message);
        mac.finalize().into_bytes().to_vec()
    }
    
    /// Verify HMAC
    pub fn verify(&self, message: &[u8], mac: &[u8]) -> bool {
        let mut hmac = HmacSha256::new_from_slice(&self.key)
            .expect("HMAC can take key of any size");
        
        hmac.update(message);
        
        hmac.verify_slice(mac).is_ok()
    }
}
```

## 6. Secure Storage

### 6.1 Encrypted Database

```rust
use sqlx::{Pool, Postgres, postgres::PgPoolOptions};

/// Encrypted database storage
pub struct EncryptedDatabase {
    pool: Pool<Postgres>,
    encryption_key: [u8; 32],
}

impl EncryptedDatabase {
    pub async fn new(
        database_url: &str,
        encryption_key: [u8; 32],
    ) -> Result<Self, StorageError> {
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(database_url)
            .await
            .map_err(|e| StorageError::ConnectionError(e.to_string()))?;
        
        Ok(Self {
            pool,
            encryption_key,
        })
    }
    
    /// Store encrypted data
    pub async fn store(
        &self,
        key: &str,
        value: &[u8],
    ) -> Result<(), StorageError> {
        let encrypted = self.encrypt(value)?;
        
        sqlx::query(
            "INSERT INTO encrypted_data (key, value, nonce) VALUES ($1, $2, $3)
             ON CONFLICT (key) DO UPDATE SET value = $2, nonce = $3"
        )
        .bind(key)
        .bind(&encrypted.ciphertext)
        .bind(&encrypted.nonce)
        .execute(&self.pool)
        .await
        .map_err(|e| StorageError::DatabaseError(e.to_string()))?;
        
        Ok(())
    }
    
    /// Load and decrypt data
    pub async fn load(&self, key: &str) -> Result<Vec<u8>, StorageError> {
        let row = sqlx::query_as::<_, (Vec<u8>, Vec<u8>)>(
            "SELECT value, nonce FROM encrypted_data WHERE key = $1"
        )
        .bind(key)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| StorageError::DatabaseError(e.to_string()))?
        .ok_or(StorageError::KeyNotFound(key.to_string()))?;
        
        let (ciphertext, nonce) = row;
        self.decrypt(&ciphertext, &nonce)
    }
    
    fn encrypt(&self, plaintext: &[u8]) -> Result<EncryptedData, StorageError> {
        let cipher = Aes256Gcm::new(&self.encryption_key.into());
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
        
        let ciphertext = cipher
            .encrypt(&nonce, plaintext)
            .map_err(|e| StorageError::EncryptionError(e.to_string()))?;
        
        Ok(EncryptedData {
            ciphertext,
            nonce: nonce.to_vec(),
        })
    }
    
    fn decrypt(&self, ciphertext: &[u8], nonce: &[u8]) -> Result<Vec<u8>, StorageError> {
        let cipher = Aes256Gcm::new(&self.encryption_key.into());
        let nonce = Nonce::from_slice(nonce);
        
        cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| StorageError::DecryptionError(e.to_string()))
    }
}

struct EncryptedData {
    ciphertext: Vec<u8>,
    nonce: Vec<u8>,
}
```

## 7. Audit and Logging

### 7.1 Secure Audit Log

```rust
/// Audit log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogEntry {
    pub timestamp: DateTime<Utc>,
    pub operation: String,
    pub user_id: Option<String>,
    pub wallet_id: Option<String>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub success: bool,
    pub error: Option<String>,
    pub metadata: HashMap<String, String>,
}

/// Audit logger
pub struct AuditLogger {
    storage: Box<dyn AuditStorage>,
}

impl AuditLogger {
    pub fn new(storage: Box<dyn AuditStorage>) -> Self {
        Self { storage }
    }
    
    pub async fn log_operation(
        &self,
        entry: AuditLogEntry,
    ) -> Result<(), AuditError> {
        self.storage.store(entry).await
    }
    
    pub async fn query_logs(
        &self,
        filter: AuditFilter,
    ) -> Result<Vec<AuditLogEntry>, AuditError> {
        self.storage.query(filter).await
    }
}

/// Audit storage trait
#[async_trait]
pub trait AuditStorage: Send + Sync {
    async fn store(&self, entry: AuditLogEntry) -> Result<(), AuditError>;
    async fn query(&self, filter: AuditFilter) -> Result<Vec<AuditLogEntry>, AuditError>;
}

/// Audit filter
#[derive(Debug, Clone)]
pub struct AuditFilter {
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
    pub user_id: Option<String>,
    pub wallet_id: Option<String>,
    pub operation: Option<String>,
    pub success: Option<bool>,
}
```

## 8. Error Handling

```rust
use thiserror::Error;

/// Security error types
#[derive(Error, Debug)]
pub enum SecurityError {
    #[error("Unauthorized access")]
    Unauthorized,
    
    #[error("Token expired")]
    TokenExpired,
    
    #[error("Insufficient permissions")]
    InsufficientPermissions,
    
    #[error("Payload too large: {size} > {max}")]
    PayloadTooLarge { size: usize, max: usize },
    
    #[error("Fee too high: {actual} > {max}")]
    FeeTooHigh { actual: u64, max: u64 },
    
    #[error("Gas limit too high: {actual} > {max}")]
    GasLimitTooHigh { actual: u64, max: u64 },
    
    #[error("Gas price too low: {actual} < {min}")]
    GasPriceTooLow { actual: u64, min: u64 },
    
    #[error("Blacklisted address: {0}")]
    BlacklistedAddress(Address),
    
    #[error("Unauthorized contract: {0}")]
    UnauthorizedContract(ContractId),
    
    #[error("Invalid nonce: {0}")]
    InvalidNonce(u64),
    
    #[error("Invalid signature")]
    InvalidSignature,
    
    #[error("Signature verification failed")]
    SignatureVerificationFailed,
    
    #[error("User rejected operation")]
    UserRejected,
    
    #[error("Signing error: {0}")]
    SigningError(String),
    
    #[error("Rate limit exceeded")]
    RateLimitExceeded,
    
    #[error("Suspicious activity detected")]
    SuspiciousActivity,
    
    #[error("Key not found: {0}")]
    KeyNotFound(String),
    
    #[error("Invalid key format")]
    InvalidKeyFormat,
    
    #[error("Encryption error: {0}")]
    EncryptionError(String),
    
    #[error("Decryption error: {0}")]
    DecryptionError(String),
    
    #[error("Configuration error: {0}")]
    ConfigError(String),
    
    #[error("Internal error: {0}")]
    Internal(String),
}
```

## 9. Security Testing

### 9.1 Fuzz Testing

```rust
/// Fuzz test for transaction validation
#[cfg(test)]
mod fuzz_tests {
    use super::*;
    
    #[test]
    fn fuzz_transaction_validation() {
        let validator = SecureTransactionValidator::new();
        
        // Fuzz with random transactions
        for _ in 0..1000 {
            let tx = generate_random_transaction();
            
            let result = validator.validate(&tx);
            
            // Should either succeed or fail with a security error
            match result {
                Ok(()) => {}
                Err(e) => {
                    // Verify it's a security error
                    assert!(matches!(e, SecurityError::FeeTooHigh { .. }
                        | SecurityError::GasLimitTooHigh { .. }
                        | SecurityError::GasPriceTooLow { .. }
                        | SecurityError::InvalidNonce(_)
                        | SecurityError::InvalidSignature));
                }
            }
        }
    }
    
    fn generate_random_transaction() -> Transaction {
        // Generate random transaction for fuzzing
        Transaction {
            from: Address::random(),
            to: Address::random(),
            value: rand::random::<u64>(),
            fee: rand::random::<u64>(),
            gas_limit: rand::random::<u64>(),
            gas_price: rand::random::<u64>(),
            nonce: rand::random::<u64>(),
            data: vec![0u8; rand::random::<usize>() % 1000],
            signature: None,
        }
    }
}
```

### 9.2 Security Audits

```rust
/// Security audit checklist
pub struct SecurityAuditChecklist {
    items: Vec<AuditItem>,
}

impl SecurityAuditChecklist {
    pub fn new() -> Self {
        Self {
            items: vec![
                AuditItem {
                    id: "SEC-001".to_string(),
                    category: "Key Management".to_string(),
                    description: "Private keys are never logged or exposed".to_string(),
                    status: AuditStatus::Pending,
                },
                AuditItem {
                    id: "SEC-002".to_string(),
                    category: "Key Management".to_string(),
                    description: "Keys are encrypted at rest".to_string(),
                    status: AuditStatus::Pending,
                },
                AuditItem {
                    id: "SEC-003".to_string(),
                    category: "Transaction Security".to_string(),
                    description: "All transactions are validated before signing".to_string(),
                    status: AuditStatus::Pending,
                },
                AuditItem {
                    id: "SEC-004".to_string(),
                    category: "Transaction Security".to_string(),
                    description: "User approval required for all transactions".to_string(),
                    status: AuditStatus::Pending,
                },
                AuditItem {
                    id: "SEC-005".to_string(),
                    category: "Communication Security".to_string(),
                    description: "TLS 1.3 or higher for all network communication".to_string(),
                    status: AuditStatus::Pending,
                },
                AuditItem {
                    id: "SEC-006".to_string(),
                    category: "Communication Security".to_string(),
                    description: "Message authentication for all API calls".to_string(),
                    status: AuditStatus::Pending,
                },
                AuditItem {
                    id: "SEC-007".to_string(),
                    category: "Storage Security".to_string(),
                    description: "Sensitive data encrypted at rest".to_string(),
                    status: AuditStatus::Pending,
                },
                AuditItem {
                    id: "SEC-008".to_string(),
                    category: "Storage Security".to_string(),
                    description: "Secure key derivation (HKDF, PBKDF2)".to_string(),
                    status: AuditStatus::Pending,
                },
                AuditItem {
                    id: "SEC-009".to_string(),
                    category: "Audit and Logging".to_string(),
                    description: "All security events logged".to_string(),
                    status: AuditStatus::Pending,
                },
                AuditItem {
                    id: "SEC-010".to_string(),
                    category: "Audit and Logging".to_string(),
                    description: "Audit logs are tamper-evident".to_string(),
                    status: AuditStatus::Pending,
                },
            ],
        }
    }
    
    pub fn mark_passed(&mut self, id: &str) {
        if let Some(item) = self.items.iter_mut().find(|i| i.id == id) {
            item.status = AuditStatus::Passed;
        }
    }
    
    pub fn mark_failed(&mut self, id: &str, reason: String) {
        if let Some(item) = self.items.iter_mut().find(|i| i.id == id) {
            item.status = AuditStatus::Failed(reason);
        }
    }
    
    pub fn generate_report(&self) -> AuditReport {
        let passed = self.items.iter()
            .filter(|i| matches!(i.status, AuditStatus::Passed))
            .count();
        
        let failed = self.items.iter()
            .filter(|i| matches!(i.status, AuditStatus::Failed(_)))
            .count();
        
        let pending = self.items.iter()
            .filter(|i| matches!(i.status, AuditStatus::Pending))
            .count();
        
        AuditReport {
            total: self.items.len(),
            passed,
            failed,
            pending,
            items: self.items.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct AuditItem {
    pub id: String,
    pub category: String,
    pub description: String,
    pub status: AuditStatus,
}

#[derive(Debug, Clone)]
pub enum AuditStatus {
    Pending,
    Passed,
    Failed(String),
}

#[derive(Debug, Clone)]
pub struct AuditReport {
    pub total: usize,
    pub passed: usize,
    pub failed: usize,
    pub pending: usize,
    pub items: Vec<AuditItem>,
}
```

## 10. Summary

### 10.1 Key Security Practices
- Use secure key derivation (HKDF, PBKDF2)
- Encrypt keys at rest
- Support HSM integration
- Never log or expose private keys
- Use zeroization for sensitive data

### 10.2 Transaction Security
- Validate all transactions before signing
- Require user approval
- Implement rate limiting
- Use transaction whitelists/blacklists
- Verify signatures

### 10.3 Communication Security
- Use TLS 1.3 or higher
- Implement message authentication
- Validate all inputs
- Use secure random generation

### 10.4 Storage Security
- Encrypt sensitive data at rest
- Use secure key storage
- Implement audit logging
- Use tamper-evident storage

### 10.5 Testing and Auditing
- Implement fuzz testing
- Conduct security audits
- Use penetration testing
- Monitor for suspicious activity
