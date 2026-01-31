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

//! Configuration for OmniChain integration
//!
//! This module provides configuration types for multi-chain wallet
//! and bridge management.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;

use crate::error::Result;
use crate::types::ChainId;

/// OmniChain configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OmniChainConfig {
    /// Bridge configuration
    pub bridge: BridgeConfig,

    /// Chain configurations
    pub chains: HashMap<String, ChainConfig>,

    /// Timeout for cross-chain operations
    pub cross_chain_timeout: Duration,

    /// Maximum retry attempts for cross-chain operations
    pub max_cross_chain_retries: u32,

    /// Enable proof verification
    pub enable_proof_verification: bool,

    /// Proof verification timeout
    pub proof_verification_timeout: Duration,
}

impl OmniChainConfig {
    /// Create new OmniChain configuration
    pub fn new(bridge: BridgeConfig) -> Self {
        Self {
            bridge,
            chains: HashMap::new(),
            cross_chain_timeout: Duration::from_secs(300),
            max_cross_chain_retries: 3,
            enable_proof_verification: true,
            proof_verification_timeout: Duration::from_secs(60),
        }
    }

    /// Add chain configuration
    pub fn with_chain(mut self, name: String, config: ChainConfig) -> Self {
        self.chains.insert(name, config);
        self
    }

    /// Set cross-chain timeout
    pub fn with_cross_chain_timeout(mut self, timeout: Duration) -> Self {
        self.cross_chain_timeout = timeout;
        self
    }

    /// Set max cross-chain retries
    pub fn with_max_cross_chain_retries(mut self, retries: u32) -> Self {
        self.max_cross_chain_retries = retries;
        self
    }

    /// Enable/disable proof verification
    pub fn with_proof_verification(mut self, enabled: bool) -> Self {
        self.enable_proof_verification = enabled;
        self
    }

    /// Set proof verification timeout
    pub fn with_proof_verification_timeout(mut self, timeout: Duration) -> Self {
        self.proof_verification_timeout = timeout;
        self
    }

    /// Get chain configuration by name
    pub fn get_chain_config(&self, name: &str) -> Option<&ChainConfig> {
        self.chains.get(name)
    }

    /// Validate configuration
    pub fn validate(&self) -> Result<()> {
        if self.chains.is_empty() {
            return Err(crate::error::OmniChainError::ConfigurationError(
                "At least one chain configuration is required".to_string(),
            ));
        }

        for (name, config) in &self.chains {
            config.validate()?;
        }

        self.bridge.validate()?;

        Ok(())
    }
}

impl Default for OmniChainConfig {
    fn default() -> Self {
        Self {
            bridge: BridgeConfig::default(),
            chains: HashMap::new(),
            cross_chain_timeout: Duration::from_secs(300),
            max_cross_chain_retries: 3,
            enable_proof_verification: true,
            proof_verification_timeout: Duration::from_secs(60),
        }
    }
}

/// Bridge configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeConfig {
    /// Bridge contract addresses
    pub contracts: HashMap<ChainId, BridgeContractConfig>,

    /// Bridge fee percentage (in basis points, 10000 = 100%)
    pub fee_percentage: u32,

    /// Minimum bridge amount
    pub min_bridge_amount: String,

    /// Maximum bridge amount
    pub max_bridge_amount: String,

    /// Confirmation blocks required
    pub confirmation_blocks: u64,

    /// Enable automatic fee payment
    pub auto_pay_fees: bool,

    /// Fee payment timeout
    pub fee_payment_timeout: Duration,
}

impl BridgeConfig {
    /// Create new bridge configuration
    pub fn new() -> Self {
        Self {
            contracts: HashMap::new(),
            fee_percentage: 30, // 0.3%
            min_bridge_amount: "1000000000000000000".to_string(), // 1 token
            max_bridge_amount: "1000000000000000000000000".to_string(), // 1M tokens
            confirmation_blocks: 12,
            auto_pay_fees: true,
            fee_payment_timeout: Duration::from_secs(60),
        }
    }

