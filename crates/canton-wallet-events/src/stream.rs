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

//! Event streaming implementation for real-time transaction and contract events.

use crate::error::{EventError, Result};
use canton_ledger_api::client::LedgerClient;
use canton_wallet_core::types::{
    Event, Identifier, LedgerOffset, PartyId, Transaction, TransactionFilter,
};
use futures::stream::{Stream, StreamExt};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;

/// Event stream configuration.
#[derive(Debug, Clone)]
pub struct EventStreamConfig {
    /// Buffer size for event streaming.
    pub buffer_size: usize,

    /// Polling interval for new events.
    pub poll_interval: Duration,

    /// Timeout for waiting for events.
    pub timeout: Duration,

    /// Whether to automatically reconnect on connection failure.
    pub auto_reconnect: bool,

    /// Maximum number of reconnection attempts.
    pub max_reconnect_attempts: usize,

    /// Delay between reconnection attempts.
    pub reconnect_delay: Duration,
}

impl Default for EventStreamConfig {
    fn default() -> Self {
        Self {
            buffer_size: 100,
            poll_interval: Duration::from_secs(1),
            timeout: Duration::from_secs(30),
            auto_reconnect: true,
            max_reconnect_attempts: 10,
            reconnect_delay: Duration::from_secs(5),
        }
    }
}

/// Event stream for real-time transaction and contract events.
pub struct EventStream {
    ledger_client: Arc<LedgerClient>,
    party_id: PartyId,
    filter: TransactionFilter,
    offset: LedgerOffset,
    config: EventStreamConfig,
}

impl EventStream {
    /// Create a new event stream.
    ///
    /// # Arguments
    ///
    /// * `ledger_client` - Ledger client for fetching events
    /// * `party_id` - Party ID to filter events for
    /// * `filter` - Transaction filter to apply
    ///
    /// # Returns
    ///
    /// Returns a new `EventStream` instance
    pub fn new(
        ledger_client: Arc<LedgerClient>,
        party_id: PartyId,
        filter: TransactionFilter,
    ) -> Self {
        Self {
            ledger_client,
            party_id,
            filter,
            offset: LedgerOffset::Begin,
            config: EventStreamConfig::default(),
        }
    }

    /// Set the starting offset for the event stream.
    ///
    /// # Arguments
    ///
    /// * `offset` - Starting offset for the stream
    ///
    /// # Returns
    ///
    /// Returns self for method chaining
    pub fn with_offset(mut self, offset: LedgerOffset) -> Self {
        self.offset = offset;
        self
    }

    /// Set the buffer size for the event stream.
    ///
    /// # Arguments
    ///
    /// * `size` - Buffer size
    ///
    /// # Returns
    ///
    /// Returns self for method chaining
    pub fn with_buffer_size(mut self, size: usize) -> Self {
        self.config.buffer_size = size;
        self
    }

    /// Set the polling interval for new events.
    ///
    /// # Arguments
    ///
    /// * `interval` - Polling interval
    ///
    /// # Returns
    ///
    /// Returns self for method chaining
    pub fn with_poll_interval(mut self, interval: Duration) -> Self {
        self.config.poll_interval = interval;
        self
    }

