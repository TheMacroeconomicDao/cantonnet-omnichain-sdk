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

//! Ledger API error types

use std::fmt;
use thiserror::Error;

/// Result type alias for ledger operations
pub type LedgerResult<T> = Result<T, LedgerError>;

/// Errors that can occur during ledger API operations
#[derive(Error, Debug)]
pub enum LedgerError {
    /// Connection error
    #[error("Connection error: {0}")]
    ConnectionError(String),

    /// Authentication error
    #[error("Authentication error: {0}")]
    AuthenticationError(String),

    /// Authorization error
    #[error("Authorization error: {0}")]
    AuthorizationError(String),

    /// Invalid request
    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    /// Transaction error
    #[error("Transaction error: {0}")]
    TransactionError(String),

    /// Command submission error
    #[error("Command submission error: {0}")]
    CommandError(String),

    /// Transaction not found
    #[error("Transaction not found: {0}")]
    TransactionNotFound(String),

    /// Contract not found
    #[error("Contract not found: {0}")]
    ContractNotFound(String),

    /// Party not found
    #[error("Party not found: {0}")]
    PartyNotFound(String),

    /// Timeout error
    #[error("Timeout error: {0}")]
    TimeoutError(String),

    /// Serialization error
    #[error("Serialization error: {0}")]
    SerializationError(String),

    /// Deserialization error
    #[error("Deserialization error: {0}")]
    DeserializationError(String),

    /// gRPC error
    #[error("gRPC error: {0}")]
    GrpcError(#[from] tonic::Status),

    /// gRPC transport error
    #[error("gRPC transport error: {0}")]
    GrpcTransportError(#[from] tonic::transport::Error),

    /// Invalid ledger offset
    #[error("Invalid ledger offset: {0}")]
    InvalidOffset(String),

    /// Stream error
    #[error("Stream error: {0}")]
    StreamError(String),

    /// Pool exhausted
    #[error("Connection pool exhausted")]
    PoolExhausted,

    /// Invalid configuration
    #[error("Invalid configuration: {0}")]
    InvalidConfiguration(String),

    /// Internal error
    #[error("Internal error: {0}")]
    InternalError(String),
}

impl LedgerError {
    /// Returns true if this error is retryable
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            LedgerError::ConnectionError(_)
                | LedgerError::TimeoutError(_)
                | LedgerError::GrpcError(_)
                | LedgerError::GrpcTransportError(_)
                | LedgerError::StreamError(_)
        )
    }

    /// Returns true if this error is a client error (4xx)
    pub fn is_client_error(&self) -> bool {
        matches!(
            self,
            LedgerError::InvalidRequest(_)
                | LedgerError::AuthenticationError(_)
                | LedgerError::AuthorizationError(_)
                | LedgerError::InvalidOffset(_)
                | LedgerError::InvalidConfiguration(_)
        )
    }

    /// Returns true if this error is a server error (5xx)
    pub fn is_server_error(&self) -> bool {
        matches!(
            self,
            LedgerError::TransactionError(_)
                | LedgerError::CommandError(_)
                | LedgerError::InternalError(_)
        )
    }
}

impl fmt::Display for LedgerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LedgerError::GrpcError(status) => write!(f, "gRPC error: {} (code: {:?})", status.message(), status.code()),
            LedgerError::GrpcTransportError(err) => write!(f, "gRPC transport error: {}", err),
            _ => write!(f, "{}", self.to_string()),
        }
    }
}
