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

//! Chain adapters for different blockchain networks
//!
//! This module provides adapter implementations for interacting with
//! various blockchain networks in cross-chain operations.

use std::collections::HashMap;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tracing::{debug, instrument};

use crate::error::{OmniChainError, Result};
use crate::types::{AssetAmount, ChainAddress, ChainId, LockReceipt, ReleaseReceipt};

/// Chain wallet trait for cross-chain operations
#[async_trait]
pub trait ChainWallet: Send + Sync {
    /// Get chain ID
    fn chain_id(&self) -> ChainId;

    /// Get wallet address
    async fn address(&self) -> Result<ChainAddress>;

    /// Get balance
    async fn balance(&self, token: Option<String>) -> Result<AssetAmount>;

    /// Lock asset for cross-chain transfer
    async fn lock_asset(
        &self,
        asset: AssetAmount,
        target_chain: ChainId,
        recipient: ChainAddress,
    ) -> Result<LockReceipt>;

    /// Release asset from lock
    async fn release_asset(
        &self,
        proof_id: String,
        recipient: ChainAddress,
    ) -> Result<ReleaseReceipt>;

    /// Get transaction status
    async fn get_transaction_status(&self, tx_id: &str) -> Result<TransactionStatus>;

    /// Estimate gas/fees
    async fn estimate_fees(&self, operation: &str) -> Result<AssetAmount>;
}

/// Transaction status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransactionStatus {
    /// Pending
    Pending,
    /// Confirmed
    Confirmed,
    /// Failed
    Failed,
    /// Reverted
    Reverted,
}

/// Chain adapter configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainAdapterConfig {
    /// Chain ID
    pub chain_id: ChainId,

    /// RPC endpoint
    pub rpc_endpoint: String,

    /// WebSocket endpoint (optional)
    pub ws_endpoint: Option<String>,

    /// Chain-specific configuration
    pub chain_config: ChainSpecificConfig,
}

/// Chain-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ChainSpecificConfig {
    /// Ethereum configuration
    Ethereum(EthereumConfig),
    /// Cosmos configuration
    Cosmos(CosmosConfig),
    /// Polkadot configuration
    Polkadot(PolkadotConfig),
    /// Custom configuration
    Custom(HashMap<String, String>),
}

/// Ethereum configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EthereumConfig {
    /// Chain ID
    pub chain_id: u64,

    /// Bridge contract address
    pub bridge_address: String,

    /// Gas price multiplier
    pub gas_price_multiplier: f64,

    /// Confirmation blocks
    pub confirmation_blocks: u64,
}

/// Cosmos configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CosmosConfig {
    /// Chain ID
    pub chain_id: String,

    /// Bridge module name
    pub bridge_module: String,

    /// Gas adjustment
    pub gas_adjustment: f64,

    /// Gas prices
    pub gas_prices: String,
}

/// Polkadot configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolkadotConfig {
    /// Chain ID
    pub chain_id: u32,

    /// Bridge pallet name
    pub bridge_pallet: String,

    /// Tip amount
    pub tip: u128,
}

/// Ethereum chain adapter
pub struct EthereumAdapter {
    /// Configuration
    config: ChainAdapterConfig,

    /// Private key
    private_key: Vec<u8>,

    /// Address
    address: ChainAddress,
}

impl EthereumAdapter {
    /// Create new Ethereum adapter
    pub fn new(config: ChainAdapterConfig, private_key: Vec<u8>) -> Result<Self> {
        // Derive address from private key
        let address = Self::derive_address(&private_key)?;

        Ok(Self {
            config,
            private_key,
            address,
        })
    }

