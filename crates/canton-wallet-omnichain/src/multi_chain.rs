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

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use crate::error::{OmniChainError, OmniChainResult};
use crate::types::{CantonAsset, ChainAddress, ChainId, CrossChainTx};
use crate::bridge::{BridgeManager, LockReceipt, ReleaseReceipt};
use crate::chain::ChainWallet;
use canton_wallet_core::{CantonWallet, PartyId};

/// Multi-chain wallet
pub struct MultiChainWallet {
    /// Canton wallet
    canton_wallet: Arc<dyn CantonWallet>,
    /// Chain wallets
    chain_wallets: Arc<RwLock<HashMap<ChainId, Box<dyn ChainWallet>>>>,
    /// Bridge manager
    bridge_manager: Arc<BridgeManager>,
}

impl MultiChainWallet {
    /// Create a new multi-chain wallet
    pub fn new(
        canton_wallet: Arc<dyn CantonWallet>,
        bridge_manager: Arc<BridgeManager>,
    ) -> Self {
        Self {
            canton_wallet,
            chain_wallets: Arc::new(RwLock::new(HashMap::new())),
            bridge_manager,
        }
    }

    /// Add a chain wallet
    pub async fn add_chain_wallet(&self, chain_id: ChainId, wallet: Box<dyn ChainWallet>) {
        let mut chain_wallets = self.chain_wallets.write().await;
        chain_wallets.insert(chain_id, wallet);
        info!("Added chain wallet for {:?}", chain_id);
    }

    /// Get chain wallet
    pub async fn get_chain_wallet(&self, chain_id: ChainId) -> Option<Box<dyn ChainWallet>> {
        let chain_wallets = self.chain_wallets.read().await;
        chain_wallets.get(&chain_id).map(|w| w.clone_box())
    }

    /// Get Canton wallet
    pub fn canton_wallet(&self) -> Arc<dyn CantonWallet> {
        self.canton_wallet.clone()
    }

    /// Get bridge manager
    pub fn bridge_manager(&self) -> Arc<BridgeManager> {
        self.bridge_manager.clone()
    }

    /// Transfer asset from Canton to another chain
    pub async fn transfer_to_chain(
        &self,
        asset: CantonAsset,
        target_chain: ChainId,
        recipient: ChainAddress,
    ) -> OmniChainResult<CrossChainTx> {
        debug!(
            "Transferring asset {} from Canton to {:?}",
            asset.asset_id, target_chain
        );

        // Lock asset on Canton
        let lock_receipt = self
            .bridge_manager
            .lock_on_canton(asset.clone(), target_chain.clone(), recipient.clone())
            .await?;

        // Generate proof
        let proof = self
            .bridge_manager
            .generate_lock_proof(&lock_receipt)
            .await?;

        // Release on target chain
        let release_receipt = self
            .bridge_manager
            .release_on_chain(target_chain, proof, recipient)
            .await?;

        let tx = CrossChainTx::new(
            lock_receipt.tx_id.clone(),
            release_receipt.tx_id,
            asset,
            ChainId::Canton,
            target_chain,
        );

        info!(
            "Completed transfer from Canton to {:?}: {}",
            target_chain, tx.canton_tx_id
        );

        Ok(tx)
    }

    /// Transfer asset from another chain to Canton
    pub async fn transfer_from_chain(
        &self,
        asset: CantonAsset,
        source_chain: ChainId,
        recipient: PartyId,
    ) -> OmniChainResult<CrossChainTx> {
        debug!(
            "Transferring asset {} from {:?} to Canton",
            asset.asset_id, source_chain
        );

        // Lock on source chain
        let lock_receipt = self
            .bridge_manager
            .lock_on_chain(
                source_chain.clone(),
                asset.clone(),
                ChainId::Canton,
                ChainAddress::Canton(recipient.to_string()),
            )
            .await?;

        // Generate proof
        let proof = self
            .bridge_manager
            .generate_lock_proof(&lock_receipt)
            .await?;

        // Release on Canton
        let release_receipt = self
            .bridge_manager
            .release_on_canton(proof, recipient)
            .await?;

        let tx = CrossChainTx::new(
            release_receipt.tx_id.clone(),
            lock_receipt.tx_id,
            asset,
            source_chain,
            ChainId::Canton,
        );

        info!(
            "Completed transfer from {:?} to Canton: {}",
            source_chain, tx.canton_tx_id
        );

        Ok(tx)
    }

    /// Transfer asset between two non-Canton chains
    pub async fn transfer_between_chains(
        &self,
        asset: CantonAsset,
        source_chain: ChainId,
        target_chain: ChainId,
        recipient: ChainAddress,
    ) -> OmniChainResult<CrossChainTx> {
        debug!(
            "Transferring asset {} from {:?} to {:?}",
            asset.asset_id, source_chain, target_chain
        );

        // Lock on source chain
        let lock_receipt = self
            .bridge_manager
            .lock_on_chain(
                source_chain.clone(),
                asset.clone(),
                target_chain.clone(),
                recipient.clone(),
            )
            .await?;

        // Generate proof
        let proof = self
            .bridge_manager
            .generate_lock_proof(&lock_receipt)
            .await?;

        // Release on target chain
        let release_receipt = self
            .bridge_manager
            .release_on_chain(target_chain, proof, recipient)
            .await?;

        let tx = CrossChainTx::new(
            lock_receipt.tx_id.clone(),
            release_receipt.tx_id,
            asset,
            source_chain,
            target_chain,
        );

        info!(
            "Completed transfer from {:?} to {:?}: {}",
            source_chain, target_chain, tx.canton_tx_id
        );

        Ok(tx)
    }

