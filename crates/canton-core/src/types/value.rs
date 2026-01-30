//! Daml value types.
//! See research/08-sdk-architecture-design.md §3.2.

use crate::types::identifier::{ContractId, Identifier, PartyId};
use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use std::collections::HashMap;

/// Daml value representation.
#[derive(Debug, Clone, PartialEq)]
pub enum DamlValue {
    Unit,
    Bool(bool),
    Int64(i64),
    Numeric(Decimal),
    Text(String),
    Timestamp(DateTime<Utc>),
    Date(NaiveDate),
    Party(PartyId),
    ContractId(ContractId),
    List(Vec<DamlValue>),
    Optional(Option<Box<DamlValue>>),
    TextMap(HashMap<String, DamlValue>),
    GenMap(Vec<(DamlValue, DamlValue)>),
    Record(DamlRecord),
    Variant(DamlVariant),
    Enum(DamlEnum),
}

#[derive(Debug, Clone, PartialEq)]
pub struct DamlRecord {
    pub record_id: Option<Identifier>,
    pub fields: Vec<RecordField>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RecordField {
    pub label: String,
    pub value: DamlValue,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DamlVariant {
    pub variant_id: Option<Identifier>,
    pub constructor: String,
    pub value: Box<DamlValue>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DamlEnum {
    pub enum_id: Option<Identifier>,
    pub constructor: String,
}

impl DamlValue {
    pub fn unit() -> Self { Self::Unit }
    pub fn bool(v: bool) -> Self { Self::Bool(v) }
    pub fn int64(v: i64) -> Self { Self::Int64(v) }
    pub fn text(v: impl Into<String>) -> Self { Self::Text(v.into()) }
    pub fn party(v: PartyId) -> Self { Self::Party(v) }
    pub fn contract_id(v: ContractId) -> Self { Self::ContractId(v) }

    pub fn is_unit(&self) -> bool { matches!(self, Self::Unit) }
    pub fn is_bool(&self) -> bool { matches!(self, Self::Bool(_)) }
    pub fn is_int64(&self) -> bool { matches!(self, Self::Int64(_)) }
    pub fn is_text(&self) -> bool { matches!(self, Self::Text(_)) }
    pub fn is_record(&self) -> bool { matches!(self, Self::Record(_)) }

    pub fn as_bool(&self) -> Option<bool> {
        match self { Self::Bool(v) => Some(*v), _ => None }
    }
    pub fn as_int64(&self) -> Option<i64> {
        match self { Self::Int64(v) => Some(*v), _ => None }
    }
    pub fn as_text(&self) -> Option<&str> {
        match self { Self::Text(v) => Some(v), _ => None }
    }
    pub fn as_record(&self) -> Option<&DamlRecord> {
        match self { Self::Record(v) => Some(v), _ => None }
    }

    pub fn get_field(&self, name: &str) -> Option<&DamlValue> {
        self.as_record()?.fields.iter()
            .find(|f| f.label == name)
            .map(|f| &f.value)
    }
}

impl DamlRecord {
    pub fn new() -> Self {
        Self { record_id: None, fields: Vec::new() }
    }

    pub fn with_id(mut self, id: Identifier) -> Self {
        self.record_id = Some(id);
        self
    }

    pub fn field(mut self, label: impl Into<String>, value: impl Into<DamlValue>) -> Self {
        self.fields.push(RecordField {
            label: label.into(),
            value: value.into(),
        });
        self
    }

    pub fn get(&self, name: &str) -> Option<&DamlValue> {
        self.fields.iter()
            .find(|f| f.label == name)
            .map(|f| &f.value)
    }
}

impl Default for DamlRecord {
    fn default() -> Self { Self::new() }
}

impl From<bool> for DamlValue {
    fn from(v: bool) -> Self { Self::Bool(v) }
}
impl From<i64> for DamlValue {
    fn from(v: i64) -> Self { Self::Int64(v) }
}
impl From<&str> for DamlValue {
    fn from(v: &str) -> Self { Self::Text(v.to_string()) }
}
impl From<String> for DamlValue {
    fn from(v: String) -> Self { Self::Text(v) }
}
impl<T: Into<DamlValue>> From<Vec<T>> for DamlValue {
    fn from(v: Vec<T>) -> Self {
        Self::List(v.into_iter().map(Into::into).collect())
    }
}
impl<T: Into<DamlValue>> From<Option<T>> for DamlValue {
    fn from(v: Option<T>) -> Self {
        Self::Optional(v.map(|x| Box::new(x.into())))
    }
}
impl From<DamlRecord> for DamlValue {
    fn from(v: DamlRecord) -> Self { Self::Record(v) }
}
