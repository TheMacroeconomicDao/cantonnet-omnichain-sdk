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
    types::{ChainId, ChainAddress, CantonAsset, ChainAsset, CrossChainTx, CrossChainTxStatus},
    adapter::ChainWallet,
    bridge::{BridgeManager, CantonWallet},
    proof::{ProofGenerator, LockProof},
};
use canton_wallet_core::PartyId;

/// Multi-chain wallet for managing assets across different blockchain networks
pub struct MultiChainWallet {
    canton_wallet: Arc<dyn CantonWallet>,
    chain_wallets: DashMap<ChainId, Arc<dyn ChainWallet>>,
    bridge_manager: Arc<BridgeManager>,
    proof_generator: ProofGenerator,
}

impl MultiChainWallet {
    /// Create a new multi-chain wallet
    pub fn new(
        canton_wallet: Arc<dyn CantonWallet>,
        bridge_manager: Arc<BridgeManager>,
    ) -> Self {
        Self {
            canton_wallet,
            chain_wallets: DashMap::new(),
            bridge_manager,
            proof_generator: ProofGenerator::new(),
        }
    }

    /// Set proof generator
    pub fn with_proof_generator(mut self, generator: ProofGenerator) -> Self {
        self.proof_generator = generator;
        self
    }

    /// Add a chain wallet
    pub fn add_chain(&self, chain_id: ChainId, wallet: Arc<dyn ChainWallet>) {
        self.chain_wallets.insert(chain_id, wallet);
    }

    /// Remove a chain wallet
    pub fn remove_chain(&self, chain_id: &ChainId) {
        self.chain_wallets.remove(chain_id);
    }

    /// Get chain wallet
    pub fn get_chain_wallet(&self, chain_id: &ChainId) -> OmniChainResult<Arc<dyn ChainWallet>> {
        self.chain_wallets
            .get(chain_id)
            .map(|entry| entry.value().clone())
            .ok_or_else(|| OmniChainError::AdapterNotFound(chain_id.to_string()))
    }

    /// Get Canton wallet
    pub fn canton_wallet(&self) -> &Arc<dyn CantonWallet> {
        &self.canton_wallet
    }

    /// Get bridge manager
    pub fn bridge_manager(&self) -> &Arc<BridgeManager> {
        &self.bridge_manager
    }

    /// Get supported chains
    pub fn supported_chains(&self) -> Vec<ChainId> {
        self.chain_wallets
            .iter()
            .map(|entry| entry.key().clone())
            .collect()
    }

    /// Check if chain is supported
    pub fn supports_chain(&self, chain_id: &ChainId) -> bool {
        self.chain_wallets.contains_key(chain_id)
    }

    /// Transfer asset from Canton to another chain
    pub async fn transfer_to_chain(
        &self,
        asset: CantonAsset,
        target_chain: ChainId,
        recipient: ChainAddress,
    ) -> OmniChainResult<CrossChainTx> {
        if !self.supports_chain(&target_chain) {
            return Err(OmniChainError::UnsupportedChain(target_chain.to_string()));
        }

        let mut cross_chain_tx = CrossChainTx {
            canton_tx_id: String::new(),
            target_tx_id: String::new(),
            asset: asset.clone(),
            source_chain: ChainId::Canton,
            target_chain: target_chain.clone(),
            timestamp: chrono::Utc::now(),
            status: CrossChainTxStatus::Initiated,
        };

        cross_chain_tx.status = CrossChainTxStatus::Locked;

        let lock_receipt = self
            .bridge_manager
            .lock_on_canton(&self.canton_wallet, asset.clone(), target_chain.clone(), recipient.clone())
            .await?;

        cross_chain_tx.canton_tx_id = lock_receipt.tx_id.clone();

        cross_chain_tx.status = CrossChainTxStatus::ProofGenerated;

        let proof = self
            .bridge_manager
            .generate_lock_proof(&lock_receipt)
            .await?;

        cross_chain_tx.status = CrossChainTxStatus::Released;

        let target_wallet = self.get_chain_wallet(&target_chain)?;
        let release_receipt = self
            .bridge_manager
            .release_on_chain(&target_wallet, proof, recipient)
            .await?;

        cross_chain_tx.target_tx_id = release_receipt.tx_id;
        cross_chain_tx.status = CrossChainTxStatus::Completed;

        Ok(cross_chain_tx)
    }