    /// Get balance across all chains
    pub async fn get_balance(&self, asset_id: &str) -> OmniChainResult<HashMap<ChainId, u128>> {
        let mut balances = HashMap::new();

        // Get Canton balance
        let canton_balance = self.canton_wallet.get_balance(asset_id).await?;
        balances.insert(ChainId::Canton, canton_balance);

        // Get chain balances
        let chain_wallets = self.chain_wallets.read().await;
        for (chain_id, wallet) in chain_wallets.iter() {
            let balance = wallet.get_balance(asset_id).await?;
            balances.insert(chain_id.clone(), balance);
        }

        Ok(balances)
    }

    /// Get supported chains
    pub async fn supported_chains(&self) -> Vec<ChainId> {
        let chain_wallets = self.chain_wallets.read().await;
        let mut chains = chain_wallets.keys().cloned().collect::<Vec<_>>();
        chains.push(ChainId::Canton);
        chains.sort();
        chains
    }

    /// Check if chain is supported
    pub async fn is_chain_supported(&self, chain_id: ChainId) -> bool {
        if chain_id == ChainId::Canton {
            return true;
        }

        let chain_wallets = self.chain_wallets.read().await;
        chain_wallets.contains_key(&chain_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bridge::BridgeManager;
    use crate::proof::MockProofGenerator;
    use crate::proof::MockProofVerifier;
    use crate::chain::MockChainWallet;
    use canton_wallet_core::MockCantonWallet;

    #[tokio::test]
    async fn test_multi_chain_wallet_creation() {
        let canton_wallet = Arc::new(MockCantonWallet::new());
        let proof_generator = Arc::new(MockProofGenerator::new());
        let proof_verifier = Arc::new(MockProofVerifier::new());
        let bridge_manager = Arc::new(BridgeManager::new(
            canton_wallet.clone(),
            proof_generator,
            proof_verifier,
        ));

        let wallet = MultiChainWallet::new(canton_wallet, bridge_manager);
        assert_eq!(wallet.supported_chains().await.len(), 1);
    }

    #[tokio::test]
    async fn test_add_chain_wallet() {
        let canton_wallet = Arc::new(MockCantonWallet::new());
        let proof_generator = Arc::new(MockProofGenerator::new());
        let proof_verifier = Arc::new(MockProofVerifier::new());
        let bridge_manager = Arc::new(BridgeManager::new(
            canton_wallet.clone(),
            proof_generator,
            proof_verifier,
        ));

        let wallet = MultiChainWallet::new(canton_wallet, bridge_manager);
        let chain_wallet = Box::new(MockChainWallet::new());

        wallet.add_chain_wallet(ChainId::Ethereum, chain_wallet).await;
        assert!(wallet.is_chain_supported(ChainId::Ethereum).await);
    }

    #[tokio::test]
    async fn test_supported_chains() {
        let canton_wallet = Arc::new(MockCantonWallet::new());
        let proof_generator = Arc::new(MockProofGenerator::new());
        let proof_verifier = Arc::new(MockProofVerifier::new());
        let bridge_manager = Arc::new(BridgeManager::new(
            canton_wallet.clone(),
            proof_generator,
            proof_verifier,
        ));

        let wallet = MultiChainWallet::new(canton_wallet, bridge_manager);
        let chain_wallet = Box::new(MockChainWallet::new());

        wallet.add_chain_wallet(ChainId::Ethereum, chain_wallet).await;
        let chains = wallet.supported_chains().await;
        assert_eq!(chains.len(), 2);
        assert!(chains.contains(&ChainId::Canton));
        assert!(chains.contains(&ChainId::Ethereum));
    }

    #[tokio::test]
    async fn test_is_chain_supported() {
        let canton_wallet = Arc::new(MockCantonWallet::new());
        let proof_generator = Arc::new(MockProofGenerator::new());
        let proof_verifier = Arc::new(MockProofVerifier::new());
        let bridge_manager = Arc::new(BridgeManager::new(
            canton_wallet.clone(),
            proof_generator,
            proof_verifier,
        ));

        let wallet = MultiChainWallet::new(canton_wallet, bridge_manager);

        assert!(wallet.is_chain_supported(ChainId::Canton).await);
        assert!(!wallet.is_chain_supported(ChainId::Ethereum).await);

        let chain_wallet = Box::new(MockChainWallet::new());
        wallet.add_chain_wallet(ChainId::Ethereum, chain_wallet).await;

        assert!(wallet.is_chain_supported(ChainId::Ethereum).await);
    }
}
