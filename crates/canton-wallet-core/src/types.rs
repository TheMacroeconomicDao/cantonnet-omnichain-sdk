// Core types for Canton Wallet SDK

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;
use validator::Validate;

/// Wallet identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct WalletId(String);

impl WalletId {
    /// Create a new wallet ID from a string
    pub fn new(id: impl Into<String>) -> WalletResult<Self> {
        let id = id.into();
        if id.is_empty() {
            return Err(WalletError::InvalidWalletId("Wallet ID cannot be empty".to_string()));
        }
        if id.len() > 256 {
            return Err(WalletError::InvalidWalletId(
                "Wallet ID too long (max 256 characters)".to_string(),
            ));
        }
        Ok(Self(id))
    }

    /// Create a wallet ID without validation (use with caution)
    pub fn new_unchecked(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Get the wallet ID as a string
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Generate a new random wallet ID
    pub fn generate() -> Self {
        Self(Uuid::new_v4().to_string())
    }
}

impl AsRef<str> for WalletId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for WalletId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Party identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PartyId(String);

impl PartyId {
    /// Create a new party ID from a string
    pub fn new(id: impl Into<String>) -> WalletResult<Self> {
        let id = id.into();
        if id.is_empty() {
            return Err(WalletError::InvalidPartyId("Party ID cannot be empty".to_string()));
        }
        if id.len() > 256 {
            return Err(WalletError::InvalidPartyId(
                "Party ID too long (max 256 characters)".to_string(),
            ));
        }
        Ok(Self(id))
    }

    /// Create a party ID without validation (use with caution)
    pub fn new_unchecked(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Get the party ID as a string
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Generate a new random party ID
    pub fn generate() -> Self {
        Self(Uuid::new_v4().to_string())
    }
}

impl AsRef<str> for PartyId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for PartyId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Participant identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ParticipantId(String);

impl ParticipantId {
    /// Create a new participant ID from a string
    pub fn new(id: impl Into<String>) -> WalletResult<Self> {
        let id = id.into();
        if id.is_empty() {
            return Err(WalletError::InvalidPartyId(
                "Participant ID cannot be empty".to_string(),
            ));
        }
        if id.len() > 256 {
            return Err(WalletError::InvalidPartyId(
                "Participant ID too long (max 256 characters)".to_string(),
            ));
        }
        Ok(Self(id))
    }

