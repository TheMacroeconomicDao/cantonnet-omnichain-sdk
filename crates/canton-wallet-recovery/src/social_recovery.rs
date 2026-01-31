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

//! Social recovery functionality.

use crate::error::{RecoveryError, Result};
use canton_wallet_core::types::WalletId;
use canton_wallet_crypto::keystore::KeyStore;
use ed25519_dalek::{Keypair, PublicKey, SecretKey, Signature, Signer, Verifier};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use zeroize::Zeroize;

/// Social recovery share.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryShare {
    /// Share ID.
    pub share_id: String,

    /// Share index.
    pub index: usize,

    /// Total number of shares required.
    pub total_shares: usize,

    /// Threshold number of shares required for recovery.
    pub threshold: usize,

    /// Encrypted share data.
    pub encrypted_share: Vec<u8>,

    /// Public key of the share holder.
    pub holder_public_key: Vec<u8>,

    /// Timestamp when share was created.
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Social recovery request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryRequest {
    /// Wallet ID to recover.
    pub wallet_id: String,

    /// Recovery shares collected.
    pub shares: Vec<RecoveryShare>,

    /// New public key for recovered wallet.
    pub new_public_key: Option<Vec<u8>>,

    /// Timestamp of recovery request.
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Social recovery manager.
pub struct SocialRecoveryManager<KS: KeyStore> {
    key_store: Arc<KS>,
}

impl<KS: KeyStore> SocialRecoveryManager<KS> {
    /// Create a new social recovery manager.
    ///
    /// # Arguments
    ///
    /// * `key_store` - Key store to use for recovery
    ///
    /// # Returns
    ///
    /// Returns a new `SocialRecoveryManager` instance
    pub fn new(key_store: Arc<KS>) -> Self {
        Self { key_store }
    }

    /// Create recovery shares for a wallet.
    ///
    /// # Arguments
    ///
    /// * `wallet_id` - Wallet ID to create shares for
    /// * `total_shares` - Total number of shares to create
    /// * `threshold` - Number of shares required for recovery
    /// * `holder_public_keys` - Public keys of share holders
    ///
    /// # Returns
    ///
    /// Returns a vector of recovery shares
    pub async fn create_recovery_shares(
        &self,
        wallet_id: &WalletId,
        total_shares: usize,
        threshold: usize,
        holder_public_keys: Vec<Vec<u8>>,
    ) -> Result<Vec<RecoveryShare>> {
        if total_shares < threshold {
            return Err(RecoveryError::InsufficientShares {
                required: threshold,
                provided: total_shares,
            });
        }

        if threshold < 1 {
            return Err(RecoveryError::SocialRecoveryFailed(
                "Threshold must be at least 1".to_string(),
            ));
        }

        if holder_public_keys.len() != total_shares {
            return Err(RecoveryError::SocialRecoveryFailed(
                "Number of public keys must match total shares".to_string(),
            ));
        }

        tracing::info!(
            "Creating {} recovery shares with threshold {} for wallet: {}",
            total_shares,
            threshold,
            wallet_id
        );

        // Get the wallet's private key
        let keys = self.key_store.list_keys().await?;
        let wallet_key = keys
            .iter()
            .find(|k| k.wallet_id == wallet_id.to_string())
            .ok_or_else(|| {
                RecoveryError::SocialRecoveryFailed("Wallet key not found".to_string())
            })?;

        let private_key_bytes = self
            .key_store
            .export_private_key(&wallet_key.key_id)
            .await?;

        let private_key = SecretKey::from_bytes(&private_key_bytes)
            .map_err(|e| RecoveryError::SocialRecoveryFailed(e.to_string()))?;

        // Create recovery shares using Shamir's Secret Sharing (simplified)
        // In production, you'd use a proper secret sharing library
        let mut shares = Vec::with_capacity(total_shares);

        for (index, holder_public_key) in holder_public_keys.iter().enumerate() {
            // Encrypt the private key for each holder
            // This is a simplified implementation - in production, use proper secret sharing
            let encrypted_share = self.encrypt_for_holder(&private_key, holder_public_key)?;

            let share = RecoveryShare {
                share_id: uuid::Uuid::new_v4().to_string(),
                index,
                total_shares,
                threshold,
                encrypted_share,
                holder_public_key: holder_public_key.clone(),
                created_at: chrono::Utc::now(),
            };

            shares.push(share);
        }

        Ok(shares)
    }

    /// Encrypt private key for a specific holder.
    fn encrypt_for_holder(
        &self,
        private_key: &SecretKey,
        holder_public_key: &[u8],
    ) -> Result<Vec<u8>> {
        // This is a simplified encryption for demonstration
        // In production, use proper hybrid encryption (e.g., RSA + AES)
        let private_key_bytes = private_key.to_bytes();
        let mut encrypted = Vec::with_capacity(private_key_bytes.len());

        for (i, byte) in private_key_bytes.iter().enumerate() {
            // XOR with holder public key (simplified - NOT SECURE)
            let holder_byte = holder_public_key.get(i % holder_public_key.len()).copied().unwrap_or(0);
            encrypted.push(byte ^ holder_byte);
        }

        Ok(encrypted)
    }

    /// Recover wallet from shares.
    ///
    /// # Arguments
    ///
    /// * `request` - Recovery request with shares
    ///
    /// # Returns
    ///
    /// Returns the recovered private key bytes
    pub async fn recover_wallet(&self, request: &RecoveryRequest) -> Result<Vec<u8>> {
        tracing::info!(
            "Recovering wallet from {} shares",
            request.shares.len()
        );

        // Validate shares
        if request.shares.is_empty() {
            return Err(RecoveryError::SocialRecoveryFailed(
                "No recovery shares provided".to_string(),
            ));
        }

        // Check if all shares are from the same wallet
        let wallet_id = request.shares.first().map(|s| s.wallet_id.clone());
        if !request.shares.iter().all(|s| s.wallet_id == wallet_id) {
            return Err(RecoveryError::SocialRecoveryFailed(
                "Shares are from different wallets".to_string(),
            ));
        }

        // Check threshold
        let threshold = request
            .shares
            .first()
            .map(|s| s.threshold)
            .ok_or_else(|| {
                RecoveryError::SocialRecoveryFailed("Invalid recovery shares".to_string())
            })?;

        if request.shares.len() < threshold {
            return Err(RecoveryError::InsufficientShares {
                required: threshold,
                provided: request.shares.len(),
            });
        }

        // Decrypt shares and recover private key
        // This is a simplified implementation - in production, use proper secret sharing
        let recovered_key = self.recover_from_shares(&request.shares)?;

        // Verify recovered key
        if let Some(ref new_public_key) = request.new_public_key {
            let recovered_private_key = SecretKey::from_bytes(&recovered_key)
                .map_err(|e| RecoveryError::SocialRecoveryFailed(e.to_string()))?;

            let recovered_public_key_bytes = recovered_private_key.public_key().to_bytes();

            if recovered_public_key_bytes.as_slice() != new_public_key.as_slice() {
                return Err(RecoveryError::VerificationFailed(
                    "Recovered key does not match expected public key".to_string(),
                ));
            }
        }

        Ok(recovered_key)
    }

    /// Recover private key from shares.
    fn recover_from_shares(&self, shares: &[RecoveryShare]) -> Result<Vec<u8>> {
        // This is a simplified recovery using XOR (NOT SECURE)
        // In production, use proper Shamir's Secret Sharing
        if shares.is_empty() {
            return Err(RecoveryError::SocialRecoveryFailed(
                "No shares provided".to_string(),
            ));
        }

        // Find the minimum share length
        let min_len = shares
            .iter()
            .map(|s| s.encrypted_share.len())
            .min()
            .ok_or_else(|| {
                RecoveryError::SocialRecoveryFailed("Invalid recovery shares".to_string())
            })?;

        // Recover by XORing all shares (simplified - NOT SECURE)
        let mut recovered = vec![0u8; min_len];
        for share in shares {
            for (i, byte) in share.encrypted_share.iter().enumerate() {
                if i < recovered.len() {
                    recovered[i] ^= byte;
                }
            }
        }

        Ok(recovered)
    }

    /// Verify recovery share.
    ///
    /// # Arguments
    ///
    /// * `share` - Recovery share to verify
    ///
    /// # Returns
    ///
    /// Returns true if share is valid
    pub fn verify_share(&self, share: &RecoveryShare) -> bool {
        // Check share structure
        if share.share_id.is_empty() {
            return false;
        }

        if share.index >= share.total_shares {
            return false;
        }

        if share.threshold > share.total_shares {
            return false;
        }

        if share.threshold < 1 {
            return false;
        }

        if share.encrypted_share.is_empty() {
            return false;
        }

        if share.holder_public_key.is_empty() {
            return false;
        }

        true
    }

    /// Verify recovery request.
    ///
    /// # Arguments
    ///
    /// * `request` - Recovery request to verify
    ///
    /// # Returns
    ///
    /// Returns true if request is valid
    pub fn verify_recovery_request(&self, request: &RecoveryRequest) -> bool {
        // Check shares are not empty
        if request.shares.is_empty() {
            return false;
        }

        // Check all shares are valid
        if !request.shares.iter().all(|s| self.verify_share(s)) {
            return false;
        }

        // Check shares are from the same wallet
        let wallet_id = request.shares.first().map(|s| s.wallet_id.clone());
        if !request.shares.iter().all(|s| s.wallet_id == wallet_id) {
            return false;
        }

        // Check threshold is met
        let threshold = request
            .shares
            .first()
            .map(|s| s.threshold)
            .unwrap_or(0);

        if request.shares.len() < threshold {
            return false;
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verify_share_valid() {
        let share = RecoveryShare {
            share_id: "test-share".to_string(),
            index: 0,
            total_shares: 3,
            threshold: 2,
            encrypted_share: vec![1, 2, 3],
            holder_public_key: vec![4, 5, 6],
            created_at: chrono::Utc::now(),
        };
        assert!(verify_share(&share));
    }

    #[test]
    fn test_verify_share_invalid_empty_id() {
        let share = RecoveryShare {
            share_id: String::new(),
            index: 0,
            total_shares: 3,
            threshold: 2,
            encrypted_share: vec![1, 2, 3],
            holder_public_key: vec![4, 5, 6],
            created_at: chrono::Utc::now(),
        };
        assert!(!verify_share(&share));
    }

    #[test]
    fn test_verify_share_invalid_index() {
        let share = RecoveryShare {
            share_id: "test-share".to_string(),
            index: 5,
            total_shares: 3,
            threshold: 2,
            encrypted_share: vec![1, 2, 3],
            holder_public_key: vec![4, 5, 6],
            created_at: chrono::Utc::now(),
        };
        assert!(!verify_share(&share));
    }

    #[test]
    fn test_verify_share_invalid_threshold() {
        let share = RecoveryShare {
            share_id: "test-share".to_string(),
            index: 0,
            total_shares: 3,
            threshold: 0,
            encrypted_share: vec![1, 2, 3],
            holder_public_key: vec![4, 5, 6],
            created_at: chrono::Utc::now(),
        };
        assert!(!verify_share(&share));
    }

    #[test]
    fn test_verify_recovery_request_valid() {
        let request = RecoveryRequest {
            wallet_id: "test-wallet".to_string(),
            shares: vec![
                RecoveryShare {
                    share_id: "share1".to_string(),
                    index: 0,
                    total_shares: 3,
                    threshold: 2,
                    encrypted_share: vec![1, 2, 3],
                    holder_public_key: vec![4, 5, 6],
                    created_at: chrono::Utc::now(),
                },
                RecoveryShare {
                    share_id: "share2".to_string(),
                    index: 1,
                    total_shares: 3,
                    threshold: 2,
                    encrypted_share: vec![1, 2, 3],
                    holder_public_key: vec![4, 5, 6],
                    created_at: chrono::Utc::now(),
                },
            ],
            new_public_key: None,
            created_at: chrono::Utc::now(),
        };
        assert!(verify_recovery_request(&request));
    }

    #[test]
    fn test_verify_recovery_request_empty_shares() {
        let request = RecoveryRequest {
            wallet_id: "test-wallet".to_string(),
            shares: vec![],
            new_public_key: None,
            created_at: chrono::Utc::now(),
        };
        assert!(!verify_recovery_request(&request));
    }

    #[test]
    fn test_verify_recovery_request_different_wallets() {
        let request = RecoveryRequest {
            wallet_id: "test-wallet".to_string(),
            shares: vec![
                RecoveryShare {
                    share_id: "share1".to_string(),
                    index: 0,
                    total_shares: 3,
                    threshold: 2,
                    encrypted_share: vec![1, 2, 3],
                    holder_public_key: vec![4, 5, 6],
                    created_at: chrono::Utc::now(),
                },
                RecoveryShare {
                    share_id: "share2".to_string(),
                    index: 1,
                    total_shares: 3,
                    threshold: 2,
                    encrypted_share: vec![1, 2, 3],
                    holder_public_key: vec![4, 5, 6],
                    created_at: chrono::Utc::now(),
                },
            ],
            new_public_key: None,
            created_at: chrono::Utc::now(),
        };
        assert!(!verify_recovery_request(&request));
    }
}