    /// Derive address from private key
    fn derive_address(private_key: &[u8]) -> Result<ChainAddress> {
        use k256::ecdsa::SigningKey;
        use k256::elliptic_curve::sec1::ToEncodedPoint;

        let signing_key = SigningKey::from_slice(private_key)
            .map_err(|e| OmniChainError::InvalidKey(e.to_string()))?;

        let public_key = signing_key.verifying_key();
        let encoded = public_key.to_encoded_point(false);

        // Take last 20 bytes of hash of public key
        let hash = sha3::Keccak256::digest(&encoded.as_bytes()[1..]);
        let address_bytes = &hash[hash.len() - 20..];

        Ok(ChainAddress::Ethereum(format!("0x{}", hex::encode(address_bytes))))
    }

    /// Get Ethereum configuration
    fn eth_config(&self) -> &EthereumConfig {
        match &self.config.chain_config {
            ChainSpecificConfig::Ethereum(config) => config,
            _ => panic!("Invalid configuration for Ethereum adapter"),
        }
    }

    /// Generate transaction ID
    fn generate_tx_id(&self) -> String {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(&self.private_key);
        hasher.update(Utc::now().timestamp().to_be_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// Estimate gas
    async fn estimate_gas(&self, operation: &str) -> Result<u64> {
        // Base gas costs for different operations
        let base_gas = match operation {
            "lock_asset" => 100_000,
            "release_asset" => 80_000,
            _ => 50_000,
        };

        Ok(base_gas)
    }

    /// Get gas price
    async fn get_gas_price(&self) -> Result<u64> {
        // In production, this would query the network
        // For now, return a default value
        Ok(20_000_000_000) // 20 Gwei
    }
}

#[async_trait]
impl ChainWallet for EthereumAdapter {
    fn chain_id(&self) -> ChainId {
        self.config.chain_id
    }

    #[instrument(skip(self))]
    async fn address(&self) -> Result<ChainAddress> {
        Ok(self.address.clone())
    }

    #[instrument(skip(self))]
    async fn balance(&self, token: Option<String>) -> Result<AssetAmount> {
        debug!("Querying balance for token={:?}", token);

        // In production, this would query the blockchain
        // For now, return a mock balance
        Ok(AssetAmount {
            amount: 1_000_000_000_000_000_000u128, // 1 ETH in wei
            decimals: 18,
            token: token.unwrap_or_else(|| "ETH".to_string()),
        })
    }

    #[instrument(skip(self))]
    async fn lock_asset(
        &self,
        asset: AssetAmount,
        target_chain: ChainId,
        recipient: ChainAddress,
    ) -> Result<LockReceipt> {
        debug!(
            "Locking asset on Ethereum: amount={}, target_chain={:?}, recipient={}",
            asset.amount, target_chain, recipient
        );

        let tx_id = self.generate_tx_id();
        let gas = self.estimate_gas("lock_asset").await?;
        let gas_price = self.get_gas_price().await?;

        let fee = AssetAmount {
            amount: (gas as u128) * gas_price,
            decimals: 18,
            token: "ETH".to_string(),
        };

        let receipt = LockReceipt {
            tx_id: tx_id.clone(),
            chain_id: ChainId::Ethereum,
            asset,
            target_chain,
            recipient: recipient.to_string(),
            fee,
            timestamp: Utc::now(),
            status: crate::bridge::BridgeTxStatus::Pending,
        };

        debug!("Asset locked on Ethereum: tx_id={}", tx_id);
        Ok(receipt)
    }

    #[instrument(skip(self))]
    async fn release_asset(
        &self,
        proof_id: String,
        recipient: ChainAddress,
    ) -> Result<ReleaseReceipt> {
        debug!(
            "Releasing asset on Ethereum: proof_id={}, recipient={}",
            proof_id, recipient
        );

        let tx_id = self.generate_tx_id();
        let gas = self.estimate_gas("release_asset").await?;
        let gas_price = self.get_gas_price().await?;

        let fee = AssetAmount {
            amount: (gas as u128) * gas_price,
            decimals: 18,
            token: "ETH".to_string(),
        };

        let receipt = ReleaseReceipt {
            tx_id: tx_id.clone(),
            chain_id: ChainId::Ethereum,
            proof_id,
            recipient: recipient.to_string(),
            fee,
            timestamp: Utc::now(),
            status: crate::bridge::BridgeTxStatus::Pending,
        };

        debug!("Asset released on Ethereum: tx_id={}", tx_id);
        Ok(receipt)
    }

    #[instrument(skip(self))]
    async fn get_transaction_status(&self, tx_id: &str) -> Result<TransactionStatus> {
        debug!("Getting transaction status: tx_id={}", tx_id);

        // In production, this would query the blockchain
        // For now, return a mock status
        Ok(TransactionStatus::Confirmed)
    }

    #[instrument(skip(self))]
    async fn estimate_fees(&self, operation: &str) -> Result<AssetAmount> {
        debug!("Estimating fees for operation={}", operation);

        let gas = self.estimate_gas(operation).await?;
        let gas_price = self.get_gas_price().await?;

        Ok(AssetAmount {
            amount: (gas as u128) * gas_price,
            decimals: 18,
            token: "ETH".to_string(),
        })
    }
}

/// Cosmos chain adapter
pub struct CosmosAdapter {
    /// Configuration
    config: ChainAdapterConfig,

    /// Private key
    private_key: Vec<u8>,

    /// Address
    address: ChainAddress,
}

impl CosmosAdapter {
    /// Create new Cosmos adapter
    pub fn new(config: ChainAdapterConfig, private_key: Vecu8>) -> Result<Self> {
        // Derive address from private key
        let address = Self::derive_address(&private_key)?;

        Ok(Self {
            config,
            private_key,
            address,
        })
    }

    /// Derive address from private key
    fn derive_address(private_key: &[u8]) -> Result<ChainAddress> {
        use k256::ecdsa::SigningKey;
        use ripemd::Ripemd160;
        use sha2::{Digest, Sha256};

        let signing_key = SigningKey::from_slice(private_key)
            .map_err(|e| OmniChainError::InvalidKey(e.to_string()))?;

        let public_key = signing_key.verifying_key();
        let encoded = public_key.to_encoded_point(false);

        // SHA256 then RIPEMD160
        let sha256_hash = Sha256::digest(&encoded.as_bytes()[1..]);
        let ripemd160_hash = Ripemd160::digest(&sha256_hash);

        // Bech32 encode
        let address = bech32::encode(
            "cosmos",
            &ripemd160_hash,
            bech32::Variant::Bech32,
        )
        .map_err(|e| OmniChainError::InvalidKey(e.to_string()))?;

        Ok(ChainAddress::Cosmos(address))
    }

    /// Get Cosmos configuration
    fn cosmos_config(&self) -> &CosmosConfig {
        match &self.config.chain_config {
            ChainSpecificConfig::Cosmos(config) => config,
            _ => panic!("Invalid configuration for Cosmos adapter"),
        }
    }

    /// Generate transaction ID
    fn generate_tx_id(&self) -> String {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(&self.private_key);
        hasher.update(Utc::now().timestamp().to_be_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// Estimate gas
    async fn estimate_gas(&self, operation: &str) -> Result<u64> {
        let base_gas = match operation {
            "lock_asset" => 150_000,
            "release_asset" => 120_000,
            _ => 60_000,
        };

        Ok(base_gas)
    }
}

#[async_trait]
impl ChainWallet for CosmosAdapter {
    fn chain_id(&self) -> ChainId {
        self.config.chain_id
    }

    #[instrument(skip(self))]
    async fn address(&self) -> Result<ChainAddress> {
        Ok(self.address.clone())
    }

    #[instrument(skip(self))]
    async fn balance(&self, token: Option<String>) -> Result<AssetAmount> {
        debug!("Querying balance for token={:?}", token);

        Ok(AssetAmount {
            amount: 1_000_000_000u128, // 1000 ATOM
            decimals: 6,
            token: token.unwrap_or_else(|| "ATOM".to_string()),
        })
    }

    #[instrument(skip(self))]
    async fn lock_asset(
        &self,
        asset: AssetAmount,
        target_chain: ChainId,
        recipient: ChainAddress,
    ) -> Result<LockReceipt> {
        debug!(
            "Locking asset on Cosmos: amount={}, target_chain={:?}, recipient={}",
            asset.amount, target_chain, recipient
        );

        let tx_id = self.generate_tx_id();
        let gas = self.estimate_gas("lock_asset").await?;

        let fee = AssetAmount {
            amount: gas as u128,
            decimals: 6,
            token: "ATOM".to_string(),
        };

        let receipt = LockReceipt {
            tx_id: tx_id.clone(),
            chain_id: ChainId::Cosmos,
            asset,
            target_chain,
            recipient: recipient.to_string(),
            fee,
            timestamp: Utc::now(),
            status: crate::bridge::BridgeTxStatus::Pending,
        };

        debug!("Asset locked on Cosmos: tx_id={}", tx_id);
        Ok(receipt)
    }

    #[instrument(skip(self))]
    async fn release_asset(
        &self,
        proof_id: String,
        recipient: ChainAddress,
    ) -> Result<ReleaseReceipt> {
        debug!(
            "Releasing asset on Cosmos: proof_id={}, recipient={}",
            proof_id, recipient
        );

        let tx_id = self.generate_tx_id();
        let gas = self.estimate_gas("release_asset").await?;

        let fee = AssetAmount {
            amount: gas as u128,
            decimals: 6,
            token: "ATOM".to_string(),
        };

        let receipt = ReleaseReceipt {
            tx_id: tx_id.clone(),
            chain_id: ChainId::Cosmos,
            proof_id,
            recipient: recipient.to_string(),
            fee,
            timestamp: Utc::now(),
            status: crate::bridge::BridgeTxStatus::Pending,
        };

        debug!("Asset released on Cosmos: tx_id={}", tx_id);
        Ok(receipt)
    }

    #[instrument(skip(self))]
    async fn get_transaction_status(&self, tx_id: &str) -> Result<TransactionStatus> {
        debug!("Getting transaction status: tx_id={}", tx_id);

        Ok(TransactionStatus::Confirmed)
    }

    #[instrument(skip(self))]
    async fn estimate_fees(&self, operation: &str) -> Result<AssetAmount> {
        debug!("Estimating fees for operation={}", operation);

        let gas = self.estimate_gas(operation).await?;

        Ok(AssetAmount {
            amount: gas as u128,
            decimals: 6,
            token: "ATOM".to_string(),
        })
    }
}

/// Polkadot chain adapter
pub struct PolkadotAdapter {
    /// Configuration
    config: ChainAdapterConfig,

    /// Private key
    private_key: Vec<u8>,

    /// Address
    address: ChainAddress,
}

impl PolkadotAdapter {
    /// Create new Polkadot adapter
    pub fn new(config: ChainAdapterConfig, private_key: Vec<u8>) -> Result<Self> {
        // Derive address from private key
        let address = Self::derive_address(&private_key)?;

        Ok(Self {
            config,
            private_key,
            address,
        })
    }

    /// Derive address from private key
    fn derive_address(private_key: &[u8]) -> Result<ChainAddress> {
        use schnorrkel::{SecretKey, PublicKey};

        let secret_key = SecretKey::from_bytes(private_key)
            .map_err(|e| OmniChainError::InvalidKey(e.to_string()))?;

        let public_key: PublicKey = secret_key.into();

        // SS58 encode
        let address = ss58_registry::ss58::encode(
            &public_key.to_bytes(),
            ss58_registry::Ss58AddressFormat::custom(0),
        );

        Ok(ChainAddress::Polkadot(address))
    }

    /// Get Polkadot configuration
    fn polkadot_config(&self) -> &PolkadotConfig {
        match &self.config.chain_config {
            ChainSpecificConfig::Polkadot(config) => config,
            _ => panic!("Invalid configuration for Polkadot adapter"),
        }
    }

    /// Generate transaction ID
    fn generate_tx_id(&self) -> String {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(&self.private_key);
        hasher.update(Utc::now().timestamp().to_be_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// Estimate weight
    async fn estimate_weight(&self, operation: &str) -> Result<u64> {
        let base_weight = match operation {
            "lock_asset" => 200_000_000,
            "release_asset" => 150_000_000,
            _ => 100_000_000,
        };

        Ok(base_weight)
    }
}

#[async_trait]
impl ChainWallet for PolkadotAdapter {
    fn chain_id(&self) -> ChainId {
        self.config.chain_id
    }

    #[instrument(skip(self))]
    async fn address(&self) -> Result<ChainAddress> {
        Ok(self.address.clone())
    }

    #[instrument(skip(self))]
    async fn balance(&self, token: Option<String>) -> Result<AssetAmount> {
        debug!("Querying balance for token={:?}", token);

        Ok(AssetAmount {
            amount: 10_000_000_000_000u128, // 10 DOT (planck)
            decimals: 10,
            token: token.unwrap_or_else(|| "DOT".to_string()),
        })
    }

    #[instrument(skip(self))]
    async fn lock_asset(
        &self,
        asset: AssetAmount,
        target_chain: ChainId,
        recipient: ChainAddress,
    ) -> Result<LockReceipt> {
        debug!(
            "Locking asset on Polkadot: amount={}, target_chain={:?}, recipient={}",
            asset.amount, target_chain, recipient
        );

        let tx_id = self.generate_tx_id();
        let weight = self.estimate_weight("lock_asset").await?;

        let fee = AssetAmount {
            amount: weight,
            decimals: 10,
            token: "DOT".to_string(),
        };

        let receipt = LockReceipt {
            tx_id: tx_id.clone(),
            chain_id: ChainId::Polkadot,
            asset,
            target_chain,
            recipient: recipient.to_string(),
            fee,
            timestamp: Utc::now(),
            status: crate::bridge::BridgeTxStatus::Pending,
        };

        debug!("Asset locked on Polkadot: tx_id={}", tx_id);
        Ok(receipt)
    }

    #[instrument(skip(self))]
    async fn release_asset(
        &self,
        proof_id: String,
        recipient: ChainAddress,
    ) -> Result<ReleaseReceipt> {
        debug!(
            "Releasing asset on Polkadot: proof_id={}, recipient={}",
            proof_id, recipient
        );

        let tx_id = self.generate_tx_id();
        let weight = self.estimate_weight("release_asset").await?;

        let fee = AssetAmount {
            amount: weight,
            decimals: 10,
            token: "DOT".to_string(),
        };

        let receipt = ReleaseReceipt {
            tx_id: tx_id.clone(),
            chain_id: ChainId::Polkadot,
            proof_id,
            recipient: recipient.to_string(),
            fee,
            timestamp: Utc::now(),
            status: crate::bridge::BridgeTxStatus::Pending,
        };

        debug!("Asset released on Polkadot: tx_id={}", tx_id);
        Ok(receipt)
    }

    #[instrument(skip(self))]
    async fn get_transaction_status(&self, tx_id: &str) -> Result<TransactionStatus> {
        debug!("Getting transaction status: tx_id={}", tx_id);

        Ok(TransactionStatus::Confirmed)
    }

    #[instrument(skip(self))]
    async fn estimate_fees(&self, operation: &str) -> Result<AssetAmount> {
        debug!("Estimating fees for operation={}", operation);

        let weight = self.estimate_weight(operation).await?;

        Ok(AssetAmount {
            amount: weight,
            decimals: 10,
            token: "DOT".to_string(),
        })
    }
}

/// Chain adapter factory
pub struct ChainAdapterFactory;

impl ChainAdapterFactory {
    /// Create chain adapter from configuration
    pub fn create_adapter(
        config: ChainAdapterConfig,
        private_key: Vec<u8>,
    ) -> Result<Box<dyn ChainWallet>> {
        match config.chain_id {
            ChainId::Ethereum => Ok(Box::new(EthereumAdapter::new(config, private_key)?)),
            ChainId::Cosmos => Ok(Box::new(CosmosAdapter::new(config, private_key)?)),
            ChainId::Polkadot => Ok(Box::new(PolkadotAdapter::new(config, private_key)?)),
            _ => Err(OmniChainError::UnsupportedChain(config.chain_id)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ethereum_address_derivation() {
        let private_key = vec![1u8; 32];
        let address = EthereumAdapter::derive_address(&private_key).unwrap();

        match address {
            ChainAddress::Ethereum(addr) => {
                assert!(addr.starts_with("0x"));
                assert_eq!(addr.len(), 42);
            }
            _ => panic!("Expected Ethereum address"),
        }
    }

    #[test]
    fn test_cosmos_address_derivation() {
        let private_key = vec![1u8; 32];
        let address = CosmosAdapter::derive_address(&private_key).unwrap();

        match address {
            ChainAddress::Cosmos(addr) => {
                assert!(addr.starts_with("cosmos"));
            }
            _ => panic!("Expected Cosmos address"),
        }
    }

    #[test]
    fn test_chain_adapter_factory() {
        let config = ChainAdapterConfig {
            chain_id: ChainId::Ethereum,
            rpc_endpoint: "http://localhost:8545".to_string(),
            ws_endpoint: None,
            chain_config: ChainSpecificConfig::Ethereum(EthereumConfig {
                chain_id: 1,
                bridge_address: "0x123".to_string(),
                gas_price_multiplier: 1.0,
                confirmation_blocks: 12,
            }),
        };

        let private_key = vec![1u8; 32];
        let adapter = ChainAdapterFactory::create_adapter(config, private_key).unwrap();

        assert_eq!(adapter.chain_id(), ChainId::Ethereum);
    }

    #[tokio::test]
    async fn test_ethereum_adapter_balance() {
        let config = ChainAdapterConfig {
            chain_id: ChainId::Ethereum,
            rpc_endpoint: "http://localhost:8545".to_string(),
            ws_endpoint: None,
            chain_config: ChainSpecificConfig::Ethereum(EthereumConfig {
                chain_id: 1,
                bridge_address: "0x123".to_string(),
                gas_price_multiplier: 1.0,
                confirmation_blocks: 12,
            }),
        };

        let private_key = vec![1u8; 32];
        let adapter = EthereumAdapter::new(config, private_key).unwrap();

        let balance = adapter.balance(None).await.unwrap();
        assert_eq!(balance.token, "ETH");
        assert_eq!(balance.decimals, 18);
    }

    #[tokio::test]
    async fn test_ethereum_adapter_lock_asset() {
        let config = ChainAdapterConfig {
            chain_id: ChainId::Ethereum,
            rpc_endpoint: "http://localhost:8545".to_string(),
            ws_endpoint: None,
            chain_config: ChainSpecificConfig::Ethereum(EthereumConfig {
                chain_id: 1,
                bridge_address: "0x123".to_string(),
                gas_price_multiplier: 1.0,
                confirmation_blocks: 12,
            }),
        };

        let private_key = vec![1u8; 32];
        let adapter = EthereumAdapter::new(config, private_key).unwrap();

        let asset = AssetAmount {
            amount: 1_000_000_000_000_000_000u128,
            decimals: 18,
            token: "USDC".to_string(),
        };

        let receipt = adapter
            .lock_asset(asset, ChainId::Canton, ChainAddress::Canton("party::test".to_string()))
            .await
            .unwrap();

        assert_eq!(receipt.chain_id, ChainId::Ethereum);
        assert_eq!(receipt.target_chain, ChainId::Canton);
        assert_eq!(receipt.asset.token, "USDC");
    }
}
