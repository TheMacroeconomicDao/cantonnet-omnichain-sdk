# Daml/Canton Ledger API Research

## 1. Ledger API Overview

The Ledger API is the primary interface for interacting with Canton Network. It provides a gRPC-based API for submitting commands, reading transactions, and managing parties.

## 2. API Services Specification

### 2.1 Service Architecture

```
┌─────────────────────────────────────────────────────────────────────┐
│                        Ledger API Services                          │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  ┌─────────────────────────────────────────────────────────────────┐│
│  │                    Command Services                              ││
│  │  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐ ││
│  │  │ CommandService  │  │CommandSubmission│  │CommandCompletion│ ││
│  │  │                 │  │    Service      │  │    Service      │ ││
│  │  └─────────────────┘  └─────────────────┘  └─────────────────┘ ││
│  └─────────────────────────────────────────────────────────────────┘│
│                                                                      │
│  ┌─────────────────────────────────────────────────────────────────┐│
│  │                    Query Services                                ││
│  │  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐ ││
│  │  │ Transaction     │  │ ActiveContracts │  │ EventQuery      │ ││
│  │  │ Service         │  │ Service         │  │ Service         │ ││
│  │  └─────────────────┘  └─────────────────┘  └─────────────────┘ ││
│  └─────────────────────────────────────────────────────────────────┘│
│                                                                      │
│  ┌─────────────────────────────────────────────────────────────────┐│
│  │                    Management Services                           ││
│  │  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐ ││
│  │  │ PartyManagement │  │ PackageService  │  │ LedgerIdentity  │ ││
│  │  │ Service         │  │                 │  │ Service         │ ││
│  │  └─────────────────┘  └─────────────────┘  └─────────────────┘ ││
│  └─────────────────────────────────────────────────────────────────┘│
│                                                                      │
│  ┌─────────────────────────────────────────────────────────────────┐│
│  │                    Utility Services                              ││
│  │  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐ ││
│  │  │ LedgerConfig    │  │ Version         │  │ Time            │ ││
│  │  │ Service         │  │ Service         │  │ Service         │ ││
│  │  └─────────────────┘  └─────────────────┘  └─────────────────┘ ││
│  └─────────────────────────────────────────────────────────────────┘│
└─────────────────────────────────────────────────────────────────────┘
```

### 2.2 Command Service

```protobuf
// Command Service - Synchronous command submission
service CommandService {
  // Submit a command and wait for the result
  rpc SubmitAndWait(SubmitAndWaitRequest) returns (SubmitAndWaitResponse);
  
  // Submit and wait for transaction ID
  rpc SubmitAndWaitForTransactionId(SubmitAndWaitRequest) 
      returns (SubmitAndWaitForTransactionIdResponse);
  
  // Submit and wait for transaction
  rpc SubmitAndWaitForTransaction(SubmitAndWaitRequest) 
      returns (SubmitAndWaitForTransactionResponse);
  
  // Submit and wait for transaction tree
  rpc SubmitAndWaitForTransactionTree(SubmitAndWaitRequest) 
      returns (SubmitAndWaitForTransactionTreeResponse);
}

message SubmitAndWaitRequest {
  Commands commands = 1;
}

message Commands {
  string ledger_id = 1;
  string workflow_id = 2;
  string application_id = 3;
  string command_id = 4;
  string party = 5;
  repeated Command commands = 6;
  google.protobuf.Timestamp min_ledger_time_abs = 7;
  google.protobuf.Duration min_ledger_time_rel = 8;
  string deduplication_period = 9;
  repeated string act_as = 10;
  repeated string read_as = 11;
  string submission_id = 12;
}
```

### 2.3 Command Submission Service

```protobuf
// Asynchronous command submission
service CommandSubmissionService {
  // Submit a command without waiting
  rpc Submit(SubmitRequest) returns (google.protobuf.Empty);
}

message SubmitRequest {
  Commands commands = 1;
}
```

### 2.4 Command Completion Service

```protobuf
// Track command completions
service CommandCompletionService {
  // Stream completions for commands
  rpc CompletionStream(CompletionStreamRequest) 
      returns (stream CompletionStreamResponse);
  
  // Get completion end offset
  rpc CompletionEnd(CompletionEndRequest) returns (CompletionEndResponse);
}

message CompletionStreamRequest {
  string ledger_id = 1;
  string application_id = 2;
  repeated string parties = 3;
  LedgerOffset offset = 4;
}

message CompletionStreamResponse {
  Checkpoint checkpoint = 1;
  repeated Completion completions = 2;
}

message Completion {
  string command_id = 1;
  google.rpc.Status status = 2;
  string transaction_id = 3;
  string application_id = 4;
  repeated string act_as = 5;
  string submission_id = 6;
  google.protobuf.Timestamp deduplication_period = 7;
}
```

### 2.5 Transaction Service

