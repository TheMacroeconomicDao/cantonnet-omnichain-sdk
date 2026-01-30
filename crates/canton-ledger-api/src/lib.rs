//! Canton Ledger API — gRPC client for Daml Ledger API v2.
//! Enable proto compilation by placing Ledger API v2 .proto files in proto/ (see proto/README.md).

#[cfg(proto_compiled)]
pub mod generated {
    #![allow(clippy::all)]
    include!(concat!(env!("OUT_DIR"), "/generated.rs"));
}

#[cfg(proto_compiled)]
pub mod client;

#[cfg(proto_compiled)]
pub use client::LedgerClient;

/// Stub client when proto is not compiled (no proto files in proto/).
#[cfg(not(proto_compiled))]
#[derive(Debug)]
pub struct LedgerClient {
    pub ledger_id: String,
}

#[cfg(not(proto_compiled))]
impl LedgerClient {
    pub fn ledger_id(&self) -> &str {
        &self.ledger_id
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(not(proto_compiled))]
    #[test]
    fn stub_ledger_client_ledger_id() {
        let c = LedgerClient {
            ledger_id: "test".into(),
        };
        assert_eq!(c.ledger_id(), "test");
    }
    #[cfg(proto_compiled)]
    #[tokio::test]
    async fn ledger_client_connect_invalid_endpoint_fails() {
        let res = LedgerClient::connect("http://127.0.0.1:1", "ledger-id").await;
        assert!(res.is_err(), "expected connection error, got ok");
        if let Err(e) = res {
            assert!(matches!(e, canton_core::SdkError::Connection { .. }));
        }
    }
}
