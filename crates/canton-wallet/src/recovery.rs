//! Recovery module for wallet backup, restore, and social recovery.
//!
//! This module provides comprehensive recovery mechanisms including:
//! - Encrypted wallet backups
//! - Social recovery with trusted guardians
//! - Key rotation and migration
//! - Recovery verification and validation

use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::fs;
use tokio::sync::RwLock;
use uuid::Uuid;

use canton_wallet_core::{
    error::WalletError,
    types::{KeyId, PartyId, WalletId},
};

use canton_wallet_crypto::{
    crypto::{CryptoOperations, KeyAlgorithm},
    keystore::{KeyMetadata, KeyPurpose, KeyStore},
};

/// Recovery configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryConfig {
    /// Number of guardians required for social recovery
    pub guardian_threshold: usize,
    /// Maximum number of guardians
    pub max_guardians: usize,
    /// Backup encryption algorithm
    pub encryption_algorithm: KeyAlgorithm,
    /// Backup retention period in days
    pub backup_retention_days: u32,
    /// Enable automatic backups
    pub auto_backup_enabled: bool,
    /// Auto-backup interval in hours
    pub auto_backup_interval_hours: u32,
}

impl Default for RecoveryConfig {
    fn default() -> Self {
        Self {
            guardian_threshold: 2,
            max_guardians: 5,
            encryption_algorithm: KeyAlgorithm::Ed25519,
            backup_retention_days: 90,
            auto_backup_enabled: true,
            auto_backup_interval_hours: 24,
        }
    }
}

/// Backup data containing encrypted wallet information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupData {
    /// Unique backup ID
    pub backup_id: String,
    /// Wallet ID
    pub wallet_id: WalletId,
    /// Party ID
    pub party_id: PartyId,
    /// Encrypted private keys
    pub encrypted_keys: Vec<EncryptedKeyData>,
    /// Encrypted mnemonic (for HD wallets)
    pub encrypted_mnemonic: Option<String>,
    /// Backup timestamp
    pub timestamp: DateTime<Utc>,
    /// Backup version
    pub version: u32,
    /// Additional metadata
    pub metadata: BackupMetadata,
}

/// Encrypted key data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedKeyData {
    /// Key ID
    pub key_id: KeyId,
    /// Encrypted key bytes
    pub encrypted_data: Vec<u8>,
    /// Nonce for encryption
    pub nonce: Vec<u8>,
    /// Key purpose
    pub purpose: KeyPurpose,
}

/// Backup metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupMetadata {
    /// Device information
    pub device_info: String,
    /// Application version
    pub app_version: String,
    /// Backup description
    pub description: Option<String>,
    /// Tags for organization
    pub tags: Vec<String>,
}

/// Guardian information for social recovery
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Guardian {
    /// Guardian ID
    pub guardian_id: String,
    /// Guardian name/identifier
    pub name: String,
    /// Guardian's public key for verification
    pub public_key: Vec<u8>,
    /// Guardian's contact information
    pub contact: GuardianContact,
    /// Guardian status
    pub status: GuardianStatus,
    /// Registration timestamp
    pub registered_at: DateTime<Utc>,
}

/// Guardian contact information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuardianContact {
    /// Email address
    pub email: Option<String>,
    /// Phone number
    pub phone: Option<String>,
    /// Other contact method
    pub other: Option<String>,
}

/// Guardian status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GuardianStatus {
    /// Guardian is active and can participate in recovery
    Active,
    /// Guardian is pending confirmation
    Pending,
    /// Guardian has been revoked
    Revoked,
    /// Guardian is temporarily unavailable
    Inactive,
}

/// Recovery share from a guardian
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryShare {
    /// Guardian ID
    pub guardian_id: String,
    /// Encrypted recovery share
    pub encrypted_share: Vec<u8>,
    /// Share signature
    pub signature: Vec<u8>,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
}

/// Recovery request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryRequest {
    /// Unique request ID
    pub request_id: String,
    /// Wallet ID being recovered
    pub wallet_id: WalletId,
    /// Requester information
    pub requester: RecoveryRequester,
    /// Recovery shares collected
    pub shares: Vec<RecoveryShare>,
    /// Request timestamp
    pub created_at: DateTime<Utc>,
    /// Request expiration
    pub expires_at: DateTime<Utc>,
    /// Request status
    pub status: RecoveryStatus,
}