```protobuf
// Read transactions from the ledger
service TransactionService {
  // Stream flat transactions
  rpc GetTransactions(GetTransactionsRequest) 
      returns (stream GetTransactionsResponse);
  
  // Stream transaction trees
  rpc GetTransactionTrees(GetTransactionsRequest) 
      returns (stream GetTransactionTreesResponse);
  
  // Get single transaction by event ID
  rpc GetTransactionByEventId(GetTransactionByEventIdRequest) 
      returns (GetTransactionResponse);
  
  // Get single transaction by ID
  rpc GetTransactionById(GetTransactionByIdRequest) 
      returns (GetTransactionResponse);
  
  // Get flat transaction by event ID
  rpc GetFlatTransactionByEventId(GetTransactionByEventIdRequest) 
      returns (GetFlatTransactionResponse);
  
  // Get flat transaction by ID
  rpc GetFlatTransactionById(GetTransactionByIdRequest) 
      returns (GetFlatTransactionResponse);
  
  // Get ledger end offset
  rpc GetLedgerEnd(GetLedgerEndRequest) returns (GetLedgerEndResponse);
}

message GetTransactionsRequest {
  string ledger_id = 1;
  LedgerOffset begin = 2;
  LedgerOffset end = 3;
  TransactionFilter filter = 4;
  bool verbose = 5;
}

message TransactionFilter {
  map<string, Filters> filters_by_party = 1;
}

message Filters {
  InclusiveFilters inclusive = 1;
}

message InclusiveFilters {
  repeated Identifier template_ids = 1;
  repeated InterfaceFilter interface_filters = 2;
}

message Transaction {
  string transaction_id = 1;
  string command_id = 2;
  string workflow_id = 3;
  google.protobuf.Timestamp effective_at = 4;
  repeated Event events = 5;
  string offset = 6;
}

message TransactionTree {
  string transaction_id = 1;
  string command_id = 2;
  string workflow_id = 3;
  google.protobuf.Timestamp effective_at = 4;
  map<string, TreeEvent> events_by_id = 5;
  repeated string root_event_ids = 6;
  string offset = 7;
}
```

### 2.6 Active Contracts Service

```protobuf
// Query active contracts
service ActiveContractsService {
  // Get all active contracts matching filter
  rpc GetActiveContracts(GetActiveContractsRequest) 
      returns (stream GetActiveContractsResponse);
}

message GetActiveContractsRequest {
  string ledger_id = 1;
  TransactionFilter filter = 2;
  bool verbose = 3;
  string active_at_offset = 4;
}

message GetActiveContractsResponse {
  string offset = 1;
  string workflow_id = 2;
  repeated CreatedEvent active_contracts = 3;
}
```

### 2.7 Party Management Service

```protobuf
// Manage parties
service PartyManagementService {
  // Get participant ID
  rpc GetParticipantId(GetParticipantIdRequest) 
      returns (GetParticipantIdResponse);
  
  // List known parties
  rpc ListKnownParties(ListKnownPartiesRequest) 
      returns (ListKnownPartiesResponse);
  
  // Allocate a new party
  rpc AllocateParty(AllocatePartyRequest) returns (AllocatePartyResponse);
  
  // Update party details
  rpc UpdatePartyDetails(UpdatePartyDetailsRequest) 
      returns (UpdatePartyDetailsResponse);
}

message AllocatePartyRequest {
  string party_id_hint = 1;
  string display_name = 2;
  string local_metadata = 3;
}

message AllocatePartyResponse {
  PartyDetails party_details = 1;
}

message PartyDetails {
  string party = 1;
  string display_name = 2;
  bool is_local = 3;
  string local_metadata = 4;
  string identity_provider_id = 5;
}
```

### 2.8 Package Service

```protobuf
// Manage Daml packages
service PackageService {
  // List all packages
  rpc ListPackages(ListPackagesRequest) returns (ListPackagesResponse);
  
  // Get package contents
  rpc GetPackage(GetPackageRequest) returns (GetPackageResponse);
  
  // Get package status
  rpc GetPackageStatus(GetPackageStatusRequest) 
      returns (GetPackageStatusResponse);
}

message ListPackagesRequest {
  string ledger_id = 1;
}

message ListPackagesResponse {
  repeated string package_ids = 1;
}

message GetPackageRequest {
  string ledger_id = 1;
  string package_id = 2;
}

message GetPackageResponse {
  HashFunction hash_function = 1;
  bytes archive_payload = 2;
  string hash = 3;
}
```

## 3. Data Types

### 3.1 Daml Values

```protobuf
// Daml value representation
message Value {
  oneof sum {
    Record record = 1;
    Variant variant = 2;
    string contract_id = 3;
    List list = 4;
    sint64 int64 = 5;
    string numeric = 6;
    string text = 7;
    sfixed64 timestamp = 8;
    string party = 9;
    bool bool = 10;
    google.protobuf.Empty unit = 11;
    int32 date = 12;
    Optional optional = 13;
    Map map = 14;
    Enum enum = 15;
    GenMap gen_map = 16;
  }
}

message Record {
  Identifier record_id = 1;
  repeated RecordField fields = 2;
}

message RecordField {
  string label = 1;
  Value value = 2;
}

message Variant {
  Identifier variant_id = 1;
  string constructor = 2;
  Value value = 3;
}

message List {
  repeated Value elements = 1;
}

message Optional {
  Value value = 1;
}

message Map {
  repeated Map.Entry entries = 1;
  
  message Entry {
    string key = 1;
    Value value = 2;
  }
}

message GenMap {
  repeated GenMap.Entry entries = 1;
  
  message Entry {
    Value key = 1;
    Value value = 2;
  }
}

message Enum {
  Identifier enum_id = 1;
  string constructor = 2;
}
```

