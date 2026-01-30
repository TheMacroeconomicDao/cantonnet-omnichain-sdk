//! Proto conversion traits for Ledger API types.
//! Implementations live in canton-ledger-api when proto is available.

use crate::error::SdkResult;

/// Convert from Ledger API proto type.
pub trait FromProto<T>: Sized {
    fn from_proto(proto: T) -> SdkResult<Self>;
}

/// Convert to Ledger API proto type.
pub trait ToProto<T>: Sized {
    fn to_proto(&self) -> SdkResult<T>;
}