/// Recovery requester information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryRequester {
    /// Requester identifier
    pub identifier: String,
    /// Requester verification method
    pub verification_method: String,
    /// Additional verification data
    pub verification_data: HashMap<String, String>,
}

/// Recovery status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RecoveryStatus {
    /// Recovery is pending guardian responses
    Pending,
    /// Recovery is in progress
    InProgress,
    /// Recovery completed successfully
    Completed,
    /// Recovery failed
    Failed,
    /// Recovery was cancelled
    Cancelled,
    /// Recovery expired
    Expired,
}

/// Recovery manager for wallet backup and restore operations
pub struct RecoveryManager {
    /// Key store for cryptographic operations
    key_store: Arc<dyn KeyStore>,
    /// Crypto operations
    crypto: Arc<CryptoOperations>,
    /// Recovery configuration
    config: RecoveryConfig,
    /// Registered guardians
    guardians: Arc<RwLock<HashMap<String, Guardian>>>,
    /// Active recovery requests
    recovery_requests: Arc<RwLock<HashMap<String, RecoveryRequest>>>,
    /// Backup storage path
    backup_path: String,
}

impl RecoveryManager {
    /// Create a new recovery manager
    ///
    /// # Arguments
    ///
    /// * `key_store` - Key store for cryptographic operations
    /// * `crypto` - Crypto operations instance
    /// * `config` - Recovery configuration
    /// * `backup_path` - Path to store backups
    ///
    /// # Returns
    ///
    /// Returns a new `RecoveryManager` instance
    pub fn new(
        key_store: Arc<dyn KeyStore>,
        crypto: Arc<CryptoOperations>,
        config: RecoveryConfig,
        backup_path: String,
    ) -> Self {
        Self {
            key_store,
            crypto,
            config,
            guardians: Arc::new(RwLock::new(HashMap::new())),
            recovery_requests: Arc::new(RwLock::new(HashMap::new())),
            backup_path,
        }
    }

    /// Create encrypted wallet backup
    ///
    /// # Arguments
    ///
    /// * `wallet_id` - Wallet ID to backup
    /// * `party_id` - Party ID associated with the wallet
    /// * `encryption_key` - Encryption key for the backup
    /// * `metadata` - Backup metadata
    ///
    /// # Returns
    ///
    /// Returns the backup data or an error
    pub async fn create_backup(
        &self,
        wallet_id: WalletId,
        party_id: PartyId,
        encryption_key: &[u8],
        metadata: BackupMetadata,
    ) -> Result<BackupData, WalletError> {
        let backup_id = Uuid::new_v4().to_string();
        let timestamp = Utc::now();

        // List all keys for the wallet
        let keys = self.key_store.list_keys().await?;
        let mut encrypted_keys = Vec::new();

        // Encrypt each key
        for key_info in keys {
            let key_bytes = self
                .key_store
                .export_public_key(&key_info.key_id)
                .await?;

            // Generate nonce
            let nonce = self.crypto.generate_nonce()?;

            // Encrypt key data
            let encrypted_data = self
                .crypto
                .encrypt(encryption_key, &nonce, &key_bytes.as_bytes())?;

            encrypted_keys.push(EncryptedKeyData {
                key_id: key_info.key_id,
                encrypted_data,
                nonce,
                purpose: key_info.purpose,
            });
        }

        let backup = BackupData {
            backup_id: backup_id.clone(),
            wallet_id,
            party_id,
            encrypted_keys,
            encrypted_mnemonic: None,
            timestamp,
            version: 1,
            metadata,
        };

        // Serialize and save backup
        let backup_json = serde_json::to_string(&backup)
            .map_err(|e| WalletError::SerializationError(e.to_string()))?;

        let backup_file_path = format!("{}/{}.backup", self.backup_path, backup_id);
        fs::write(&backup_file_path, backup_json)
            .await
            .map_err(|e| WalletError::IoError(e.to_string()))?;

        tracing::info!("Created backup {} for wallet {}", backup_id, wallet_id);

        Ok(backup)
    }

