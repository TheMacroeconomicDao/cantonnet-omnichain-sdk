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

//! Audit logging for security and compliance.

use crate::error::{Result, SecurityError};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Audit log entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogEntry {
    /// Timestamp of the log entry.
    pub timestamp: DateTime<Utc>,

    /// Operation that was performed.
    pub operation: String,

    /// Wallet ID that performed the operation.
    #[serde(default)]
    pub wallet_id: String,

    /// Transaction ID if applicable.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transaction_id: Option<String>,

    /// Additional details about the operation.
    #[serde(default)]
    pub details: serde_json::Value,

    /// User ID if applicable.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<String>,

    /// IP address if applicable.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ip_address: Option<String>,

    /// User agent if applicable.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_agent: Option<String>,
}

impl AuditLogEntry {
    /// Create a new audit log entry.
    pub fn new(operation: String) -> Self {
        Self {
            timestamp: Utc::now(),
            operation,
            wallet_id: String::new(),
            transaction_id: None,
            details: serde_json::Value::Null,
            user_id: None,
            ip_address: None,
            user_agent: None,
        }
    }

    /// Set the wallet ID.
    pub fn with_wallet_id(mut self, wallet_id: String) -> Self {
        self.wallet_id = wallet_id;
        self
    }

    /// Set the transaction ID.
    pub fn with_transaction_id(mut self, transaction_id: String) -> Self {
        self.transaction_id = Some(transaction_id);
        self
    }

    /// Set the user ID.
    pub fn with_user_id(mut self, user_id: String) -> Self {
        self.user_id = Some(user_id);
        self
    }

    /// Set the IP address.
    pub fn with_ip_address(mut self, ip_address: String) -> Self {
        self.ip_address = Some(ip_address);
        self
    }

    /// Set the user agent.
    pub fn with_user_agent(mut self, user_agent: String) -> Self {
        self.user_agent = Some(user_agent);
        self
    }

    /// Set the details.
    pub fn with_details(mut self, details: serde_json::Value) -> Self {
        self.details = details;
        self
    }
}

/// Audit logger trait for custom audit logging implementations.
#[async_trait]
pub trait AuditLogger: Send + Sync {
    /// Log an audit entry.
    ///
    /// # Arguments
    ///
    /// * `entry` - Audit log entry to log
    ///
    /// # Returns
    ///
    /// Returns a result indicating success or failure
    async fn log(&self, entry: AuditLogEntry) -> Result<()>;

    /// Query audit logs.
    ///
    /// # Arguments
    ///
    /// * `filter` - Filter to apply to logs
    ///
    /// # Returns
    ///
    /// Returns a vector of matching audit log entries
    async fn query(&self, filter: AuditFilter) -> Result<Vec<AuditLogEntry>>;

    /// Get audit log statistics.
    ///
    /// # Returns
    ///
    /// Returns audit log statistics
    async fn statistics(&self) -> Result<AuditStatistics>;
}

/// Filter for querying audit logs.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AuditFilter {
    /// Start timestamp for filtering.
    pub start_time: Option<DateTime<Utc>>,

    /// End timestamp for filtering.
    pub end_time: Option<DateTime<Utc>>,

    /// Operation to filter by.
    pub operation: Option<String>,

    /// Wallet ID to filter by.
    pub wallet_id: Option<String>,

    /// Transaction ID to filter by.
    pub transaction_id: Option<String>,

    /// User ID to filter by.
    pub user_id: Option<String>,

    /// Maximum number of results to return.
    pub limit: Option<usize>,

    /// Offset for pagination.
    pub offset: Option<usize>,
}

impl AuditFilter {
    /// Create a new audit filter.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the start time.
    pub fn with_start_time(mut self, time: DateTime<Utc>) -> Self {
        self.start_time = Some(time);
        self
    }

    /// Set the end time.
    pub fn with_end_time(mut self, time: DateTime<Utc>) -> Self {
        self.end_time = Some(time);
        self
    }

