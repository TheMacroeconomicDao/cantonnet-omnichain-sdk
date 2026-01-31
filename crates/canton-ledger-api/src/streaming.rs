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

//! Transaction streaming

use crate::client::LedgerClient;
use crate::error::{LedgerError, LedgerResult};
use crate::types::{LedgerOffset, Transaction, TransactionFilter};
use futures::stream::{Stream, StreamExt};
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::time::sleep;
use tracing::{debug, warn};

/// Transaction stream configuration
#[derive(Debug, Clone)]
pub struct StreamConfig {
    /// Buffer size for the stream
    pub buffer_size: usize,

    /// Polling interval for reconnecting
    pub poll_interval: Duration,

    /// Maximum reconnect attempts
    pub max_reconnect_attempts: u32,

    /// Whether to auto-reconnect on failure
    pub auto_reconnect: bool,
}

impl Default for StreamConfig {
    fn default() -> Self {
        Self {
            buffer_size: 100,
            poll_interval: Duration::from_secs(1),
            max_reconnect_attempts: 10,
            auto_reconnect: true,
        }
    }
}

impl StreamConfig {
    /// Create a new stream configuration
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the buffer size
    pub fn with_buffer_size(mut self, size: usize) -> Self {
        self.buffer_size = size;
        self
    }

    /// Set the polling interval
    pub fn with_poll_interval(mut self, interval: Duration) -> Self {
        self.poll_interval = interval;
        self
    }

    /// Set the maximum reconnect attempts
    pub fn with_max_reconnect_attempts(mut self, attempts: u32) -> Self {
        self.max_reconnect_attempts = attempts;
        self
    }

    /// Set whether to auto-reconnect
    pub fn with_auto_reconnect(mut self, auto: bool) -> Self {
        self.auto_reconnect = auto;
        self
    }
}

/// Transaction stream
pub struct TransactionStream {
    client: Arc<LedgerClient>,
    filter: TransactionFilter,
    config: StreamConfig,
    current_offset: LedgerOffset,
}

impl TransactionStream {
    /// Create a new transaction stream
    pub fn new(
        client: Arc<LedgerClient>,
        filter: TransactionFilter,
    ) -> Self {
        Self {
            client,
            filter,
            config: StreamConfig::default(),
            current_offset: LedgerOffset::Begin,
        }
    }

    /// Set the starting offset
    pub fn with_offset(mut self, offset: LedgerOffset) -> Self {
        self.current_offset = offset;
        self
    }

    /// Set the stream configuration
    pub fn with_config(mut self, config: StreamConfig) -> Self {
        self.config = config;
        self
    }

    /// Subscribe to transactions
    pub fn subscribe(
        self,
    ) -> Pin<Box<dyn Stream<Item = LedgerResult<Transaction>> + Send>> {
        Box::pin(async_stream::try_stream! {
            let mut reconnect_attempts = 0;
            let mut current_offset = self.current_offset;

            loop {
                // Check if we should reconnect
                if reconnect_attempts > 0 {
                    if reconnect_attempts >= self.config.max_reconnect_attempts {
                        return Err(LedgerError::StreamError(
                            "Max reconnect attempts reached".to_string(),
                        ));
                    }

                    warn!(
                        "Reconnect attempt {} for transaction stream",
                        reconnect_attempts
                    );
                    sleep(self.config.poll_interval).await;
                }

                // Check if client is connected
                if !self.client.is_connected().await {
                    if self.config.auto_reconnect {
                        debug!("Client not connected, attempting to reconnect");
                        if let Err(e) = self.client.reconnect().await {
                            warn!("Failed to reconnect: {}", e);
                            reconnect_attempts += 1;
                            continue;
                        }
                    } else {
                        return Err(LedgerError::StreamError(
                            "Client not connected and auto-reconnect disabled".to_string(),
                        ));
                    }
                }

                // Get transactions
                match self
                    .client
                    .get_transactions(current_offset.clone(), None, self.filter.clone())
                    .await
                {
                    Ok(transactions) => {
                        reconnect_attempts = 0;

                        if transactions.is_empty() {
                            // No new transactions, wait before polling again
                            sleep(self.config.poll_interval).await;
                            continue;
                        }

                        // Yield transactions and update offset
                        for tx in transactions {
                            current_offset = LedgerOffset::Absolute(tx.offset.clone());
                            yield tx;
                        }
                    }
                    Err(e) => {
                        warn!("Error getting transactions: {}", e);
                        reconnect_attempts += 1;

                        if !self.config.auto_reconnect {
                            return Err(e);
                        }
                    }
                }
            }
        })
    }