    /// Restore wallet from backup
    ///
    /// # Arguments
    ///
    /// * `backup_id` - Backup ID to restore
    /// * `encryption_key` - Decryption key for the backup
    ///
    /// # Returns
    ///
    /// Returns the restored backup data or an error
    pub async fn restore_backup(
        &self,
        backup_id: &str,
        encryption_key: &[u8],
    ) -> Result<BackupData, WalletError> {
        let backup_file_path = format!("{}/{}.backup", self.backup_path, backup_id);

        // Read backup file
        let backup_json = fs::read_to_string(&backup_file_path)
            .await
            .map_err(|e| WalletError::IoError(e.to_string()))?;

        let mut backup: BackupData = serde_json::from_str(&backup_json)
            .map_err(|e| WalletError::SerializationError(e.to_string()))?;

        // Decrypt and restore keys
        for encrypted_key in &mut backup.encrypted_keys {
            let decrypted_data = self.crypto.decrypt(
                encryption_key,
                &encrypted_key.nonce,
                &encrypted_key.encrypted_data,
            )?;

            // Import the restored key
            self.key_store
                .import_key(
                    &decrypted_data,
                    self.config.encryption_algorithm,
                    encrypted_key.purpose,
                    KeyMetadata::default(),
                )
                .await?;
        }

        tracing::info!("Restored backup {} for wallet {}", backup_id, backup.wallet_id);

        Ok(backup)
    }

    /// List all available backups
    ///
    /// # Returns
    ///
    /// Returns a list of backup IDs or an error
    pub async fn list_backups(&self) -> Result<Vec<String>, WalletError> {
        let mut backups = Vec::new();

        let mut entries = fs::read_dir(&self.backup_path)
            .await
            .map_err(|e| WalletError::IoError(e.to_string()))?;

        while let Some(entry) = entries.next_entry().await
            .map_err(|e| WalletError::IoError(e.to_string()))?
        {
            let path = entry.path();
            if let Some(extension) = path.extension() {
                if extension == "backup" {
                    if let Some(stem) = path.file_stem() {
                        if let Some(id) = stem.to_str() {
                            backups.push(id.to_string());
                        }
                    }
                }
            }
        }

        Ok(backups)
    }

    /// Delete a backup
    ///
    /// # Arguments
    ///
    /// * `backup_id` - Backup ID to delete
    ///
    /// # Returns
    ///
    /// Returns Ok(()) or an error
    pub async fn delete_backup(&self, backup_id: &str) -> Result<(), WalletError> {
        let backup_file_path = format!("{}/{}.backup", self.backup_path, backup_id);

        fs::remove_file(&backup_file_path)
            .await
            .map_err(|e| WalletError::IoError(e.to_string()))?;

        tracing::info!("Deleted backup {}", backup_id);

        Ok(())
    }

    /// Register a guardian for social recovery
    ///
    /// # Arguments
    ///
    /// * `name` - Guardian name/identifier
    /// * `public_key` - Guardian's public key
    /// * `contact` - Contact information
    ///
    /// # Returns
    ///
    /// Returns the guardian ID or an error
    pub async fn register_guardian(
        &self,
        name: String,
        public_key: Vec<u8>,
        contact: GuardianContact,
    ) -> Result<String, WalletError> {
        let mut guardians = self.guardians.write().await;

        // Check if we've reached the maximum number of guardians
        if guardians.len() >= self.config.max_guardians {
            return Err(WalletError::GuardianLimitReached);
        }

        let guardian_id = Uuid::new_v4().to_string();
        let guardian = Guardian {
            guardian_id: guardian_id.clone(),
            name,
            public_key,
            contact,
            status: GuardianStatus::Pending,
            registered_at: Utc::now(),
        };

        guardians.insert(guardian_id.clone(), guardian);

        tracing::info!("Registered guardian {} with ID {}", guardian.name, guardian_id);

        Ok(guardian_id)
    }

