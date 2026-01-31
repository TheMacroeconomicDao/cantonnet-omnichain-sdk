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

//! Transaction error types

use thiserror::Error;

/// Transaction error type
#[derive(Error, Debug)]
pub enum TransactionError {
    #[error("Missing party ID")]
    MissingPartyId,
    
    #[error("Missing command")]
    MissingCommand,
    
    #[error("Invalid command: {0}")]
    InvalidCommand(String),
    
    #[error("Invalid transaction: {0}")]
    InvalidTransaction(String),
    
    #[error("Transaction validation failed: {0}")]
    ValidationFailed(String),
    
    #[error("Transaction estimation failed: {0}")]
    EstimationFailed(String),
    
    #[error("Transaction submission failed: {0}")]
    SubmissionFailed(String),
    
    #[error("Transaction timeout")]
    Timeout,
    
    #[error("Transaction rejected: {0}")]
    Rejected(String),
    
    #[error("Transaction already exists: {0}")]
    AlreadyExists(String),
    
    #[error("Transaction not found: {0}")]
    NotFound(String),
    
    #[error("Serialization error: {0}")]
    SerializationError(String),
    
    #[error("Deserialization error: {0}")]
    DeserializationError(String),
    
    #[error("Invalid argument: {0}")]
    InvalidArgument(String),
    
    #[error("Invalid ledger time: {0}")]
    InvalidLedgerTime(String),
    
    #[error("Invalid deduplication period: {0}")]
    InvalidDeduplicationPeriod(String),
    
    #[error("Invalid workflow ID: {0}")]
    InvalidWorkflowId(String),
    
    #[error("Invalid application ID: {0}")]
    InvalidApplicationId(String),
    
    #[error("Invalid command ID: {0}")]
    InvalidCommandId(String),
    
    #[error("Invalid party: {0}")]
    InvalidParty(String),
    
    #[error("Invalid template: {0}")]
    InvalidTemplate(String),
    
    #[error("Invalid contract ID: {0}")]
    InvalidContractId(String),
    
    #[error("Invalid choice: {0}")]
    InvalidChoice(String),
    
    #[error("Invalid choice argument: {0}")]
    InvalidChoiceArgument(String),
    
    #[error("Invalid contract key: {0}")]
    InvalidContractKey(String),
    
    #[error("Invalid create arguments: {0}")]
    InvalidCreateArguments(String),
    
    #[error("Too many commands: {0}")]
    TooManyCommands(usize),
    
    #[error("Too few commands: {0}")]
    TooFewCommands(usize),
    
    #[error("Command limit exceeded: {0}")]
    CommandLimitExceeded(usize),
    
    #[error("Size limit exceeded: {0}")]
    SizeLimitExceeded(usize),
    
    #[error("Rate limit exceeded")]
    RateLimitExceeded,
    
    #[error("Retry limit exceeded")]
    RetryLimitExceeded,
    
    #[error("Network error: {0}")]
    NetworkError(String),
    
    #[error("Internal error: {0}")]
    InternalError(String),
    
    #[error("Unknown error: {0}")]
    Unknown(String),
}

/// Transaction result type
pub type TransactionResult<T> = Result<T, TransactionError>;