### 3.2 Identifiers

```protobuf
// Template/type identifier
message Identifier {
  string package_id = 1;
  string module_name = 2;
  string entity_name = 3;
}
```

### 3.3 Events

```protobuf
// Ledger events
message Event {
  oneof event {
    CreatedEvent created = 1;
    ArchivedEvent archived = 2;
  }
}

message CreatedEvent {
  string event_id = 1;
  string contract_id = 2;
  Identifier template_id = 3;
  string contract_key = 4;
  Record create_arguments = 5;
  repeated string witness_parties = 6;
  repeated string signatories = 7;
  repeated string observers = 8;
  google.protobuf.Timestamp created_at = 9;
}

message ArchivedEvent {
  string event_id = 1;
  string contract_id = 2;
  Identifier template_id = 3;
  repeated string witness_parties = 4;
}

message ExercisedEvent {
  string event_id = 1;
  string contract_id = 2;
  Identifier template_id = 3;
  string choice = 4;
  Value choice_argument = 5;
  repeated string acting_parties = 6;
  bool consuming = 7;
  repeated string witness_parties = 8;
  repeated string child_event_ids = 9;
  Value exercise_result = 10;
}

message TreeEvent {
  oneof kind {
    CreatedEvent created = 1;
    ExercisedEvent exercised = 2;
  }
}
```

### 3.4 Commands

```protobuf
// Command types
message Command {
  oneof command {
    CreateCommand create = 1;
    ExerciseCommand exercise = 2;
    ExerciseByKeyCommand exercise_by_key = 3;
    CreateAndExerciseCommand create_and_exercise = 4;
  }
}

message CreateCommand {
  Identifier template_id = 1;
  Record create_arguments = 2;
}

message ExerciseCommand {
  Identifier template_id = 1;
  string contract_id = 2;
  string choice = 3;
  Value choice_argument = 4;
}

message ExerciseByKeyCommand {
  Identifier template_id = 1;
  Value contract_key = 2;
  string choice = 3;
  Value choice_argument = 4;
}

message CreateAndExerciseCommand {
  Identifier template_id = 1;
  Record create_arguments = 2;
  string choice = 3;
  Value choice_argument = 4;
}
```

## 4. Rust SDK Implementation

### 4.1 Value Types

```rust
//! Daml value types for Rust SDK

use std::collections::HashMap;
use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;

/// Daml value representation
#[derive(Debug, Clone, PartialEq)]
pub enum DamlValue {
    /// Unit value
    Unit,
    /// Boolean value
    Bool(bool),
    /// 64-bit signed integer
    Int64(i64),
    /// Decimal number (Numeric)
    Numeric(Decimal),
    /// Text string
    Text(String),
    /// Timestamp
    Timestamp(DateTime<Utc>),
    /// Date
    Date(NaiveDate),
    /// Party identifier
    Party(PartyId),
    /// Contract ID
    ContractId(ContractId),
    /// List of values
    List(Vec<DamlValue>),
    /// Optional value
    Optional(Option<Box<DamlValue>>),
    /// Text map
    TextMap(HashMap<String, DamlValue>),
    /// Generic map
    GenMap(Vec<(DamlValue, DamlValue)>),
    /// Record (named fields)
    Record(DamlRecord),
    /// Variant (sum type)
    Variant(DamlVariant),
    /// Enum
    Enum(DamlEnum),
}

/// Daml record (product type)
#[derive(Debug, Clone, PartialEq)]
pub struct DamlRecord {
    /// Optional record type identifier
    pub record_id: Option<Identifier>,
    /// Record fields
    pub fields: Vec<RecordField>,
}

/// Record field
#[derive(Debug, Clone, PartialEq)]
pub struct RecordField {
    /// Field label
    pub label: String,
    /// Field value
    pub value: DamlValue,
}

/// Daml variant (sum type)
#[derive(Debug, Clone, PartialEq)]
pub struct DamlVariant {
    /// Variant type identifier
    pub variant_id: Option<Identifier>,
    /// Constructor name
    pub constructor: String,
    /// Constructor argument
    pub value: Box<DamlValue>,
}

/// Daml enum
#[derive(Debug, Clone, PartialEq)]
pub struct DamlEnum {
    /// Enum type identifier
    pub enum_id: Option<Identifier>,
    /// Constructor name
    pub constructor: String,
}

/// Template/type identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Identifier {
    /// Package ID (hash)
    pub package_id: String,
    /// Module name (dot-separated)
    pub module_name: String,
    /// Entity name
    pub entity_name: String,
}

impl Identifier {
    /// Create new identifier
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
    
    /// Parse from qualified name (package_id:module.Entity)
    pub fn parse(qualified: &str) -> Result<Self, ParseError> {
        let parts: Vec<&str> = qualified.split(':').collect();
        if parts.len() != 2 {
            return Err(ParseError::InvalidFormat);
        }
        
        let package_id = parts[0];
        let module_entity: Vec<&str> = parts[1].rsplitn(2, '.').collect();
        
        if module_entity.len() != 2 {
            return Err(ParseError::InvalidFormat);
        }
        
        Ok(Self {
            package_id: package_id.to_string(),
            module_name: module_entity[1].to_string(),
            entity_name: module_entity[0].to_string(),
        })
    }
    
    /// Get fully qualified name
    pub fn qualified_name(&self) -> String {
        format!("{}:{}.{}", self.package_id, self.module_name, self.entity_name)
    }
}

/// Party identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PartyId(String);

impl PartyId {
    pub fn new(id: impl Into<String>) -> Result<Self, ValidationError> {
        let id = id.into();
        Self::validate(&id)?;
        Ok(Self(id))
    }
    
    fn validate(id: &str) -> Result<(), ValidationError> {
        if id.is_empty() {
            return Err(ValidationError::Empty);
        }
        if id.len() > 256 {
            return Err(ValidationError::TooLong);
        }
        if !id.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_' || c == ':') {
            return Err(ValidationError::InvalidCharacters);
        }
        Ok(())
    }
    
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Contract identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ContractId(String);

impl ContractId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
    
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
```

