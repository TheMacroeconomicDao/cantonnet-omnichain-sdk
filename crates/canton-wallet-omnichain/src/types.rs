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

use serde::{Deserialize, Serialize};
use std::fmt;

/// Chain identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ChainId {
    /// Canton Network
    Canton,
    /// Ethereum Mainnet
    Ethereum,
    /// Ethereum Goerli Testnet
    EthereumGoerli,
    /// Ethereum Sepolia Testnet
    EthereumSepolia,
    /// Cosmos Hub
    Cosmos,
    /// Osmosis
    Osmosis,
    /// Juno
    Juno,
    /// Polkadot
    Polkadot,
    /// Kusama
    Kusama,
    /// Custom chain
    Custom(String),
}

impl ChainId {
    /// Create a custom chain ID
    pub fn custom(name: impl Into<String>) -> Self {
        Self::Custom(name.into())
    }

    /// Check if this is Canton
    pub fn is_canton(&self) -> bool {
        matches!(self, Self::Canton)
    }

    /// Get the chain name as a string
    pub fn as_str(&self) -> &str {
        match self {
            Self::Canton => "canton",
            Self::Ethereum => "ethereum",
            Self::EthereumGoerli => "ethereum-goerli",
            Self::EthereumSepolia => "ethereum-sepolia",
            Self::Cosmos => "cosmos",
            Self::Osmosis => "osmosis",
            Self::Juno => "juno",
            Self::Polkadot => "polkadot",
            Self::Kusama => "kusama",
            Self::Custom(name) => name,
        }
    }
}

impl fmt::Display for ChainId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Chain address
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ChainAddress {
    /// Chain ID
    pub chain_id: ChainId,
    /// Address bytes
    pub address: String,
}

impl ChainAddress {
    /// Create a new chain address
    pub fn new(chain_id: ChainId, address: impl Into<String>) -> Self {
        Self {
            chain_id,
            address: address.into(),
        }
    }

    /// Create an Ethereum address
    pub fn ethereum(address: impl Into<String>) -> Self {
        Self::new(ChainId::Ethereum, address)
    }

    /// Create a Cosmos address
    pub fn cosmos(address: impl Into<String>) -> Self {
        Self::new(ChainId::Cosmos, address)
    }

    /// Create a Polkadot address
    pub fn polkadot(address: impl Into<String>) -> Self {
        Self::new(ChainId::Polkadot, address)
    }

    /// Get the address string
    pub fn as_str(&self) -> &str {
        &self.address
    }
}

impl fmt::Display for ChainAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.chain_id, self.address)
    }
}

/// Canton asset
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CantonAsset {
    /// Asset ID
    pub asset_id: String,
    /// Amount
    pub amount: String,
    /// Owner party ID
    pub owner_party_id: String,
}

impl CantonAsset {
    /// Create a new Canton asset
    pub fn new(asset_id: impl Into<String>, amount: impl Into<String>, owner_party_id: impl Into<String>) -> Self {
        Self {
            asset_id: asset_id.into(),
            amount: amount.into(),
            owner_party_id: owner_party_id.into(),
        }
    }
}

/// Chain asset
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChainAsset {
    /// Chain ID
    pub chain_id: ChainId,
    /// Token address or ID
    pub token_id: String,
    /// Amount
    pub amount: String,
    /// Owner address
    pub owner_address: String,
}

impl ChainAsset {
    /// Create a new chain asset
    pub fn new(chain_id: ChainId, token_id: impl Into<String>, amount: impl Into<String>, owner_address: impl Into<String>) -> Self {
        Self {
            chain_id,
            token_id: token_id.into(),
            amount: amount.into(),
            owner_address: owner_address.into(),
        }
    }

    /// Convert to Canton asset
    pub fn to_canton_asset(&self, owner_party_id: impl Into<String>) -> CantonAsset {
        CantonAsset {
            asset_id: format!("{}:{}", self.chain_id, self.token_id),
            amount: self.amount.clone(),
            owner_party_id: owner_party_id.into(),
        }
    }
}

/// Cross-chain transaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossChainTx {
    /// Canton transaction ID
    pub canton_tx_id: String,
    /// Target chain transaction ID
    pub target_tx_id: String,
    /// Asset being transferred
    pub asset: CantonAsset,
    /// Source chain
    pub source_chain: ChainId,
    /// Target chain
    pub target_chain: ChainId,
    /// Timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Status
    pub status: CrossChainTxStatus,
}

/// Cross-chain transaction status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CrossChainTxStatus {
    /// Transaction initiated
    Initiated,
    /// Asset locked
    Locked,
    /// Proof generated
    ProofGenerated,
    /// Asset released
    Released,
    /// Transaction completed
    Completed,
    /// Transaction failed
    Failed,
}

/// Lock receipt
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockReceipt {
    /// Transaction ID
    pub tx_id: String,
    /// Asset being locked
    pub asset: CantonAsset,
    /// Target chain
    pub target_chain: ChainId,
    /// Recipient address
    pub recipient: ChainAddress,
    /// Lock timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Lock contract address
    pub lock_contract: String,
}

/// Release receipt
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReleaseReceipt {
    /// Transaction ID
    pub tx_id: String,
    /// Asset being released
    pub asset: ChainAsset,
    /// Source chain
    pub source_chain: ChainId,
    /// Recipient address
    pub recipient: ChainAddress,
    /// Release timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Release contract address
    pub release_contract: String,
}
