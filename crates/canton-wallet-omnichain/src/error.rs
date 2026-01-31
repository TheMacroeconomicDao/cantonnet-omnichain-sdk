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

use std::fmt;

/// Result type for OmniChain operations
pub type OmniChainResult<T> = Result<T, OmniChainError>;

/// Errors that can occur in OmniChain operations
#[derive(Debug, thiserror::Error)]
pub enum OmniChainError {
    /// Chain not supported
    #[error("Chain not supported: {0}")]
    UnsupportedChain(String),

    /// Chain adapter not found
    #[error("Chain adapter not found for chain: {0}")]
    AdapterNotFound(String),

    /// Bridge contract error
    #[error("Bridge contract error: {0}")]
    BridgeError(String),

    /// Lock operation failed
    #[error("Lock operation failed: {0}")]
    LockFailed(String),

    /// Release operation failed
    #[error("Release operation failed: {0}")]
    ReleaseFailed(String),

    /// Proof generation failed
    #[error("Proof generation failed: {0}")]
    ProofGenerationFailed(String),

    /// Proof verification failed
    #[error("Proof verification failed: {0}")]
    ProofVerificationFailed(String),

    /// Invalid asset
    #[error("Invalid asset: {0}")]
    InvalidAsset(String),

    /// Invalid address
    #[error("Invalid address: {0}")]
    InvalidAddress(String),

    /// Insufficient balance
    #[error("Insufficient balance: required {required}, available {available}")]
    InsufficientBalance {
        required: String,
        available: String,
    },

    /// Transaction timeout
    #[error("Transaction timeout after {0} seconds")]
    TransactionTimeout(u64),

    /// Network error
    #[error("Network error: {0}")]
    NetworkError(String),

    /// Serialization error
    #[error("Serialization error: {0}")]
    SerializationError(String),

    /// Deserialization error
    #[error("Deserialization error: {0}")]
    DeserializationError(String),

    /// Invalid configuration
    #[error("Invalid configuration: {0}")]
    InvalidConfiguration(String),

    /// Internal error
    #[error("Internal error: {0}")]
    InternalError(String),
}

impl fmt::Display for OmniChainError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedChain(chain) => write!(f, "Chain not supported: {}", chain),
            Self::AdapterNotFound(chain) => write!(f, "Chain adapter not found for chain: {}", chain),
            Self::BridgeError(msg) => write!(f, "Bridge contract error: {}", msg),
            Self::LockFailed(msg) => write!(f, "Lock operation failed: {}", msg),
            Self::ReleaseFailed(msg) => write!(f, "Release operation failed: {}", msg),
            Self::ProofGenerationFailed(msg) => write!(f, "Proof generation failed: {}", msg),
            Self::ProofVerificationFailed(msg) => write!(f, "Proof verification failed: {}", msg),
            Self::InvalidAsset(msg) => write!(f, "Invalid asset: {}", msg),
            Self::InvalidAddress(msg) => write!(f, "Invalid address: {}", msg),
            Self::InsufficientBalance { required, available } => {
                write!(f, "Insufficient balance: required {}, available {}", required, available)
            }
            Self::TransactionTimeout(seconds) => {
                write!(f, "Transaction timeout after {} seconds", seconds)
            }
            Self::NetworkError(msg) => write!(f, "Network error: {}", msg),
            Self::SerializationError(msg) => write!(f, "Serialization error: {}", msg),
            Self::DeserializationError(msg) => write!(f, "Deserialization error: {}", msg),
            Self::InvalidConfiguration(msg) => write!(f, "Invalid configuration: {}", msg),
            Self::InternalError(msg) => write!(f, "Internal error: {}", msg),
        }
    }
}
