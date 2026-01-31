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

//! Backup and restore functionality.

use crate::error::{RecoveryError, Result};
use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use canton_wallet_core::types::WalletId;
use canton_wallet_crypto::keystore::KeyStore;
use chrono::{DateTime, Utc};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use sha2::Digest;
use std::path::Path;
use std::sync::Arc;
use zeroize::Zeroize;

/// Backup metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupMetadata {
    /// Backup version.
    pub version: String,

    /// Wallet ID.
    pub wallet_id: String,

    /// Timestamp of backup.
    pub timestamp: DateTime<Utc>,

    /// Number of keys in backup.
    pub key_count: usize,

    /// Backup format version.
    pub format_version: u32,
}

impl Default for BackupMetadata {
    fn default() -> Self {
        Self {
            version: "0.1.0".to_string(),
            wallet_id: String::new(),
            timestamp: Utc::now(),
            key_count: 0,
            format_version: 1,
        }
    }
}

/// Encrypted backup data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedBackup {
    /// Backup metadata.
    pub metadata: BackupMetadata,

    /// Encrypted data.
    pub encrypted_data: Vec<u8>,

    /// Nonce used for encryption.
    pub nonce: Vec<u8>,
}

/// Backup manager for creating and restoring wallet backups.
pub struct BackupManager<KS: KeyStore> {
    key_store: Arc<KS>,
}

impl<KS: KeyStore> BackupManager<KS> {
    /// Create a new backup manager.
    ///
    /// # Arguments
    ///
    /// * `key_store` - Key store to backup
    ///
    /// # Returns
    ///
    /// Returns a new `BackupManager` instance
    pub fn new(key_store: Arc<KS>) -> Self {
        Self { key_store }
    }

    /// Create a backup of the wallet.
    ///
    /// # Arguments
    ///
    /// * `wallet_id` - Wallet ID to backup
    /// * `encryption_key` - Encryption key for the backup
    ///
    /// # Returns
    ///
    /// Returns encrypted backup data
    pub async fn create_backup(
        &self,
        wallet_id: &WalletId,
        encryption_key: &[u8; 32],
    ) -> Result<EncryptedBackup> {
        tracing::info!("Creating backup for wallet: {}", wallet_id);

        // Get all keys from the key store
        let keys = self.key_store.list_keys().await?;

        // Serialize keys
        let keys_data = bincode::serialize(&keys)
            .map_err(|e| RecoveryError::SerializationError(e.to_string()))?;

        // Generate nonce
        let mut nonce_bytes = [0u8; 12];
        rand::thread_rng().fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        // Encrypt data
        let cipher = Aes256Gcm::new(encryption_key);
        let encrypted_data = cipher
            .encrypt(&nonce, keys_data.as_ref())
            .map_err(|e| RecoveryError::BackupFailed(e.to_string()))?;

        // Create metadata
        let metadata = BackupMetadata {
            wallet_id: wallet_id.to_string(),
            key_count: keys.len(),
            ..Default::default()
        };

        Ok(EncryptedBackup {
            metadata,
            encrypted_data,
            nonce: nonce_bytes.to_vec(),
        })
    }

    /// Restore a wallet from backup.
    ///
    /// # Arguments
    ///
    /// * `backup` - Encrypted backup data
    /// * `encryption_key` - Encryption key for the backup
    ///
    /// # Returns
    ///
    /// Returns the number of keys restored
    pub async fn restore_backup(
        &self,
        backup: &EncryptedBackup,
        encryption_key: &[u8; 32],
    ) -> Result<usize> {
        tracing::info!(
            "Restoring backup for wallet: {}",
            backup.metadata.wallet_id
        );

        // Validate backup format version
        if backup.metadata.format_version != 1 {
            return Err(RecoveryError::InvalidBackupFormat(format!(
                "Unsupported backup format version: {}",
                backup.metadata.format_version
            )));
        }

        // Decrypt data
        let nonce = Nonce::from_slice(&backup.nonce);
        let cipher = Aes256Gcm::new(encryption_key);
        let decrypted_data = cipher
            .decrypt(&nonce, backup.encrypted_data.as_ref())
            .map_err(|e| RecoveryError::RestoreFailed(e.to_string()))?;

        // Deserialize keys
        let keys: Vec<canton_wallet_crypto::keystore::KeyInfo> = bincode::deserialize(&decrypted_data)
            .map_err(|e| RecoveryError::DeserializationError(e.to_string()))?;

        // Restore keys to key store
        let mut restored_count = 0;
        for key_info in keys {
            self.key_store
                .import_key(
                    &key_info.public_key.as_bytes(),
                    key_info.algorithm,
                    key_info.purpose,
                    key_info.metadata.clone(),
                )
                .await?;
            restored_count += 1;
        }

        tracing::info!("Restored {} keys from backup", restored_count);
        Ok(restored_count)
    }

    /// Save backup to file.
    ///
    /// # Arguments
    ///
    /// * `backup` - Encrypted backup data
    /// * `path` - Path to save backup
    ///
    /// # Returns
    ///
    /// Returns a result indicating success or failure
    pub async fn save_backup_to_file(
        &self,
        backup: &EncryptedBackup,
        path: &Path,
    ) -> Result<()> {
        tracing::info!("Saving backup to: {:?}", path);

        // Serialize backup
        let backup_data = bincode::serialize(backup)
            .map_err(|e| RecoveryError::SerializationError(e.to_string()))?;

        // Write to file
        tokio::fs::write(path, backup_data)
            .await
            .map_err(|e| RecoveryError::BackupFailed(e.to_string()))?;

        Ok(())
    }

