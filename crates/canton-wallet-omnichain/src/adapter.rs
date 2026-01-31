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

use async_trait::async_trait;
use crate::{error::OmniChainResult, types::{ChainId, ChainAddress, ChainAsset}};

/// Chain wallet trait for interacting with different blockchain networks
#[async_trait]
pub trait ChainWallet: Send + Sync {
    /// Get the chain ID
    fn chain_id(&self) -> ChainId;

    /// Get the wallet address
    async fn address(&self) -> OmniChainResult<ChainAddress>;

    /// Get balance
    async fn balance(&self, token_id: Option<String>) -> OmniChainResult<String>;

    /// Lock asset for cross-chain transfer
    async fn lock_asset(
        &self,
        asset: ChainAsset,
        target_chain: ChainId,
        recipient: ChainAddress,
    ) -> OmniChainResult<String>;

    /// Release asset from cross-chain transfer
    async fn release_asset(
        &self,
        proof: Vec<u8>,
        recipient: ChainAddress,
    ) -> OmniChainResult<String>;

    /// Get transaction status
    async fn get_transaction_status(&self, tx_id: &str) -> OmniChainResult<TransactionStatus>;
}

/// Transaction status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransactionStatus {
    /// Transaction pending
    Pending,
    /// Transaction confirmed
    Confirmed,
    /// Transaction failed
    Failed,
    /// Transaction unknown
    Unknown,
}

/// Chain adapter trait for creating chain-specific wallets
#[async_trait]
pub trait ChainAdapter: Send + Sync {
    /// Create a new chain wallet
    async fn create_wallet(&self, config: ChainConfig) -> OmniChainResult<Box<dyn ChainWallet>>;

    /// Get supported chain ID
    fn chain_id(&self) -> ChainId;

    /// Validate address
    fn validate_address(&self, address: &str) -> bool;

    /// Get chain-specific configuration
    fn chain_config(&self) -> ChainConfig;
}

/// Chain configuration
#[derive(Debug, Clone)]
pub struct ChainConfig {
    /// Chain ID
    pub chain_id: ChainId,
    /// RPC endpoint
    pub rpc_endpoint: String,
    /// WebSocket endpoint (optional)
    pub ws_endpoint: Option<String>,
    /// Chain ID (for EVM chains)
    pub evm_chain_id: Option<u64>,
    /// Bridge contract address
    pub bridge_contract: Option<String>,
    /// Gas price (optional)
    pub gas_price: Option<String>,
    /// Gas limit (optional)
    pub gas_limit: Option<u64>,
    /// Confirmation blocks
    pub confirmation_blocks: u64,
    /// Timeout in seconds
    pub timeout_seconds: u64,
}

impl ChainConfig {
    /// Create a new chain configuration
    pub fn new(chain_id: ChainId, rpc_endpoint: impl Into<String>) -> Self {
        Self {
            chain_id,
            rpc_endpoint: rpc_endpoint.into(),
            ws_endpoint: None,
            evm_chain_id: None,
            bridge_contract: None,
            gas_price: None,
            gas_limit: None,
            confirmation_blocks: 12,
            timeout_seconds: 300,
        }
    }

    /// Set WebSocket endpoint
    pub fn with_ws_endpoint(mut self, endpoint: impl Into<String>) -> Self {
        self.ws_endpoint = Some(endpoint.into());
        self
    }

    /// Set EVM chain ID
    pub fn with_evm_chain_id(mut self, chain_id: u64) -> Self {
        self.evm_chain_id = Some(chain_id);
        self
    }

    /// Set bridge contract address
    pub fn with_bridge_contract(mut self, address: impl Into<String>) -> Self {
        self.bridge_contract = Some(address.into());
        self
    }

    /// Set gas price
    pub fn with_gas_price(mut self, price: impl Into<String>) -> Self {
        self.gas_price = Some(price.into());
        self
    }

    /// Set gas limit
    pub fn with_gas_limit(mut self, limit: u64) -> Self {
        self.gas_limit = Some(limit);
        self
    }

    /// Set confirmation blocks
    pub fn with_confirmation_blocks(mut self, blocks: u64) -> Self {
        self.confirmation_blocks = blocks;
        self
    }

    /// Set timeout
    pub fn with_timeout(mut self, seconds: u64) -> Self {
        self.timeout_seconds = seconds;
        self
    }
}

/// Ethereum chain adapter
pub struct EthereumAdapter {
    config: ChainConfig,
}

impl EthereumAdapter {
    /// Create a new Ethereum adapter
    pub fn new(config: ChainConfig) -> Self {
        Self { config }
    }
}

#[async_trait]
impl ChainAdapter for EthereumAdapter {
    async fn create_wallet(&self, config: ChainConfig) -> OmniChainResult<Box<dyn ChainWallet>> {
        Ok(Box::new(EthereumWallet::new(config)))
    }

    fn chain_id(&self) -> ChainId {
        self.config.chain_id.clone()
    }

    fn validate_address(&self, address: &str) -> bool {
        address.starts_with("0x") && address.len() == 42
    }

    fn chain_config(&self) -> ChainConfig {
        self.config.clone()
    }
}

/// Ethereum wallet implementation
pub struct EthereumWallet {
    config: ChainConfig,
    address: ChainAddress,
}

impl EthereumWallet {
    /// Create a new Ethereum wallet
    pub fn new(config: ChainConfig) -> Self {
        let address = ChainAddress::ethereum("0x0000000000000000000000000000000000000000");
        Self { config, address }
    }
}

