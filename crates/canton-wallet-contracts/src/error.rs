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

//! Contract error types

use thiserror::Error;

/// Contract error type
#[derive(Error, Debug)]
pub enum ContractError {
    #[error("Contract not found: {0}")]
    NotFound(String),

    #[error("Contract creation failed: {0}")]
    CreationFailed(String),

    #[error("Contract exercise failed: {0}")]
    ExerciseFailed(String),

    #[error("Contract archive failed: {0}")]
    ArchiveFailed(String),

    #[error("Invalid contract ID: {0}")]
    InvalidContractId(String),

    #[error("Invalid template ID: {0}")]
    InvalidTemplateId(String),

    #[error("Invalid choice: {0}")]
    InvalidChoice(String),

    #[error("Invalid choice argument: {0}")]
    InvalidChoiceArgument(String),

    #[error("Invalid contract key: {0}")]
    InvalidContractKey(String),

    #[error("Invalid create arguments: {0}")]
    InvalidCreateArguments(String),

    #[error("Contract already exists: {0}")]
    AlreadyExists(String),

    #[error("Contract is archived: {0}")]
    Archived(String),

    #[error("Contract is not active: {0}")]
    NotActive(String),

    #[error("Contract query failed: {0}")]
    QueryFailed(String),

    #[error("Contract cache error: {0}")]
    CacheError(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Deserialization error: {0}")]
    DeserializationError(String),

    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("Timeout")]
    Timeout,

    #[error("Internal error: {0}")]
    InternalError(String),

    #[error("Unknown error: {0}")]
    Unknown(String),
}

/// Contract result type
pub type ContractResult<T> = Result<T, ContractError>;
