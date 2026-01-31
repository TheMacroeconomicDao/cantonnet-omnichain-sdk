// Copyright 2025 Canton Wallet SDK Contributors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Protobuf definitions for Canton Ledger API

use crate::error::{LedgerError, LedgerResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Ledger identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LedgerId {
    pub value: String,
}

impl LedgerId {
    /// Create a new ledger ID
    pub fn new(value: impl Into<String>) -> Self {
        Self {
            value: value.into(),
        }
    }

    /// Get the ledger ID value
    pub fn as_str(&self) -> &str {
        &self.value
    }
}

impl From<String> for LedgerId {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

impl From<&str> for LedgerId {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

/// Application identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ApplicationId {
    pub value: String,
}

impl ApplicationId {
    /// Create a new application ID
    pub fn new(value: impl Into<String>) -> Self {
        Self {
            value: value.into(),
        }
    }

    /// Get the application ID value
    pub fn as_str(&self) -> &str {
        &self.value
    }
}

impl From<String> for ApplicationId {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

impl From<&str> for ApplicationId {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

/// Workflow identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct WorkflowId {
    pub value: String,
}

impl WorkflowId {
    /// Create a new workflow ID
    pub fn new(value: impl Into<String>) -> Self {
        Self {
            value: value.into(),
        }
    }

    /// Get the workflow ID value
    pub fn as_str(&self) -> &str {
        &self.value
    }
}

impl From<String> for WorkflowId {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

impl From<&str> for WorkflowId {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

/// Command identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CommandId {
    pub value: String,
}

impl CommandId {
    /// Create a new command ID
    pub fn new(value: impl Into<String>) -> Self {
        Self {
            value: value.into(),
        }
    }

    /// Get the command ID value
    pub fn as_str(&self) -> &str {
        &self.value
    }
}

impl From<String> for CommandId {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

impl From<&str> for CommandId {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

/// Party identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PartyId {
    pub value: String,
}

impl PartyId {
    /// Create a new party ID
    pub fn new(value: impl Into<String>) -> LedgerResult<Self> {
        let value = value.into();
        if value.is_empty() {
            return Err(LedgerError::InvalidPartyId("Party ID cannot be empty".to_string()));
        }
        Ok(Self { value })
    }

    /// Create a new party ID without validation
    pub fn new_unchecked(value: impl Into<String>) -> Self {
        Self {
            value: value.into(),
        }
    }

    /// Get the party ID value
    pub fn as_str(&self) -> &str {
        &self.value
    }
}

impl From<String> for PartyId {
    fn from(value: String) -> Self {
        Self::new_unchecked(value)
    }
}

impl From<&str> for PartyId {
    fn from(value: &str) -> Self {
        Self::new_unchecked(value)
    }
}

/// Contract identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ContractId {
    pub value: String,
}

impl ContractId {
    /// Create a new contract ID
    pub fn new(value: impl Into<String>) -> LedgerResult<Self> {
        let value = value.into();
        if value.is_empty() {
            return Err(LedgerError::InvalidContractId("Contract ID cannot be empty".to_string()));
        }
        Ok(Self { value })
    }

    /// Create a new contract ID without validation
    pub fn new_unchecked(value: impl Into<String>) -> Self {
        Self {
            value: value.into(),
        }
    }

    /// Get the contract ID value
    pub fn as_str(&self) -> &str {
        &self.value
    }
}

impl From<String> for ContractId {
    fn from(value: String) -> Self {
        Self::new_unchecked(value)
    }
}

impl From<&str> for ContractId {
    fn from(value: &str) -> Self {
        Self::new_unchecked(value)
    }
}

/// Template identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Identifier {
    pub package_id: String,
    pub module_name: String,
    pub entity_name: String,
}

impl Identifier {
    /// Create a new identifier
    pub fn new(
        package_id: impl Into<String>,
        module_name: impl Into<String>,
        entity_name: impl Into<String>,
    ) -> Self {
        Self {
            package_id: package_id.into(),
            module_name: module_name.into(),
            entity_name: entity_name.into(),
        }
    }

    /// Get the full identifier string
    pub fn to_string(&self) -> String {
        format!("{}:{}:{}", self.package_id, self.module_name, self.entity_name)
    }

    /// Parse an identifier from a string
    pub fn from_str(s: &str) -> LedgerResult<Self> {
        let parts: Vec<&str> = s.split(':').collect();
        if parts.len() != 3 {
            return Err(LedgerError::InvalidIdentifier(format!(
                "Invalid identifier format: {}. Expected format: package:module:entity",
                s
            )));
        }
        Ok(Self {
            package_id: parts[0].to_string(),
            module_name: parts[1].to_string(),
            entity_name: parts[2].to_string(),
        })
    }
}

/// Ledger offset for transaction streaming
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LedgerOffset {
    /// Beginning of the ledger
    Begin,
    /// End of the ledger
    End,
    /// Absolute offset
    Absolute(String),
    /// Boundary
    Boundary(String),
}

impl LedgerOffset {
    /// Create a new absolute offset
    pub fn absolute(value: impl Into<String>) -> Self {
        Self::Absolute(value.into())
    }

    /// Create a new boundary offset
    pub fn boundary(value: impl Into<String>) -> Self {
        Self::Boundary(value.into())
    }
}

impl Default for LedgerOffset {
    fn default() -> Self {
        Self::Begin
    }
}

/// Transaction filter
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransactionFilter {
    /// Filter by party
    ByParty {
        party: PartyId,
    },
    /// Filter by template
    ByTemplate {
        party: PartyId,
        template_id: Identifier,
    },
    /// Filter by interface
    ByInterface {
        party: PartyId,
        interface_id: Identifier,
    },
    /// No filter
    None,
}

impl TransactionFilter {
    /// Create a filter for a party
    pub fn for_party(party: &PartyId) -> Self {
        Self::ByParty {
            party: party.clone(),
        }
    }

    /// Create a filter for a template
    pub fn for_template(party: &PartyId, template_id: Identifier) -> Self {
        Self::ByTemplate {
            party: party.clone(),
            template_id,
        }
    }

    /// Create a filter for an interface
    pub fn for_interface(party: &PartyId, interface_id: Identifier) -> Self {
        Self::ByInterface {
            party: party.clone(),
            interface_id,
        }
    }
}

impl Default for TransactionFilter {
    fn default() -> Self {
        Self::None
    }
}

/// Daml value
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum DamlValue {
    /// Unit value
    Unit,
    /// Boolean value
    Bool(bool),
    /// Integer value
    Int64(i64),
    /// Decimal value
    Decimal(String),
    /// Text value
    Text(String),
    /// Timestamp value
    Timestamp(i64),
    /// Party value
    Party(PartyId),
    /// Contract ID value
    ContractId(ContractId),
    /// List value
    List(Vec<DamlValue>),
    /// Optional value
    Optional(Option<Box<DamlValue>>),
    /// Map value
    Map(HashMap<String, DamlValue>),
    /// Record value
    Record(DamlRecord),
    /// Variant value
    Variant(DamlVariant),
    /// Enum value
    Enum(String),
    /// Gen map value
    GenMap(Vec<(DamlValue, DamlValue)>),
}

impl DamlValue {
    /// Create a unit value
    pub fn unit() -> Self {
        Self::Unit
    }

    /// Create a boolean value
    pub fn bool(value: bool) -> Self {
        Self::Bool(value)
    }

    /// Create an integer value
    pub fn int64(value: i64) -> Self {
        Self::Int64(value)
    }

    /// Create a decimal value
    pub fn decimal(value: impl Into<String>) -> Self {
        Self::Decimal(value.into())
    }

    /// Create a text value
    pub fn text(value: impl Into<String>) -> Self {
        Self::Text(value.into())
    }

    /// Create a timestamp value
    pub fn timestamp(value: i64) -> Self {
        Self::Timestamp(value)
    }

    /// Create a party value
    pub fn party(value: PartyId) -> Self {
        Self::Party(value)
    }

    /// Create a contract ID value
    pub fn contract_id(value: ContractId) -> Self {
        Self::ContractId(value)
    }

    /// Create a list value
    pub fn list(values: Vec<DamlValue>) -> Self {
        Self::List(values)
    }

    /// Create an optional value
    pub fn optional(value: Option<DamlValue>) -> Self {
        Self::Optional(value.map(Box::new))
    }

    /// Create a map value
    pub fn map(values: HashMap<String, DamlValue>) -> Self {
        Self::Map(values)
    }

    /// Create a record value
    pub fn record(value: DamlRecord) -> Self {
        Self::Record(value)
    }

    /// Create a variant value
    pub fn variant(value: DamlVariant) -> Self {
        Self::Variant(value)
    }

    /// Create an enum value
    pub fn enum_value(value: impl Into<String>) -> Self {
        Self::Enum(value.into())
    }
}

impl Default for DamlValue {
    fn default() -> Self {
        Self::Unit
    }
}

/// Daml record
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DamlRecord {
    pub record_id: Option<Identifier>,
    pub fields: Vec<DamlRecordField>,
}

impl DamlRecord {
    /// Create a new record
    pub fn new() -> Self {
        Self {
            record_id: None,
            fields: Vec::new(),
        }
    }

    /// Create a new record with an identifier
    pub fn with_id(record_id: Identifier) -> Self {
        Self {
            record_id: Some(record_id),
            fields: Vec::new(),
        }
    }

    /// Add a field to the record
    pub fn add_field(mut self, label: Option<String>, value: DamlValue) -> Self {
        self.fields.push(DamlRecordField { label, value });
        self
    }

    /// Get a field by label
    pub fn get_field(&self, label: &str) -> Option<&DamlValue> {
        self.fields
            .iter()
            .find(|f| f.label.as_deref() == Some(label))
            .map(|f| &f.value)
    }
}

impl Default for DamlRecord {
    fn default() -> Self {
        Self::new()
    }
}

/// Daml record field
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DamlRecordField {
    pub label: Option<String>,
    pub value: DamlValue,
}

/// Daml variant
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DamlVariant {
    pub variant_id: Option<Identifier>,
    pub constructor: String,
    pub value: Box<DamlValue>,
}

impl DamlVariant {
    /// Create a new variant
    pub fn new(constructor: impl Into<String>, value: DamlValue) -> Self {
        Self {
            variant_id: None,
            constructor: constructor.into(),
            value: Box::new(value),
        }
    }

    /// Create a new variant with an identifier
    pub fn with_id(variant_id: Identifier, constructor: impl Into<String>, value: DamlValue) -> Self {
        Self {
            variant_id: Some(variant_id),
            constructor: constructor.into(),
            value: Box::new(value),
        }
    }
}

/// Command
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum Command {
    /// Create command
    Create(CreateCommand),
    /// Exercise command
    Exercise(ExerciseCommand),
    /// Exercise by key command
    ExerciseByKey(ExerciseByKeyCommand),
    /// Create and exercise command
    CreateAndExercise(CreateAndExerciseCommand),
}

/// Create command
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreateCommand {
    pub template_id: Identifier,
    pub create_arguments: DamlRecord,
}

/// Exercise command
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExerciseCommand {
    pub template_id: Option<Identifier>,
    pub contract_id: String,
    pub choice: String,
    pub choice_argument: DamlValue,
}

/// Exercise by key command
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExerciseByKeyCommand {
    pub template_id: Identifier,
    pub contract_key: DamlValue,
    pub choice: String,
    pub choice_argument: DamlValue,
}

/// Create and exercise command
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreateAndExerciseCommand {
    pub template_id: Identifier,
    pub create_arguments: DamlRecord,
    pub choice: String,
    pub choice_argument: DamlValue,
}

/// Commands
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Commands {
    pub ledger_id: String,
    pub workflow_id: String,
    pub application_id: String,
    pub command_id: String,
    pub party: String,
    pub commands: Vec<Command>,
    pub act_as: Vec<String>,
    pub read_as: Vec<String>,
    pub min_ledger_time_abs: Option<i64>,
    pub min_ledger_time_rel: Option<i64>,
    pub deduplication_time: Option<i64>,
}

impl Default for Commands {
    fn default() -> Self {
        Self {
            ledger_id: String::new(),
            workflow_id: String::new(),
            application_id: String::new(),
            command_id: String::new(),
            party: String::new(),
            commands: Vec::new(),
            act_as: Vec::new(),
            read_as: Vec::new(),
            min_ledger_time_abs: None,
            min_ledger_time_rel: None,
            deduplication_time: None,
        }
    }
}

/// Event
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum Event {
    /// Created event
    Created(CreatedEvent),
    /// Archived event
    Archived(ArchivedEvent),
}

/// Created event
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreatedEvent {
    pub event_id: String,
    pub contract_id: String,
    pub template_id: Identifier,
    pub create_arguments: DamlRecord,
    pub witness_parties: Vec<String>,
    pub signatories: Vec<String>,
    pub observers: Vec<String>,
    pub agreement_text: Option<String>,
}

/// Archived event
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ArchivedEvent {
    pub event_id: String,
    pub contract_id: String,
    pub template_id: Identifier,
    pub witness_parties: Vec<String>,
}

/// Transaction
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Transaction {
    pub transaction_id: String,
    pub command_id: String,
    pub workflow_id: String,
    pub effective_at: i64,
    pub offset: String,
    pub events: Vec<Event>,
    pub party_ids: Vec<String>,
}

/// Completion
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Completion {
    pub command_id: String,
    pub status: CompletionStatus,
    pub transaction_id: Option<String>,
    pub update_id: Option<String>,
}

/// Completion status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CompletionStatus {
    Success,
    InvalidArgument,
    InvalidArgumentDaml,
    InvalidArgumentLedger,
    InvalidArgumentParty,
    InvalidArgumentTemplate,
    InvalidArgumentContractId,
    InvalidArgumentCommand,
    InvalidArgumentTime,
    InvalidArgumentDuplicate,
    InvalidArgumentMissing,
    InvalidArgumentUnsatisfied,
    InvalidArgumentInconsistent,
    InvalidArgumentPreexecution,
    InvalidArgumentSubmission,
    InvalidArgumentNamespace,
    InvalidArgumentPackage,
    InvalidArgumentService,
    InvalidArgumentRateLimit,
    InvalidArgumentQuota,
    InvalidArgumentPermission,
    InvalidArgumentAuthentication,
    InvalidArgumentAuthorization,
    InvalidArgumentValidation,
    InvalidArgumentConfiguration,
    InvalidArgumentEnvironment,
    InvalidArgumentInternal,
    InvalidArgumentUnknown,
    SubmissionError,
    SubmissionErrorDaml,
    SubmissionErrorLedger,
    SubmissionErrorParty,
    SubmissionErrorTemplate,
    SubmissionErrorContractId,
    SubmissionErrorCommand,
    SubmissionErrorTime,
    SubmissionErrorDuplicate,
    SubmissionErrorMissing,
    SubmissionErrorUnsatisfied,
    SubmissionErrorInconsistent,
    SubmissionErrorPreexecution,
    SubmissionErrorSubmission,
    SubmissionErrorNamespace,
    SubmissionErrorPackage,
    SubmissionErrorService,
    SubmissionErrorRateLimit,
    SubmissionErrorQuota,
    SubmissionErrorPermission,
    SubmissionErrorAuthentication,
    SubmissionErrorAuthorization,
    SubmissionErrorValidation,
    SubmissionErrorConfiguration,
    SubmissionErrorEnvironment,
    SubmissionErrorInternal,
    SubmissionErrorUnknown,
}

/// Active contracts
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ActiveContracts {
    pub offset: String,
    pub active_contracts: Vec<CreatedEvent>,
}

/// Get active contracts request
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GetActiveContractsRequest {
    pub ledger_id: String,
    pub filter: TransactionFilter,
    pub verbose: bool,
    pub offset: LedgerOffset,
}

/// Get transactions request
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GetTransactionsRequest {
    pub ledger_id: String,
    pub begin: LedgerOffset,
    pub end: Option<LedgerOffset>,
    pub filter: TransactionFilter,
    pub verbose: bool,
}

/// Submit request
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SubmitRequest {
    pub commands: Commands,
}

/// Submit and wait request
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SubmitAndWaitRequest {
    pub commands: Commands,
    pub timeout: Option<i64>,
}

/// Submit and wait for transaction request
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SubmitAndWaitForTransactionRequest {
    pub commands: Commands,
    pub timeout: Option<i64>,
}

/// Submit and wait for transaction tree request
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SubmitAndWaitForTransactionTreeRequest {
    pub commands: Commands,
    pub timeout: Option<i64>,
}

/// Submit and wait for update request
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SubmitAndWaitForUpdateRequest {
    pub commands: Commands,
    pub timeout: Option<i64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ledger_id() {
        let id = LedgerId::new("test-ledger");
        assert_eq!(id.as_str(), "test-ledger");
    }

    #[test]
    fn test_party_id_validation() {
        let result = PartyId::new("");
        assert!(result.is_err());

        let result = PartyId::new("test-party");
        assert!(result.is_ok());
        assert_eq!(result.unwrap().as_str(), "test-party");
    }

    #[test]
    fn test_identifier() {
        let id = Identifier::new("pkg", "mod", "tpl");
        assert_eq!(id.to_string(), "pkg:mod:tpl");

        let parsed = Identifier::from_str("pkg:mod:tpl").unwrap();
        assert_eq!(parsed.package_id, "pkg");
        assert_eq!(parsed.module_name, "mod");
        assert_eq!(parsed.entity_name, "tpl");
    }

    #[test]
    fn test_daml_value() {
        let value = DamlValue::unit();
        assert_eq!(value, DamlValue::Unit);

        let value = DamlValue::bool(true);
        assert_eq!(value, DamlValue::Bool(true));

        let value = DamlValue::int64(42);
        assert_eq!(value, DamlValue::Int64(42));

        let value = DamlValue::text("hello");
        assert_eq!(value, DamlValue::Text("hello".to_string()));
    }

    #[test]
    fn test_daml_record() {
        let record = DamlRecord::new()
            .add_field(Some("field1".to_string()), DamlValue::int64(42))
            .add_field(Some("field2".to_string()), DamlValue::text("hello"));

        assert_eq!(record.fields.len(), 2);
        assert_eq!(record.get_field("field1"), Some(&DamlValue::Int64(42)));
        assert_eq!(record.get_field("field2"), Some(&DamlValue::Text("hello".to_string())));
    }

    #[test]
    fn test_transaction_filter() {
        let party = PartyId::new_unchecked("test-party");
        let filter = TransactionFilter::for_party(&party);
        assert!(matches!(filter, TransactionFilter::ByParty { .. }));

        let template_id = Identifier::new("pkg", "mod", "tpl");
        let filter = TransactionFilter::for_template(&party, template_id);
        assert!(matches!(filter, TransactionFilter::ByTemplate { .. }));
    }
}