#[async_trait]
impl ChainWallet for EthereumWallet {
    fn chain_id(&self) -> ChainId {
        self.config.chain_id.clone()
    }

    async fn address(&self) -> OmniChainResult<ChainAddress> {
        Ok(self.address.clone())
    }

    async fn balance(&self, _token_id: Option<String>) -> OmniChainResult<String> {
        Ok("0".to_string())
    }

    async fn lock_asset(
        &self,
        _asset: ChainAsset,
        _target_chain: ChainId,
        _recipient: ChainAddress,
    ) -> OmniChainResult<String> {
        Ok("0x0000000000000000000000000000000000000000000000000000000000000000".to_string())
    }

    async fn release_asset(
        &self,
        _proof: Vec<u8>,
        _recipient: ChainAddress,
    ) -> OmniChainResult<String> {
        Ok("0x0000000000000000000000000000000000000000000000000000000000000000".to_string())
    }

    async fn get_transaction_status(&self, _tx_id: &str) -> OmniChainResult<TransactionStatus> {
        Ok(TransactionStatus::Confirmed)
    }
}

/// Cosmos chain adapter
pub struct CosmosAdapter {
    config: ChainConfig,
}

impl CosmosAdapter {
    /// Create a new Cosmos adapter
    pub fn new(config: ChainConfig) -> Self {
        Self { config }
    }
}

#[async_trait]
impl ChainAdapter for CosmosAdapter {
    async fn create_wallet(&self, config: ChainConfig) -> OmniChainResult<Box<dyn ChainWallet>> {
        Ok(Box::new(CosmosWallet::new(config)))
    }

    fn chain_id(&self) -> ChainId {
        self.config.chain_id.clone()
    }

    fn validate_address(&self, address: &str) -> bool {
        address.len() >= 40 && address.len() <= 50
    }

    fn chain_config(&self) -> ChainConfig {
        self.config.clone()
    }
}

/// Cosmos wallet implementation
pub struct CosmosWallet {
    config: ChainConfig,
    address: ChainAddress,
}

impl CosmosWallet {
    /// Create a new Cosmos wallet
    pub fn new(config: ChainConfig) -> Self {
        let address = ChainAddress::cosmos("cosmos1xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx");
        Self { config, address }
    }
}

#[async_trait]
impl ChainWallet for CosmosWallet {
    fn chain_id(&self) -> ChainId {
        self.config.chain_id.clone()
    }

    async fn address(&self) -> OmniChainResult<ChainAddress> {
        Ok(self.address.clone())
    }

    async fn balance(&self, _token_id: Option<String>) -> OmniChainResult<String> {
        Ok("0".to_string())
    }

    async fn lock_asset(
        &self,
        _asset: ChainAsset,
        _target_chain: ChainId,
        _recipient: ChainAddress,
    ) -> OmniChainResult<String> {
        Ok("0000000000000000000000000000000000000000000000000000000000000000".to_string())
    }

    async fn release_asset(
        &self,
        _proof: Vec<u8>,
        _recipient: ChainAddress,
    ) -> OmniChainResult<String> {
        Ok("0000000000000000000000000000000000000000000000000000000000000000".to_string())
    }

    async fn get_transaction_status(&self, _tx_id: &str) -> OmniChainResult<TransactionStatus> {
        Ok(TransactionStatus::Confirmed)
    }
}

/// Polkadot chain adapter
pub struct PolkadotAdapter {
    config: ChainConfig,
}

impl PolkadotAdapter {
    /// Create a new Polkadot adapter
    pub fn new(config: ChainConfig) -> Self {
        Self { config }
    }
}

#[async_trait]
impl ChainAdapter for PolkadotAdapter {
    async fn create_wallet(&self, config: ChainConfig) -> OmniChainResult<Box<dyn ChainWallet>> {
        Ok(Box::new(PolkadotWallet::new(config)))
    }

    fn chain_id(&self) -> ChainId {
        self.config.chain_id.clone()
    }

    fn validate_address(&self, address: &str) -> bool {
        address.len() == 47 && address.starts_with("1")
    }

    fn chain_config(&self) -> ChainConfig {
        self.config.clone()
    }
}

/// Polkadot wallet implementation
pub struct PolkadotWallet {
    config: ChainConfig,
    address: ChainAddress,
}

impl PolkadotWallet {
    /// Create a new Polkadot wallet
    pub fn new(config: ChainConfig) -> Self {
        let address = ChainAddress::polkadot("1xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx");
        Self { config, address }
    }
}

#[async_trait]
impl ChainWallet for PolkadotWallet {
    fn chain_id(&self) -> ChainId {
        self.config.chain_id.clone()
    }

    async fn address(&self) -> OmniChainResult<ChainAddress> {
        Ok(self.address.clone())
    }

    async fn balance(&self, _token_id: Option<String>) -> OmniChainResult<String> {
        Ok("0".to_string())
    }

    async fn lock_asset(
        &self,
        _asset: ChainAsset,
        _target_chain: ChainId,
        _recipient: ChainAddress,
    ) -> OmniChainResult<String> {
        Ok("0000000000000000000000000000000000000000000000000000000000000000".to_string())
    }

    async fn release_asset(
        &self,
        _proof: Vec<u8>,
        _recipient: ChainAddress,
    ) -> OmniChainResult<String> {
        Ok("0000000000000000000000000000000000000000000000000000000000000000".to_string())
    }

    async fn get_transaction_status(&self, _tx_id: &str) -> OmniChainResult<TransactionStatus> {
        Ok(TransactionStatus::Confirmed)
    }
}
