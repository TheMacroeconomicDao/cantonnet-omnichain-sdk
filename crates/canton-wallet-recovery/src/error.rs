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

//! Recovery error types.

use thiserror::Error;

/// Result type alias for recovery operations.
pub type Result<T> = std::result::Result<T, RecoveryError>;

/// Errors that can occur during recovery operations.
#[derive(Error, Debug)]
pub enum RecoveryError {
    /// Backup creation failed.
    #[error("Backup creation failed: {0}")]
    BackupFailed(String),

    /// Backup restoration failed.
    #[error("Backup restoration failed: {0}")]
    RestoreFailed(String),

    /// Invalid backup format.
    #[error("Invalid backup format: {0}")]
    InvalidBackupFormat(String),

    /// Backup not found.
    #[error("Backup not found: {0}")]
    BackupNotFound(String),

    /// Backup corrupted.
    #[error("Backup corrupted: {0}")]
    BackupCorrupted(String),

    /// Social recovery failed.
    #[error("Social recovery failed: {0}")]
    SocialRecoveryFailed(String),

    /// Insufficient recovery shares.
    #[error("Insufficient recovery shares: {0}/{1} required", required, provided)]
    InsufficientShares { required: usize, provided: usize },

    /// Invalid recovery share.
    #[error("Invalid recovery share: {0}")]
    InvalidShare(String),

    /// Key rotation failed.
    #[error("Key rotation failed: {0}")]
    KeyRotationFailed(String),

    /// Verification failed.
    #[error("Verification failed: {0}")]
    VerificationFailed(String),

    /// Core wallet error.
    #[error("Core wallet error: {0}")]
    CoreError(#[from] canton_wallet_core::error::WalletError),

    /// Crypto error.
    #[error("Crypto error: {0}")]
    CryptoError(String),

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

impl From<serde_json::Error> for RecoveryError {
    fn from(err: serde_json::Error) -> Self {
        RecoveryError::SerializationError(err.to_string())
    }
}

impl From<bincode::Error> for RecoveryError {
    fn from(err: bincode::Error) -> Self {
        RecoveryError::DeserializationError(err.to_string())
    }
}