    /// Add bridge contract
    pub fn with_contract(mut self, chain_id: ChainId, config: BridgeContractConfig) -> Self {
        self.contracts.insert(chain_id, config);
        self
    }

    /// Set fee percentage
    pub fn with_fee_percentage(mut self, percentage: u32) -> Self {
        self.fee_percentage = percentage;
        self
    }

    /// Set minimum bridge amount
    pub fn with_min_bridge_amount(mut self, amount: String) -> Self {
        self.min_bridge_amount = amount;
        self
    }

    /// Set maximum bridge amount
    pub fn with_max_bridge_amount(mut self, amount: String) -> Self {
        self.max_bridge_amount = amount;
        self
    }

    /// Set confirmation blocks
    pub fn with_confirmation_blocks(mut self, blocks: u64) -> Self {
        self.confirmation_blocks = blocks;
        self
    }

    /// Enable/disable automatic fee payment
    pub fn with_auto_pay_fees(mut self, enabled: bool) -> Self {
        self.auto_pay_fees = enabled;
        self
    }

    /// Set fee payment timeout
    pub fn with_fee_payment_timeout(mut self, timeout: Duration) -> Self {
        self.fee_payment_timeout = timeout;
        self
    }

    /// Get bridge contract for chain
    pub fn get_contract(&self, chain_id: &ChainId) -> Option<&BridgeContractConfig> {
        self.contracts.get(chain_id)
    }

    /// Validate configuration
    pub fn validate(&self) -> Result<()> {
        if self.fee_percentage > 10000 {
            return Err(crate::error::OmniChainError::ConfigurationError(
                "Fee percentage cannot exceed 10000 (100%)".to_string(),
            ));
        }

        if self.contracts.is_empty() {
            return Err(crate::error::OmniChainError::ConfigurationError(
                "At least one bridge contract is required".to_string(),
            ));
        }

        for (chain_id, config) in &self.contracts {
            config.validate()?;
        }

        Ok(())
    }
}

impl Default for BridgeConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// Bridge contract configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeContractConfig {
    /// Contract address
    pub address: String,

    /// Contract ABI (for EVM chains)
    pub abi: Option<String>,

    /// Contract version
    pub version: String,

    /// Supported assets
    pub supported_assets: Vec<String>,

    /// Gas limit for bridge operations
    pub gas_limit: u64,

    /// Gas price strategy
    pub gas_price_strategy: GasPriceStrategy,

    /// Enable automatic gas estimation
    pub auto_estimate_gas: bool,
}

impl BridgeContractConfig {
    /// Create new bridge contract configuration
    pub fn new(address: String, version: String) -> Self {
        Self {
            address,
            abi: None,
            version,
            supported_assets: Vec::new(),
            gas_limit: 500000,
            gas_price_strategy: GasPriceStrategy::Medium,
            auto_estimate_gas: true,
        }
    }

    /// Set ABI
    pub fn with_abi(mut self, abi: String) -> Self {
        self.abi = Some(abi);
        self
    }

    /// Add supported asset
    pub fn with_supported_asset(mut self, asset: String) -> Self {
        self.supported_assets.push(asset);
        self
    }

    /// Set gas limit
    pub fn with_gas_limit(mut self, limit: u64) -> Self {
        self.gas_limit = limit;
        self
    }

    /// Set gas price strategy
    pub fn with_gas_price_strategy(mut self, strategy: GasPriceStrategy) -> Self {
        self.gas_price_strategy = strategy;
        self
    }

    /// Enable/disable automatic gas estimation
    pub fn with_auto_estimate_gas(mut self, enabled: bool) -> Self {
        self.auto_estimate_gas = enabled;
        self
    }

    /// Validate configuration
    pub fn validate(&self) -> Result<()> {
        if self.address.is_empty() {
            return Err(crate::error::OmniChainError::ConfigurationError(
                "Bridge contract address is required".to_string(),
            ));
        }

        if self.version.is_empty() {
            return Err(crate::error::OmniChainError::ConfigurationError(
                "Bridge contract version is required".to_string(),
            ));
        }

        Ok(())
    }
}