    /// Create a participant ID without validation (use with caution)
    pub fn new_unchecked(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Get the participant ID as a string
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl AsRef<str> for ParticipantId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for ParticipantId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Contract identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ContractId(String);

impl ContractId {
    /// Create a new contract ID from a string
    pub fn new(id: impl Into<String>) -> WalletResult<Self> {
        let id = id.into();
        if id.is_empty() {
            return Err(WalletError::InvalidContractId(
                "Contract ID cannot be empty".to_string(),
            ));
        }
        if id.len() > 512 {
            return Err(WalletError::InvalidContractId(
                "Contract ID too long (max 512 characters)".to_string(),
            ));
        }
        Ok(Self(id))
    }

    /// Create a contract ID without validation (use with caution)
    pub fn new_unchecked(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Get the contract ID as a string
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl AsRef<str> for ContractId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for ContractId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Transaction identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TransactionId(String);

impl TransactionId {
    /// Create a new transaction ID from a string
    pub fn new(id: impl Into<String>) -> WalletResult<Self> {
        let id = id.into();
        if id.is_empty() {
            return Err(WalletError::InvalidTransactionId(
                "Transaction ID cannot be empty".to_string(),
            ));
        }
        if id.len() > 512 {
            return Err(WalletError::InvalidTransactionId(
                "Transaction ID too long (max 512 characters)".to_string(),
            ));
        }
        Ok(Self(id))
    }

    /// Create a transaction ID without validation (use with caution)
    pub fn new_unchecked(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Get the transaction ID as a string
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Generate a new random transaction ID
    pub fn generate() -> Self {
        Self(Uuid::new_v4().to_string())
    }
}

impl AsRef<str> for TransactionId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for TransactionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Key identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct KeyId(String);

impl KeyId {
    /// Create a new key ID from a string
    pub fn new(id: impl Into<String>) -> WalletResult<Self> {
        let id = id.into();
        if id.is_empty() {
            return Err(WalletError::InvalidKeyId("Key ID cannot be empty".to_string()));
        }
        if id.len() > 256 {
            return Err(WalletError::InvalidKeyId(
                "Key ID too long (max 256 characters)".to_string(),
            ));
        }
        Ok(Self(id))
    }

    /// Create a key ID without validation (use with caution)
    pub fn new_unchecked(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Get the key ID as a string
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Generate a new random key ID
    pub fn generate() -> Self {
        Self(Uuid::new_v4().to_string())
    }
}

impl AsRef<str> for KeyId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for KeyId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Template identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TemplateId {
    pub package_id: String,
    pub module_name: String,
    pub entity_name: String,
}

impl TemplateId {
    /// Create a new template ID
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

    /// Get the full template identifier string
    pub fn as_str(&self) -> String {
        format!("{}:{}:{}", self.package_id, self.module_name, self.entity_name)
    }
}

impl std::fmt::Display for TemplateId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
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

/// Transaction filter
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransactionFilter {
    /// Filter by party
    ByParty(PartyId),
    /// Filter by template
    ByTemplate { party_id: PartyId, template_id: TemplateId },
    /// Filter by parties
    ByParties(Vec<PartyId>),
    /// No filter
    None,
}

impl TransactionFilter {
    /// Create a filter for a specific party
    pub fn for_party(party_id: &PartyId) -> Self {
        Self::ByParty(party_id.clone())
    }

    /// Create a filter for a specific template
    pub fn for_template(party_id: &PartyId, template_id: TemplateId) -> Self {
        Self::ByTemplate {
            party_id: party_id.clone(),
            template_id,
        }
    }
}

/// Wallet balance
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WalletBalance {
    pub total_amount: String,
    pub available_amount: String,
    pub locked_amount: String,
    pub currency: String,
}

impl WalletBalance {
    /// Create a new wallet balance
    pub fn new(
        total_amount: impl Into<String>,
        available_amount: impl Into<String>,
        locked_amount: impl Into<String>,
        currency: impl Into<String>,
    ) -> Self {
        Self {
            total_amount: total_amount.into(),
            available_amount: available_amount.into(),
            locked_amount: locked_amount.into(),
            currency: currency.into(),
        }
    }

    /// Create a zero balance
    pub fn zero(currency: impl Into<String>) -> Self {
        Self {
            total_amount: "0".to_string(),
            available_amount: "0".to_string(),
            locked_amount: "0".to_string(),
            currency: currency.into(),
        }
    }
}

/// Signature
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Signature {
    pub bytes: Vec<u8>,
    pub algorithm: String,
}

impl Signature {
    /// Create a new signature
    pub fn new(bytes: Vec<u8>, algorithm: impl Into<String>) -> Self {
        Self {
            bytes,
            algorithm: algorithm.into(),
        }
    }

    /// Get the signature bytes
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// Get the signature as hex
    pub fn to_hex(&self) -> String {
        hex::encode(&self.bytes)
    }

    /// Create signature from hex
    pub fn from_hex(hex: impl AsRef<str>, algorithm: impl Into<String>) -> WalletResult<Self> {
        let bytes = hex::decode(hex.as_ref())
            .map_err(|e| WalletError::InvalidSignature)?;
        Ok(Self::new(bytes, algorithm))
    }
}

/// Public key
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PublicKey {
    pub bytes: Vec<u8>,
    pub algorithm: String,
}

impl PublicKey {
    /// Create a new public key
    pub fn new(bytes: Vec<u8>, algorithm: impl Into<String>) -> Self {
        Self {
            bytes,
            algorithm: algorithm.into(),
        }
    }

    /// Get the public key bytes
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// Get the public key as hex
    pub fn to_hex(&self) -> String {
        hex::encode(&self.bytes)
    }

    /// Create public key from hex
    pub fn from_hex(hex: impl AsRef<str>, algorithm: impl Into<String>) -> WalletResult<Self> {
        let bytes = hex::decode(hex.as_ref())
            .map_err(|e| WalletError::InvalidSignature)?;
        Ok(Self::new(bytes, algorithm))
    }
}

/// Command
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum Command {
    Create(CreateCommand),
    Exercise(ExerciseCommand),
    ExerciseByKey(ExerciseByKeyCommand),
    CreateAndExercise(CreateAndExerciseCommand),
}

/// Create command
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateCommand {
    pub template_id: TemplateId,
    #[validate(length(min = 1))]
    pub create_arguments: serde_json::Value,
}

/// Exercise command
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct ExerciseCommand {
    pub template_id: Option<TemplateId>,
    #[validate(length(min = 1))]
    pub contract_id: String,
    #[validate(length(min = 1))]
    pub choice: String,
    pub choice_argument: serde_json::Value,
}

/// Exercise by key command
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct ExerciseByKeyCommand {
    pub template_id: TemplateId,
    pub contract_key: serde_json::Value,
    #[validate(length(min = 1))]
    pub choice: String,
    pub choice_argument: serde_json::Value,
}

/// Create and exercise command
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateAndExerciseCommand {
    pub template_id: TemplateId,
    #[validate(length(min = 1))]
    pub create_arguments: serde_json::Value,
    #[validate(length(min = 1))]
    pub choice: String,
    pub choice_argument: serde_json::Value,
}

/// Transaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub transaction_id: TransactionId,
    pub command_id: String,
    pub workflow_id: String,
    pub effective_at: DateTime<Utc>,
    pub events: Vec<Event>,
    pub offset: String,
}

/// Event
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum Event {
    Created(CreatedEvent),
    Exercised(ExercisedEvent),
    Archived(ArchivedEvent),
}

/// Created event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatedEvent {
    pub contract_id: ContractId,
    pub template_id: TemplateId,
    pub create_arguments: serde_json::Value,
    pub signatories: Vec<PartyId>,
    pub observers: Vec<PartyId>,
    pub agreement_text: String,
}

/// Exercised event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExercisedEvent {
    pub contract_id: ContractId,
    pub choice: String,
    pub choice_argument: serde_json::Value,
    pub acting_parties: Vec<PartyId>,
    pub consuming: bool,
    pub exercise_result: Option<serde_json::Value>,
}

/// Archived event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchivedEvent {
    pub contract_id: ContractId,
}

/// Commands wrapper
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct Commands {
    pub ledger_id: String,
    pub workflow_id: String,
    pub application_id: String,
    pub command_id: String,
    pub party: String,
    pub commands: Vec<Command>,
    pub act_as: Vec<String>,
    pub read_as: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_ledger_time_abs: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_ledger_time_rel: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deduplication_time: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub submission_id: Option<String>,
}

impl Default for Commands {
    fn default() -> Self {
        Self {
            ledger_id: String::new(),
            workflow_id: Uuid::new_v4().to_string(),
            application_id: "canton-wallet-sdk".to_string(),
            command_id: Uuid::new_v4().to_string(),
            party: String::new(),
            commands: Vec::new(),
            act_as: Vec::new(),
            read_as: Vec::new(),
            min_ledger_time_abs: None,
            min_ledger_time_rel: None,
            deduplication_time: None,
            submission_id: None,
        }
    }
}

/// Daml value
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum DamlValue {
    Unit,
    Bool(bool),
    Int64(i64),
    Decimal(String),
    Text(String),
    Timestamp(DateTime<Utc>),
    Party(PartyId),
    ContractId(ContractId),
    List(Vec<DamlValue>),
    Optional(Option<Box<DamlValue>>),
    Map(Vec<(String, DamlValue)>),
    Record(DamlRecord),
    Variant(DamlVariant),
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

    /// Create an int64 value
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
    pub fn map(entries: Vec<(String, DamlValue)>) -> Self {
        Self::Map(entries)
    }
}

/// Daml record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DamlRecord {
    pub record_id: Option<TemplateId>,
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

    /// Create a record with a template ID
    pub fn with_template_id(template_id: TemplateId) -> Self {
        Self {
            record_id: Some(template_id),
            fields: Vec::new(),
        }
    }

    /// Add a field to the record
    pub fn add_field(mut self, label: impl Into<String>, value: DamlValue) -> Self {
        self.fields.push(DamlRecordField {
            label: Some(label.into()),
            value,
        });
        self
    }
}

impl Default for DamlRecord {
    fn default() -> Self {
        Self::new()
    }
}

/// Daml record field
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DamlRecordField {
    pub label: Option<String>,
    pub value: DamlValue,
}

/// Daml variant
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DamlVariant {
    pub variant: String,
    pub value: DamlValue,
}

/// Identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Identifier {
    pub package_id: String,
    pub module_name: String,
    pub name: String,
}

impl Identifier {
    /// Create a new identifier
    pub fn new(
        package_id: impl Into<String>,
        module_name: impl Into<String>,
        name: impl Into<String>,
    ) -> Self {
        Self {
            package_id: package_id.into(),
            module_name: module_name.into(),
            name: name.into(),
        }
    }

