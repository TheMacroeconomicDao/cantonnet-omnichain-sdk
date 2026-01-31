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

use std::sync::Arc;
use dashmap::DashMap;
use crate::{
    error::{OmniChainError, OmniChainResult},
    types::{ChainId, ChainAddress, CantonAsset, ChainAsset, LockReceipt, ReleaseReceipt},
    adapter::ChainWallet,
};
use canton_wallet_core::PartyId;

/// Bridge manager for cross-chain transfers
pub struct BridgeManager {
    config: BridgeConfig,
    canton_bridge_contracts: DashMap<ChainId, String>,
    chain_bridge_contracts: DashMap<ChainId, String>,
}

impl BridgeManager {
    /// Create a new bridge manager
    pub fn new(config: BridgeConfig) -> Self {
        Self {
            config,
            canton_bridge_contracts: DashMap::new(),
            chain_bridge_contracts: DashMap::new(),
        }
    }

    /// Register a Canton bridge contract
    pub fn register_canton_bridge(&self, chain_id: ChainId, contract_address: impl Into<String>) {
        self.canton_bridge_contracts.insert(chain_id, contract_address.into());
    }

    /// Register a chain bridge contract
    pub fn register_chain_bridge(&self, chain_id: ChainId, contract_address: impl Into<String>) {
        self.chain_bridge_contracts.insert(chain_id, contract_address.into());
    }

    /// Get Canton bridge contract address
    pub fn get_canton_bridge(&self, chain_id: &ChainId) -> OmniChainResult<String> {
        self.canton_bridge_contracts
            .get(chain_id)
            .map(|entry| entry.value().clone())
            .ok_or_else(|| OmniChainError::BridgeError(format!(
                "Canton bridge contract not found for chain: {}",
                chain_id
            )))
    }

    /// Get chain bridge contract address
    pub fn get_chain_bridge(&self, chain_id: &ChainId) -> OmniChainResult<String> {
        self.chain_bridge_contracts
            .get(chain_id)
            .map(|entry| entry.value().clone())
            .ok_or_else(|| OmniChainError::BridgeError(format!(
                "Chain bridge contract not found for chain: {}",
                chain_id
            )))
    }

    /// Lock asset on Canton
    pub async fn lock_on_canton(
        &self,
        canton_wallet: &Arc<dyn CantonWallet>,
        asset: CantonAsset,
        target_chain: ChainId,
        recipient: ChainAddress,
    ) -> OmniChainResult<LockReceipt> {
        let bridge_contract = self.get_canton_bridge(&target_chain)?;

        let tx_id = canton_wallet
            .lock_asset_on_bridge(asset.clone(), target_chain.clone(), recipient.clone(), bridge_contract)
            .await?;

        Ok(LockReceipt {
            tx_id,
            asset,
            target_chain,
            recipient,
            timestamp: chrono::Utc::now(),
            lock_contract: bridge_contract,
        })
    }

    /// Lock asset on chain
    pub async fn lock_on_chain(
        &self,
        chain_wallet: &Arc<dyn ChainWallet>,
        asset: ChainAsset,
        target_chain: ChainId,
        recipient: ChainAddress,
    ) -> OmniChainResult<LockReceipt> {
        let tx_id = chain_wallet
            .lock_asset(asset.clone(), target_chain, recipient)
            .await?;

        Ok(LockReceipt {
            tx_id,
            asset: CantonAsset {
                asset_id: format!("{}:{}", asset.chain_id, asset.token_id),
                amount: asset.amount,
                owner_party_id: asset.owner_address,
            },
            target_chain,
            recipient,
            timestamp: chrono::Utc::now(),
            lock_contract: self.get_chain_bridge(&asset.chain_id)?,
        })
    }