### 4.2 Command Builder

```rust
//! Command builder for type-safe command construction

use crate::types::*;

/// Command builder
pub struct CommandBuilder {
    workflow_id: Option<String>,
    application_id: String,
    command_id: String,
    act_as: Vec<PartyId>,
    read_as: Vec<PartyId>,
    commands: Vec<Command>,
    deduplication_period: Option<DeduplicationPeriod>,
    min_ledger_time: Option<MinLedgerTime>,
}

impl CommandBuilder {
    /// Create new command builder
    pub fn new(application_id: impl Into<String>) -> Self {
        Self {
            workflow_id: None,
            application_id: application_id.into(),
            command_id: uuid::Uuid::new_v4().to_string(),
            act_as: Vec::new(),
            read_as: Vec::new(),
            commands: Vec::new(),
            deduplication_period: None,
            min_ledger_time: None,
        }
    }
    
    /// Set workflow ID
    pub fn workflow_id(mut self, id: impl Into<String>) -> Self {
        self.workflow_id = Some(id.into());
        self
    }
    
    /// Set command ID
    pub fn command_id(mut self, id: impl Into<String>) -> Self {
        self.command_id = id.into();
        self
    }
    
    /// Add party to act as
    pub fn act_as(mut self, party: PartyId) -> Self {
        self.act_as.push(party);
        self
    }
    
    /// Add parties to act as
    pub fn act_as_parties(mut self, parties: impl IntoIterator<Item = PartyId>) -> Self {
        self.act_as.extend(parties);
        self
    }
    
    /// Add party to read as
    pub fn read_as(mut self, party: PartyId) -> Self {
        self.read_as.push(party);
        self
    }
    
    /// Set deduplication period
    pub fn deduplication_period(mut self, period: DeduplicationPeriod) -> Self {
        self.deduplication_period = Some(period);
        self
    }
    
    /// Add create command
    pub fn create(
        mut self,
        template_id: Identifier,
        arguments: DamlRecord,
    ) -> Self {
        self.commands.push(Command::Create(CreateCommand {
            template_id,
            create_arguments: arguments,
        }));
        self
    }
    
    /// Add exercise command
    pub fn exercise(
        mut self,
        template_id: Identifier,
        contract_id: ContractId,
        choice: impl Into<String>,
        argument: DamlValue,
    ) -> Self {
        self.commands.push(Command::Exercise(ExerciseCommand {
            template_id,
            contract_id,
            choice: choice.into(),
            choice_argument: argument,
        }));
        self
    }
    
    /// Add exercise by key command
    pub fn exercise_by_key(
        mut self,
        template_id: Identifier,
        contract_key: DamlValue,
        choice: impl Into<String>,
        argument: DamlValue,
    ) -> Self {
        self.commands.push(Command::ExerciseByKey(ExerciseByKeyCommand {
            template_id,
            contract_key,
            choice: choice.into(),
            choice_argument: argument,
        }));
        self
    }
    
    /// Add create and exercise command
    pub fn create_and_exercise(
        mut self,
        template_id: Identifier,
        create_arguments: DamlRecord,
        choice: impl Into<String>,
        choice_argument: DamlValue,
    ) -> Self {
        self.commands.push(Command::CreateAndExercise(CreateAndExerciseCommand {
            template_id,
            create_arguments,
            choice: choice.into(),
            choice_argument,
        }));
        self
    }
    
    /// Build the commands
    pub fn build(self) -> Result<Commands, BuildError> {
        if self.act_as.is_empty() {
            return Err(BuildError::NoActAs);
        }
        if self.commands.is_empty() {
            return Err(BuildError::NoCommands);
        }
        
        Ok(Commands {
            workflow_id: self.workflow_id,
            application_id: self.application_id,
            command_id: self.command_id,
            act_as: self.act_as,
            read_as: self.read_as,
            commands: self.commands,
            deduplication_period: self.deduplication_period,
            min_ledger_time: self.min_ledger_time,
        })
    }
}

/// Deduplication period
#[derive(Debug, Clone)]
pub enum DeduplicationPeriod {
    /// Duration-based deduplication
    Duration(std::time::Duration),
    /// Offset-based deduplication
    Offset(String),
}

/// Minimum ledger time
#[derive(Debug, Clone)]
pub enum MinLedgerTime {
    /// Absolute timestamp
    Absolute(DateTime<Utc>),
    /// Relative duration
    Relative(std::time::Duration),
}

/// Command types
#[derive(Debug, Clone)]
pub enum Command {
    Create(CreateCommand),
    Exercise(ExerciseCommand),
    ExerciseByKey(ExerciseByKeyCommand),
    CreateAndExercise(CreateAndExerciseCommand),
}

#[derive(Debug, Clone)]
pub struct CreateCommand {
    pub template_id: Identifier,
    pub create_arguments: DamlRecord,
}

#[derive(Debug, Clone)]
pub struct ExerciseCommand {
    pub template_id: Identifier,
    pub contract_id: ContractId,
    pub choice: String,
    pub choice_argument: DamlValue,
}

#[derive(Debug, Clone)]
pub struct ExerciseByKeyCommand {
    pub template_id: Identifier,
    pub contract_key: DamlValue,
    pub choice: String,
    pub choice_argument: DamlValue,
}

#[derive(Debug, Clone)]
pub struct CreateAndExerciseCommand {
    pub template_id: Identifier,
    pub create_arguments: DamlRecord,
    pub choice: String,
    pub choice_argument: DamlValue,
}

/// Built commands
#[derive(Debug, Clone)]
pub struct Commands {
    pub workflow_id: Option<String>,
    pub application_id: String,
    pub command_id: String,
    pub act_as: Vec<PartyId>,
    pub read_as: Vec<PartyId>,
    pub commands: Vec<Command>,
    pub deduplication_period: Option<DeduplicationPeriod>,
    pub min_ledger_time: Option<MinLedgerTime>,
}
```

