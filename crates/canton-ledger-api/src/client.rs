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

//! Ledger client for Canton Network

use crate::error::{LedgerError, LedgerResult};
use crate::pool::ConnectionPool;
use crate::proto::{
    ActiveContracts, ArchivedEvent, Command, Completion, CompletionStatus, CreatedEvent,
    Event, GetActiveContractsRequest, GetTransactionsRequest, Identifier, LedgerOffset,
    PartyId, SubmitAndWaitForTransactionRequest, SubmitAndWaitRequest, SubmitRequest,
    Transaction, TransactionFilter,
};
use async_trait::async_trait;
use futures::Stream;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, instrument, warn};

/// Ledger client configuration
#[derive(Debug, Clone)]
pub struct LedgerClientConfig {
    /// Ledger endpoint URL
    pub endpoint: String,
    /// Ledger ID
    pub ledger_id: String,
    /// Application ID
    pub application_id: String,
    /// Maximum number of connections in the pool
    pub max_connections: usize,
    /// Connection timeout in seconds
    pub connection_timeout_secs: u64,
    /// Request timeout in seconds
    pub request_timeout_secs: u64,
    /// Whether to use TLS
    pub use_tls: bool,
    /// TLS certificate path (optional)
    pub tls_cert_path: Option<String>,
}

impl Default for LedgerClientConfig {
    fn default() -> Self {
        Self {
            endpoint: "http://localhost:50051".to_string(),
            ledger_id: String::new(),
            application_id: "canton-wallet-sdk".to_string(),
            max_connections: 10,
            connection_timeout_secs: 30,
            request_timeout_secs: 60,
            use_tls: false,
            tls_cert_path: None,
        }
    }
}

/// Ledger client trait
#[async_trait]
pub trait LedgerClient: Send + Sync {
    /// Get ledger ID
    fn ledger_id(&self) -> &str;

    /// Get application ID
    fn application_id(&self) -> &str;

    /// Submit command
    async fn submit(&self, request: SubmitRequest) -> LedgerResult<()>;

    /// Submit and wait for completion
    async fn submit_and_wait(&self, request: SubmitAndWaitRequest) -> LedgerResult<Completion>;

    /// Submit and wait for transaction
    async fn submit_and_wait_for_transaction(
        &self,
        request: SubmitAndWaitForTransactionRequest,
    ) -> LedgerResult<Transaction>;

    /// Get active contracts
    async fn get_active_contracts(
        &self,
        request: GetActiveContractsRequest,
    ) -> LedgerResult<ActiveContracts>;

    /// Get transactions
    async fn get_transactions(
        &self,
        request: GetTransactionsRequest,
    ) -> LedgerResult<Vec<Transaction>>;

    /// Get transactions as stream
    fn get_transactions_stream(
        &self,
        request: GetTransactionsRequest,
    ) -> impl Stream<Item = LedgerResult<Transaction>> + Send;

    /// Submit command for party
    async fn submit_for_party(
        &self,
        party_id: &PartyId,
        commands: Vec<Command>,
    ) -> LedgerResult<()>;

    /// Submit and wait for party
    async fn submit_and_wait_for_party(
        &self,
        party_id: &PartyId,
        commands: Vec<Command>,
    ) -> LedgerResult<Completion>;

    /// Submit and wait for transaction for party
    async fn submit_and_wait_for_transaction_for_party(
        &self,
        party_id: &PartyId,
        commands: Vec<Command>,
    ) -> LedgerResult<Transaction>;

    /// Get active contracts for party
    async fn get_active_contracts_for_party(
        &self,
        party_id: &PartyId,
        filter: Option<TransactionFilter>,
    ) -> LedgerResult<Vec<CreatedEvent>>;

    /// Get transactions for party
    async fn get_transactions_for_party(
        &self,
        party_id: &PartyId,
        begin: LedgerOffset,
        end: Option<LedgerOffset>,
        filter: Option<TransactionFilter>,
    ) -> LedgerResult<Vec<Transaction>>;

    /// Get transactions stream for party
    fn get_transactions_stream_for_party(
        &self,
        party_id: &PartyId,
        begin: LedgerOffset,
        end: Option<LedgerOffset>,
        filter: Option<TransactionFilter>,
    ) -> impl Stream<Item = LedgerResult<Transaction>> + Send;
}