    /// Subscribe to transactions with a callback
    pub async fn subscribe_with_callback<F, Fut>(
        self,
        callback: F,
    ) -> LedgerResult<()>
    where
        F: Fn(Transaction) -> Fut + Send + Sync,
        Fut: std::future::Future<Output = LedgerResult<()>> + Send,
    {
        let mut stream = self.subscribe();

        while let Some(result) = stream.next().await {
            match result {
                Ok(tx) => {
                    if let Err(e) = callback(tx).await {
                        warn!("Callback error: {}", e);
                    }
                }
                Err(e) => {
                    warn!("Stream error: {}", e);
                    if !self.config.auto_reconnect {
                        return Err(e);
                    }
                }
            }
        }

        Ok(())
    }
}

/// Active contract stream
pub struct ActiveContractStream {
    client: Arc<LedgerClient>,
    filter: TransactionFilter,
    config: StreamConfig,
}

impl ActiveContractStream {
    /// Create a new active contract stream
    pub fn new(
        client: Arc<LedgerClient>,
        filter: TransactionFilter,
    ) -> Self {
        Self {
            client,
            filter,
            config: StreamConfig::default(),
        }
    }

    /// Set the stream configuration
    pub fn with_config(mut self, config: StreamConfig) -> Self {
        self.config = config;
        self
    }

    /// Subscribe to active contract updates
    pub fn subscribe(
        self,
    ) -> Pin<Box<dyn Stream<Item = LedgerResult<crate::types::CreatedEvent>> + Send>> {
        Box::pin(async_stream::try_stream! {
            let mut reconnect_attempts = 0;
            let mut last_seen = std::collections::HashSet::new();

            loop {
                // Check if we should reconnect
                if reconnect_attempts > 0 {
                    if reconnect_attempts >= self.config.max_reconnect_attempts {
                        return Err(LedgerError::StreamError(
                            "Max reconnect attempts reached".to_string(),
                        ));
                    }

                    warn!(
                        "Reconnect attempt {} for active contract stream",
                        reconnect_attempts
                    );
                    sleep(self.config.poll_interval).await;
                }

                // Check if client is connected
                if !self.client.is_connected().await {
                    if self.config.auto_reconnect {
                        debug!("Client not connected, attempting to reconnect");
                        if let Err(e) = self.client.reconnect().await {
                            warn!("Failed to reconnect: {}", e);
                            reconnect_attempts += 1;
                            continue;
                        }
                    } else {
                        return Err(LedgerError::StreamError(
                            "Client not connected and auto-reconnect disabled".to_string(),
                        ));
                    }
                }

                // Get active contracts
                match self
                    .client
                    .get_active_contracts(self.filter.clone())
                    .await
                {
                    Ok(contracts) => {
                        reconnect_attempts = 0;

                        // Yield new contracts
                        for contract in contracts {
                            if last_seen.insert(contract.contract_id.clone()) {
                                yield contract;
                            }
                        }

                        // Wait before polling again
                        sleep(self.config.poll_interval).await;
                    }
                    Err(e) => {
                        warn!("Error getting active contracts: {}", e);
                        reconnect_attempts += 1;

                        if !self.config.auto_reconnect {
                            return Err(e);
                        }
                    }
                }
            }
        })
    }

