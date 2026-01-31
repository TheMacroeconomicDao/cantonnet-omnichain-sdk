// Copyright 2025 Canton Wallet SDK Contributors
//
// Licensed under the Apache License, Version 2.0 ( (the "License");
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

//! Chain wallet traits and implementations
//!
//! This module defines the ChainWallet trait and provides implementations
//! for different blockchain networks.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::error::{OmniChainError, Result};
use crate::types::{ChainAddress, ChainAsset, ChainId, TransactionId};

/// Chain wallet trait for interacting with different blockchains
#[async_trait]
pub trait ChainWallet: Send + Sync {
    /// Get chain ID
    fn chain_id(&self) -> ChainId;

    /// Get wallet address
    async fn address(&self) -> Result<ChainAddress>;

    /// Get balance
    async fn balance(&self) -> Result<ChainAsset>;

    /// Get balance for specific asset
    async fn balance_of(&self, asset_id: &str) -> Result<ChainAsset>;

    /// Transfer asset to address
    async fn transfer(
        &self,
        to: ChainAddress,
        asset: ChainAsset,
    ) -> Result<TransactionId>;

    /// Lock asset for cross-chain transfer
    async fn lock_asset(
        &self,
        asset: ChainAsset,
        target_chain: ChainId,
        recipient: ChainAddress,
    ) -> Result<LockReceipt>;

    /// Release asset from lock
    async fn release_asset(
        &self,
        proof: LockProof,
        recipient: ChainAddress,
    ) -> Result<ReleaseReceipt>;

    /// Get transaction status
    async fn transaction_status(&self, tx_id: &TransactionId) -> Result<TransactionStatus>;

    /// Estimate gas for transaction
    async fn estimate_gas(&self, tx: &ChainTransaction) -> Result<u64>;

    /// Get current block height
    async fn block_height(&self) -> Result<u64>;

    /// Get transaction by ID
    async fn get_transaction(&self, tx_id: &TransactionId) -> Result<ChainTransaction>;

    /// Subscribe to events
    async fn subscribe_events(&self, filter: EventFilter) -> Result<ChainEventStream>;
}

/// Lock receipt for cross-chain transfers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockReceipt {
    /// Transaction ID
    pub tx_id: TransactionId,

    /// Chain ID
    pub chain_id: ChainId,

    /// Locked asset
    pub asset: ChainAsset,

    /// Target chain
    pub target_chain: ChainId,

    /// Recipient address
    pub recipient: ChainAddress,

    /// Lock timestamp
    pub timestamp: DateTime<Utc>,

    /// Block height
    pub block_height: u64,

    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

impl LockReceipt {
    /// Create new lock receipt
    pub fn new(
        tx_id: TransactionId,
        chain_id: ChainId,
        asset: ChainAsset,
        target_chain: ChainId,
        recipient: ChainAddress,
        block_height: u64,
    ) -> Self {
        Self {
            tx_id,
            chain_id,
            asset,
            target_chain,
            recipient,
            timestamp: Utc::now(),
            block_height,
            metadata: HashMap::new(),
        }
    }

    /// Add metadata
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
}

/// Release receipt for cross-chain transfers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReleaseReceipt {
    /// Transaction ID
    pub tx_id: TransactionId,

    /// Chain ID
    pub chain_id: ChainId,

    /// Released asset
    pub asset: ChainAsset,

    /// Recipient address
    pub recipient: ChainAddress,

    /// Release timestamp
    pub timestamp: DateTime<Utc>,

    /// Block height
    pub block_height: u64,

    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

impl ReleaseReceipt {
    /// Create new release receipt
    pub fn new(
        tx_id: TransactionId,
        chain_id: ChainId,
        asset: ChainAsset,
        recipient: ChainAddress,
        block_height: u64,
    ) -> Self {
        Self {
            tx_id,
            chain_id,
            asset,
            recipient,
            timestamp: Utc::now(),
            block_height,
            metadata: HashMap::new(),
        }
    }

    /// Add metadata
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
}