    /// Get the full identifier string
    pub fn as_str(&self) -> String {
        format!("{}:{}:{}", self.package_id, self.module_name, self.name)
    }
}

impl std::fmt::Display for Identifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Wallet configuration
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct WalletConfig {
    pub ledger_endpoint: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub participant_id: Option<ParticipantId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub party_id: Option<PartyId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub application_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_retries: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout_ms: Option<u64>,
}

impl Default for WalletConfig {
    fn default() -> Self {
        Self {
            ledger_endpoint: String::new(),
            participant_id: None,
            party_id: None,
            application_id: Some("canton-wallet-sdk".to_string()),
            max_retries: Some(3),
            timeout_ms: Some(30000),
        }
    }
}

/// Contract info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractInfo {
    pub contract_id: ContractId,
    pub template_id: TemplateId,
    pub arguments: serde_json::Value,
    pub signatories: Vec<PartyId>,
    pub observers: Vec<PartyId>,
    pub created_at: DateTime<Utc>,
}

impl ContractInfo {
    /// Create contract info from a created event
    pub fn from_created_event(event: &CreatedEvent) -> Self {
        Self {
            contract_id: event.contract_id.clone(),
            template_id: event.template_id.clone(),
            arguments: event.create_arguments.clone(),
            signatories: event.signatories.clone(),
            observers: event.observers.clone(),
            created_at: Utc::now(),
        }
    }
}

/// Approval response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalResponse {
    pub approved: bool,
    pub timestamp: DateTime<Utc>,
    pub metadata: HashMap<String, String>,
}