    /// Subscribe to active contract updates with a callback
    pub async fn subscribe_with_callback<F, Fut>(
        self,
        callback: F,
    ) -> LedgerResult<()>
    where
        F: Fn(crate::types::CreatedEvent) -> Fut + Send + Sync,
        Fut: std::future::Future<Output = LedgerResult<()>> + Send,
    {
        let mut stream = self.subscribe();

        while let Some(result) = stream.next().await {
            match result {
                Ok(contract) => {
                    if let Err(e) = callback(contract).await {
                        warn!("Callback error: {}", e);
                    }
                }
                Err(e) => {
                    warn!("Stream error: {}", e);
                    if !self.config.auto_reconnect {
                        return Err(e);
                    }
                }
            }
        }

        Ok(())
    }
}

/// Event stream that combines transactions and active contracts
pub struct EventStream {
    client: Arc<LedgerClient>,
    filter: TransactionFilter,
    config: StreamConfig,
}

impl EventStream {
    /// Create a new event stream
    pub fn new(
        client: Arc<LedgerClient>,
        filter: TransactionFilter,
    ) -> Self {
        Self {
            client,
            filter,
            config: StreamConfig::default(),
        }
    }

    /// Set the stream configuration
    pub fn with_config(mut self, config: StreamConfig) -> Self {
        self.config = config;
        self
    }

    /// Subscribe to all events
    pub fn subscribe(
        self,
    ) -> Pin<Box<dyn Stream<Item = LedgerResult<Event>> + Send>> {
        Box::pin(async_stream::try_stream! {
            let mut tx_stream = TransactionStream::new(
                self.client.clone(),
                self.filter.clone(),
            )
            .with_config(self.config.clone())
            .subscribe();

            let mut ac_stream = ActiveContractStream::new(
                self.client.clone(),
                self.filter.clone(),
            )
            .with_config(self.config.clone())
            .subscribe();

            loop {
                tokio::select! {
                    result = tx_stream.next() => {
                        match result {
                            Some(Ok(tx)) => yield Event::Transaction(tx),
                            Some(Err(e)) => return Err(e),
                            None => return Ok(()),
                        }
                    }
                    result = ac_stream.next() => {
                        match result {
                            Some(Ok(contract)) => yield Event::ActiveContract(contract),
                            Some(Err(e)) => return Err(e),
                            None => return Ok(()),
                        }
                    }
                }
            }
        })
    }

    /// Subscribe to events with a callback
    pub async fn subscribe_with_callback<F, Fut>(
        self,
        callback: F,
    ) -> LedgerResult<()>
    where
        F: Fn(Event) -> Fut + Send + Sync,
        Fut: std::future::Future<Output = LedgerResult<()>> + Send,
    {
        let mut stream = self.subscribe();

        while let Some(result) = stream.next().await {
            match result {
                Ok(event) => {
                    if let Err(e) = callback(event).await {
                        warn!("Callback error: {}", e);
                    }
                }
                Err(e) => {
                    warn!("Stream error: {}", e);
                    return Err(e);
                }
            }
        }

        Ok(())
    }
}

/// Event type
#[derive(Debug, Clone)]
pub enum Event {
    /// Transaction event
    Transaction(Transaction),
    /// Active contract event
    ActiveContract(crate::types::CreatedEvent),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stream_config_default() {
        let config = StreamConfig::default();
        assert_eq!(config.buffer_size, 100);
        assert_eq!(config.max_reconnect_attempts, 10);
        assert!(config.auto_reconnect);
    }

    #[test]
    fn test_stream_config_builder() {
        let config = StreamConfig::new()
            .with_buffer_size(200)
            .with_max_reconnect_attempts(5)
            .with_auto_reconnect(false);

        assert_eq!(config.buffer_size, 200);
        assert_eq!(config.max_reconnect_attempts, 5);
        assert!(!config.auto_reconnect);
    }
}
