// Error types for Canton Wallet SDK

use std::fmt;
use thiserror::Error;

/// Result type alias for wallet operations
pub type WalletResult<T> = Result<T, WalletError>;

/// Comprehensive error type for wallet operations
#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum WalletError {
    /// Invalid wallet ID format
    #[error("Invalid wallet ID: {0}")]
    InvalidWalletId(String),

    /// Invalid party ID format
    #[error("Invalid party ID: {0}")]
    InvalidPartyId(String),

    /// Invalid contract ID format
    #[error("Invalid contract ID: {0}")]
    InvalidContractId(String),

    /// Invalid transaction ID format
    #[error("Invalid transaction ID: {0}")]
    InvalidTransactionId(String),

    /// Invalid key ID format
    #[error("Invalid key ID: {0}")]
    InvalidKeyId(String),

    /// Wallet not found
    #[error("Wallet not found: {0}")]
    WalletNotFound(String),

    /// Party not found
    #[error("Party not found: {0}")]
    PartyNotFound(String),

    /// Contract not found
    #[error("Contract not found: {0}")]
    ContractNotFound(String),

    /// Key not found
    #[error("Key not found: {0}")]
    KeyNotFound(String),

    /// Transaction failed
    #[error("Transaction failed: {0}")]
    TransactionFailed(String),

    /// Contract creation failed
    #[error("Contract creation failed: {0}")]
    ContractCreationFailed(String),

    /// Choice exercise failed
    #[error("Choice exercise failed: {0}")]
    ChoiceExerciseFailed(String),

    /// Invalid command
    #[error("Invalid command: {0}")]
    InvalidCommand(String),

    /// Invalid argument
    #[error("Invalid argument: {0}")]
    InvalidArgument(String),

    /// Invalid signature
    #[error("Invalid signature")]
    InvalidSignature,

    /// Signature verification failed
    #[error("Signature verification failed")]
    SignatureVerificationFailed,

    /// Key generation failed
    #[error("Key generation failed: {0}")]
    KeyGenerationFailed(String),

    /// Key import failed
    #[error("Key import failed: {0}")]
    KeyImportFailed(String),

    /// Key export failed
    #[error("Key export failed: {0}")]
    KeyExportFailed(String),

    /// Key rotation failed
    #[error("Key rotation failed: {0}")]
    KeyRotationFailed(String),

    /// Encryption failed
    #[error("Encryption failed: {0}")]
    EncryptionFailed(String),

    /// Decryption failed
    #[error("Decryption failed: {0}")]
    DecryptionFailed(String),

    /// Invalid mnemonic phrase
    #[error("Invalid mnemonic phrase: {0}")]
    InvalidMnemonic(String),

    /// Invalid derivation path
    #[error("Invalid derivation path: {0}")]
    InvalidDerivationPath(String),

    /// Derivation error
    #[error("Derivation error: {0}")]
    DerivationError(String),

    /// Network error
    #[error("Network error: {0}")]
    NetworkError(String),

    /// Connection error
    #[error("Connection error: {0}")]
    ConnectionError(String),

    /// Timeout error
    #[error("Timeout error: {0}")]
    TimeoutError(String),

    /// Ledger error
    #[error("Ledger error: {0}")]
    LedgerError(String),

    /// Serialization error
    #[error("Serialization error: {0}")]
    SerializationError(String),

    /// Deserialization error
    #[error("Deserialization error: {0}")]
    DeserializationError(String),

    /// Validation error
    #[error("Validation error: {0}")]
    ValidationError(String),

    /// Configuration error
    #[error("Configuration error: {0}")]
    ConfigurationError(String),

    /// Storage error
    #[error("Storage error: {0}")]
    StorageError(String),

    /// I/O error
    #[error("I/O error: {0}")]
    IoError(String),

    /// User rejected operation
    #[error("User rejected operation")]
    UserRejected,

    /// Operation not supported
    #[error("Operation not supported: {0}")]
    NotSupported(String),

    /// Insufficient balance
    #[error("Insufficient balance: required {required}, available {available}")]
    InsufficientBalance {
        required: String,
        available: String,
    },

    /// Rate limit exceeded
    #[error("Rate limit exceeded: {0}")]
    RateLimitExceeded(String),

    /// Authentication error
    #[error("Authentication error: {0}")]
    AuthenticationError(String),

    /// Authorization error
    #[error("Authorization error: {0}")]
    AuthorizationError(String),

    /// Permission denied
    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    /// Internal error
    #[error("Internal error: {0}")]
    InternalError(String),

    /// Custom error with message
    #[error("{0}")]
    Custom(String),
}

impl WalletError {
    /// Create a new custom error
    pub fn custom(message: impl Into<String>) -> Self {
        Self::Custom(message.into())
    }

    /// Check if this is a retryable error
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            Self::NetworkError(_)
                | Self::ConnectionError(_)
                | Self::TimeoutError(_)
                | Self::RateLimitExceeded(_)
        )
    }

    /// Check if this is a user-facing error
    pub fn is_user_facing(&self) -> bool {
        matches!(
            self,
            Self::UserRejected
                | Self::InsufficientBalance { .. }
                | Self::ValidationError(_)
                | Self::ConfigurationError(_)
        )
    }
}

impl From<serde_json::Error> for WalletError {
    fn from(err: serde_json::Error) -> Self {
        Self::SerializationError(err.to_string())
    }
}

impl From<validator::ValidationErrors> for WalletError {
    fn from(err: validator::ValidationErrors) -> Self {
        Self::ValidationError(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = WalletError::WalletNotFound("test-wallet".to_string());
        assert_eq!(err.to_string(), "Wallet not found: test-wallet");
    }

    #[test]
    fn test_error_custom() {
        let err = WalletError::custom("custom error message");
        assert_eq!(err.to_string(), "custom error message");
    }

    #[test]
    fn test_error_retryable() {
        assert!(WalletError::NetworkError("test".to_string()).is_retryable());
        assert!(WalletError::ConnectionError("test".to_string()).is_retryable());
        assert!(WalletError::TimeoutError("test".to_string()).is_retryable());
        assert!(!WalletError::InvalidWalletId("test".to_string()).is_retryable());
    }

    #[test]
    fn test_error_user_facing() {
        assert!(WalletError::UserRejected.is_user_facing());
        assert!(WalletError::InsufficientBalance {
            required: "100".to_string(),
            available: "50".to_string()
        }
        .is_user_facing());
        assert!(!WalletError::InternalError("test".to_string()).is_user_facing());
    }

    #[test]
    fn test_insufficient_balance() {
        let err = WalletError::InsufficientBalance {
            required: "100".to_string(),
            available: "50".to_string(),
        };
        assert!(err.to_string().contains("required 100"));
        assert!(err.to_string().contains("available 50"));
    }
}