    /// Set the operation.
    pub fn with_operation(mut self, operation: String) -> Self {
        self.operation = Some(operation);
        self
    }

    /// Set the wallet ID.
    pub fn with_wallet_id(mut self, wallet_id: String) -> Self {
        self.wallet_id = Some(wallet_id);
        self
    }

    /// Set the transaction ID.
    pub fn with_transaction_id(mut self, transaction_id: String) -> Self {
        self.transaction_id = Some(transaction_id);
        self
    }

    /// Set the user ID.
    pub fn with_user_id(mut self, user_id: String) -> Self {
        self.user_id = Some(user_id);
        self
    }

    /// Set the limit.
    pub fn with_limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Set the offset.
    pub fn with_offset(mut self, offset: usize) -> Self {
        self.offset = Some(offset);
        self
    }
}

/// Audit log statistics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditStatistics {
    /// Total number of log entries.
    pub total_entries: usize,

    /// Number of entries by operation type.
    pub entries_by_operation: std::collections::HashMap<String, usize>,

    /// Number of entries by wallet ID.
    pub entries_by_wallet: std::collections::HashMap<String, usize>,

    /// Number of entries by user ID.
    pub entries_by_user: std::collections::HashMap<String, usize>,

    /// Earliest log entry timestamp.
    pub earliest_entry: Option<DateTime<Utc>>,

    /// Latest log entry timestamp.
    pub latest_entry: Option<DateTime<Utc>>,
}

impl Default for AuditStatistics {
    fn default() -> Self {
        Self {
            total_entries: 0,
            entries_by_operation: std::collections::HashMap::new(),
            entries_by_wallet: std::collections::HashMap::new(),
            entries_by_user: std::collections::HashMap::new(),
            earliest_entry: None,
            latest_entry: None,
        }
    }
}

/// In-memory audit logger for development and testing.
pub struct InMemoryAuditLogger {
    entries: Arc<parking_lot::RwLock<Vec<AuditLogEntry>>>,
}

impl InMemoryAuditLogger {
    /// Create a new in-memory audit logger.
    pub fn new() -> Self {
        Self {
            entries: Arc::new(parking_lot::RwLock::new(Vec::new())),
        }
    }

    /// Clear all audit log entries.
    pub fn clear(&self) {
        self.entries.write().clear();
    }

    /// Get the number of audit log entries.
    pub fn len(&self) -> usize {
        self.entries.read().len()
    }

    /// Check if the audit log is empty.
    pub fn is_empty(&self) -> bool {
        self.entries.read().is_empty()
    }
}

impl Default for InMemoryAuditLogger {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl AuditLogger for InMemoryAuditLogger {
    async fn log(&self, entry: AuditLogEntry) -> Result<()> {
        tracing::info!(
            operation = %entry.operation,
            wallet_id = %entry.wallet_id,
            transaction_id = ?entry.transaction_id,
            "Audit log entry"
        );

        self.entries.write().push(entry);
        Ok(())
    }

    async fn query(&self, filter: AuditFilter) -> Result<Vec<AuditLogEntry>> {
        let entries = self.entries.read();

        let mut results: Vec<AuditLogEntry> = entries
            .iter()
            .filter(|entry| {
                if let Some(start_time) = filter.start_time {
                    if entry.timestamp < start_time {
                        return false;
                    }
                }
                if let Some(end_time) = filter.end_time {
                    if entry.timestamp > end_time {
                        return false;
                    }
                }
                if let Some(ref operation) = filter.operation {
                    if entry.operation != *operation {
                        return false;
                    }
                }
                if let Some(ref wallet_id) = filter.wallet_id {
                    if entry.wallet_id != *wallet_id {
                        return false;
                    }
                }
                if let Some(ref transaction_id) = filter.transaction_id {
                    if entry.transaction_id.as_ref() != Some(transaction_id) {
                        return false;
                    }
                }
                if let Some(ref user_id) = filter.user_id {
                    if entry.user_id.as_ref() != Some(user_id) {
                        return false;
                    }
                }
                true
            })
            .cloned()
            .collect();

        // Apply pagination
        if let Some(offset) = filter.offset {
            if offset < results.len() {
                results = results.into_iter().skip(offset).collect();
            } else {
                results.clear();
            }
        }

        if let Some(limit) = filter.limit {
            results.truncate(limit);
        }

        Ok(results)
    }

