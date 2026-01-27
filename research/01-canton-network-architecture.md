# Canton Network Architecture Research

## 1. Overview

Canton Network is a privacy-enabled interoperable blockchain network designed for institutional use cases. It is built on the Daml smart contract language and provides a unique approach to distributed ledger technology.

## 2. Core Architecture Components

### 2.1 Canton Protocol

Canton is the underlying protocol that enables:
- **Privacy by design**: Transactions are only visible to involved parties
- **Interoperability**: Different Canton-based applications can interact seamlessly
- **Composability**: Smart contracts can be composed across different domains

### 2.2 Key Architectural Elements

```
┌─────────────────────────────────────────────────────────────────┐
│                     Canton Network                               │
├─────────────────────────────────────────────────────────────────┤
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐              │
│  │  Domain 1   │  │  Domain 2   │  │  Domain N   │              │
│  │  (Sequencer)│  │  (Sequencer)│  │  (Sequencer)│              │
│  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘              │
│         │                │                │                      │
│  ┌──────┴──────┐  ┌──────┴──────┐  ┌──────┴──────┐              │
│  │ Participant │  │ Participant │  │ Participant │              │
│  │   Node 1    │  │   Node 2    │  │   Node N    │              │
│  └─────────────┘  └─────────────┘  └─────────────┘              │
│                                                                  │
│  ┌─────────────────────────────────────────────────────────────┐│
│  │                    Ledger API (gRPC)                        ││
│  └─────────────────────────────────────────────────────────────┘│
│                                                                  │
│  ┌─────────────────────────────────────────────────────────────┐│
│  │                    Daml Runtime                             ││
│  └─────────────────────────────────────────────────────────────┘│
└─────────────────────────────────────────────────────────────────┘
```

### 2.3 Domain Architecture

A **Domain** in Canton consists of:
- **Sequencer**: Orders transactions and provides global ordering
- **Mediator**: Validates transaction confirmations
- **Domain Manager**: Manages domain topology and identity

### 2.4 Participant Node

A **Participant Node** is the entry point for applications:
- Hosts Daml applications
- Manages local state
- Communicates with domains
- Exposes Ledger API

## 3. Canton Protocol Details

### 3.1 Transaction Processing Flow

```
1. Command Submission
   └── Client submits command via Ledger API
   
2. Transaction Preparation
   └── Participant prepares transaction
   └── Computes views for each stakeholder
   
3. Confirmation Request
   └── Participant sends to Sequencer
   └── Sequencer orders and distributes
   
4. Confirmation Response
   └── Stakeholders validate and respond
   └── Mediator collects responses
   
5. Result Distribution
   └── Mediator computes result
   └── Sequencer distributes final result
   
6. Commitment
   └── Participants commit to local ledger
```

### 3.2 Privacy Model

Canton implements **sub-transaction privacy**:
- Each party sees only their relevant portion
- Cryptographic commitments ensure integrity
- Zero-knowledge proofs for validation

### 3.3 Synchronization Protocol

```rust
// Conceptual representation of Canton sync protocol
pub struct SyncProtocol {
    // Domain time synchronization
    domain_time: DomainTime,
    // Sequencer counter for ordering
    sequencer_counter: SequencerCounter,
    // Participant's local view
    local_ledger_state: LedgerState,
}

pub struct DomainTime {
    timestamp: Timestamp,
    domain_id: DomainId,
}

pub struct SequencerCounter(u64);
```

## 4. Ledger API Specification

### 4.1 Core Services

| Service | Description | gRPC Methods |
|---------|-------------|--------------|
| CommandService | Submit commands | SubmitAndWait, SubmitAndWaitForTransaction |
| CommandSubmissionService | Async command submission | Submit |
| CommandCompletionService | Track command completion | CompletionStream |
| TransactionService | Read transactions | GetTransactions, GetTransactionTrees |
| ActiveContractsService | Get active contracts | GetActiveContracts |
| LedgerIdentityService | Get ledger identity | GetLedgerIdentity |
| PackageService | Manage Daml packages | ListPackages, GetPackage, UploadDarFile |
| PartyManagementService | Manage parties | AllocateParty, ListKnownParties |
| LedgerConfigurationService | Get ledger config | GetLedgerConfiguration |

### 4.2 gRPC Protocol Buffers

```protobuf
// Key message types from Ledger API

message Command {
  oneof command {
    CreateCommand create = 1;
    ExerciseCommand exercise = 2;
    ExerciseByKeyCommand exerciseByKey = 3;
    CreateAndExerciseCommand createAndExercise = 4;
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

message Transaction {
  string transaction_id = 1;
  string command_id = 2;
  string workflow_id = 3;
  google.protobuf.Timestamp effective_at = 4;
  repeated Event events = 5;
  string offset = 6;
}

message Event {
  oneof event {
    CreatedEvent created = 1;
    ArchivedEvent archived = 2;
    ExercisedEvent exercised = 3;
  }
}
```