### 4.3 Transaction Filter

```rust
//! Transaction filtering

use crate::types::*;
use std::collections::HashMap;

/// Transaction filter builder
pub struct TransactionFilterBuilder {
    filters_by_party: HashMap<PartyId, PartyFilters>,
}

impl TransactionFilterBuilder {
    pub fn new() -> Self {
        Self {
            filters_by_party: HashMap::new(),
        }
    }
    
    /// Add filter for party
    pub fn for_party(mut self, party: PartyId) -> PartyFilterBuilder {
        PartyFilterBuilder {
            parent: self,
            party,
            templates: Vec::new(),
            interfaces: Vec::new(),
        }
    }
    
    /// Add wildcard filter for party (all templates)
    pub fn all_for_party(mut self, party: PartyId) -> Self {
        self.filters_by_party.insert(party, PartyFilters::Wildcard);
        self
    }
    
    /// Build the filter
    pub fn build(self) -> TransactionFilter {
        TransactionFilter {
            filters_by_party: self.filters_by_party,
        }
    }
}

pub struct PartyFilterBuilder {
    parent: TransactionFilterBuilder,
    party: PartyId,
    templates: Vec<Identifier>,
    interfaces: Vec<InterfaceFilter>,
}

impl PartyFilterBuilder {
    /// Add template filter
    pub fn template(mut self, template_id: Identifier) -> Self {
        self.templates.push(template_id);
        self
    }
    
    /// Add interface filter
    pub fn interface(
        mut self,
        interface_id: Identifier,
        include_created_event_blob: bool,
    ) -> Self {
        self.interfaces.push(InterfaceFilter {
            interface_id,
            include_created_event_blob,
        });
        self
    }
    
    /// Finish party filter and return to parent builder
    pub fn done(mut self) -> TransactionFilterBuilder {
        let filters = PartyFilters::Inclusive {
            template_ids: self.templates,
            interface_filters: self.interfaces,
        };
        self.parent.filters_by_party.insert(self.party, filters);
        self.parent
    }
}

/// Transaction filter
#[derive(Debug, Clone)]
pub struct TransactionFilter {
    pub filters_by_party: HashMap<PartyId, PartyFilters>,
}

/// Party-specific filters
#[derive(Debug, Clone)]
pub enum PartyFilters {
    /// Match all templates
    Wildcard,
    /// Match specific templates/interfaces
    Inclusive {
        template_ids: Vec<Identifier>,
        interface_filters: Vec<InterfaceFilter>,
    },
}

/// Interface filter
#[derive(Debug, Clone)]
pub struct InterfaceFilter {
    pub interface_id: Identifier,
    pub include_created_event_blob: bool,
}
```

### 4.4 Event Types