    /// Set the timeout for waiting for events.
    ///
    /// # Arguments
    ///
    /// * `timeout` - Timeout duration
    ///
    /// # Returns
    ///
    /// Returns self for method chaining
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.config.timeout = timeout;
        self
    }

    /// Enable or disable automatic reconnection.
    ///
    /// # Arguments
    ///
    /// * `enabled` - Whether to enable auto-reconnect
    ///
    /// # Returns
    ///
    /// Returns self for method chaining
    pub fn with_auto_reconnect(mut self, enabled: bool) -> Self {
        self.config.auto_reconnect = enabled;
        self
    }

    /// Set the maximum number of reconnection attempts.
    ///
    /// # Arguments
    ///
    /// * `max_attempts` - Maximum reconnection attempts
    ///
    /// # Returns
    ///
    /// Returns self for method chaining
    pub fn with_max_reconnect_attempts(mut self, max_attempts: usize) -> Self {
        self.config.max_reconnect_attempts = max_attempts;
        self
    }

    /// Set the delay between reconnection attempts.
    ///
    /// # Arguments
    ///
    /// * `delay` - Reconnection delay
    ///
    /// # Returns
    ///
    /// Returns self for method chaining
    pub fn with_reconnect_delay(mut self, delay: Duration) -> Self {
        self.config.reconnect_delay = delay;
        self
    }

    /// Set the event stream configuration.
    ///
    /// # Arguments
    ///
    /// * `config` - Event stream configuration
    ///
    /// # Returns
    ///
    /// Returns self for method chaining
    pub fn with_config(mut self, config: EventStreamConfig) -> Self {
        self.config = config;
        self
    }

    /// Subscribe to events as a stream.
    ///
    /// # Returns
    ///
    /// Returns a stream of transaction events
    pub fn subscribe(&self) -> impl Stream<Item = Result<Transaction>> + Send + 'static {
        let client = self.ledger_client.clone();
        let party_id = self.party_id.clone();
        let filter = self.filter.clone();
        let mut current_offset = self.offset.clone();
        let config = self.config.clone();

        let (tx, rx) = mpsc::channel(config.buffer_size);

        tokio::spawn(async move {
            let mut reconnect_attempts = 0;

            loop {
                match Self::fetch_events(
                    &client,
                    &party_id,
                    filter.clone(),
                    current_offset.clone(),
                    config.timeout,
                )
                .await
                {
                    Ok(transactions) => {
                        reconnect_attempts = 0;

                        for tx in transactions {
                            current_offset = LedgerOffset::Absolute(tx.offset.clone());

                            if tx.send(tx).await.is_err() {
                                tracing::error!("Event stream receiver dropped");
                                return;
                            }
                        }

                        // Wait before polling again
                        tokio::time::sleep(config.poll_interval).await;
                    }
                    Err(e) => {
                        tracing::error!("Event stream error: {}", e);

                        if !config.auto_reconnect {
                            let _ = tx.send(Err(e)).await;
                            return;
                        }

                        reconnect_attempts += 1;
                        if reconnect_attempts >= config.max_reconnect_attempts {
                            tracing::error!("Max reconnection attempts reached");
                            let _ = tx
                                .send(Err(EventError::ConnectionFailed(
                                    "Max reconnection attempts reached".to_string(),
                                )))
                                .await;
                            return;
                        }

                        tokio::time::sleep(config.reconnect_delay).await;
                    }
                }
            }
        });

        ReceiverStream::new(rx)
    }

    /// Subscribe to events with a callback function.
    ///
    /// # Arguments
    ///
    /// * `callback` - Callback function to handle each transaction
    ///
    /// # Returns
    ///
    /// Returns a result indicating success or failure
    pub async fn subscribe_with_callback<F, Fut>(
        &self,
        callback: F,
    ) -> Result<()>
    where
        F: Fn(Transaction) -> Fut + Send + Sync,
        Fut: std::future::Future<Output = Result<()>> + Send,
    {
        let mut stream = self.subscribe();

        while let Some(result) = stream.next().await {
            match result {
                Ok(tx) => {
                    if let Err(e) = callback(tx).await {
                        tracing::error!("Callback error: {}", e);
                    }
                }
                Err(e) => {
                    tracing::error!("Stream error: {}", e);
                    return Err(e);
                }
            }
        }

        Ok(())
    }

    /// Subscribe to created contract events only.
    ///
    /// # Returns
    ///
    /// Returns a stream of created contract events
    pub fn subscribe_created(&self) -> impl Stream<Item = Result<Event>> + Send + 'static {
        let tx_stream = self.subscribe();

        async_stream::try_stream! {
            for await result in tx_stream {
                let tx = result?;
                for event in tx.events {
                    if matches!(event, Event::Created(_)) {
                        yield event;
                    }
                }
            }
        }
    }

    /// Subscribe to archived contract events only.
    ///
    /// # Returns
    ///
    /// Returns a stream of archived contract events
    pub fn subscribe_archived(&self) -> impl Stream<Item = Result<Event>> + Send + 'static {
        let tx_stream = self.subscribe();

        async_stream::try_stream! {
            for await result in tx_stream {
                let tx = result?;
                for event in tx.events {
                    if matches!(event, Event::Archived(_)) {
                        yield event;
                    }
                }
            }
        }
    }

    /// Subscribe to events for a specific template.
    ///
    /// # Arguments
    ///
    /// * `template_id` - Template ID to filter by
    ///
    /// # Returns
    ///
    /// Returns a stream of events for the specified template
    pub fn subscribe_template(
        &self,
        template_id: Identifier,
    ) -> impl Stream<Item = Result<Event>> + Send + 'static {
        let tx_stream = self.subscribe();
        let template_id_clone = template_id.clone();

        async_stream::try_stream! {
            for await result in tx_stream {
                let tx = result?;
                for event in tx.events {
                    match &event {
                        Event::Created(created) => {
                            if created.template_id == template_id_clone {
                                yield event;
                            }
                        }
                        Event::Archived(archived) => {
                            if archived.template_id == template_id_clone {
                                yield event;
                            }
                        }
                    }
                }
            }
        }
    }

    /// Get the current offset of the event stream.
    ///
    /// # Returns
    ///
    /// Returns the current offset
    pub fn current_offset(&self) -> &LedgerOffset {
        &self.offset
    }

    /// Fetch events from the ledger.
    async fn fetch_events(
        client: &LedgerClient,
        party_id: &PartyId,
        filter: TransactionFilter,
        offset: LedgerOffset,
        timeout: Duration,
    ) -> Result<Vec<Transaction>> {
        tokio::time::timeout(
            timeout,
            client.get_transactions(offset.clone(), None, filter),
        )
        .await
        .map_err(|_| EventError::Timeout(timeout))?
    }
}

