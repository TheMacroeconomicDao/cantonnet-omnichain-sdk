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

//! Event streaming error types.

use thiserror::Error;

/// Result type alias for event streaming operations.
pub type Result<T> = std::result::Result<T, EventError>;

/// Errors that can occur during event streaming operations.
#[derive(Error, Debug)]
pub enum EventError {
    /// Invalid offset provided for event streaming.
    #[error("Invalid offset: {0}")]
    InvalidOffset(String),

    /// Stream connection failed.
    #[error("Stream connection failed: {0}")]
    ConnectionFailed(String),

    /// Stream subscription failed.
    #[error("Stream subscription failed: {0}")]
    SubscriptionFailed(String),

    /// Stream read error.
    #[error("Stream read error: {0}")]
    StreamReadError(String),

    /// Invalid filter provided.
    #[error("Invalid filter: {0}")]
    InvalidFilter(String),

    /// Buffer overflow - too many events buffered.
    #[error("Buffer overflow: {0} events buffered, max is {1}")]
    BufferOverflow(usize, usize),

    /// Stream closed unexpectedly.
    #[error("Stream closed unexpectedly")]
    StreamClosed,

    /// Timeout waiting for events.
    #[error("Timeout waiting for events after {0:?}")]
    Timeout(std::time::Duration),

    /// Ledger API error.
    #[error("Ledger API error: {0}")]
    LedgerApiError(#[from] canton_ledger_api::error::LedgerError),

    /// Core wallet error.
    #[error("Core wallet error: {0}")]
    CoreError(#[from] canton_wallet_core::error::WalletError),

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

impl From<serde_json::Error> for EventError {
    fn from(err: serde_json::Error) -> Self {
        EventError::SerializationError(err.to_string())
    }
}

impl From<bincode::Error> for EventError {
    fn from(err: bincode::Error) -> Self {
        EventError::DeserializationError(err.to_string())
    }
}