```rust
//! Ledger event types

use crate::types::*;
use chrono::{DateTime, Utc};

/// Ledger event
#[derive(Debug, Clone)]
pub enum Event {
    Created(CreatedEvent),
    Archived(ArchivedEvent),
}

/// Contract created event
#[derive(Debug, Clone)]
pub struct CreatedEvent {
    /// Event ID
    pub event_id: String,
    /// Contract ID
    pub contract_id: ContractId,
    /// Template ID
    pub template_id: Identifier,
    /// Contract key (if any)
    pub contract_key: Option<DamlValue>,
    /// Create arguments
    pub create_arguments: DamlRecord,
    /// Witness parties
    pub witness_parties: Vec<PartyId>,
    /// Signatories
    pub signatories: Vec<PartyId>,
    /// Observers
    pub observers: Vec<PartyId>,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
}

/// Contract archived event
#[derive(Debug, Clone)]
pub struct ArchivedEvent {
    /// Event ID
    pub event_id: String,
    /// Contract ID
    pub contract_id: ContractId,
    /// Template ID
    pub template_id: Identifier,
    /// Witness parties
    pub witness_parties: Vec<PartyId>,
}

/// Exercised event (in transaction trees)
#[derive(Debug, Clone)]
pub struct ExercisedEvent {
    /// Event ID
    pub event_id: String,
    /// Contract ID
    pub contract_id: ContractId,
    /// Template ID
    pub template_id: Identifier,
    /// Choice name
    pub choice: String,
    /// Choice argument
    pub choice_argument: DamlValue,
    /// Acting parties
    pub acting_parties: Vec<PartyId>,
    /// Whether the choice is consuming
    pub consuming: bool,
    /// Witness parties
    pub witness_parties: Vec<PartyId>,
    /// Child event IDs
    pub child_event_ids: Vec<String>,
    /// Exercise result
    pub exercise_result: DamlValue,
}

/// Flat transaction
#[derive(Debug, Clone)]
pub struct Transaction {
    /// Transaction ID
    pub transaction_id: String,
    /// Command ID
    pub command_id: String,
    /// Workflow ID
    pub workflow_id: String,
    /// Effective timestamp
    pub effective_at: DateTime<Utc>,
    /// Events
    pub events: Vec<Event>,
    /// Ledger offset
    pub offset: String,
}

/// Transaction tree
#[derive(Debug, Clone)]
pub struct TransactionTree {
    /// Transaction ID
    pub transaction_id: String,
    /// Command ID
    pub command_id: String,
    /// Workflow ID
    pub workflow_id: String,
    /// Effective timestamp
    pub effective_at: DateTime<Utc>,
    /// Events by ID
    pub events_by_id: HashMap<String, TreeEvent>,
    /// Root event IDs
    pub root_event_ids: Vec<String>,
    /// Ledger offset
    pub offset: String,
}

/// Tree event
#[derive(Debug, Clone)]
pub enum TreeEvent {
    Created(CreatedEvent),
    Exercised(ExercisedEvent),
}

/// Ledger offset
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LedgerOffset {
    /// Beginning of the ledger
    Begin,
    /// End of the ledger
    End,
    /// Absolute offset
    Absolute(String),
}
```

## 5. Client Implementation

### 5.1 Ledger Client