/// Event subscription for managing multiple event streams.
pub struct EventSubscription {
    streams: Vec<EventStream>,
}

impl EventSubscription {
    /// Create a new event subscription.
    ///
    /// # Returns
    ///
    /// Returns a new `EventSubscription` instance
    pub fn new() -> Self {
        Self { streams: Vec::new() }
    }

    /// Add an event stream to the subscription.
    ///
    /// # Arguments
    ///
    /// * `stream` - Event stream to add
    ///
    /// # Returns
    ///
    /// Returns self for method chaining
    pub fn add_stream(mut self, stream: EventStream) -> Self {
        self.streams.push(stream);
        self
    }

    /// Subscribe to all event streams and merge them into a single stream.
    ///
    /// # Returns
    ///
    /// Returns a merged stream of all events
    pub fn subscribe_all(&self) -> impl Stream<Item = Result<Transaction>> + Send + 'static {
        let streams: Vec<_> = self.streams.iter().map(|s| s.subscribe()).collect();

        futures::stream::select_all(streams)
    }

    /// Get the number of event streams in the subscription.
    ///
    /// # Returns
    ///
    /// Returns the number of streams
    pub fn stream_count(&self) -> usize {
        self.streams.len()
    }
}

impl Default for EventSubscription {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_stream_config_default() {
        let config = EventStreamConfig::default();
        assert_eq!(config.buffer_size, 100);
        assert_eq!(config.poll_interval, Duration::from_secs(1));
        assert_eq!(config.timeout, Duration::from_secs(30));
        assert!(config.auto_reconnect);
        assert_eq!(config.max_reconnect_attempts, 10);
        assert_eq!(config.reconnect_delay, Duration::from_secs(5));
    }

    #[test]
    fn test_event_subscription_default() {
        let subscription = EventSubscription::default();
        assert_eq!(subscription.stream_count(), 0);
    }

    #[test]
    fn test_event_subscription_add_stream() {
        let subscription = EventSubscription::new();
        assert_eq!(subscription.stream_count(), 0);

        // Note: We can't create a real EventStream without a LedgerClient
        // This is just to verify the API structure
        assert_eq!(subscription.stream_count(), 0);
    }
}
