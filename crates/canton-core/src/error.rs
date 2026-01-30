//! SDK error types.
//! See research/08-sdk-architecture-design.md §6.

use std::backtrace::Backtrace;
use std::collections::HashMap;

/// Main SDK error type.
#[derive(Debug)]
pub enum SdkError {
    Connection {
        message: String,
        cause: Option<Box<dyn std::error::Error + Send + Sync>>,
        backtrace: Backtrace,
    },

    Authentication {
        reason: String,
        cause: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    Transaction {
        kind: TransactionErrorKind,
        transaction_id: Option<String>,
        details: HashMap<String, String>,
        cause: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    Validation { field: String, message: String },

    Config(String),

    Serialization(String),

    Crypto(String),

    Timeout {
        duration: std::time::Duration,
        operation: String,
    },

    RateLimited {
        retry_after: Option<std::time::Duration>,
    },

    CircuitOpen,

    CrossChain {
        message: String,
        source_chain: Option<String>,
        target_chain: Option<String>,
        cause: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    Internal {
        message: String,
        backtrace: Backtrace,
    },
}

impl std::fmt::Display for SdkError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SdkError::Connection { message, .. } => write!(f, "Connection error: {}", message),
            SdkError::Authentication { reason, .. } => write!(f, "Authentication failed: {}", reason),
            SdkError::Transaction { kind, .. } => write!(f, "Transaction error: {}", kind),
            SdkError::Validation { field, message } => write!(f, "Validation error: {} - {}", field, message),
            SdkError::Config(s) => write!(f, "Configuration error: {}", s),
            SdkError::Serialization(s) => write!(f, "Serialization error: {}", s),
            SdkError::Crypto(s) => write!(f, "Cryptographic error: {}", s),
            SdkError::Timeout { duration, operation } => {
                write!(f, "Operation timed out after {:?}: {}", duration, operation)
            }
            SdkError::RateLimited { retry_after } => write!(f, "Rate limited, retry after {:?}", retry_after),
            SdkError::CircuitOpen => write!(f, "Circuit breaker open"),
            SdkError::CrossChain { message, .. } => write!(f, "Cross-chain error: {}", message),
            SdkError::Internal { message, .. } => write!(f, "Internal error: {}", message),
        }
    }
}

impl std::error::Error for SdkError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            SdkError::Connection { cause: Some(c), .. }
            | SdkError::Authentication { cause: Some(c), .. }
            | SdkError::Transaction { cause: Some(c), .. }
            | SdkError::CrossChain { cause: Some(c), .. } => Some(c.as_ref()),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransactionErrorKind {
    InvalidCommand,
    ContractNotFound,
    ChoiceNotFound,
    AuthorizationFailed,
    Conflict,
    Timeout,
    Rejected,
    Unknown,
}

impl std::fmt::Display for TransactionErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::InvalidCommand => "InvalidCommand",
            Self::ContractNotFound => "ContractNotFound",
            Self::ChoiceNotFound => "ChoiceNotFound",
            Self::AuthorizationFailed => "AuthorizationFailed",
            Self::Conflict => "Conflict",
            Self::Timeout => "Timeout",
            Self::Rejected => "Rejected",
            Self::Unknown => "Unknown",
        })
    }
}

impl SdkError {
    /// Whether the error is retryable.
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            SdkError::Connection { .. }
                | SdkError::Timeout { .. }
                | SdkError::RateLimited { .. }
                | SdkError::Transaction {
                    kind: TransactionErrorKind::Conflict,
                    ..
                }
        )
    }

    /// Error type for metrics.
    pub fn error_type(&self) -> &'static str {
        match self {
            SdkError::Connection { .. } => "connection",
            SdkError::Authentication { .. } => "authentication",
            SdkError::Transaction { .. } => "transaction",
            SdkError::Validation { .. } => "validation",
            SdkError::Config(_) => "config",
            SdkError::Serialization(_) => "serialization",
            SdkError::Crypto(_) => "crypto",
            SdkError::Timeout { .. } => "timeout",
            SdkError::RateLimited { .. } => "rate_limited",
            SdkError::CircuitOpen => "circuit_open",
            SdkError::CrossChain { .. } => "cross_chain",
            SdkError::Internal { .. } => "internal",
        }
    }
}

/// Result alias.
pub type SdkResult<T> = Result<T, SdkError>;