/// Lock proof for cross-chain transfers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockProof {
    /// Source chain ID
    pub source_chain: ChainId,

    /// Lock transaction ID
    pub lock_tx_id: TransactionId,

    /// Locked asset
    pub asset: ChainAsset,

    /// Target chain
    pub target_chain: ChainId,

    /// Recipient address
    pub recipient: ChainAddress,

    /// Block height
    pub block_height: u64,

    /// Block hash
    pub block_hash: String,

    /// Merkle proof
    pub merkle_proof: Vec<String>,

    /// Signature
    pub signature: Vec<u8>,

    /// Additional data
    pub additional_data: HashMap<String, String>,
}

impl LockProof {
    /// Create new lock proof
    pub fn new(
        source_chain: ChainId,
        lock_tx_id: TransactionId,
        asset: ChainAsset,
        target_chain: ChainId,
        recipient: ChainAddress,
        block_height: u64,
        block_hash: String,
        signature: Vec<u8>,
    ) -> Self {
        Self {
            source_chain,
            lock_tx_id,
            asset,
            target_chain,
            recipient,
            block_height,
            block_hash,
            merkle_proof: Vec::new(),
            signature,
            additional_data: HashMap::new(),
        }
    }

    /// Add merkle proof
    pub fn with_merkle_proof(mut self, proof: Vec<String>) -> Self {
        self.merkle_proof = proof;
        self
    }

    /// Add additional data
    pub fn with_additional_data(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.additional_data.insert(key.into(), value.into());
        self
    }
}

/// Transaction status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransactionStatus {
    /// Transaction is pending
    Pending,
    /// Transaction is confirmed
    Confirmed,
    /// Transaction failed
    Failed,
    /// Transaction is unknown
    Unknown,
}

/// Chain transaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainTransaction {
    /// Transaction ID
    pub tx_id: TransactionId,

    /// Chain ID
    pub chain_id: ChainId,

    /// From address
    pub from: ChainAddress,

    /// To address
    pub to: Option<ChainAddress>,

    /// Value/amount
    pub value: String,

    /// Gas limit
    pub gas_limit: u64,

    /// Gas used
    pub gas_used: Option<u64>,

    /// Gas price
    pub gas_price: Option<String>,

    /// Nonce
    pub nonce: Option<u64>,

    /// Block height
    pub block_height: Option<u64>,

    /// Timestamp
    pub timestamp: Option<DateTime<Utc>>,

    /// Status
    pub status: TransactionStatus,

    /// Input data
    pub input_data: Option<String>,

    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

impl ChainTransaction {
    /// Create new transaction
    pub fn new(
        tx_id: TransactionId,
        chain_id: ChainId,
        from: ChainAddress,
        value: String,
    ) -> Self {
        Self {
            tx_id,
            chain_id,
            from,
            to: None,
            value,
            gas_limit: 0,
            gas_used: None,
            gas_price: None,
            nonce: None,
            block_height: None,
            timestamp: None,
            status: TransactionStatus::Pending,
            input_data: None,
            metadata: HashMap::new(),
        }
    }

    /// Set to address
    pub fn with_to(mut self, to: ChainAddress) -> Self {
        self.to = Some(to);
        self
    }

    /// Set gas limit
    pub fn with_gas_limit(mut self, gas_limit: u64) -> Self {
        self.gas_limit = gas_limit;
        self
    }

    /// Set gas price
    pub fn with_gas_price(mut self, gas_price: String) -> Self {
        self.gas_price = Some(gas_price);
        self
    }

    /// Set nonce
    pub fn with_nonce(mut self, nonce: u64) -> Self {
        self.nonce = Some(nonce);
        self
    }

    /// Set status
    pub fn with_status(mut self, status: TransactionStatus) -> Self {
        self.status = status;
        self
    }

    /// Set block height
    pub fn with_block_height(mut self, block_height: u64) -> Self {
        self.block_height = Some(block_height);
        self
    }

    /// Add metadata
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
}

/// Event filter for chain events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventFilter {
    /// Contract addresses to filter
    pub addresses: Vec<ChainAddress>,

    /// Event topics to filter
    pub topics: Vec<String>,

    /// Block range
    pub from_block: Option<u64>,
    pub to_block: Option<u64>,
}

impl EventFilter {
    /// Create new event filter
    pub fn new() -> Self {
        Self {
            addresses: Vec::new(),
            topics: Vec::new(),
            from_block: None,
            to_block: None,
        }
    }