    /// Release asset on Canton
    pub async fn release_on_canton(
        &self,
        canton_wallet: &Arc<dyn CantonWallet>,
        proof: Vec<u8>,
        recipient: PartyId,
    ) -> OmniChainResult<ReleaseReceipt> {
        let tx_id = canton_wallet
            .release_asset_from_bridge(proof, recipient.clone())
            .await?;

        Ok(ReleaseReceipt {
            tx_id,
            asset: ChainAsset {
                chain_id: ChainId::Canton,
                token_id: "native".to_string(),
                amount: "0".to_string(),
                owner_address: recipient.to_string(),
            },
            source_chain: ChainId::Canton,
            recipient: ChainAddress::new(ChainId::Canton, recipient.to_string()),
            timestamp: chrono::Utc::now(),
            release_contract: self.get_canton_bridge(&ChainId::Canton)?,
        })
    }

    /// Release asset on chain
    pub async fn release_on_chain(
        &self,
        chain_wallet: &Arc<dyn ChainWallet>,
        proof: Vec<u8>,
        recipient: ChainAddress,
    ) -> OmniChainResult<ReleaseReceipt> {
        let tx_id = chain_wallet
            .release_asset(proof, recipient.clone())
            .await?;

        Ok(ReleaseReceipt {
            tx_id,
            asset: ChainAsset {
                chain_id: chain_wallet.chain_id(),
                token_id: "native".to_string(),
                amount: "0".to_string(),
                owner_address: recipient.address.clone(),
            },
            source_chain: chain_wallet.chain_id(),
            recipient,
            timestamp: chrono::Utc::now(),
            release_contract: self.get_chain_bridge(&chain_wallet.chain_id())?,
        })
    }

    /// Generate lock proof
    pub async fn generate_lock_proof(&self, receipt: &LockReceipt) -> OmniChainResult<Vec<u8>> {
        let proof_data = serde_json::to_vec(receipt)
            .map_err(|e| OmniChainError::ProofGenerationFailed(e.to_string()))?;

        Ok(proof_data)
    }

    /// Verify lock proof
    pub async fn verify_lock_proof(&self, proof: &[u8]) -> OmniChainResult<LockReceipt> {
        serde_json::from_slice(proof)
            .map_err(|e| OmniChainError::ProofVerificationFailed(e.to_string()))
    }
}

/// Bridge configuration
#[derive(Debug, Clone)]
pub struct BridgeConfig {
    /// Confirmation timeout in seconds
    pub confirmation_timeout: u64,
    /// Maximum retry attempts
    pub max_retry_attempts: u32,
    /// Retry delay in milliseconds
    pub retry_delay_ms: u64,
    /// Enable proof verification
    pub enable_proof_verification: bool,
}

impl BridgeConfig {
    /// Create a new bridge configuration
    pub fn new() -> Self {
        Self {
            confirmation_timeout: 300,
            max_retry_attempts: 3,
            retry_delay_ms: 1000,
            enable_proof_verification: true,
        }
    }

    /// Set confirmation timeout
    pub fn with_confirmation_timeout(mut self, timeout: u64) -> Self {
        self.confirmation_timeout = timeout;
        self
    }

    /// Set maximum retry attempts
    pub fn with_max_retry_attempts(mut self, attempts: u32) -> Self {
        self.max_retry_attempts = attempts;
        self
    }

    /// Set retry delay
    pub fn with_retry_delay(mut self, delay: u64) -> Self {
        self.retry_delay_ms = delay;
        self
    }

    /// Enable or disable proof verification
    pub fn with_proof_verification(mut self, enabled: bool) -> Self {
        self.enable_proof_verification = enabled;
        self
    }
}

impl Default for BridgeConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// Canton wallet trait for bridge operations
#[async_trait::async_trait]
pub trait CantonWallet: Send + Sync {
    /// Lock asset on bridge
    async fn lock_asset_on_bridge(
        &self,
        asset: CantonAsset,
        target_chain: ChainId,
        recipient: ChainAddress,
        bridge_contract: String,
    ) -> OmniChainResult<String>;

    /// Release asset from bridge
    async fn release_asset_from_bridge(
        &self,
        proof: Vec<u8>,
        recipient: PartyId,
    ) -> OmniChainResult<String>;
}