    /// Transfer asset from another chain to Canton
    pub async fn transfer_from_chain(
        &self,
        asset: ChainAsset,
        source_chain: ChainId,
        recipient: PartyId,
    ) -> OmniChainResult<CrossChainTx> {
        if !self.supports_chain(&source_chain) {
            return Err(OmniChainError::UnsupportedChain(source_chain.to_string()));
        }

        let mut cross_chain_tx = CrossChainTx {
            canton_tx_id: String::new(),
            target_tx_id: String::new(),
            asset: asset.to_canton_asset(recipient.clone()),
            source_chain: source_chain.clone(),
            target_chain: ChainId::Canton,
            timestamp: chrono::Utc::now(),
            status: CrossChainTxStatus::Initiated,
        };

        cross_chain_tx.status = CrossChainTxStatus::Locked;

        let source_wallet = self.get_chain_wallet(&source_chain)?;
        let lock_receipt = self
            .bridge_manager
            .lock_on_chain(&source_wallet, asset.clone(), ChainId::Canton, ChainAddress::new(ChainId::Canton, recipient.clone()))
            .await?;

        cross_chain_tx.target_tx_id = lock_receipt.tx_id.clone();

        cross_chain_tx.status = CrossChainTxStatus::ProofGenerated;

        let proof = self
            .bridge_manager
            .generate_lock_proof(&lock_receipt)
            .await?;

        cross_chain_tx.status = CrossChainTxStatus::Released;

        let release_receipt = self
            .bridge_manager
            .release_on_canton(&self.canton_wallet, proof, recipient)
            .await?;

        cross_chain_tx.canton_tx_id = release_receipt.tx_id;
        cross_chain_tx.status = CrossChainTxStatus::Completed;

        Ok(cross_chain_tx)
    }

    /// Transfer asset between two non-Canton chains
    pub async fn transfer_between_chains(
        &self,
        asset: ChainAsset,
        source_chain: ChainId,
        target_chain: ChainId,
        recipient: ChainAddress,
    ) -> OmniChainResult<CrossChainTx> {
        if !self.supports_chain(&source_chain) {
            return Err(OmniChainError::UnsupportedChain(source_chain.to_string()));
        }

        if !self.supports_chain(&target_chain) {
            return Err(OmniChainError::UnsupportedChain(target_chain.to_string()));
        }

        let mut cross_chain_tx = CrossChainTx {
            canton_tx_id: String::new(),
            target_tx_id: String::new(),
            asset: asset.to_canton_asset(recipient.address.clone()),
            source_chain: source_chain.clone(),
            target_chain: target_chain.clone(),
            timestamp: chrono::Utc::now(),
            status: CrossChainTxStatus::Initiated,
        };

        cross_chain_tx.status = CrossChainTxStatus::Locked;

        let source_wallet = self.get_chain_wallet(&source_chain)?;
        let lock_receipt = self
            .bridge_manager
            .lock_on_chain(&source_wallet, asset.clone(), target_chain.clone(), recipient.clone())
            .await?;

        cross_chain_tx.target_tx_id = lock_receipt.tx_id.clone();

        cross_chain_tx.status = CrossChainTxStatus::ProofGenerated;

        let proof = self
            .bridge_manager
            .generate_lock_proof(&lock_receipt)
            .await?;

        cross_chain_tx.status = CrossChainTxStatus::Released;

        let target_wallet = self.get_chain_wallet(&target_chain)?;
        let release_receipt = self
            .bridge_manager
            .release_on_chain(&target_wallet, proof, recipient)
            .await?;

        cross_chain_tx.canton_tx_id = release_receipt.tx_id;
        cross_chain_tx.status = CrossChainTxStatus::Completed;

        Ok(cross_chain_tx)
    }

    /// Get balance across all chains
    pub async fn get_total_balance(&self, asset_id: Option<String>) -> OmniChainResult<String> {
        let mut total = "0".to_string();

        for entry in self.chain_wallets.iter() {
            let wallet = entry.value();
            let balance = wallet.balance(asset_id.clone()).await?;
            total = self.add_balances(&total, &balance)?;
        }

        Ok(total)
    }

    /// Get balance on specific chain
    pub async fn get_chain_balance(
        &self,
        chain_id: &ChainId,
        asset_id: Option<String>,
    ) -> OmniChainResult<String> {
        let wallet = self.get_chain_wallet(chain_id)?;
        wallet.balance(asset_id).await
    }

    /// Add two balance strings
    fn add_balances(&self, a: &str, b: &str) -> OmniChainResult<String> {
        let a_val: u128 = a.parse().unwrap_or(0);
        let b_val: u128 = b.parse().unwrap_or(0);
        Ok((a_val + b_val).to_string())
    }

    /// Generate lock proof with custom block info
    pub async fn generate_lock_proof_with_block(
        &self,
        receipt: crate::types::LockReceipt,
        block_number: u64,
        block_hash: impl Into<String>,
    ) -> OmniChainResult<LockProof> {
        self.proof_generator
            .generate_lock_proof(receipt, block_number, block_hash)
    }
}

impl Clone for MultiChainWallet {
    fn clone(&self) -> Self {
        Self {
            canton_wallet: Arc::clone(&self.canton_wallet),
            chain_wallets: self.chain_wallets.clone(),
            bridge_manager: Arc::clone(&self.bridge_manager),
            proof_generator: self.proof_generator.clone(),
        }
    }
}