/// Ledger client implementation
pub struct CantonLedgerClient {
    config: LedgerClientConfig,
    pool: Arc<ConnectionPool>,
    ledger_id: String,
    application_id: String,
}

impl CantonLedgerClient {
    /// Create a new ledger client
    pub async fn new(config: LedgerClientConfig) -> LedgerResult<Self> {
        let ledger_id = config.ledger_id.clone();
        let application_id = config.application_id.clone();

        let pool = ConnectionPool::new(config.clone()).await?;

        Ok(Self {
            config,
            pool,
            ledger_id,
            application_id,
        })
    }

    /// Connect to a ledger endpoint
    pub async fn connect(endpoint: impl Into<String>) -> LedgerResult<Self> {
        let config = LedgerClientConfig {
            endpoint: endpoint.into(),
            ..Default::default()
        };
        Self::new(config).await
    }

    /// Get the client configuration
    pub fn config(&self) -> &LedgerClientConfig {
        &self.config
    }

    /// Get the connection pool
    pub fn pool(&self) -> &Arc<ConnectionPool> {
        &self.pool
    }

    /// Check if the client is connected
    pub async fn is_connected(&self) -> bool {
        self.pool.is_connected().await
    }

    /// Disconnect from the ledger
    pub async fn disconnect(&self) -> LedgerResult<()> {
        self.pool.close().await
    }

    /// Get ledger ID from the server
    async fn fetch_ledger_id(&self) -> LedgerResult<String> {
        let mut conn = self.pool.acquire().await?;

        let ledger_id = conn.get_ledger_id().await?;

        Ok(ledger_id)
    }

    /// Validate party exists
    async fn validate_party(&self, party_id: &PartyId) -> LedgerResult<bool> {
        let mut conn = self.pool.acquire().await?;

        conn.party_exists(party_id).await
    }

    /// Get party details
    async fn get_party_details(&self, party_id: &PartyId) -> LedgerResult<PartyDetails> {
        let mut conn = self.pool.acquire().await?;

        conn.get_party_details(party_id).await
    }

    /// List all parties
    async fn list_parties(&self) -> LedgerResult<Vec<PartyDetails>> {
        let mut conn = self.pool.acquire().await?;

        conn.list_parties().await
    }
}

#[async_trait]
impl LedgerClient for CantonLedgerClient {
    fn ledger_id(&self) -> &str {
        &self.ledger_id
    }

    fn application_id(&self) -> &str {
        &self.application_id
    }

    #[instrument(skip(self, request))]
    async fn submit(&self, request: SubmitRequest) -> LedgerResult<()> {
        debug!("Submitting command");

        let mut conn = self.pool.acquire().await?;

        conn.submit(request).await?;

        debug!("Command submitted successfully");

        Ok(())
    }

    #[instrument(skip(self, request))]
    async fn submit_and_wait(&self, request: SubmitAndWaitRequest) -> LedgerResult<Completion> {
        debug!("Submitting command and waiting for completion");

        let mut conn = self.pool.acquire().await?;

        let completion = conn.submit_and_wait(request).await?;

        debug!("Command completed with status: {:?}", completion.status);

        Ok(completion)
    }

    #[instrument(skip(self, request))]
    async fn submit_and_wait_for_transaction(
        &self,
        request: SubmitAndWaitForTransactionRequest,
    ) -> LedgerResult<Transaction> {
        debug!("Submitting command and waiting for transaction");

        let mut conn = self.pool.acquire().await?;

        let transaction = conn.submit_and_wait_for_transaction(request).await?;

        debug!("Transaction completed: {}", transaction.transaction_id);

        Ok(transaction)
    }

    #[instrument(skip(self, request))]
    async fn get_active_contracts(
        &self,
        request: GetActiveContractsRequest,
    ) -> LedgerResult<ActiveContracts> {
        debug!("Getting active contracts");

        let mut conn = self.pool.acquire().await?;

        let contracts = conn.get_active_contracts(request).await?;

        debug!("Retrieved {} active contracts", contracts.active_contracts.len());

        Ok(contracts)
    }

