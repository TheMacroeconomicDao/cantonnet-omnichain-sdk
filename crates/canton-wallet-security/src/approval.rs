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

//! Transaction approval management.

use crate::error::{Result, SecurityError};
use async_trait::async_trait;
use canton_wallet_core::types::Transaction;
use canton_wallet_transactions::validator::TransactionValidator;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

/// Approval response from user.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalResponse {
    /// Whether the transaction was approved.
    pub approved: bool,

    /// Timestamp of the approval.
    pub timestamp: DateTime<Utc>,

    /// Additional metadata.
    #[serde(default)]
    pub metadata: HashMap<String, String>,
}

impl ApprovalResponse {
    /// Create a new approval response.
    pub fn new(approved: bool) -> Self {
        Self {
            approved,
            timestamp: Utc::now(),
            metadata: HashMap::new(),
        }
    }

    /// Create an approved response.
    pub fn approved() -> Self {
        Self::new(true)
    }

    /// Create a rejected response.
    pub fn rejected() -> Self {
        Self::new(false)
    }

    /// Add metadata to the response.
    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }
}

/// User approval trait for custom approval implementations.
#[async_trait]
pub trait UserApproval: Send + Sync {
    /// Request approval for a transaction.
    ///
    /// # Arguments
    ///
    /// * `tx` - Transaction to approve
    ///
    /// # Returns
    ///
    /// Returns the approval response
    async fn request_approval(&self, tx: &Transaction) -> Result<ApprovalResponse>;
}

/// Default user approval that auto-approves all transactions.
/// This should only be used in development/testing environments.
pub struct AutoApproval;

#[async_trait]
impl UserApproval for AutoApproval {
    async fn request_approval(&self, _tx: &Transaction) -> Result<ApprovalResponse> {
        Ok(ApprovalResponse::approved())
    }
}

/// User approval that always rejects transactions.
/// This can be used for testing or in specific security scenarios.
pub struct RejectAllApproval;

#[async_trait]
impl UserApproval for RejectAllApproval {
    async fn request_approval(&self, _tx: &Transaction) -> Result<ApprovalResponse> {
        Ok(ApprovalResponse::rejected())
    }
}

/// Approval policy for determining when approval is required.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ApprovalPolicy {
    /// Approval required for all transactions.
    Always,

    /// Approval required only for transactions above a threshold.
    Threshold(u64),

    /// No approval required.
    Never,
}

impl Default for ApprovalPolicy {
    fn default() -> Self {
        Self::Always
    }
}

/// Transaction approval manager.
pub struct ApprovalManager {
    user_approval: Arc<dyn UserApproval>,
    validator: TransactionValidator,
    audit_logger: Arc<AuditLogger>,
    policy: ApprovalPolicy,
}

impl ApprovalManager {
    /// Create a new approval manager.
    ///
    /// # Arguments
    ///
    /// * `user_approval` - User approval implementation
    /// * `validator` - Transaction validator
    /// * `audit_logger` - Audit logger
    ///
    /// # Returns
    ///
    /// Returns a new `ApprovalManager` instance
    pub fn new(
        user_approval: Arc<dyn UserApproval>,
        validator: TransactionValidator,
        audit_logger: Arc<AuditLogger>,
    ) -> Self {
        Self {
            user_approval,
            validator,
            audit_logger,
            policy: ApprovalPolicy::default(),
        }
    }

    /// Set the approval policy.
    ///
    /// # Arguments
    ///
    /// * `policy` - Approval policy to use
    ///
    /// # Returns
    ///
    /// Returns self for method chaining
    pub fn with_policy(mut self, policy: ApprovalPolicy) -> Self {
        self.policy = policy;
        self
    }

    /// Check if approval is required for a transaction.
    ///
    /// # Arguments
    ///
    /// * `tx` - Transaction to check
    ///
    /// # Returns
    ///
    /// Returns true if approval is required
    fn requires_approval(&self, tx: &Transaction) -> bool {
        match self.policy {
            ApprovalPolicy::Always => true,
            ApprovalPolicy::Threshold(threshold) => {
                // Check if transaction value exceeds threshold
                // This is a simplified check - in production, you'd need to
                // properly extract the value from the transaction
                tx.commands.len() as u64 > threshold
            }
            ApprovalPolicy::Never => false,
        }
    }

    /// Request approval for a transaction.
    ///
    /// # Arguments
    ///
    /// * `tx` - Transaction to approve
    ///
    /// # Returns
    ///
    /// Returns the approval response
    pub async fn request_approval(&self, tx: &Transaction) -> Result<ApprovalResponse> {
        // Validate transaction first
        self.validator.validate_commands(&tx.commands)?;

        // Check if approval is required
        if !self.requires_approval(tx) {
            tracing::debug!("Transaction does not require approval based on policy");
            return Ok(ApprovalResponse::approved());
        }

        // Log approval request
        self.audit_logger
            .log(AuditLogEntry {
                timestamp: Utc::now(),
                operation: "transaction_approval_request".to_string(),
                wallet_id: tx.party_id.clone(),
                transaction_id: Some(tx.transaction_id.clone()),
                details: serde_json::to_value(tx).unwrap_or_default(),
            })
            .await?;

        // Request user approval
        let response = self.user_approval.request_approval(tx).await?;

        // Log approval response
        self.audit_logger
            .log(AuditLogEntry {
                timestamp: Utc::now(),
                operation: "transaction_approval_response".to_string(),
                wallet_id: tx.party_id.clone(),
                transaction_id: Some(tx.transaction_id.clone()),
                details: serde_json::json!({
                    "approved": response.approved,
                    "timestamp": response.timestamp,
                }),
            })
            .await?;

        if !response.approved {
            return Err(SecurityError::UserRejected);
        }

        Ok(response)
    }

    /// Validate and approve a transaction.
    ///
    /// # Arguments
    ///
    /// * `tx` - Transaction to validate and approve
    ///
    /// # Returns
    ///
    /// Returns the approval response
    pub async fn validate_and_approve(&self, tx: &Transaction) -> Result<ApprovalResponse> {
        self.validator.validate_commands(&tx.commands)?;
        self.request_approval(tx).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_approval_response_approved() {
        let response = ApprovalResponse::approved();
        assert!(response.approved);
    }

    #[tokio::test]
    async fn test_approval_response_rejected() {
        let response = ApprovalResponse::rejected();
        assert!(!response.approved);
    }

    #[tokio::test]
    async fn test_approval_response_with_metadata() {
        let response = ApprovalResponse::new(true)
            .with_metadata("key".to_string(), "value".to_string());
        assert!(response.approved);
        assert_eq!(response.metadata.get("key"), Some(&"value".to_string()));
    }

    #[tokio::test]
    async fn test_auto_approval() {
        let approval = AutoApproval;
        let tx = Transaction::default();
        let response = approval.request_approval(&tx).await.unwrap();
        assert!(response.approved);
    }

    #[tokio::test]
    async fn test_reject_all_approval() {
        let approval = RejectAllApproval;
        let tx = Transaction::default();
        let response = approval.request_approval(&tx).await.unwrap();
        assert!(!response.approved);
    }

    #[test]
    fn test_approval_policy_default() {
        let policy = ApprovalPolicy::default();
        assert_eq!(policy, ApprovalPolicy::Always);
    }
}