## 5. Canton-Specific Extensions

### 5.1 Admin API

Canton provides additional Admin API for:
- Domain management
- Participant configuration
- Health monitoring
- Pruning operations

### 5.2 Topology Management

```protobuf
// Canton topology transactions
message TopologyTransaction {
  TopologyMapping mapping = 1;
  TopologyChangeOp operation = 2;
  bytes signature = 3;
}

enum TopologyChangeOp {
  ADD = 0;
  REMOVE = 1;
  REPLACE = 2;
}

message TopologyMapping {
  oneof mapping {
    NamespaceDelegation namespace_delegation = 1;
    IdentifierDelegation identifier_delegation = 2;
    OwnerToKeyMapping owner_to_key_mapping = 3;
    PartyToParticipant party_to_participant = 4;
    DomainParametersChange domain_parameters_change = 5;
  }
}
```

### 5.3 Multi-Domain Support

Canton enables cross-domain transactions:
- Atomic transactions across domains
- Domain routing
- Automatic domain selection

## 6. Security Model

### 6.1 Identity and Authentication

- **X.509 Certificates**: PKI-based identity
- **Namespace Hierarchy**: Hierarchical identity management
- **Key Management**: Multiple key types (signing, encryption)

### 6.2 Cryptographic Primitives

| Primitive | Algorithm | Usage |
|-----------|-----------|-------|
| Signing | Ed25519, ECDSA P-256 | Transaction signing |
| Hashing | SHA-256, Blake2b | Content addressing |
| Encryption | AES-GCM | Data encryption |
| Key Exchange | X25519 | Secure communication |

### 6.3 Authorization Model

```rust
// Conceptual authorization model
pub enum Permission {
    CanActAs(PartyId),
    CanReadAs(PartyId),
    CanSubmit,
    CanAllocateParty,
    CanUploadPackages,
}

pub struct AccessToken {
    pub user_id: UserId,
    pub permissions: Vec<Permission>,
    pub expiry: Timestamp,
}
```

## 7. Network Topology

### 7.1 Global Synchronizer

The Canton Network uses a **Global Synchronizer** for:
- Cross-domain coordination
- Global time synchronization
- Network-wide identity management

### 7.2 Super Validator Nodes

Super Validators provide:
- Network governance
- Domain hosting
- Transaction validation

## 8. Integration Points for SDK

### 8.1 Primary Integration: Ledger API

```
SDK Client
    │
    ├── gRPC Connection (TLS)
    │       │
    │       └── Ledger API Services
    │               ├── Command Submission
    │               ├── Transaction Streaming
    │               └── Active Contracts Query
    │
    └── Authentication
            ├── JWT Tokens
            └── mTLS Certificates
```

### 8.2 Secondary Integration: Admin API

For advanced operations:
- Health checks
- Metrics collection
- Configuration management

### 8.3 Event Streaming

```rust
// Event streaming model
pub trait TransactionStream {
    type Item = Transaction;
    type Error = LedgerError;
    
    fn subscribe(
        &self,
        begin: LedgerOffset,
        end: Option<LedgerOffset>,
        filter: TransactionFilter,
    ) -> impl Stream<Item = Result<Self::Item, Self::Error>>;
}
```

## 9. Performance Characteristics

### 9.1 Throughput

- Single domain: 1000+ TPS
- Multi-domain: Depends on coordination overhead
- Latency: Sub-second for simple transactions

### 9.2 Scalability

- Horizontal scaling via multiple domains
- Participant node clustering
- Read replicas for query scaling

## 10. SDK Requirements Derived from Architecture

Based on Canton architecture, the SDK must support:

1. **gRPC Client Implementation**
   - Full Ledger API coverage
   - Streaming support
   - Connection pooling

2. **Transaction Building**
   - Command construction
   - Daml value serialization
   - Template/choice identification

3. **Event Processing**
   - Transaction stream handling
   - Event filtering
   - Offset management

4. **Identity Management**
   - Party allocation
   - Key management
   - Certificate handling

5. **Multi-Domain Support**
   - Domain discovery
   - Cross-domain transactions
   - Domain routing

6. **Error Handling**
   - Retry logic
   - Conflict resolution
   - Timeout management

## References

- Canton Protocol Specification
- Daml Ledger API Documentation
- Canton Network Documentation
- Digital Asset Developer Resources