/// Gas price strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GasPriceStrategy {
    /// Low gas price (slow transactions)
    Low,
    /// Medium gas price (balanced)
    Medium,
    /// High gas price (fast transactions)
    High,
    /// Custom gas price
    Custom(u64),
}

impl GasPriceStrategy {
    /// Get gas price multiplier
    pub fn multiplier(&self) -> f64 {
        match self {
            GasPriceStrategy::Low => 0.8,
            GasPriceStrategy::Medium => 1.0,
            GasPriceStrategy::High => 1.2,
            GasPriceStrategy::Custom(price) => *price as f64 / 1e9, // Convert from Gwei
        }
    }
}

/// Chain configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainConfig {
    /// Chain ID
    pub chain_id: ChainId,

    /// Chain name
    pub name: String,

    /// RPC endpoint
    pub rpc_endpoint: String,

    /// WebSocket endpoint (optional)
    pub ws_endpoint: Option<String>,

    /// Chain type
    pub chain_type: ChainType,

    /// Native currency
    pub native_currency: CurrencyConfig,

    /// Block time (in seconds)
    pub block_time: u64,

    /// Confirmation blocks
    pub confirmation_blocks: u64,

    /// Gas configuration
    pub gas_config: GasConfig,

    /// Enable EIP-1559 (for EVM chains)
    pub enable_eip1559: bool,

    /// Chain-specific configuration
    pub chain_specific: ChainSpecificConfig,
}

impl ChainConfig {
    /// Create new chain configuration
    pub fn new(
        chain_id: ChainId,
        name: String,
        rpc_endpoint: String,
        chain_type: ChainType,
    ) -> Self {
        Self {
            chain_id,
            name,
            rpc_endpoint,
            ws_endpoint: None,
            chain_type,
            native_currency: CurrencyConfig::default(),
            block_time: 15,
            confirmation_blocks: 12,
            gas_config: GasConfig::default(),
            enable_eip1559: true,
            chain_specific: ChainSpecificConfig::default(),
        }
    }

    /// Set WebSocket endpoint
    pub fn with_ws_endpoint(mut self, endpoint: String) -> Self {
        self.ws_endpoint = Some(endpoint);
        self
    }

    /// Set native currency
    pub fn with_native_currency(mut self, currency: CurrencyConfig) -> Self {
        self.native_currency = currency;
        self
    }

    /// Set block time
    pub fn with_block_time(mut self, block_time: u64) -> Self {
        self.block_time = block_time;
        self
    }

    /// Set confirmation blocks
    pub fn with_confirmation_blocks(mut self, blocks: u64) -> Self {
        self.confirmation_blocks = blocks;
        self
    }

    /// Set gas configuration
    pub fn with_gas_config(mut self, config: GasConfig) -> Self {
        self.gas_config = config;
        self
    }

    /// Enable/disable EIP-1559
    pub fn with_eip1559(mut self, enabled: bool) -> Self {
        self.enable_eip1559 = enabled;
        self
    }

    /// Set chain-specific configuration
    pub fn with_chain_specific(mut self, config: ChainSpecificConfig) -> Self {
        self.chain_specific = config;
        self
    }

    /// Validate configuration
    pub fn validate(&self) -> Result<()> {
        if self.rpc_endpoint.is_empty() {
            return Err(crate::error::OmniChainError::ConfigurationError(
                "RPC endpoint is required".to_string(),
            ));
        }

        if self.name.is_empty() {
            return Err(crate::error::OmniChainError::ConfigurationError(
                "Chain name is required".to_string(),
            ));
        }

        self.native_currency.validate()?;
        self.gas_config.validate()?;

        Ok(())
    }
}

/// Chain type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChainType {
    /// Ethereum Virtual Machine
    EVM,
    /// Cosmos SDK
    Cosmos,
    /// Substrate (Polkadot, Kusama)
    Substrate,
    /// Solana
    Solana,
    /// Canton Network
    Canton,
    /// Custom chain
    Custom,
}

