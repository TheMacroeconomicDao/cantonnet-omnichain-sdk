//! Multi-chain wallet implementation

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

use canton_wallet_core::{
    WalletError, PartyId, ContractId, Identifier, DamlValue, DamlRecord,
};

use super::bridge::{
    BridgeManager, BridgeResult, CantonWalletBridge, ChainWalletBridge,
    LockReceipt, ReleaseReceipt, LockProof,
};
use super::types::{
    ChainId, ChainAddress, CantonAsset, ChainAsset, ChainWallet,
    CrossChainTx, TransferStatus, TransferDirection,
};

/// Multi-chain wallet error
#[derive(Debug, thiserror::Error)]
pub enum MultiChainError {
    #[error("Chain wallet not found: {0}")]
    ChainWalletNotFound(ChainId),

    #[error("Canton wallet not configured")]
    CantonWalletNotConfigured,

    #[error("Bridge manager not configured")]
    BridgeManagerNotConfigured,

    #[error("Transfer failed: {0}")]
    TransferFailed(String),

    #[error("Unsupported operation: {0}")]
    UnsupportedOperation(String),

    #[error("Wallet error: {0}")]
    WalletError(#[from] WalletError),

    #[error("Bridge error: {0}")]
    BridgeError(#[from] super::bridge::BridgeError),
}

/// Multi-chain wallet result type
pub type MultiChainResult<T> = Result<T, MultiChainError>;

/// Multi-chain wallet configuration
#[derive(Debug, Clone)]
pub struct MultiChainWalletConfig {
    /// Canton wallet
    pub canton_wallet: Option<Arc<dyn CantonWalletBridge>>,
    /// Bridge manager
    pub bridge_manager: Option<Arc<BridgeManager>>,
    /// Default timeout for operations in seconds
    pub default_timeout: u64,
    /// Maximum concurrent transfers
    pub max_concurrent_transfers: usize,
}

impl Default for MultiChainWalletConfig {
    fn default() -> Self {
        Self {
            canton_wallet: None,
            bridge_manager: None,
            default_timeout: 300, // 5 minutes
            max_concurrent_transfers: 10,
        }
    }
}

/// Multi-chain wallet
pub struct MultiChainWallet {
    /// Canton wallet
    canton_wallet: Option<Arc<dyn CantonWalletBridge>>,
    /// Chain wallets
    chain_wallets: Arc<RwLock<HashMap<ChainId, Box<dyn ChainWallet>>>>,
    /// Bridge manager
    bridge_manager: Option<Arc<BridgeManager>>,
    /// Active transfers
    active_transfers: Arc<RwLock<HashMap<String, CrossChainTx>>>,
    /// Configuration
    config: MultiChainWalletConfig,
}

impl MultiChainWallet {
    /// Create a new multi-chain wallet
    pub fn new(config: MultiChainWalletConfig) -> Self {
        Self {
            canton_wallet: config.canton_wallet.clone(),
            chain_wallets: Arc::new(RwLock::new(HashMap::new())),
            bridge_manager: config.bridge_manager.clone(),
            active_transfers: Arc::new(RwLock::new(HashMap::new())),
            config,
        }
    }

    /// Create with default configuration
    pub fn default() -> Self {
        Self::new(MultiChainWalletConfig::default())
    }

    /// Set Canton wallet
    pub fn with_canton_wallet(mut self, wallet: Arc<dyn CantonWalletBridge>) -> Self {
        self.canton_wallet = Some(wallet);
        self
    }

    /// Set bridge manager
    pub fn with_bridge_manager(mut self, manager: Arc<BridgeManager>) -> Self {
        self.bridge_manager = Some(manager);
        self
    }

    /// Add chain wallet
    pub async fn add_chain(&self, chain_id: ChainId, wallet: Box<dyn ChainWallet>) -> MultiChainResult<()> {
        info!("Adding chain wallet for: {}", chain_id);

        let mut wallets = self.chain_wallets.write().await;
        wallets.insert(chain_id.clone(), wallet);

        debug!("Chain wallet added for: {}", chain_id);

        Ok(())
    }

    /// Remove chain wallet
    pub async fn remove_chain(&self, chain_id: &ChainId) -> MultiChainResult<()> {
        info!("Removing chain wallet for: {}", chain_id);

        let mut wallets = self.chain_wallets.write().await;
        wallets.remove(chain_id);

        debug!("Chain wallet removed for: {}", chain_id);

        Ok(())
    }

    /// Get chain wallet
    pub async fn get_chain_wallet(&self, chain_id: &ChainId) -> MultiChainResult<Arc<dyn ChainWallet>> {
        let wallets = self.chain_wallets.read().await;

        wallets
            .get(chain_id)
            .map(|w| Arc::clone(w) as Arc<dyn ChainWallet>)
            .ok_or_else(|| MultiChainError::ChainWalletNotFound(chain_id.clone()))
    }

    /// Check if chain is supported
    pub async fn supports_chain(&self, chain_id: &ChainId) -> bool {
        let wallets = self.chain_wallets.read().await;
        wallets.contains_key(chain_id)
    }

    /// Get supported chains
    pub async fn supported_chains(&self) -> Vec<ChainId> {
        let wallets = self.chain_wallets.read().await;
        wallets.keys().cloned().collect()
    }

    /// Transfer asset from Canton to another chain
    pub async fn transfer_to_chain(
        &self,
        asset: CantonAsset,
        target_chain: ChainId,
        recipient: ChainAddress,
    ) -> MultiChainResult<CrossChainTx> {
        info!(
            "Transferring asset from Canton to {}: {}",
            target_chain, asset.contract_id
        );

        // Get Canton wallet
        let canton_wallet = self
            .canton_wallet
            .as_ref()
            .ok_or(MultiChainError::CantonWalletNotConfigured)?;

        // Get bridge manager
        let bridge_manager = self
            .bridge_manager
            .as_ref()
            .ok_or(MultiChainError::BridgeManagerNotConfigured)?;

        // Lock asset on Canton
        let lock_receipt = bridge_manager
            .lock_on_canton(canton_wallet.as_ref(), asset.clone(), target_chain.clone(), recipient.clone())
            .await?;

        // Generate proof
        let proof = bridge_manager
            .generate_lock_proof(&lock_receipt)
            .await?;

        // Get target chain wallet
        let target_wallet = self.get_chain_wallet(&target_chain).await?;

        // Release on target chain
        let release_receipt = bridge_manager
            .release_on_chain(
                target_wallet.as_ref(),
                proof,
                recipient,
            )
            .await?;

        // Create cross-chain transaction
        let tx = CrossChainTx::new(
            lock_receipt.tx_id.clone(),
            release_receipt.tx_id.clone(),
            asset,
            ChainId::Canton,
            target_chain,
        );

        // Store active transfer
        let mut transfers = self.active_transfers.write().await;
        transfers.insert(lock_receipt.tx_id.clone(), tx.clone());

        info!(
            "Transfer completed: Canton -> {}: {}",
            target_chain, lock_receipt.tx_id
        );

        Ok(tx)
    }

    /// Transfer asset from another chain to Canton
    pub async fn transfer_from_chain(
        &self,
        asset: ChainAsset,
        source_chain: ChainId,
        recipient: PartyId,
    ) -> MultiChainResult<CrossChainTx> {
        info!(
            "Transferring asset from {} to Canton: {}",
            source_chain, asset.owner
        );

        // Get Canton wallet
        let canton_wallet = self
            .canton_wallet
            .as_ref()
            .ok_or(MultiChainError::CantonWalletNotConfigured)?;

        // Get bridge manager
        let bridge_manager = self
            .bridge_manager
            .as_ref()
            .ok_or(MultiChainError::BridgeManagerNotConfigured)?;

        // Get source chain wallet
        let source_wallet = self.get_chain_wallet(&source_chain).await?;

        // Lock on source chain
        let lock_receipt = bridge_manager
            .lock_on_chain(
                source_wallet.as_ref(),
                asset.clone(),
                ChainId::Canton,
                recipient.clone(),
            )
            .await?;

        // Generate proof
        let proof = bridge_manager
            .generate_lock_proof(&lock_receipt)
            .await?;

        // Release on Canton
        let release_receipt = bridge_manager
            .release_on_canton(
                canton_wallet.as_ref(),
                proof,
                recipient,
            )
            .await?;

        // Create cross-chain transaction
        let tx = CrossChainTx::new(
            lock_receipt.tx_id.clone(),
            release_receipt.tx_id.clone(),
            CantonAsset::from_chain_asset(asset),
            source_chain,
            ChainId::Canton,
        );

        // Store active transfer
        let mut transfers = self.active_transfers.write().await;
        transfers.insert(lock_receipt.tx_id.clone(), tx.clone());

        info!(
            "Transfer completed: {} -> Canton: {}",
            source_chain, lock_receipt.tx_id
        );

        Ok(tx)
    }

    /// Transfer asset between two non-Canton chains
    pub async fn transfer_between_chains(
        &self,
        asset: ChainAsset,
        source_chain: ChainId,
        target_chain: ChainId,
        recipient: ChainAddress,
    ) -> MultiChainResult<CrossChainTx> {
        info!(
            "Transferring asset from {} to {}: {}",
            source_chain, target_chain, asset.owner
        );

        // Get bridge manager
        let bridge_manager = self
            .bridge_manager
            .as_ref()
            .ok_or(MultiChainError::BridgeManagerNotConfigured)?;

        // Get source chain wallet
        let source_wallet = self.get_chain_wallet(&source_chain).await?;

        // Lock on source chain
        let lock_receipt = bridge_manager
            .lock_on_chain(
                source_wallet.as_ref(),
                asset.clone(),
                target_chain.clone(),
                PartyId::new_unchecked(&recipient.display),
            )
            .await?;

        // Generate proof
        let proof = bridge_manager
            .generate_lock_proof(&lock_receipt)
            .await?;

        // Get target chain wallet
        let target_wallet = self.get_chain_wallet(&target_chain).await?;

        // Release on target chain
        let release_receipt = bridge_manager
            .release_on_chain(
                target_wallet.as_ref(),
                proof,
                recipient,
            )
            .await?;

        // Create cross-chain transaction
        let tx = CrossChainTx::new(
            lock_receipt.tx_id.clone(),
            release_receipt.tx_id.clone(),
            CantonAsset::from_chain_asset(asset),
            source_chain,
            target_chain,
        );

        // Store active transfer
        let mut transfers = self.active_transfers.write().await;
        transfers.insert(lock_receipt.tx_id.clone(), tx.clone());

        info!(
            "Transfer completed: {} -> {}: {}",
            source_chain, target_chain, lock_receipt.tx_id
        );

        Ok(tx)
    }

    /// Get transfer status
    pub async fn get_transfer(&self, tx_id: &str) -> MultiChainResult<Option<CrossChainTx>> {
        let transfers = self.active_transfers.read().await;
        Ok(transfers.get(tx_id).cloned())
    }

    /// Get all active transfers
    pub async fn get_active_transfers(&self) -> MultiChainResult<Vec<CrossChainTx>> {
        let transfers = self.active_transfers.read().await;
        Ok(transfers.values().cloned().collect())
    }

    /// Get transfers by status
    pub async fn get_transfers_by_status(
        &self,
        status: TransferStatus,
    ) -> MultiChainResult<Vec<CrossChainTx>> {
        let transfers = self.active_transfers.read().await;
        Ok(transfers
            .values()
            .filter(|t| t.status == status)
            .cloned()
            .collect())
    }

    /// Get transfers by direction
    pub async fn get_transfers_by_direction(
        &self,
        direction: TransferDirection,
    ) -> MultiChainResult<Vec<CrossChainTx>> {
        let transfers = self.active_transfers.read().await;
        Ok(transfers
            .values()
            .filter(|t| t.direction == direction)
            .cloned()
            .collect())
    }

    /// Get transfers by chain
    pub async fn get_transfers_by_chain(
        &self,
        chain_id: &ChainId,
    ) -> MultiChainResult<Vec<CrossChainTx>> {
        let transfers = self.active_transfers.read().await;
        Ok(transfers
            .values()
            .filter(|t| t.source_chain == *chain_id || t.target_chain == *chain_id)
            .cloned()
            .collect())
    }

    /// Cancel transfer
    pub async fn cancel_transfer(&self, tx_id: &str) -> MultiChainResult<()> {
        info!("Cancelling transfer: {}", tx_id);

        let mut transfers = self.active_transfers.write().await;

        if let Some(transfer) = transfers.get_mut(tx_id) {
            transfer.mark_failed("Transfer cancelled by user".to_string());
            info!("Transfer cancelled: {}", tx_id);
            Ok(())
        } else {
            Err(MultiChainError::TransferFailed(format!(
                "Transfer not found: {}",
                tx_id
            )))
        }
    }

    /// Retry failed transfer
    pub async fn retry_transfer(&self, tx_id: &str) -> MultiChainResult<CrossChainTx> {
        info!("Retrying transfer: {}", tx_id);

        let transfer = {
            let transfers = self.active_transfers.read().await;
            transfers
                .get(tx_id)
                .cloned()
                .ok_or_else(|| MultiChainError::TransferFailed(format!("Transfer not found: {}", tx_id)))?
        };

        if transfer.status != TransferStatus::Failed {
            return Err(MultiChainError::TransferFailed(
                "Only failed transfers can be retried".to_string(),
            ));
        }

        // Remove old transfer
        let mut transfers = self.active_transfers.write().await;
        transfers.remove(tx_id);

        // Retry based on direction
        match transfer.direction {
            TransferDirection::Outbound => {
                // Canton -> Chain
                let canton_wallet = self
                    .canton_wallet
                    .as_ref()
                    .ok_or(MultiChainError::CantonWalletNotConfigured)?;

                let bridge_manager = self
                    .bridge_manager
                    .as_ref()
                    .ok_or(MultiChainError::BridgeManagerNotConfigured)?;

                let recipient = ChainAddress::new(
                    transfer.target_chain.clone(),
                    vec![],
                    "retry-recipient".to_string(),
                );

                self.transfer_to_chain(transfer.asset, transfer.target_chain, recipient)
                    .await
            }
            TransferDirection::Inbound => {
                // Chain -> Canton
                let asset = ChainAsset::new(
                    transfer.source_chain.clone(),
                    transfer.asset.amount,
                    ChainAddress::new(
                        transfer.source_chain.clone(),
                        vec![],
                        "retry-owner".to_string(),
                    ),
                );

                let recipient = PartyId::new_unchecked("retry-recipient");

                self.transfer_from_chain(asset, transfer.source_chain, recipient)
                    .await
            }
        }
    }

    /// Get wallet summary
    pub async fn summary(&self) -> MultiChainResult<MultiChainWalletSummary> {
        let supported_chains = self.supported_chains().await;
        let active_transfers = self.get_active_transfers().await?;

        let pending_count = active_transfers
            .iter()
            .filter(|t| t.status == TransferStatus::Pending)
            .count();

        let completed_count = active_transfers
            .iter()
            .filter(|t| t.status == TransferStatus::Completed)
            .count();

        let failed_count = active_transfers
            .iter()
            .filter(|t| t.status == TransferStatus::Failed)
            .count();

        Ok(MultiChainWalletSummary {
            supported_chains,
            active_transfers: active_transfers.len(),
            pending_transfers: pending_count,
            completed_transfers: completed_count,
            failed_transfers: failed_count,
        })
    }
}

/// Multi-chain wallet summary
#[derive(Debug, Clone)]
pub struct MultiChainWalletSummary {
    /// Supported chains
    pub supported_chains: Vec<ChainId>,
    /// Number of active transfers
    pub active_transfers: usize,
    /// Number of pending transfers
    pub pending_transfers: usize,
    /// Number of completed transfers
    pub completed_transfers: usize,
    /// Number of failed transfers
    pub failed_transfers: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_multi_chain_wallet_config_default() {
        let config = MultiChainWalletConfig::default();
        assert!(config.canton_wallet.is_none());
        assert!(config.bridge_manager.is_none());
        assert_eq!(config.default_timeout, 300);
        assert_eq!(config.max_concurrent_transfers, 10);
    }

    #[test]
    fn test_multi_chain_wallet_creation() {
        let wallet = MultiChainWallet::default();
        assert!(wallet.canton_wallet.is_none());
    }

    #[tokio::test]
    async fn test_supported_chains() {
        let wallet = MultiChainWallet::default();
        let chains = wallet.supported_chains().await;
        assert!(chains.is_empty());
    }

    #[tokio::test]
    async fn test_supports_chain() {
        let wallet = MultiChainWallet::default();
        assert!(!wallet.supports_chain(&ChainId::Ethereum).await);
    }

    #[tokio::test]
    async fn test_summary() {
        let wallet = MultiChainWallet::default();
        let summary = wallet.summary().await.unwrap();
        assert_eq!(summary.active_transfers, 0);
        assert_eq!(summary.pending_transfers, 0);
        assert_eq!(summary.completed_transfers, 0);
        assert_eq!(summary.failed_transfers, 0);
    }
}
