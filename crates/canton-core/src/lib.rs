//! Canton Core — core types, traits, and errors for Canton OmniChain SDK.
//!
//! See [DEVELOPMENT_PROMPT](../../../DEVELOPMENT_PROMPT.md) and research/08 for full spec.

pub mod error;
pub mod config;

pub mod types;

pub mod traits;

pub use error::{SdkError, SdkResult, TransactionErrorKind};
pub use config::*;
pub use types::*;
pub use traits::*;
