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

//! Canton Wallet SDK - Security
//!
//! This crate provides security features for the Canton Wallet SDK, including:
//!
//! - Transaction approval management
//! - Audit logging
//! - Rate limiting
//! - Input validation
//!
//! # Features
//!
//! - User approval for sensitive operations
//! - Comprehensive audit logging
//! - Rate limiting to prevent abuse
//! - Input validation and sanitization
//!
//! # Example
//!
//! ```no_run
//! use canton_wallet_security::{
//!     ApprovalManager, AuditLogger, InMemoryAuditLogger,
//!     AutoApproval, InputValidator,
//! };
//! use canton_wallet_transactions::validator::TransactionValidator;
//! use std::sync::Arc;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let audit_logger = Arc::new(InMemoryAuditLogger::new());
//! let user_approval = Arc::new(AutoApproval);
//! let validator = TransactionValidator::default();
//!
//! let approval_manager = ApprovalManager::new(
//!     user_approval,
//!     validator,
//!     audit_logger,
//! );
//!
//! let input_validator = InputValidator::new();
//! input_validator.validate_party_id(&party_id)?;
//! # Ok(())
//! # }
//! ```

pub mod approval;
pub mod audit;
pub mod error;
pub mod rate_limit;
pub mod validation;

pub use approval::{
    ApprovalManager, ApprovalPolicy, ApprovalResponse, AutoApproval, RejectAllApproval, UserApproval,
};
pub use audit::{
    AuditFilter, AuditLogger, AuditLogEntry, AuditStatistics, InMemoryAuditLogger,
};
pub use error::{Result, SecurityError};
pub use rate_limit::{KeyedRateLimiter, RateLimiter, RateLimiterConfig};
pub use validation::InputValidator;
