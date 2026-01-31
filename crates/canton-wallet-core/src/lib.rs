// Canton Wallet SDK - Core Types and Traits
//
// This crate provides the foundational types, traits, and error handling
// for the Canton Wallet SDK.

#![warn(missing_docs)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![allow(clippy::module_name_repetitions)]

pub mod error;
pub mod types;
pub mod traits;

#[cfg(test)]
mod types_test;

pub use error::{WalletError, WalletResult};
pub use types::*;
pub use traits::Wallet;

/// Re-export commonly used types for convenience
pub mod prelude {
    pub use crate::error::{WalletError, WalletResult};
    pub use crate::types::*;
    pub use crate::traits::Wallet;
}