    async fn statistics(&self) -> Result<AuditStatistics> {
        let entries = self.entries.read();

        let mut stats = AuditStatistics::default();
        stats.total_entries = entries.len();

        for entry in entries.iter() {
            *stats
                .entries_by_operation
                .entry(entry.operation.clone())
                .or_insert(0) += 1;
            *stats
                .entries_by_wallet
                .entry(entry.wallet_id.clone())
                .or_insert(0) += 1;
            if let Some(ref user_id) = entry.user_id {
                *stats
                    .entries_by_user
                    .entry(user_id.clone())
                    .or_insert(0) += 1;
            }

            if stats.earliest_entry.is_none() || entry.timestamp < stats.earliest_entry.unwrap() {
                stats.earliest_entry = Some(entry.timestamp);
            }
            if stats.latest_entry.is_none() || entry.timestamp > stats.latest_entry.unwrap() {
                stats.latest_entry = Some(entry.timestamp);
            }
        }

        Ok(stats)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_audit_log_entry_new() {
        let entry = AuditLogEntry::new("test_operation".to_string());
        assert_eq!(entry.operation, "test_operation");
        assert!(entry.wallet_id.is_empty());
        assert!(entry.transaction_id.is_none());
    }

    #[tokio::test]
    async fn test_audit_log_entry_with_wallet_id() {
        let entry = AuditLogEntry::new("test_operation".to_string())
            .with_wallet_id("wallet-123".to_string());
        assert_eq!(entry.wallet_id, "wallet-123");
    }

    #[tokio::test]
    async fn test_audit_filter_new() {
        let filter = AuditFilter::new();
        assert!(filter.start_time.is_none());
        assert!(filter.end_time.is_none());
        assert!(filter.operation.is_none());
    }

    #[tokio::test]
    async fn test_audit_filter_with_operation() {
        let filter = AuditFilter::new().with_operation("test_operation".to_string());
        assert_eq!(filter.operation, Some("test_operation".to_string()));
    }

    #[tokio::test]
    async fn test_in_memory_audit_logger_log() {
        let logger = InMemoryAuditLogger::new();
        let entry = AuditLogEntry::new("test_operation".to_string());

        logger.log(entry.clone()).await.unwrap();
        assert_eq!(logger.len(), 1);
    }

    #[tokio::test]
    async fn test_in_memory_audit_logger_query() {
        let logger = InMemoryAuditLogger::new();
        let entry1 = AuditLogEntry::new("operation1".to_string())
            .with_wallet_id("wallet1".to_string());
        let entry2 = AuditLogEntry::new("operation2".to_string())
            .with_wallet_id("wallet2".to_string());

        logger.log(entry1).await.unwrap();
        logger.log(entry2).await.unwrap();

        let filter = AuditFilter::new().with_operation("operation1".to_string());
        let results = logger.query(filter).await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].operation, "operation1");
    }

    #[tokio::test]
    async fn test_in_memory_audit_logger_statistics() {
        let logger = InMemoryAuditLogger::new();
        let entry1 = AuditLogEntry::new("operation1".to_string())
            .with_wallet_id("wallet1".to_string());
        let entry2 = AuditLogEntry::new("operation2".to_string())
            .with_wallet_id("wallet1".to_string());

        logger.log(entry1).await.unwrap();
        logger.log(entry2).await.unwrap();

        let stats = logger.statistics().await.unwrap();
        assert_eq!(stats.total_entries, 2);
        assert_eq!(stats.entries_by_wallet.get("wallet1"), Some(&2));
    }
}