    /// Add address to filter
    pub fn with_address(mut self, address: ChainAddress) -> Self {
        self.addresses.push(address);
        self
    }

    /// Add topic to filter
    pub fn with_topic(mut self, topic: String) -> Self {
        self.topics.push(topic);
        self
    }

    /// Set block range
    pub fn with_block_range(mut self, from: u64, to: u64) -> Self {
        self.from_block = Some(from);
        self.to_block = Some(to);
        self
    }
}

impl Default for EventFilter {
    fn default() -> Self {
        Self::new()
    }
}

/// Chain event stream
pub struct ChainEventStream {
    /// Chain ID
    pub chain_id: ChainId,

    /// Event filter
    pub filter: EventFilter,

    /// Current block height
    pub current_block: u64,
}

impl ChainEventStream {
    /// Create new event stream
    pub fn new(chain_id: ChainId, filter: EventFilter) -> Self {
        Self {
            chain_id,
            filter,
            current_block: 0,
        }
    }
}

/// Chain event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainEvent {
    /// Chain ID
    pub chain_id: ChainId,

    /// Transaction ID
    pub tx_id: TransactionId,

    /// Block height
    pub block_height: u64,

    /// Log index
    pub log_index: u64,

    /// Contract address
    pub address: ChainAddress,

    /// Event topics
    pub topics: Vec<String>,

    /// Event data
    pub data: String,

    /// Timestamp
    pub timestamp: DateTime<Utc>,
}

impl ChainEvent {
    /// Create new chain event
    pub fn new(
        chain_id: ChainId,
        tx_id: TransactionId,
        block_height: u64,
        address: ChainAddress,
    ) -> Self {
        Self {
            chain_id,
            tx_id,
            block_height,
            log_index: 0,
            address,
            topics: Vec::new(),
            data: String::new(),
            timestamp: Utc::now(),
        }
    }

    /// Add topic
    pub fn with_topic(mut self, topic: String) -> Self {
        self.topics.push(topic);
        self
    }

    /// Set data
    pub fn with_data(mut self, data: String) -> Self {
        self.data = data;
        self
    }

    /// Set log index
    pub fn with_log_index(mut self, log_index: u64) -> Self {
        self.log_index = log_index;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lock_receipt_creation() {
        let receipt = LockReceipt::new(
            TransactionId::new("0x123"),
            ChainId::Ethereum,
            ChainAsset::new("ETH", "1000000000000000000"),
            ChainId::Canton,
            ChainAddress::new("0xabc"),
            1000,
        );

        assert_eq!(receipt.tx_id.as_str(), "0x123");
        assert_eq!(receipt.chain_id, ChainId::Ethereum);
        assert_eq!(receipt.target_chain, ChainId::Canton);
    }

    #[test]
    fn test_lock_receipt_with_metadata() {
        let receipt = LockReceipt::new(
            TransactionId::new("0x123"),
            ChainId::Ethereum,
            ChainAsset::new("ETH", "1000000000000000000"),
            ChainId::Canton,
            ChainAddress::new("0xabc"),
            1000,
        )
        .with_metadata("key", "value");

        assert_eq!(receipt.metadata.get("key"), Some(&"value".to_string()));
    }

    #[test]
    fn test_chain_transaction_builder() {
        let tx = ChainTransaction::new(
            TransactionId::new("0x456"),
            ChainId::Ethereum,
            ChainAddress::new("0xdef"),
            "1000000000000000000".to_string(),
        )
        .with_to(ChainAddress::new("0xabc"))
        .with_gas_limit(21000)
        .with_status(TransactionStatus::Confirmed);

        assert_eq!(tx.tx_id.as_str(), "0x456");
        assert_eq!(tx.gas_limit, 21000);
        assert_eq!(tx.status, TransactionStatus::Confirmed);
    }

    #[test]
    fn test_event_filter() {
        let filter = EventFilter::new()
            .with_address(ChainAddress::new("0xabc"))
            .with_topic("Transfer(address,address,uint256)".to_string())
            .with_block_range(1000, 2000);

        assert_eq!(filter.addresses.len(), 1);
        assert_eq!(filter.topics.len(), 1);
        assert_eq!(filter.from_block, Some(1000));
        assert_eq!(filter.to_block, Some(2000));
    }
}
