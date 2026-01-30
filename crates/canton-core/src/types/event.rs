//! Event types (Created, Archived, Exercised).
//! See research/08, 04.

use crate::types::identifier::{ContractId, Identifier};
use crate::types::value::{DamlRecord, DamlValue};

#[derive(Debug, Clone)]
pub enum Event {
    Created(CreatedEvent),
    Archived(ArchivedEvent),
    Exercised(ExercisedEvent),
}

#[derive(Debug, Clone)]
pub struct CreatedEvent {
    pub event_id: String,
    pub contract_id: ContractId,
    pub template_id: Identifier,
    pub create_arguments: DamlRecord,
    pub contract_key: Option<DamlValue>,
    pub signatories: Vec<String>,
    pub observers: Vec<String>,
    pub agreement_text: String,
}

#[derive(Debug, Clone)]
pub struct ArchivedEvent {
    pub event_id: String,
    pub contract_id: ContractId,
    pub template_id: Identifier,
    pub contract_key: Option<DamlValue>,
}

#[derive(Debug, Clone)]
pub struct ExercisedEvent {
    pub event_id: String,
    pub contract_id: ContractId,
    pub template_id: Identifier,
    pub choice: String,
    pub choice_argument: DamlValue,
    pub exercising_party: String,
    pub acting_parties: Vec<String>,
    pub consumed_contracts: Vec<String>,
    pub created_event_ids: Vec<String>,
    pub exercise_result: Option<DamlValue>,
}