/// Currency configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurrencyConfig {
    /// Currency symbol
    pub symbol: String,

    /// Currency name
    pub name: String,

    /// Decimals
    pub decimals: u8,

    /// Contract address (for tokens)
    pub contract_address: Option<String>,
}

impl CurrencyConfig {
    /// Create new currency configuration
    pub fn new(symbol: String, name: String, decimals: u8) -> Self {
        Self {
            symbol,
            name,
            decimals,
            contract_address: None,
        }
    }

    /// Set contract address
    pub fn with_contract_address(mut self, address: String) -> Self {
        self.contract_address = Some(address);
        self
    }

    /// Validate configuration
    pub fn validate(&self) -> Result<()> {
        if self.symbol.is_empty() {
            return Err(crate::error::OmniChainError::ConfigurationError(
                "Currency symbol is required".to_string(),
            ));
        }

        if self.name.is_empty() {
            return Err(crate::error::OmniChainError::ConfigurationError(
                "Currency name is required".to_string(),
            ));
        }

        if self.decimals > 18 {
            return Err(crate::error::OmniChainError::ConfigurationError(
                "Currency decimals cannot exceed 18".to_string(),
            ));
        }

        Ok(())
    }
}

impl Default for CurrencyConfig {
    fn default() -> Self {
        Self::new("ETH".to_string(), "Ether".to_string(), 18)
    }
}

/// Gas configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GasConfig {
    /// Default gas limit
    pub default_gas_limit: u64,

    /// Minimum gas price (in Gwei)
    pub min_gas_price: u64,

    /// Maximum gas price (in Gwei)
    pub max_gas_price: u64,

    /// Gas price update interval (in seconds)
    pub gas_price_update_interval: u64,
}

impl GasConfig {
    /// Create new gas configuration
    pub fn new() -> Self {
        Self {
            default_gas_limit: 21000,
            min_gas_price: 1,
            max_gas_price: 1000,
            gas_price_update_interval: 60,
        }
    }

    /// Set default gas limit
    pub fn with_default_gas_limit(mut self, limit: u64) -> Self {
        self.default_gas_limit = limit;
        self
    }

    /// Set minimum gas price
    pub fn with_min_gas_price(mut self, price: u64) -> Self {
        self.min_gas_price = price;
        self
    }

    /// Set maximum gas price
    pub fn with_max_gas_price(mut self, price: u64) -> Self {
        self.max_gas_price = price;
        self
    }

    /// Set gas price update interval
    pub fn with_gas_price_update_interval(mut self, interval: u64) -> Self {
        self.gas_price_update_interval = interval;
        self
    }

    /// Validate configuration
    pub fn validate(&self) -> Result<()> {
        if self.min_gas_price > self.max_gas_price {
            return Err(crate::error::OmniChainError::ConfigurationError(
                "Minimum gas price cannot exceed maximum gas price".to_string(),
            ));
        }

        if self.default_gas_limit == 0 {
            return Err(crate::error::OmniChainError::ConfigurationError(
                "Default gas limit must be greater than 0".to_string(),
            ));
        }

        Ok(())
    }
}

impl Default for GasConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// Chain-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ChainSpecificConfig {
    /// EVM-specific configuration
    EVM {
        /// Chain ID (EIP-155)
        eip155_chain_id: u64,
        /// Enable EIP-1559
        enable_eip1559: bool,
    },
    /// Cosmos-specific configuration
    Cosmos {
        /// Chain ID
        chain_id: String,
        /// Bech32 prefix
        bech32_prefix: String,
    },
    /// Substrate-specific configuration
    Substrate {
        /// Genesis hash
        genesis_hash: String,
        /// Runtime version
        runtime_version: u32,
    },
    /// Solana-specific configuration
    Solana {
        /// Commitment level
        commitment: String,
    },
    /// Canton-specific configuration
    Canton {
        /// Ledger API endpoint
        ledger_api_endpoint: String,
    },
    /// Custom configuration
    Custom {
        /// Custom parameters
        params: HashMap<String, String>,
    },
}

impl Default for ChainSpecificConfig {
    fn default() -> Self {
        Self::EVM {
            eip155_chain_id: 1,
            enable_eip1559: true,
        }
    }
}