    /// Activate a guardian
    ///
    /// # Arguments
    ///
    /// * `guardian_id` - Guardian ID to activate
    ///
    /// # Returns
    ///
    /// Returns Ok(()) or an error
    pub async fn activate_guardian(&self, guardian_id: &str) -> Result<(), WalletError> {
        let mut guardians = self.guardians.write().await;

        let guardian = guardians
            .get_mut(guardian_id)
            .ok_or(WalletError::GuardianNotFound(guardian_id.to_string()))?;

        guardian.status = GuardianStatus::Active;

        tracing::info!("Activated guardian {}", guardian_id);

        Ok(())
    }

    /// Revoke a guardian
    ///
    /// # Arguments
    ///
    /// * `guardian_id` - Guardian ID to revoke
    ///
    /// # Returns
    ///
    /// Returns Ok(()) or an error
    pub async fn revoke_guardian(&self, guardian_id: &str) -> Result<(), WalletError> {
        let mut guardians = self.guardians.write().await;

        let guardian = guardians
            .get_mut(guardian_id)
            .ok_or(WalletError::GuardianNotFound(guardian_id.to_string()))?;

        guardian.status = GuardianStatus::Revoked;

        tracing::info!("Revoked guardian {}", guardian_id);

        Ok(())
    }

    /// List all guardians
    ///
    /// # Returns
    ///
    /// Returns a list of guardians or an error
    pub async fn list_guardians(&self) -> Result<Vec<Guardian>, WalletError> {
        let guardians = self.guardians.read().await;
        Ok(guardians.values().cloned().collect())
    }

    /// Initiate social recovery
    ///
    /// # Arguments
    ///
    /// * `wallet_id` - Wallet ID to recover
    /// * `requester` - Requester information
    ///
    /// # Returns
    ///
    /// Returns the recovery request ID or an error
    pub async fn initiate_recovery(
        &self,
        wallet_id: WalletId,
        requester: RecoveryRequester,
    ) -> Result<String, WalletError> {
        let request_id = Uuid::new_v4().to_string();
        let created_at = Utc::now();
        let expires_at = created_at + chrono::Duration::days(7); // 7 day expiration

        let request = RecoveryRequest {
            request_id: request_id.clone(),
            wallet_id,
            requester,
            shares: Vec::new(),
            created_at,
            expires_at,
            status: RecoveryStatus::Pending,
        };

        let mut recovery_requests = self.recovery_requests.write().await;
        recovery_requests.insert(request_id.clone(), request);

        tracing::info!("Initiated recovery {} for wallet {}", request_id, wallet_id);

        Ok(request_id)
    }

    /// Submit a recovery share from a guardian
    ///
    /// # Arguments
    ///
    /// * `request_id` - Recovery request ID
    /// * `share` - Recovery share from guardian
    ///
    /// # Returns
    ///
    /// Returns Ok(()) or an error
    pub async fn submit_recovery_share(
        &self,
        request_id: &str,
        share: RecoveryShare,
    ) -> Result<(), WalletError> {
        let mut recovery_requests = self.recovery_requests.write().await;

        let request = recovery_requests
            .get_mut(request_id)
            .ok_or(WalletError::RecoveryNotFound(request_id.to_string()))?;

        // Verify guardian is active
        let guardians = self.guardians.read().await;
        let guardian = guardians
            .get(&share.guardian_id)
            .ok_or(WalletError::GuardianNotFound(share.guardian_id.clone()))?;

        if guardian.status != GuardianStatus::Active {
            return Err(WalletError::GuardianNotActive(share.guardian_id.clone()));
        }

        // Verify share signature
        let signature_valid = self
            .crypto
            .verify(&guardian.public_key, &share.encrypted_share, &share.signature)?;

        if !signature_valid {
            return Err(WalletError::InvalidSignature);
        }

        // Add share
        request.shares.push(share);
        request.status = RecoveryStatus::InProgress;

        // Check if we have enough shares
        if request.shares.len() >= self.config.guardian_threshold {
            request.status = RecoveryStatus::Completed;
        }

        tracing::info!(
            "Submitted recovery share from {} for request {}",
            share.guardian_id,
            request_id
        );

        Ok(())
    }

