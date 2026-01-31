// Copyright 2025 Canton Wallet SDK Contributors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Security error types.

use thiserror::Error;

/// Result type alias for security operations.
pub type Result<T> = std::result::Result<T, SecurityError>;

/// Errors that can occur during security operations.
#[derive(Error, Debug)]
pub enum SecurityError {
    /// Transaction approval failed.
    #[error("Transaction approval failed: {0}")]
    ApprovalFailed(String),

    /// User rejected the transaction.
    #[error("User rejected transaction")]
    UserRejected,

    /// Transaction validation failed.
    #[error("Transaction validation failed: {0}")]
    ValidationFailed(String),

    /// Rate limit exceeded.
    #[error("Rate limit exceeded: {0} requests per {1:?}")]
    RateLimitExceeded(u32, std::time::Duration),

    /// Input validation failed.
    #[error("Input validation failed: {0}")]
    InputValidationFailed(String),

    /// Audit log error.
    #[error("Audit log error: {0}")]
    AuditLogError(String),

    /// Unauthorized operation.
    #[error("Unauthorized operation: {0}")]
    Unauthorized(String),

    /// Permission denied.
    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    /// Invalid signature.
    #[error("Invalid signature")]
    InvalidSignature,

    /// Signature verification failed.
    #[error("Signature verification failed: {0}")]
    SignatureVerificationFailed(String),

    /// Core wallet error.
    #[error("Core wallet error: {0}")]
    CoreError(#[from] canton_wallet_core::error::WalletError),

    /// Transaction error.
    #[error("Transaction error: {0}")]
    TransactionError(#[from] canton_wallet_transactions::error::TransactionError),

    /// IO error.
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    /// Serialization error.
    #[error("Serialization error: {0}")]
    SerializationError(String),

    /// Deserialization error.
    #[error("Deserialization error: {0}")]
    DeserializationError(String),

    /// Other error.
    #[error("Error: {0}")]
    Other(String),
}

impl From<serde_json::Error> for SecurityError {
    fn from(err: serde_json::Error) -> Self {
        SecurityError::SerializationError(err.to_string())
    }
}

impl From<bincode::Error> for SecurityError {
    fn from(err: bincode::Error) -> Self {
        SecurityError::DeserializationError(err.to_string())
    }
}
