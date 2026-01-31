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

//! Canton Wallet SDK - Recovery
//!
//! This crate provides recovery mechanisms for Canton Wallet SDK, including:
//!
//! - Backup and restore functionality
//! - Social recovery
//! - Key rotation
//! - Recovery verification
//!
//! # Features
//!
//! - Encrypted wallet backups
//! - Social recovery using secret sharing
//! - Key rotation for enhanced security
//! - Recovery verification
//!
//! # Example
//!
//! ```no_run
//! use canton_wallet_recovery::{
//!     BackupManager, SocialRecoveryManager,
//!     generate_encryption_key, generate_salt,
//! };
//! use std::sync::Arc;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let backup_manager = BackupManager::new(key_store);
//! let social_recovery = SocialRecoveryManager::new(key_store);
//!
//! let encryption_key = generate_encryption_key();
//! let backup = backup_manager.create_backup(&wallet_id, &encryption_key).await?;
//! # Ok(())
//! # }
//! ```

pub mod backup;
pub mod error;
pub mod social_recovery;

pub use backup::{
    BackupManager, EncryptedBackup, BackupMetadata,
    generate_encryption_key, derive_encryption_key, generate_salt,
};
pub use error::{RecoveryError, Result};
pub use social_recovery::{RecoveryShare, RecoveryRequest, SocialRecoveryManager};