impl ApprovalResponse {
    /// Create a new approval response
    pub fn new(approved: bool) -> Self {
        Self {
            approved,
            timestamp: Utc::now(),
            metadata: HashMap::new(),
        }
    }

    /// Add metadata to the approval response
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
}

/// Key metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyMetadata {
    pub created_at: DateTime<Utc>,
    pub purpose: String,
    pub algorithm: String,
    pub tags: HashMap<String, String>,
}

impl KeyMetadata {
    /// Create new key metadata
    pub fn new(purpose: impl Into<String>, algorithm: impl Into<String>) -> Self {
        Self {
            created_at: Utc::now(),
            purpose: purpose.into(),
            algorithm: algorithm.into(),
            tags: HashMap::new(),
        }
    }

    /// Add a tag to the metadata
    pub fn with_tag(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.tags.insert(key.into(), value.into());
        self
    }
}

impl Default for KeyMetadata {
    fn default() -> Self {
        Self::new("signing", "ed25519")
    }
}

/// Key info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyInfo {
    pub key_id: KeyId,
    pub public_key: PublicKey,
    pub metadata: KeyMetadata,
}

/// Key algorithm
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum KeyAlgorithm {
    Ed25519,
    Secp256k1,
    Secp256r1,
}

impl KeyAlgorithm {
    /// Get the algorithm name
    pub fn as_str(&self) -> &str {
        match self {
            Self::Ed25519 => "ed25519",
            Self::Secp256k1 => "secp256k1",
            Self::Secp256r1 => "secp256r1",
        }
    }

    /// Parse algorithm from string
    pub fn from_str(s: &str) -> WalletResult<Self> {
        match s.to_lowercase().as_str() {
            "ed25519" => Ok(Self::Ed25519),
            "secp256k1" => Ok(Self::Secp256k1),
            "secp256r1" => Ok(Self::Secp256r1),
            _ => Err(WalletError::InvalidArgument(format!("Unknown key algorithm: {}", s))),
        }
    }
}

/// Key purpose
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum KeyPurpose {
    Signing,
    Encryption,
    Derivation,
}

impl KeyPurpose {
    /// Get the purpose name
    pub fn as_str(&self) -> &str {
        match self {
            Self::Signing => "signing",
            Self::Encryption => "encryption",
            Self::Derivation => "derivation",
        }
    }