/// Proof configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofConfig {
    /// Enable proof generation
    pub enable_generation: bool,

    /// Enable proof verification
    pub enable_verification: bool,

    /// Proof storage path
    pub storage_path: Option<PathBuf>,

    /// Proof retention period (in seconds)
    pub retention_period: u64,

    /// Maximum proof size (in bytes)
    pub max_proof_size: usize,
}

impl ProofConfig {
    /// Create new proof configuration
    pub fn new() -> Self {
        Self {
            enable_generation: true,
            enable_verification: true,
            storage_path: None,
            retention_period: 86400, // 24 hours
            max_proof_size: 1024 * 1024, // 1MB
        }
    }

    /// Enable/disable proof generation
    pub fn with_generation(mut self, enabled: bool) -> Self {
        self.enable_generation = enabled;
        self
    }

    /// Enable/disable proof verification
    pub fn with_verification(mut self, enabled: bool) -> Self {
        self.enable_verification = enabled;
        self
    }

    /// Set storage path
    pub fn with_storage_path(mut self, path: PathBuf) -> Self {
        self.storage_path = Some(path);
        self
    }

    /// Set retention period
    pub fn with_retention_period(mut self, period: u64) -> Self {
        self.retention_period = period;
        self
    }

    /// Set maximum proof size
    pub fn with_max_proof_size(mut self, size: usize) -> Self {
        self.max_proof_size = size;
        self
    }
}

impl Default for ProofConfig {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_omnichain_config_creation() {
        let bridge = BridgeConfig::new();
        let config = OmniChainConfig::new(bridge);

        assert_eq!(config.max_cross_chain_retries, 3);
        assert!(config.enable_proof_verification);
    }

    #[test]
    fn test_bridge_config_creation() {
        let config = BridgeConfig::new();

        assert_eq!(config.fee_percentage, 30);
        assert_eq!(config.confirmation_blocks, 12);
        assert!(config.auto_pay_fees);
    }

    #[test]
    fn test_bridge_contract_config() {
        let config = BridgeContractConfig::new("0x123".to_string(), "1.0.0".to_string())
            .with_supported_asset("ETH".to_string())
            .with_gas_limit(300000);

        assert_eq!(config.address, "0x123");
        assert_eq!(config.version, "1.0.0");
        assert_eq!(config.gas_limit, 300000);
        assert_eq!(config.supported_assets.len(), 1);
    }

    #[test]
    fn test_chain_config() {
        let config = ChainConfig::new(
            ChainId::Ethereum,
            "Ethereum".to_string(),
            "https://eth.example.com".to_string(),
            ChainType::EVM,
        )
        .with_block_time(12)
        .with_confirmation_blocks(6);

        assert_eq!(config.chain_id, ChainId::Ethereum);
        assert_eq!(config.block_time, 12);
        assert_eq!(config.confirmation_blocks, 6);
    }

    #[test]
    fn test_currency_config() {
        let config = CurrencyConfig::new("ETH".to_string(), "Ether".to_string(), 18);

        assert_eq!(config.symbol, "ETH");
        assert_eq!(config.decimals, 18);
    }

    #[test]
    fn test_gas_config() {
        let config = GasConfig::new()
            .with_min_gas_price(5)
            .with_max_gas_price(100);

        assert_eq!(config.min_gas_price, 5);
        assert_eq!(config.max_gas_price, 100);
    }

    #[test]
    fn test_gas_price_strategy() {
        assert_eq!(GasPriceStrategy::Low.multiplier(), 0.8);
        assert_eq!(GasPriceStrategy::Medium.multiplier(), 1.0);
        assert_eq!(GasPriceStrategy::High.multiplier(), 1.2);
    }

    #[test]
    fn test_proof_config() {
        let config = ProofConfig::new()
            .with_retention_period(3600)
            .with_max_proof_size(512 * 1024);

        assert_eq!(config.retention_period, 3600);
        assert_eq!(config.max_proof_size, 512 * 1024);
    }
}