    #[instrument(skip(self, request))]
    async fn get_transactions(
        &self,
        request: GetTransactionsRequest,
    ) -> LedgerResult<Vec<Transaction>> {
        debug!("Getting transactions");

        let mut conn = self.pool.acquire().await?;

        let transactions = conn.get_transactions(request).await?;

        debug!("Retrieved {} transactions", transactions.len());

        Ok(transactions)
    }

    fn get_transactions_stream(
        &self,
        request: GetTransactionsRequest,
    ) -> impl Stream<Item = LedgerResult<Transaction>> + Send {
        use futures::stream;

        let pool = self.pool.clone();

        async_stream::try_stream! {
            let mut conn = pool.acquire().await?;
            let mut stream = conn.get_transactions_stream(request).await?;

            while let Some(result) = stream.next().await {
                let transaction = result?;
                yield transaction;
            }
        }
    }

    #[instrument(skip(self, commands))]
    async fn submit_for_party(
        &self,
        party_id: &PartyId,
        commands: Vec<Command>,
    ) -> LedgerResult<()> {
        debug!("Submitting commands for party: {}", party_id.as_str());

        let request = SubmitRequest {
            commands: crate::proto::Commands {
                ledger_id: self.ledger_id.clone(),
                workflow_id: uuid::Uuid::new_v4().to_string(),
                application_id: self.application_id.clone(),
                command_id: uuid::Uuid::new_v4().to_string(),
                party: party_id.as_str().to_string(),
                commands,
                act_as: vec![party_id.as_str().to_string()],
                read_as: vec![],
                min_ledger_time_abs: None,
                min_ledger_time_rel: None,
                deduplication_time: None,
            },
        };

        self.submit(request).await
    }

    #[instrument(skip(self, commands))]
    async fn submit_and_wait_for_party(
        &self,
        party_id: &PartyId,
        commands: Vec<Command>,
    ) -> LedgerResult<Completion> {
        debug!("Submitting commands for party and waiting: {}", party_id.as_str());

        let request = SubmitAndWaitRequest {
            commands: crate::proto::Commands {
                ledger_id: self.ledger_id.clone(),
                workflow_id: uuid::Uuid::new_v4().to_string(),
                application_id: self.application_id.clone(),
                command_id: uuid::Uuid::new_v4().to_string(),
                party: party_id.as_str().to_string(),
                commands,
                act_as: vec![party_id.as_str().to_string()],
                read_as: vec![],
                min_ledger_time_abs: None,
                min_ledger_time_rel: None,
                deduplication_time: None,
            },
            timeout: None,
        };

        self.submit_and_wait(request).await
    }

    #[instrument(skip(self, commands))]
    async fn submit_and_wait_for_transaction_for_party(
        &self,
        party_id: &PartyId,
        commands: Vec<Command>,
    ) -> LedgerResult<Transaction> {
        debug!("Submitting commands for party and waiting for transaction: {}", party_id.as_str());

        let request = SubmitAndWaitForTransactionRequest {
            commands: crate::proto::Commands {
                ledger_id: self.ledger_id.clone(),
                workflow_id: uuid::Uuid::new_v4().to_string(),
                application_id: self.application_id.clone(),
                command_id: uuid::Uuid::new_v4().to_string(),
                party: party_id.as_str().to_string(),
                commands,
                act_as: vec![party_id.as_str().to_string()],
                read_as: vec![],
                min_ledger_time_abs: None,
                min_ledger_time_rel: None,
                deduplication_time: None,
            },
            timeout: None,
        };

        self.submit_and_wait_for_transaction(request).await
    }

    #[instrument(skip(self))]
    async fn get_active_contracts_for_party(
        &self,
        party_id: &PartyId,
        filter: Option<TransactionFilter>,
    ) -> LedgerResult<Vec<CreatedEvent>> {
        debug!("Getting active contracts for party: {}", party_id.as_str());

        let filter = filter.unwrap_or_else(|| TransactionFilter::for_party(party_id));

        let request = GetActiveContractsRequest {
            ledger_id: self.ledger_id.clone(),
            filter,
            verbose: true,
            offset: LedgerOffset::Begin,
        };

        let result = self.get_active_contracts(request).await?;

        Ok(result.active_contracts)
    }

