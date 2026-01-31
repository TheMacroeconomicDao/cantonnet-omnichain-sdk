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

//! Canton Ledger API Integration
//!
//! This crate provides integration with the Canton Ledger API through gRPC,
//! enabling wallet operations such as command submission, transaction streaming,
//! and contract queries.

pub mod client;
pub mod connection;
pub mod error;
pub mod pool;
pub mod proto;

pub use client::LedgerClient;
pub use connection::{ConnectionConfig, ConnectionManager};
pub use error::{LedgerError, LedgerResult};
pub use pool::{ConnectionPool, PoolConfig};
