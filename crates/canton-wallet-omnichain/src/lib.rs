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

//! # Canton Wallet OmniChain Integration
//!
//! This crate provides multi-chain support for the Canton Wallet SDK, enabling
//! cross-chain transfers and interoperability with other blockchain networks.
//!
//! ## Features
//!
//! - Multi-chain wallet management
//! - Chain adapters for Ethereum, Cosmos, Polkadot, and custom chains
//! - Bridge contract management
//! - Atomic cross-chain transfers
//! - Proof generation and verification
//!
//! ## Example
//!
//! ```no_run
//! use canton_wallet_omnichain::{MultiChainWallet, ChainId, ChainAddress};
//! use canton_wallet_core::PartyId;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let canton_wallet = /* ... */;
//! let bridge_manager = /* ... */;
//!
//! let mut multi_chain_wallet = MultiChainWallet::new(canton_wallet, bridge_manager);
//!
//! // Add chain adapter
//! let eth_adapter = /* ... */;
//! multi_chain_wallet.add_chain(ChainId::Ethereum, eth_adapter);
//!
//! // Transfer asset to Ethereum
//! let asset = /* ... */;
//! let recipient = ChainAddress::ethereum("0x1234...");
//! let tx = multi_chain_wallet.transfer_to_chain(asset, ChainId::Ethereum, recipient).await?;
//! # Ok(())
//! # }
//! ```

pub mod error;
pub mod types;
pub mod wallet;
pub mod adapter;
pub mod bridge;
pub mod proof;

pub use error::{OmniChainError, OmniChainResult};
pub use types::{
    ChainId, ChainAddress, CantonAsset, ChainAsset, CrossChainTx, LockReceipt, ReleaseReceipt,
};
pub use wallet::MultiChainWallet;
pub use adapter::{ChainWallet, ChainAdapter};
pub use bridge::{BridgeManager, BridgeConfig};
pub use proof::{LockProof, ProofVerifier};