```rust
//! Main ledger client implementation

use crate::types::*;
use crate::services::*;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Main Canton ledger client
pub struct LedgerClient {
    /// gRPC channel
    channel: Channel,
    /// Ledger ID
    ledger_id: String,
    /// Command service
    command_service: CommandServiceClient,
    /// Transaction service
    transaction_service: TransactionServiceClient,
    /// Active contracts service
    active_contracts_service: ActiveContractsServiceClient,
    /// Party management service
    party_management_service: PartyManagementServiceClient,
    /// Package service
    package_service: PackageServiceClient,
    /// Configuration
    config: ClientConfig,
    /// Metrics
    metrics: Arc<ClientMetrics>,
}

impl LedgerClient {
    /// Create new client
    pub async fn connect(config: ClientConfig) -> SdkResult<Self> {
        let channel = Self::create_channel(&config).await?;
        let ledger_id = Self::get_ledger_id(&channel).await?;
        
        Ok(Self {
            channel: channel.clone(),
            ledger_id,
            command_service: CommandServiceClient::new(channel.clone()),
            transaction_service: TransactionServiceClient::new(channel.clone()),
            active_contracts_service: ActiveContractsServiceClient::new(channel.clone()),
            party_management_service: PartyManagementServiceClient::new(channel.clone()),
            package_service: PackageServiceClient::new(channel.clone()),
            config,
            metrics: Arc::new(ClientMetrics::new()),
        })
    }
    
    /// Get ledger ID
    pub fn ledger_id(&self) -> &str {
        &self.ledger_id
    }
    
    /// Submit command and wait for result
    pub async fn submit_and_wait(
        &self,
        commands: Commands,
    ) -> SdkResult<Transaction> {
        let start = std::time::Instant::now();
        
        let result = self.command_service
            .submit_and_wait_for_transaction(commands)
            .await;
        
        self.metrics.record_command(start.elapsed(), result.is_ok());
        
        result
    }
    
    /// Submit command asynchronously
    pub async fn submit(&self, commands: Commands) -> SdkResult<()> {
        self.command_service.submit(commands).await
    }
    
    /// Get transaction stream
    pub fn get_transactions(
        &self,
        begin: LedgerOffset,
        end: Option<LedgerOffset>,
        filter: TransactionFilter,
    ) -> impl Stream<Item = SdkResult<Transaction>> {
        self.transaction_service.get_transactions(
            &self.ledger_id,
            begin,
            end,
            filter,
        )
    }
    
    /// Get transaction trees stream
    pub fn get_transaction_trees(
        &self,
        begin: LedgerOffset,
        end: Option<LedgerOffset>,
        filter: TransactionFilter,
    ) -> impl Stream<Item = SdkResult<TransactionTree>> {
        self.transaction_service.get_transaction_trees(
            &self.ledger_id,
            begin,
            end,
            filter,
        )
    }
    
    /// Get active contracts
    pub async fn get_active_contracts(
        &self,
        filter: TransactionFilter,
    ) -> SdkResult<ActiveContractsResponse> {
        self.active_contracts_service
            .get_active_contracts(&self.ledger_id, filter)
            .await
    }
    
    /// Allocate party
    pub async fn allocate_party(
        &self,
        party_id_hint: Option<&str>,
        display_name: Option<&str>,
    ) -> SdkResult<PartyDetails> {
        self.party_management_service
            .allocate_party(party_id_hint, display_name)
            .await
    }
    
    /// List known parties
    pub async fn list_known_parties(&self) -> SdkResult<Vec<PartyDetails>> {
        self.party_management_service.list_known_parties().await
    }
    
    /// List packages
    pub async fn list_packages(&self) -> SdkResult<Vec<String>> {
        self.package_service.list_packages(&self.ledger_id).await
    }
    
    /// Get ledger end
    pub async fn get_ledger_end(&self) -> SdkResult<LedgerOffset> {
        self.transaction_service.get_ledger_end(&self.ledger_id).await
    }
    
    async fn create_channel(config: &ClientConfig) -> SdkResult<Channel> {
        let mut endpoint = Channel::from_shared(config.endpoint.clone())
            .map_err(|e| SdkError::Connection {
                message: format!("Invalid endpoint: {}", e),
                source: Some(Box::new(e)),
                backtrace: std::backtrace::Backtrace::capture(),
            })?;
        
        // Configure TLS
        if let Some(tls_config) = &config.tls {
            let tls = Self::create_tls_config(tls_config)?;
            endpoint = endpoint.tls_config(tls)
                .map_err(|e| SdkError::Connection {
                    message: format!("TLS configuration error: {}", e),
                    source: Some(Box::new(e)),
                    backtrace: std::backtrace::Backtrace::capture(),
                })?;
        }
        
        // Configure timeouts
        endpoint = endpoint
            .connect_timeout(config.connect_timeout)
            .timeout(config.request_timeout);
        
        // Connect
        endpoint.connect().await
            .map_err(|e| SdkError::Connection {
                message: format!("Failed to connect: {}", e),
                source: Some(Box::new(e)),
                backtrace: std::backtrace::Backtrace::capture(),
            })
    }
    
    async fn get_ledger_id(channel: &Channel) -> SdkResult<String> {
        let mut client = LedgerIdentityServiceClient::new(channel.clone());
        let response = client.get_ledger_identity(GetLedgerIdentityRequest {})
            .await
            .map_err(|e| SdkError::Connection {
                message: format!("Failed to get ledger identity: {}", e),
                source: Some(Box::new(e)),
                backtrace: std::backtrace::Backtrace::capture(),
            })?;
        
        Ok(response.into_inner().ledger_id)
    }
}
```

## 6. Streaming Patterns

### 6.1 Transaction Stream Handler

```rust
//! Transaction stream handling

use crate::types::*;
use futures::{Stream, StreamExt};
use tokio::sync::mpsc;

/// Transaction stream handler
pub struct TransactionStreamHandler {
    client: Arc<LedgerClient>,
    filter: TransactionFilter,
    offset_tracker: OffsetTracker,
}

impl TransactionStreamHandler {
    /// Create new handler
    pub fn new(
        client: Arc<LedgerClient>,
        filter: TransactionFilter,
    ) -> Self {
        Self {
            client,
            filter,
            offset_tracker: OffsetTracker::new(),
        }
    }
    
    /// Start processing transactions
    pub async fn process<F, Fut>(
        &mut self,
        mut handler: F,
    ) -> SdkResult<()>
    where
        F: FnMut(Transaction) -> Fut,
        Fut: Future<Output = SdkResult<()>>,
    {
        let begin = self.offset_tracker.get_offset();
        
        let mut stream = self.client.get_transactions(
            begin,
            None,
            self.filter.clone(),
        );
        
        while let Some(result) = stream.next().await {
            let transaction = result?;
            let offset = transaction.offset.clone();
            
            handler(transaction).await?;
            
            self.offset_tracker.update(offset);
        }
        
        Ok(())
    }
    
    /// Process with automatic reconnection
    pub async fn process_with_reconnect<F, Fut>(
        &mut self,
        handler: F,
        reconnect_config: ReconnectConfig,
    ) -> SdkResult<()>
    where
        F: FnMut(Transaction) -> Fut + Clone,
        Fut: Future<Output = SdkResult<()>>,
    {
        let mut attempts = 0;
        
        loop {
            match self.process(handler.clone()).await {
                Ok(()) => return Ok(()),
                Err(e) if e.is_retryable() && attempts < reconnect_config.max_attempts => {
                    attempts += 1;
                    let delay = reconnect_config.delay_for_attempt(attempts);
                    
                    tracing::warn!(
                        error = %e,
                        attempt = attempts,
                        delay = ?delay,
                        "Stream disconnected, reconnecting"
                    );
                    
                    tokio::time::sleep(delay).await;
                }
                Err(e) => return Err(e),
            }
        }
    }
}

/// Offset tracker for resumable streams
pub struct OffsetTracker {
    current_offset: RwLock<LedgerOffset>,
    persistence: Option<Box<dyn OffsetPersistence>>,
}

impl OffsetTracker {
    pub fn new() -> Self {
        Self {
            current_offset: RwLock::new(LedgerOffset::Begin),
            persistence: None,
        }
    }
    
    pub fn with_persistence(persistence: impl OffsetPersistence + 'static) -> Self {
        let offset = persistence.load().unwrap_or(LedgerOffset::Begin);
        Self {
            current_offset: RwLock::new(offset),
            persistence: Some(Box::new(persistence)),
        }
    }
    
    pub fn get_offset(&self) -> LedgerOffset {
        self.current_offset.read().unwrap().clone()
    }
    
    pub fn update(&self, offset: String) {
        let new_offset = LedgerOffset::Absolute(offset);
        *self.current_offset.write().unwrap() = new_offset.clone();
        
        if let Some(persistence) = &self.persistence {
            if let Err(e) = persistence.save(&new_offset) {
                tracing::error!(error = %e, "Failed to persist offset");
            }
        }
    }
}

/// Offset persistence trait
pub trait OffsetPersistence: Send + Sync {
    fn load(&self) -> SdkResult<LedgerOffset>;
    fn save(&self, offset: &LedgerOffset) -> SdkResult<()>;
}
```