    /// Load backup from file.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to load backup from
    ///
    /// # Returns
    ///
    /// Returns encrypted backup data
    pub async fn load_backup_from_file(&self, path: &Path) -> Result<EncryptedBackup> {
        tracing::info!("Loading backup from: {:?}", path);

        // Check if file exists
        if !path.exists() {
            return Err(RecoveryError::BackupNotFound(path.display().to_string()));
        }

        // Read from file
        let backup_data = tokio::fs::read(path)
            .await
            .map_err(|e| RecoveryError::BackupFailed(e.to_string()))?;

        // Deserialize backup
        let backup: EncryptedBackup = bincode::deserialize(&backup_data)
            .map_err(|e| RecoveryError::DeserializationError(e.to_string()))?;

        Ok(backup)
    }

    /// Verify backup integrity.
    ///
    /// # Arguments
    ///
    /// * `backup` - Encrypted backup data
    ///
    /// # Returns
    ///
    /// Returns true if backup is valid
    pub fn verify_backup(&self, backup: &EncryptedBackup) -> bool {
        // Check format version
        if backup.metadata.format_version != 1 {
            return false;
        }

        // Check nonce length
        if backup.nonce.len() != 12 {
            return false;
        }

        // Check encrypted data is not empty
        if backup.encrypted_data.is_empty() {
            return false;
        }

        // Check wallet ID is not empty
        if backup.metadata.wallet_id.is_empty() {
            return false;
        }

        true
    }

    /// Get backup size in bytes.
    ///
    /// # Arguments
    ///
    /// * `backup` - Encrypted backup data
    ///
    /// # Returns
    ///
    /// Returns backup size in bytes
    pub fn backup_size(&self, backup: &EncryptedBackup) -> usize {
        bincode::serialized_size(backup).unwrap_or(0)
    }
}

/// Generate a random encryption key for backup.
///
/// # Returns
///
/// Returns a 32-byte encryption key
pub fn generate_encryption_key() -> [u8; 32] {
    let mut key = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut key);
    key
}

/// Derive encryption key from password.
///
/// # Arguments
///
/// * `password` - Password to derive key from
/// * `salt` - Salt for key derivation
///
/// # Returns
///
/// Returns a 32-byte encryption key
pub fn derive_encryption_key(password: &str, salt: &[u8; 16]) -> [u8; 32] {
    use sha2::Sha256;

    // Simple key derivation using SHA256
    let mut hasher = Sha256::new();
    hasher.update(password.as_bytes());
    hasher.update(salt);
    let result = hasher.finalize();

    let mut key = [0u8; 32];
    key.copy_from_slice(&result[..32.min(key.len())]);
    key
}

/// Generate a random salt for key derivation.
///
/// # Returns
///
/// Returns a 16-byte salt
pub fn generate_salt() -> [u8; 16] {
    let mut salt = [0u8; 16];
    rand::thread_rng().fill_bytes(&mut salt);
    salt
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backup_metadata_default() {
        let metadata = BackupMetadata::default();
        assert_eq!(metadata.version, "0.1.0");
        assert!(metadata.wallet_id.is_empty());
        assert_eq!(metadata.format_version, 1);
    }

    #[test]
    fn test_generate_encryption_key() {
        let key1 = generate_encryption_key();
        let key2 = generate_encryption_key();
        assert_ne!(key1, key2);
    }

    #[test]
    fn test_generate_salt() {
        let salt1 = generate_salt();
        let salt2 = generate_salt();
        assert_ne!(salt1, salt2);
    }

    #[test]
    fn test_derive_encryption_key() {
        let salt = generate_salt();
        let key1 = derive_encryption_key("password", &salt);
        let key2 = derive_encryption_key("password", &salt);
        assert_eq!(key1, key2);
    }

    #[test]
    fn test_verify_backup_valid() {
        let backup = EncryptedBackup {
            metadata: BackupMetadata {
                wallet_id: "test-wallet".to_string(),
                format_version: 1,
                ..Default::default()
            },
            encrypted_data: vec![1, 2, 3],
            nonce: vec![0; 12],
        };
        assert!(verify_backup(&backup));
    }

    #[test]
    fn test_verify_backup_invalid_format() {
        let backup = EncryptedBackup {
            metadata: BackupMetadata {
                wallet_id: "test-wallet".to_string(),
                format_version: 2,
                ..Default::default()
            },
            encrypted_data: vec![1, 2, 3],
            nonce: vec![0; 12],
        };
        assert!(!verify_backup(&backup));
    }

    #[test]
    fn test_verify_backup_invalid_nonce() {
        let backup = EncryptedBackup {
            metadata: BackupMetadata {
                wallet_id: "test-wallet".to_string(),
                format_version: 1,
                ..Default::default()
            },
            encrypted_data: vec![1, 2, 3],
            nonce: vec![0; 10],
        };
        assert!(!verify_backup(&backup));
    }

    #[test]
    fn test_verify_backup_empty_data() {
        let backup = EncryptedBackup {
            metadata: BackupMetadata {
                wallet_id: "test-wallet".to_string(),
                format_version: 1,
                ..Default::default()
            },
            encrypted_data: vec![],
            nonce: vec![0; 12],
        };
        assert!(!verify_backup(&backup));
    }

    #[test]
    fn test_verify_backup_empty_wallet_id() {
        let backup = EncryptedBackup {
            metadata: BackupMetadata {
                wallet_id: String::new(),
                format_version: 1,
                ..Default::default()
            },
            encrypted_data: vec![1, 2, 3],
            nonce: vec![0; 12],
        };
        assert!(!verify_backup(&backup));
    }
}
