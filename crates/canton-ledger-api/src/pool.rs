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

//! Connection pool for ledger API connections

use crate::client::LedgerClientConfig;
use crate::error::{LedgerError, LedgerResult};
use crate::proto::{
    ActiveContracts, ArchivedEvent, Command, Completion, CompletionStatus, CreatedEvent,
    Event, GetActiveContractsRequest, GetTransactionsRequest, Identifier, LedgerOffset,
    PartyId, SubmitAndWaitForTransactionRequest, SubmitAndWaitRequest, SubmitRequest,
    Transaction, TransactionFilter,
};
use async_trait::async_trait;
use futures::Stream;
use std::sync::Arc;
use tokio::sync::{Mutex, Semaphore};
use tokio::time::{timeout, Duration};
use tracing::{debug, error, info, instrument, warn};

/// Connection pool configuration
#[derive(Debug, Clone)]
pub struct PoolConfig {
    /// Maximum number of connections
    pub max_connections: usize,
    /// Minimum number of connections to maintain
    pub min_connections: usize,
    /// Connection timeout in seconds
    pub connection_timeout_secs: u64,
    /// Idle timeout in seconds
    pub idle_timeout_secs: u64,
    /// Maximum lifetime of a connection in seconds
    pub max_lifetime_secs: u64,
}

impl Default for PoolConfig {
    fn default() -> Self {
        Self {
            max_connections: 10,
            min_connections: 1,
            connection_timeout_secs: 30,
            idle_timeout_secs: 300,
            max_lifetime_secs: 3600,
        }
    }
}

/// Connection pool for ledger API
pub struct ConnectionPool {
    config: LedgerClientConfig,
    pool_config: PoolConfig,
    connections: Arc<Mutex<Vec<PooledConnection>>>,
    semaphore: Arc<Semaphore>,
    is_connected: Arc<Mutex<bool>>,
}

impl ConnectionPool {
    /// Create a new connection pool
    pub async fn new(config: LedgerClientConfig) -> LedgerResult<Self> {
        let pool_config = PoolConfig {
            max_connections: config.max_connections,
            ..Default::default()
        };

        let semaphore = Arc::new(Semaphore::new(pool_config.max_connections));
        let connections = Arc::new(Mutex::new(Vec::new()));
        let is_connected = Arc::new(Mutex::new(false));

        let pool = Self {
            config,
            pool_config,
            connections,
            semaphore,
            is_connected,
        };

        // Initialize minimum connections
        pool.initialize().await?;

        Ok(pool)
    }

    /// Initialize the pool with minimum connections
    async fn initialize(&self) -> LedgerResult<()> {
        debug!("Initializing connection pool");

        let mut connections = self.connections.lock().await;
        for _ in 0..self.pool_config.min_connections {
            let conn = self.create_connection().await?;
            connections.push(conn);
        }

        *self.is_connected.lock().await = true;

        info!(
            "Connection pool initialized with {} connections",
            self.pool_config.min_connections
        );

        Ok(())
    }

    /// Create a new connection
    async fn create_connection(&self) -> LedgerResult<PooledConnection> {
        debug!("Creating new ledger connection");

        let conn = LedgerConnection::connect(&self.config).await?;

        Ok(PooledConnection {
            conn,
            created_at: std::time::Instant::now(),
            last_used: std::time::Instant::now(),
        })
    }

    /// Acquire a connection from the pool
    pub async fn acquire(&self) -> LedgerResult<ConnectionGuard> {
        let _permit = self.semaphore.acquire().await.map_err(|e| {
            LedgerError::ConnectionError(format!("Failed to acquire semaphore: {}", e))
        })?;

        let mut connections = self.connections.lock().await;

        // Find an available connection
        let index = connections
            .iter()
            .position(|c| !c.in_use);

        let conn = if let Some(index) = index {
            // Reuse existing connection
            let mut pooled = connections.remove(index);
            pooled.in_use = true;
            pooled.last_used = std::time::Instant::now();
            connections.push(pooled);
            connections.len() - 1
        } else if connections.len() < self.pool_config.max_connections {
            // Create new connection
            let pooled = self.create_connection().await?;
            connections.push(pooled);
            connections.len() - 1
        } else {
            // Wait for a connection to become available
            drop(connections);
            tokio::time::sleep(Duration::from_millis(100)).await;
            return self.acquire().await;
        };

        let pool = self.clone();
        let guard = ConnectionGuard {
            pool,
            index,
        };

        Ok(guard)
    }

    /// Release a connection back to the pool
    async fn release(&self, index: usize) {
        let mut connections = self.connections.lock().await;

        if let Some(conn) = connections.get_mut(index) {
            conn.in_use = false;
            conn.last_used = std::time::Instant::now();
        }
    }

    /// Check if the pool is connected
    pub async fn is_connected(&self) -> bool {
        *self.is_connected.lock().await
    }

    /// Close all connections
    pub async fn close(&self) -> LedgerResult<()> {
        debug!("Closing connection pool");

        let mut connections = self.connections.lock().await;
        connections.clear();

        *self.is_connected.lock().await = false;

        info!("Connection pool closed");

        Ok(())
    }

    /// Get pool statistics
    pub async fn stats(&self) -> PoolStats {
        let connections = self.connections.lock().await;
        let active = connections.iter().filter(|c| c.in_use).count();
        let idle = connections.len() - active;

        PoolStats {
            total: connections.len(),
            active,
            idle,
            max_connections: self.pool_config.max_connections,
        }
    }
}