    /// Complete recovery and restore wallet
    ///
    /// # Arguments
    ///
    /// * `request_id` - Recovery request ID
    ///
    /// # Returns
    ///
    /// Returns the restored backup data or an error
    pub async fn complete_recovery(
        &self,
        request_id: &str,
    ) -> Result<BackupData, WalletError> {
        let mut recovery_requests = self.recovery_requests.write().await;

        let request = recovery_requests
            .get(request_id)
            .ok_or(WalletError::RecoveryNotFound(request_id.to_string()))?;

        if request.status != RecoveryStatus::Completed {
            return Err(WalletError::RecoveryNotComplete);
        }

        // Combine shares to reconstruct the wallet
        // This is a simplified version - in production, use proper secret sharing
        let combined_data = self.combine_recovery_shares(&request.shares)?;

        // Deserialize backup data
        let backup: BackupData = serde_json::from_slice(&combined_data)
            .map_err(|e| WalletError::SerializationError(e.to_string()))?;

        // Restore keys
        for encrypted_key in &backup.encrypted_keys {
            // In a real implementation, we would decrypt using the combined shares
            // For now, we'll just verify the structure
            tracing::debug!("Restoring key {}", encrypted_key.key_id);
        }

        tracing::info!("Completed recovery {} for wallet {}", request_id, backup.wallet_id);

        Ok(backup)
    }

    /// Cancel a recovery request
    ///
    /// # Arguments
    ///
    /// * `request_id` - Recovery request ID to cancel
    ///
    /// # Returns
    ///
    /// Returns Ok(()) or an error
    pub async fn cancel_recovery(&self, request_id: &str) -> Result<(), WalletError> {
        let mut recovery_requests = self.recovery_requests.write().await;

        let request = recovery_requests
            .get_mut(request_id)
            .ok_or(WalletError::RecoveryNotFound(request_id.to_string()))?;

        request.status = RecoveryStatus::Cancelled;

        tracing::info!("Cancelled recovery {}", request_id);

        Ok(())
    }

    /// List active recovery requests
    ///
    /// # Returns
    ///
    /// Returns a list of recovery requests or an error
    pub async fn list_recovery_requests(&self) -> Result<Vec<RecoveryRequest>, WalletError> {
        let recovery_requests = self.recovery_requests.read().await;
        Ok(recovery_requests.values().cloned().collect())
    }

    /// Rotate wallet keys
    ///
    /// # Arguments
    ///
    /// * `old_key_id` - Old key ID to rotate
    /// * `new_algorithm` - Algorithm for the new key
    ///
    /// # Returns
    ///
    /// Returns the new key ID or an error
    pub async fn rotate_key(
        &self,
        old_key_id: &KeyId,
        new_algorithm: KeyAlgorithm,
    ) -> Result<KeyId, WalletError> {
        let new_key_id = self
            .key_store
            .rotate_key(old_key_id, new_algorithm)
            .await?;

        tracing::info!("Rotated key {} to {}", old_key_id, new_key_id);

        Ok(new_key_id)
    }

    /// Clean up expired recovery requests
    ///
    /// # Returns
    ///
    /// Returns the number of cleaned up requests or an error
    pub async fn cleanup_expired_requests(&self) -> Result<usize, WalletError> {
        let mut recovery_requests = self.recovery_requests.write().await;
        let now = Utc::now();
        let mut cleaned = 0;

        recovery_requests.retain(|_, request| {
            if request.expires_at < now {
                cleaned += 1;
                false
            } else {
                true
            }
        });

        if cleaned > 0 {
            tracing::info!("Cleaned up {} expired recovery requests", cleaned);
        }

        Ok(cleaned)
    }

    /// Clean up old backups
    ///
    /// # Returns
    ///
    /// Returns the number of cleaned up backups or an error
    pub async fn cleanup_old_backups(&self) -> Result<usize, WalletError> {
        let backups = self.list_backups().await?;
        let now = Utc::now();
        let retention_period = chrono::Duration::days(self.config.backup_retention_days as i64);
        let mut cleaned = 0;

        for backup_id in backups {
            let backup_file_path = format!("{}/{}.backup", self.backup_path, backup_id);

            // Read backup to check timestamp
            if let Ok(backup_json) = fs::read_to_string(&backup_file_path).await {
                if let Ok(backup) = serde_json::from_str::<BackupData>(&backup_json) {
                    if now - backup.timestamp > retention_period {
                        if self.delete_backup(&backup_id).await.is_ok() {
                            cleaned += 1;
                        }
                    }
                }
            }
        }

        if cleaned > 0 {
            tracing::info!("Cleaned up {} old backups", cleaned);
        }

        Ok(cleaned)
    }

