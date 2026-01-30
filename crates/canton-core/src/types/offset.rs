//! Ledger offset for streaming.
//! See research/08, 04.

#[derive(Debug, Clone)]
pub struct LedgerOffset {
    pub value: OffsetValue,
}

#[derive(Debug, Clone)]
pub enum OffsetValue {
    Absolute(String),
    Begin,
    End,
}

impl LedgerOffset {
    pub fn absolute(s: impl Into<String>) -> Self {
        Self { value: OffsetValue::Absolute(s.into()) }
    }
    pub fn begin() -> Self {
        Self { value: OffsetValue::Begin }
    }
    pub fn end() -> Self {
        Self { value: OffsetValue::End }
    }
}