## 7. Error Handling

### 7.1 gRPC Error Mapping

```rust
//! Error mapping from gRPC status codes

use tonic::Status;

impl From<Status> for SdkError {
    fn from(status: Status) -> Self {
        match status.code() {
            tonic::Code::NotFound => SdkError::Transaction {
                kind: TransactionErrorKind::ContractNotFound,
                transaction_id: None,
                source: Some(Box::new(status)),
            },
            tonic::Code::PermissionDenied => SdkError::Authentication {
                reason: status.message().to_string(),
                source: Some(Box::new(status)),
            },
            tonic::Code::InvalidArgument => SdkError::Transaction {
                kind: TransactionErrorKind::InvalidCommand,
                transaction_id: None,
                source: Some(Box::new(status)),
            },
            tonic::Code::Aborted => SdkError::Transaction {
                kind: TransactionErrorKind::Conflict,
                transaction_id: None,
                source: Some(Box::new(status)),
            },
            tonic::Code::DeadlineExceeded => SdkError::Timeout {
                duration: std::time::Duration::from_secs(0),
                operation: "gRPC call".to_string(),
            },
            tonic::Code::ResourceExhausted => SdkError::RateLimited {
                retry_after: None,
            },
            tonic::Code::Unavailable => SdkError::Connection {
                message: status.message().to_string(),
                source: Some(Box::new(status)),
                backtrace: std::backtrace::Backtrace::capture(),
            },
            _ => SdkError::Internal {
                message: format!("gRPC error: {}", status.message()),
                backtrace: std::backtrace::Backtrace::capture(),
            },
        }
    }
}

impl SdkError {
    /// Check if error is retryable
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            SdkError::Connection { .. }
                | SdkError::Timeout { .. }
                | SdkError::RateLimited { .. }
                | SdkError::Transaction {
                    kind: TransactionErrorKind::Conflict,
                    ..
                }
        )
    }
}
```

## 8. Macros for Ergonomics

### 8.1 Value Construction Macros

```rust
//! Macros for ergonomic value construction

/// Create a Daml record
#[macro_export]
macro_rules! daml_record {
    ($($field:ident : $value:expr),* $(,)?) => {
        $crate::types::DamlRecord {
            record_id: None,
            fields: vec![
                $(
                    $crate::types::RecordField {
                        label: stringify!($field).to_string(),
                        value: $crate::types::DamlValue::from($value),
                    }
                ),*
            ],
        }
    };
    
    ($record_id:expr => $($field:ident : $value:expr),* $(,)?) => {
        $crate::types::DamlRecord {
            record_id: Some($record_id),
            fields: vec![
                $(
                    $crate::types::RecordField {
                        label: stringify!($field).to_string(),
                        value: $crate::types::DamlValue::from($value),
                    }
                ),*
            ],
        }
    };
}

/// Create a Daml list
#[macro_export]
macro_rules! daml_list {
    ($($value:expr),* $(,)?) => {
        $crate::types::DamlValue::List(vec![
            $($crate::types::DamlValue::from($value)),*
        ])
    };
}

/// Create a Daml variant
#[macro_export]
macro_rules! daml_variant {
    ($constructor:ident) => {
        $crate::types::DamlVariant {
            variant_id: None,
            constructor: stringify!($constructor).to_string(),
            value: Box::new($crate::types::DamlValue::Unit),
        }
    };
    
    ($constructor:ident : $value:expr) => {
        $crate::types::DamlVariant {
            variant_id: None,
            constructor: stringify!($constructor).to_string(),
            value: Box::new($crate::types::DamlValue::from($value)),
        }
    };
}

// Usage example:
// let record = daml_record! {
//     owner: "party-123",
//     amount: 100i64,
//     description: "Test",
// };
```