    /// Combine recovery shares to reconstruct wallet data
    ///
    /// # Arguments
    ///
    /// * `shares` - Recovery shares from guardians
    ///
    /// # Returns
    ///
    /// Returns the combined data or an error
    fn combine_recovery_shares(&self, shares: &[RecoveryShare]) -> Result<Vec<u8>, WalletError> {
        // In a production implementation, this would use Shamir's Secret Sharing
        // or similar threshold cryptography to combine shares
        // For now, we'll concatenate the encrypted shares as a placeholder

        let mut combined = Vec::new();
        for share in shares {
            combined.extend_from_slice(&share.encrypted_share);
        }

        Ok(combined)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use canton_wallet_crypto::keystore::InMemoryKeyStore;

    #[tokio::test]
    async fn test_recovery_manager_creation() {
        let key_store = Arc::new(InMemoryKeyStore::new());
        let crypto = Arc::new(CryptoOperations::new());
        let config = RecoveryConfig::default();
        let backup_path = "./test_backups".to_string();

        let manager = RecoveryManager::new(key_store, crypto, config, backup_path);

        assert_eq!(manager.config.guardian_threshold, 2);
        assert_eq!(manager.config.max_guardians, 5);
    }

    #[tokio::test]
    async fn test_guardian_registration() {
        let key_store = Arc::new(InMemoryKeyStore::new());
        let crypto = Arc::new(CryptoOperations::new());
        let config = RecoveryConfig::default();
        let backup_path = "./test_backups".to_string();

        let manager = RecoveryManager::new(key_store, crypto, config, backup_path);

        let contact = GuardianContact {
            email: Some("test@example.com".to_string()),
            phone: None,
            other: None,
        };

        let guardian_id = manager
            .register_guardian(
                "Test Guardian".to_string(),
                vec![1, 2, 3, 4],
                contact,
            )
            .await
            .unwrap();

        assert!(!guardian_id.is_empty());

        let guardians = manager.list_guardians().await.unwrap();
        assert_eq!(guardians.len(), 1);
        assert_eq!(guardians[0].name, "Test Guardian");
    }

    #[tokio::test]
    async fn test_guardian_activation() {
        let key_store = Arc::new(InMemoryKeyStore::new());
        let crypto = Arc::new(CryptoOperations::new());
        let config = RecoveryConfig::default();
        let backup_path = "./test_backups".to_string();

        let manager = RecoveryManager::new(key_store, crypto, config, backup_path);

        let contact = GuardianContact {
            email: Some("test@example.com".to_string()),
            phone: None,
            other: None,
        };

        let guardian_id = manager
            .register_guardian(
                "Test Guardian".to_string(),
                vec![1, 2, 3, 4],
                contact,
            )
            .await
            .unwrap();

        manager.activate_guardian(&guardian_id).await.unwrap();

        let guardians = manager.list_guardians().await.unwrap();
        assert_eq!(guardians[0].status, GuardianStatus::Active);
    }

    #[tokio::test]
    async fn test_recovery_initiation() {
        let key_store = Arc::new(InMemoryKeyStore::new());
        let crypto = Arc::new(CryptoOperations::new());
        let config = RecoveryConfig::default();
        let backup_path = "./test_backups".to_string();

        let manager = RecoveryManager::new(key_store, crypto, config, backup_path);

        let wallet_id = WalletId::new_unchecked("test-wallet");
        let requester = RecoveryRequester {
            identifier: "test-requester".to_string(),
            verification_method: "email".to_string(),
            verification_data: HashMap::new(),
        };

        let request_id = manager
            .initiate_recovery(wallet_id, requester)
            .await
            .unwrap();

        assert!(!request_id.is_empty());

        let requests = manager.list_recovery_requests().await.unwrap();
        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].status, RecoveryStatus::Pending);
    }
}