impl Clone for ConnectionPool {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            pool_config: self.pool_config.clone(),
            connections: self.connections.clone(),
            semaphore: self.semaphore.clone(),
            is_connected: self.is_connected.clone(),
        }
    }
}

/// Pooled connection
struct PooledConnection {
    conn: LedgerConnection,
    created_at: std::time::Instant,
    last_used: std::time::Instant,
    in_use: bool,
}

/// Connection guard that releases the connection when dropped
pub struct ConnectionGuard {
    pool: ConnectionPool,
    index: usize,
}

impl ConnectionGuard {
    /// Get the underlying connection
    pub async fn connection(&mut self) -> LedgerResult<&mut LedgerConnection> {
        let mut connections = self.pool.connections.lock().await;
        let pooled = connections
            .get_mut(self.index)
            .ok_or_else(|| LedgerError::ConnectionError("Connection not found".to_string()))?;

        Ok(&mut pooled.conn)
    }
}

impl Drop for ConnectionGuard {
    fn drop(&mut self) {
        let pool = self.pool.clone();
        let index = self.index;
        tokio::spawn(async move {
            pool.release(index).await;
        });
    }
}

/// Pool statistics
#[derive(Debug, Clone)]
pub struct PoolStats {
    pub total: usize,
    pub active: usize,
    pub idle: usize,
    pub max_connections: usize,
}

/// Ledger connection
pub struct LedgerConnection {
    endpoint: String,
    ledger_id: String,
    application_id: String,
}

impl LedgerConnection {
    /// Connect to the ledger
    pub async fn connect(config: &LedgerClientConfig) -> LedgerResult<Self> {
        debug!("Connecting to ledger at: {}", config.endpoint);

        // In a real implementation, this would establish a gRPC connection
        // For now, we simulate a connection

        let conn = Self {
            endpoint: config.endpoint.clone(),
            ledger_id: config.ledger_id.clone(),
            application_id: config.application_id.clone(),
        };

        info!("Connected to ledger at: {}", config.endpoint);

        Ok(conn)
    }

    /// Get ledger ID
    pub async fn get_ledger_id(&mut self) -> LedgerResult<String> {
        Ok(self.ledger_id.clone())
    }

    /// Check if party exists
    pub async fn party_exists(&mut self, party_id: &PartyId) -> LedgerResult<bool> {
        // In a real implementation, this would query the ledger
        Ok(true)
    }

    /// Get party details
    pub async fn get_party_details(&mut self, party_id: &PartyId) -> LedgerResult<crate::client::PartyDetails> {
        Ok(crate::client::PartyDetails {
            party_id: party_id.clone(),
            display_name: None,
            is_local: true,
        })
    }

    /// List all parties
    pub async fn list_parties(&mut self) -> LedgerResult<Vec<crate::client::PartyDetails>> {
        Ok(Vec::new())
    }

    /// Submit command
    pub async fn submit(&mut self, request: SubmitRequest) -> LedgerResult<()> {
        debug!("Submitting command");

        // In a real implementation, this would submit via gRPC
        Ok(())
    }

    /// Submit and wait for completion
    pub async fn submit_and_wait(&mut self, request: SubmitAndWaitRequest) -> LedgerResult<Completion> {
        debug!("Submitting command and waiting for completion");

        // In a real implementation, this would submit via gRPC and wait
        Ok(Completion {
            status: CompletionStatus::Ok,
            transaction_id: String::new(),
            update_id: String::new(),
        })
    }

    /// Submit and wait for transaction
    pub async fn submit_and_wait_for_transaction(
        &mut self,
        request: SubmitAndWaitForTransactionRequest,
    ) -> LedgerResult<Transaction> {
        debug!("Submitting command and waiting for transaction");

        // In a real implementation, this would submit via gRPC and wait
        Ok(Transaction {
            transaction_id: String::new(),
            command_id: String::new(),
            workflow_id: String::new(),
            effective_at: None,
            ledger_offset: LedgerOffset::Begin,
            events: Vec::new(),
        })
    }

    /// Get active contracts
    pub async fn get_active_contracts(
        &mut self,
        request: GetActiveContractsRequest,
    ) -> LedgerResult<ActiveContracts> {
        debug!("Getting active contracts");

        // In a real implementation, this would query via gRPC
        Ok(ActiveContracts {
            active_contracts: Vec::new(),
            offset: LedgerOffset::Begin,
        })
    }

    /// Get transactions
    pub async fn get_transactions(
        &mut self,
        request: GetTransactionsRequest,
    ) -> LedgerResult<Vec<Transaction>> {
        debug!("Getting transactions");

        // In a real implementation, this would query via gRPC
        Ok(Vec::new())
    }

    /// Get transactions as stream
    pub async fn get_transactions_stream(
        &mut self,
        request: GetTransactionsRequest,
    ) -> LedgerResult<impl Stream<Item = LedgerResult<Transaction>> + Send> {
        use futures::stream;

        Ok(stream::empty())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_pool_config_default() {
        let config = PoolConfig::default();
        assert_eq!(config.max_connections, 10);
        assert_eq!(config.min_connections, 1);
    }

    #[tokio::test]
    async fn test_pool_stats() {
        let stats = PoolStats {
            total: 5,
            active: 2,
            idle: 3,
            max_connections: 10,
        };

        assert_eq!(stats.total, 5);
        assert_eq!(stats.active, 2);
        assert_eq!(stats.idle, 3);
        assert_eq!(stats.max_connections, 10);
    }
}