    #[instrument(skip(self))]
    async fn get_transactions_for_party(
        &self,
        party_id: &PartyId,
        begin: LedgerOffset,
        end: Option<LedgerOffset>,
        filter: Option<TransactionFilter>,
    ) -> LedgerResult<Vec<Transaction>> {
        debug!("Getting transactions for party: {}", party_id.as_str());

        let filter = filter.unwrap_or_else(|| TransactionFilter::for_party(party_id));

        let request = GetTransactionsRequest {
            ledger_id: self.ledger_id.clone(),
            begin,
            end,
            filter,
            verbose: true,
        };

        self.get_transactions(request).await
    }

    fn get_transactions_stream_for_party(
        &self,
        party_id: &PartyId,
        begin: LedgerOffset,
        end: Option<LedgerOffset>,
        filter: Option<TransactionFilter>,
    ) -> impl Stream<Item = LedgerResult<Transaction>> + Send {
        let filter = filter.unwrap_or_else(|| TransactionFilter::for_party(party_id));

        let request = GetTransactionsRequest {
            ledger_id: self.ledger_id.clone(),
            begin,
            end,
            filter,
            verbose: true,
        };

        self.get_transactions_stream(request)
    }
}

/// Party details
#[derive(Debug, Clone)]
pub struct PartyDetails {
    pub party_id: PartyId,
    pub display_name: Option<String>,
    pub is_local: bool,
}

/// Ledger client builder
pub struct LedgerClientBuilder {
    config: LedgerClientConfig,
}

impl LedgerClientBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self {
            config: LedgerClientConfig::default(),
        }
    }

    /// Set the endpoint
    pub fn endpoint(mut self, endpoint: impl Into<String>) -> Self {
        self.config.endpoint = endpoint.into();
        self
    }

    /// Set the ledger ID
    pub fn ledger_id(mut self, ledger_id: impl Into<String>) -> Self {
        self.config.ledger_id = ledger_id.into();
        self
    }

    /// Set the application ID
    pub fn application_id(mut self, application_id: impl Into<String>) -> Self {
        self.config.application_id = application_id.into();
        self
    }

    /// Set the maximum number of connections
    pub fn max_connections(mut self, max: usize) -> Self {
        self.config.max_connections = max;
        self
    }

    /// Set the connection timeout
    pub fn connection_timeout_secs(mut self, timeout: u64) -> Self {
        self.config.connection_timeout_secs = timeout;
        self
    }

    /// Set the request timeout
    pub fn request_timeout_secs(mut self, timeout: u64) -> Self {
        self.config.request_timeout_secs = timeout;
        self
    }

    /// Enable TLS
    pub fn use_tls(mut self, use_tls: bool) -> Self {
        self.config.use_tls = use_tls;
        self
    }

    /// Set the TLS certificate path
    pub fn tls_cert_path(mut self, path: impl Into<String>) -> Self {
        self.config.tls_cert_path = Some(path.into());
        self
    }

    /// Build the ledger client
    pub async fn build(self) -> LedgerResult<CantonLedgerClient> {
        CantonLedgerClient::new(self.config).await
    }
}

impl Default for LedgerClientBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ledger_client_config_default() {
        let config = LedgerClientConfig::default();
        assert_eq!(config.endpoint, "http://localhost:50051");
        assert_eq!(config.application_id, "canton-wallet-sdk");
        assert_eq!(config.max_connections, 10);
    }

    #[test]
    fn test_ledger_client_builder() {
        let builder = LedgerClientBuilder::new()
            .endpoint("http://localhost:50052")
            .ledger_id("test-ledger")
            .application_id("test-app")
            .max_connections(20)
            .connection_timeout_secs(60)
            .request_timeout_secs(120)
            .use_tls(true);

        assert_eq!(builder.config.endpoint, "http://localhost:50052");
        assert_eq!(builder.config.ledger_id, "test-ledger");
        assert_eq!(builder.config.application_id, "test-app");
        assert_eq!(builder.config.max_connections, 20);
        assert_eq!(builder.config.connection_timeout_secs, 60);
        assert_eq!(builder.config.request_timeout_secs, 120);
        assert!(builder.config.use_tls);
    }
}