    /// Parse purpose from string
    pub fn from_str(s: &str) -> WalletResult<Self> {
        match s.to_lowercase().as_str() {
            "signing" => Ok(Self::Signing),
            "encryption" => Ok(Self::Encryption),
            "derivation" => Ok(Self::Derivation),
            _ => Err(WalletError::InvalidArgument(format!("Unknown key purpose: {}", s))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wallet_id() {
        let id = WalletId::new("test-wallet").unwrap();
        assert_eq!(id.as_str(), "test-wallet");
        assert_eq!(id.to_string(), "test-wallet");
    }

    #[test]
    fn test_wallet_id_invalid() {
        assert!(WalletId::new("").is_err());
    }

    #[test]
    fn test_wallet_id_generate() {
        let id1 = WalletId::generate();
        let id2 = WalletId::generate();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_party_id() {
        let id = PartyId::new("test-party").unwrap();
        assert_eq!(id.as_str(), "test-party");
    }

    #[test]
    fn test_contract_id() {
        let id = ContractId::new("test-contract").unwrap();
        assert_eq!(id.as_str(), "test-contract");
    }

    #[test]
    fn test_transaction_id() {
        let id = TransactionId::new("test-tx").unwrap();
        assert_eq!(id.as_str(), "test-tx");
    }

    #[test]
    fn test_key_id() {
        let id = KeyId::new("test-key").unwrap();
        assert_eq!(id.as_str(), "test-key");
    }

    #[test]
    fn test_template_id() {
        let id = TemplateId::new("pkg", "mod", "tpl");
        assert_eq!(id.as_str(), "pkg:mod:tpl");
    }

    #[test]
    fn test_wallet_balance() {
        let balance = WalletBalance::new("100", "80", "20", "USD");
        assert_eq!(balance.total_amount, "100");
        assert_eq!(balance.available_amount, "80");
        assert_eq!(balance.locked_amount, "20");
        assert_eq!(balance.currency, "USD");
    }

    #[test]
    fn test_wallet_balance_zero() {
        let balance = WalletBalance::zero("USD");
        assert_eq!(balance.total_amount, "0");
        assert_eq!(balance.available_amount, "0");
        assert_eq!(balance.locked_amount, "0");
        assert_eq!(balance.currency, "USD");
    }

    #[test]
    fn test_signature() {
        let sig = Signature::new(vec![1, 2, 3], "ed25519");
        assert_eq!(sig.as_bytes(), &[1, 2, 3]);
        assert_eq!(sig.to_hex(), "010203");
    }

    #[test]
    fn test_signature_from_hex() {
        let sig = Signature::from_hex("010203", "ed25519").unwrap();
        assert_eq!(sig.as_bytes(), &[1, 2, 3]);
    }

    #[test]
    fn test_public_key() {
        let key = PublicKey::new(vec![1, 2, 3], "ed25519");
        assert_eq!(key.as_bytes(), &[1, 2, 3]);
        assert_eq!(key.to_hex(), "010203");
    }

    #[test]
    fn test_daml_value() {
        let val = DamlValue::text("hello");
        assert!(matches!(val, DamlValue::Text(_)));
    }

    #[test]
    fn test_daml_record() {
        let record = DamlRecord::new()
            .add_field("field1", DamlValue::int64(42))
            .add_field("field2", DamlValue::text("hello"));
        assert_eq!(record.fields.len(), 2);
    }

    #[test]
    fn test_identifier() {
        let id = Identifier::new("pkg", "mod", "name");
        assert_eq!(id.as_str(), "pkg:mod:name");
    }

    #[test]
    fn test_wallet_config_default() {
        let config = WalletConfig::default();
        assert_eq!(config.application_id, Some("canton-wallet-sdk".to_string()));
        assert_eq!(config.max_retries, Some(3));
        assert_eq!(config.timeout_ms, Some(30000));
    }

    #[test]
    fn test_approval_response() {
        let response = ApprovalResponse::new(true).with_metadata("key", "value");
        assert!(response.approved);
        assert_eq!(response.metadata.get("key"), Some(&"value".to_string()));
    }

    #[test]
    fn test_key_metadata() {
        let metadata = KeyMetadata::new("signing", "ed25519").with_tag("tag", "value");
        assert_eq!(metadata.purpose, "signing");
        assert_eq!(metadata.algorithm, "ed25519");
        assert_eq!(metadata.tags.get("tag"), Some(&"value".to_string()));
    }

    #[test]
    fn test_key_algorithm() {
        assert_eq!(KeyAlgorithm::Ed25519.as_str(), "ed25519");
        assert_eq!(KeyAlgorithm::from_str("ed25519").unwrap(), KeyAlgorithm::Ed25519);
    }

    #[test]
    fn test_key_purpose() {
        assert_eq!(KeyPurpose::Signing.as_str(), "signing");
        assert_eq!(KeyPurpose::from_str("signing").unwrap(), KeyPurpose::Signing);
    }
}
