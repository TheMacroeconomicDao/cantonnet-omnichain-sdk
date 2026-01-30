//! Canton Ledger API — gRPC client for Daml Ledger API (stub).
//! Enable proto compilation by placing Ledger API v2 .proto files in proto/ (see proto/README.md).

#[cfg(feature = "generated")]
pub mod generated;

// Stub client type until proto is compiled
#[derive(Debug)]
pub struct LedgerClient {
    pub ledger_id: String,
}

impl LedgerClient {
    pub fn ledger_id(&self) -> &str {
        &self.ledger_id
    }
}
