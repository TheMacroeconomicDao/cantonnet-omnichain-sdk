//! Transaction filter for Ledger API.
//! See research/08, 04.

use crate::types::identifier::Identifier;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct TransactionFilter {
    pub filters_by_party: HashMap<String, Filters>,
}

#[derive(Debug, Clone)]
pub struct Filters {
    pub inclusive: Option<InclusiveFilters>,
}

#[derive(Debug, Clone)]
pub struct InclusiveFilters {
    pub template_ids: Vec<Identifier>,
    pub interface_filters: Vec<InterfaceFilter>,
}

#[derive(Debug, Clone)]
pub struct InterfaceFilter {
    pub interface_id: Identifier,
    pub include_created_event_blob: bool,
}
