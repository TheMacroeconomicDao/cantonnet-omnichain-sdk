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

//! Canton Wallet SDK - Event Streaming
//!
//! This crate provides event streaming functionality for real-time transaction
//! and contract events from the Canton Network ledger.
//!
//! # Features
//!
//! - Real-time event streaming
//! - Offset management for resume capability
//! - Event filtering by template and event type
//! - Automatic reconnection with configurable retry logic
//! - Efficient buffering
//! - Multiple stream subscription support
//!
//! # Example
//!
//! ```no_run
//! use canton_wallet_events::{EventStream, EventStreamConfig};
//! use canton_ledger_api::client::LedgerClient;
//! use canton_wallet_core::types::{PartyId, TransactionFilter};
//! use std::sync::Arc;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let client = Arc::new(LedgerClient::connect("http://localhost:50051").await?);
//! let party_id = PartyId::new_unchecked("my-party");
//! let filter = TransactionFilter::for_party(&party_id);
//!
//! let stream = EventStream::new(client, party_id, filter)
//!     .with_buffer_size(100)
//!     .with_poll_interval(std::time::Duration::from_secs(1));
//!
//! let mut event_stream = stream.subscribe();
//! while let Some(result) = event_stream.next().await {
//!     match result {
//!         Ok(tx) => println!("Received transaction: {}", tx.transaction_id),
//!         Err(e) => eprintln!("Error: {}", e),
//!     }
//! }
//! # Ok(())
//! # }
//! ```

pub mod error;
pub mod stream;

pub use error::{EventError, Result};
pub use stream::{EventStream, EventStreamConfig, EventSubscription};
