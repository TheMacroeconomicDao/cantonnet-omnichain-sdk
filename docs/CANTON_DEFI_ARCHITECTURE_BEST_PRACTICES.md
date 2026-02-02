# Canton DeFi Ecosystem - Architecture & Best Practices Documentation

**By Gybernaty Community** 🌐

A comprehensive decentralized finance platform built on Canton Network, providing institutional-grade financial services including treasury bills, real estate tokenization, privacy vaults, and compliance integration.

---

## 📋 Table of Contents

1. [Overview](#overview)
2. [System Architecture](#system-architecture)
3. [SDK Architecture](#sdk-architecture)
4. [DeFi Platform Components](#defi-platform-components)
5. [Blockchain Infrastructure](#blockchain-infrastructure)
6. [Data Flows](#data-flows)
7. [Security Architecture](#security-architecture)
8. [Best Practices](#best-practices)
9. [Deployment Patterns](#deployment-patterns)
10. [Monitoring & Observability](#monitoring--observability)
11. [Testing Strategy](#testing-strategy)
12. [Performance Optimization](#performance-optimization)

---

## 🎯 Overview

### Project Description

Canton DeFi Ecosystem is a comprehensive decentralized finance platform built on the Canton Network, providing institutional-grade financial services including:

- **Treasury Bills**: Tokenized government securities
- **Real Estate**: Fractional property ownership
- **Privacy Vaults**: Zero-knowledge proof-based private asset storage
- **Compliance**: Integrated KYC/AML and regulatory compliance
- **Oracle Services**: Real-time price feeds and market data

### Technology Stack

| Layer | Technology | Purpose |
|-------|-----------|---------|
| **Frontend** | Next.js 14, TypeScript, React | Web application UI |
| **Backend API** | Next.js API Routes | RESTful API endpoints |
| **SDK** | Rust (cantonnet-omnichain-sdk) | Ledger API client & domain services |
| **Blockchain** | Canton Network 0.5.3 | Distributed ledger |
| **Database** | PostgreSQL 14 | Contract and state storage |
| **Infrastructure** | Kubernetes, Docker | Container orchestration |
| **Authentication** | Supabase OIDC | User authentication |
| **Monitoring** | Prometheus, Grafana | Metrics and observability |

### Network Environments

| Network | IP Address | API Endpoint | Status |
|---------|------------|--------------|--------|
| **DevNet** | 65.108.15.30 | https://sv.sv-1.dev.global.canton.network.sync.global | ✅ Active |
| **TestNet** | 65.108.15.20 | https://sv.sv-1.testnet.global.canton.network.sync.global | ⏳ Partial |
| **MainNet** | 65.108.15.19 | https://sv.sv-1.global.canton.network.sync.global | ✅ Validated |

---

## 🏗️ System Architecture

### High-Level Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                         USERS                                  │
└────────────────────────────┬────────────────────────────────────┘
                             │
┌────────────────────────────▼────────────────────────────────────┐
│                    FRONTEND LAYER                               │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐        │
│  │ Treasury     │  │ Real Estate  │  │ Privacy      │        │
│  │ Bills Panel  │  │ Panel        │  │ Vaults Panel │        │
│  └──────────────┘  └──────────────┘  └──────────────┘        │
└────────────────────────────┬────────────────────────────────────┘
                             │
┌────────────────────────────▼────────────────────────────────────┐
│                    API LAYER (Next.js)                           │
│  /api/defi/treasury/*  /api/defi/realestate/*                 │
│  /api/defi/privacy/*   /api/defi/compliance/*                  │
│  /api/defi/oracle/*    /api/defi/auth/*                        │
└────────────────────────────┬────────────────────────────────────┘
                             │
┌────────────────────────────▼────────────────────────────────────┐
│                 BUSINESS LOGIC LAYER                            │
│  TreasuryBillsService  RealEstateService  PrivacyVaultService  │
│  DamlIntegrationService  ComplianceService  OracleService      │
└────────────────────────────┬────────────────────────────────────┘
                             │
┌────────────────────────────▼────────────────────────────────────┐
│                    SDK LAYER (Rust)                             │
│  CantonClient  TreasuryService  RealEstateService  Privacy     │
│  LedgerClient  Conversion  Transport  Config                    │
└────────────────────────────┬────────────────────────────────────┘
                             │
┌────────────────────────────▼────────────────────────────────────┐
│              BLOCKCHAIN INFRASTRUCTURE                         │
│  Canton Participant  Canton Validator  PostgreSQL  Nginx        │
└────────────────────────────┬────────────────────────────────────┘
                             │
┌────────────────────────────▼────────────────────────────────────┐
│                 CANTON NETWORK                                  │
│  DevNet  TestNet  MainNet  Super Validators  Sequencers         │
└─────────────────────────────────────────────────────────────────┘
```

### Layer Responsibilities

#### 1. Frontend Layer
- **Responsibility**: User interface and interaction
- **Components**: React components, hooks, state management
- **Communication**: API calls to Next.js routes
- **Key Files**: 
  - `src/app/defi/page.tsx`
  - `src/components/defi/*.tsx`
  - `src/lib/canton/hooks/*.ts`

#### 2. API Layer
- **Responsibility**: HTTP request handling and routing
- **Components**: Next.js API routes, middleware
- **Communication**: Business logic services
- **Key Files**:
  - `src/app/api/defi/treasury/*/route.ts`
  - `src/app/api/defi/realestate/*/route.ts`
  - `src/app/api/defi/privacy/*/route.ts`

#### 3. Business Logic Layer
- **Responsibility**: Domain logic and orchestration
- **Components**: Service classes, business rules
- **Communication**: SDK or direct Canton API
- **Key Files**:
  - `src/lib/canton/services/treasuryBillsService.ts`
  - `src/lib/canton/services/realEstateService.ts`
  - `src/lib/canton/services/privacyVaultService.ts`

#### 4. SDK Layer
- **Responsibility**: Ledger API abstraction and domain services
- **Components**: Rust crates, type-safe API
- **Communication**: gRPC to Canton Participant
- **Key Crates**:
  - `canton-core`
  - `canton-ledger-api`
  - `canton-wallet`
  - `canton-crypto`

#### 5. Blockchain Infrastructure
- **Responsibility**: Ledger operations and consensus
- **Components**: Canton nodes, database, reverse proxy
- **Communication**: P2P network, gRPC
- **Key Services**:
  - Canton Participant (Ledger API)
  - Canton Validator (Consensus)
  - PostgreSQL (Storage)

#### 6. Canton Network
- **Responsibility**: Global distributed ledger
- **Components**: Super Validators, Sequencers, Mediators
- **Communication**: P2P protocol
- **Networks**: DevNet, TestNet, MainNet

---

## 🔧 SDK Architecture

### Crate Structure

```
cantonnet-omnichain-sdk/
├── canton-core/              # Core types and configuration
│   ├── src/
│   │   ├── types/            # Commands, DamlValue, ContractId
│   │   ├── error.rs          # SdkError, SdkResult
│   │   ├── config.rs         # LedgerApiConfig
│   │   └── domain/           # Domain types (Treasury, RealEstate)
│   └── Cargo.toml
│
├── canton-ledger-api/        # Ledger API client
│   ├── src/
│   │   ├── client.rs         # LedgerClient
│   │   ├── conversion.rs     # Domain ↔ Proto conversion
│   │   ├── services/         # Ledger API services
│   │   │   ├── command_submission.rs
│   │   │   ├── state_service.rs
│   │   │   ├── active_contracts.rs
│   │   │   └── completion_stream.rs
│   │   └── proto/           # Generated protobuf code
│   └── Cargo.toml
│
├── canton-wallet/            # Wallet functionality
│   ├── src/
│   │   ├── wallet.rs        # Wallet trait
│   │   ├── keystore.rs      # KeyStore implementation
│   │   └── signing.rs      # Signing operations
│   └── Cargo.toml
│
├── canton-crypto/            # Cryptographic operations
│   ├── src/
│   │   ├── keys.rs          # Key types
│   │   ├── signature.rs     # Signature operations
│   │   └── encryption.rs    # Encryption utilities
│   └── Cargo.toml
│
├── canton-transport/         # Transport layer
│   ├── src/
│   │   ├── channel.rs       # gRPC channel builder
│   │   ├── tls.rs           # TLS configuration
│   │   └── timeout.rs       # Timeout handling
│   └── Cargo.toml
│
├── canton-reliability/       # Reliability patterns
│   ├── src/
│   │   ├── retry.rs         # Retry logic
│   │   ├── circuit_breaker.rs
│   │   └── backoff.rs       # Backoff strategies
│   └── Cargo.toml
│
├── canton-observability/     # Observability
│   ├── src/
│   │   ├── logging.rs       # Logging utilities
│   │   ├── metrics.rs       # Metrics collection
│   │   └── tracing.rs       # Distributed tracing
│   └── Cargo.toml
│
├── config/                   # Configuration files
│   ├── example.yaml
│   ├── example-production.yaml
│   └── local.yaml
│
└── docs/                     # Documentation
    ├── DEVNET_PARTICIPANT.md
    ├── SDK_ARCHITECTURE.md
    └── API_REFERENCE.md
```

### Core Types

#### Commands
```rust
pub enum Command {
    Create(CreateCommand),
    Exercise(ExerciseCommand),
    CreateAndExercise(CreateAndExerciseCommand),
    ExerciseByKey(ExerciseByKeyCommand),
}

pub struct CreateCommand {
    pub template_id: Identifier,
    pub create_arguments: DamlRecord,
}

pub struct ExerciseCommand {
    pub contract_id: ContractId,
    pub choice: String,
    pub choice_argument: DamlValue,
}
```

#### DamlValue
```rust
pub enum DamlValue {
    Record(DamlRecord),
    Variant(DamlVariant),
    List(Vec<DamlValue>),
    Optional(Option<Box<DamlValue>>),
    Text(String),
    Int64(i64),
    Numeric(BigDecimal),
    Party(PartyId),
    ContractId(ContractId),
    Timestamp(Timestamp),
    Date(Date),
    // ... more types
}
```

#### Domain Types
```rust
// Treasury Bills
pub struct TreasuryBill {
    pub bill_id: String,
    pub name: String,
    pub symbol: String,
    pub issuer: String,
    pub custodian: String,
    pub maturity: Maturity,
    pub maturity_date: DateTime<Utc>,
    pub total_supply: Decimal,
    pub available_supply: Decimal,
    pub price_per_token: Decimal,
    pub minimum_investment: Decimal,
    pub current_yield: Decimal,
    pub expected_yield: Decimal,
    pub yield_to_maturity: Decimal,
    pub status: BillStatus,
    pub contract_id: Option<ContractId>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// Real Estate
pub struct PropertyInfo {
    pub id: String,
    pub name: String,
    pub address: String,
    pub property_type: PropertyType,
    pub total_value: Decimal,
    pub token_supply: u64,
    pub available_supply: u64,
    pub price_per_token: Decimal,
    pub expected_dividend_yield: Decimal,
    pub location: Location,
    pub property_manager: String,
    pub legal_structure: String,
    pub jurisdiction: String,
    pub status: PropertyStatus,
    // ... more fields
}

// Privacy Vaults
pub struct PrivacyVault {
    pub id: String,
    pub name: String,
    pub description: String,
    pub owner: PartyId,
    pub custodian: PartyId,
    pub privacy_level: PrivacyLevel,
    pub encryption_standard: EncryptionStandard,
    pub zk_proof_protocol: ZKProofProtocol,
    pub total_value: Decimal,
    pub asset_count: u64,
    pub multi_sig_threshold: u8,
    pub timelock: Option<Duration>,
    pub status: VaultStatus,
    // ... more fields
}
```

### LedgerClient API

#### Connection
```rust
use canton_ledger_api::LedgerClient;

// Connect from config
let config = LedgerApiConfig::from_yaml("config/example.yaml")?;
let mut client = LedgerClient::connect_from_config(&config).await?;

// Or connect directly
let mut client = LedgerClient::connect(
    "http://65.108.15.30:30501",
    "participant"
).await?;
```

#### Contract Operations
```rust
// Create contract
let contract_id = client.create_contract(
    &template_id,
    create_arguments
).await?;

// Exercise choice
client.exercise(
    &contract_id,
    "Approve",
    choice_argument
).await?;

// Query active contracts
let contracts = client.get_active_contracts(
    &template_id,
    Some(filter)
).await?;
```

#### Domain Services
```rust
use canton_ledger_api::services::{TreasuryService, RealEstateService};

// Treasury operations
let treasury = TreasuryService::new(client.clone(), template_ids);
let bill = treasury.create_bill(CreateBillInput {
    name: "US Treasury 1Y".to_string(),
    symbol: "UST1Y".to_string(),
    total_supply: dec!(1000000),
    price_per_token: dec!(0.99),
    // ... more fields
}).await?;

let portfolio = treasury.get_portfolio_summary(&investor_id).await?;

// Real Estate operations
let real_estate = RealEstateService::new(client.clone(), template_ids);
let properties = real_estate.list_properties(None).await?;
let holding = real_estate.create_purchase_request(
    &property_id,
    &investor_id,
    100,
    payment_data
).await?;
```

---

## 💼 DeFi Platform Components

### Treasury Bills Module

#### Features
- **Bill Creation**: Create tokenized treasury bills
- **Purchase Requests**: Submit purchase requests with compliance checks
- **Portfolio Management**: Track holdings and portfolio performance
- **Yield Distribution**: Automatic yield distribution to holders
- **Maturity Handling**: Automatic maturity processing

#### Data Model
```typescript
interface TreasuryBill {
  billId: string;
  name: string;
  symbol: string;
  description: string;
  issuer: string;
  custodian: string;
  maturity: Maturity;
  maturityDate: Date;
  issueDate: Date;
  totalSupply: Decimal;
  availableSupply: Decimal;
  pricePerToken: Decimal;
  minimumInvestment: Decimal;
  currentYield: Decimal;
  expectedYield: Decimal;
  yieldToMaturity: Decimal;
  status: BillStatus;
  contractId?: string;
  createdAt: Date;
  updatedAt: Date;
}

interface TreasuryBillHolding {
  holdingId: string;
  billId: string;
  investor: string;
  tokensOwned: Decimal;
  averageCostBasis: Decimal;
  currentMarketValue: Decimal;
  unrealizedGainLoss: Decimal;
  unrealizedGainLossPercent: Decimal;
  purchaseDate: Date;
  purchasePrice: Decimal;
  accumulatedYield: Decimal;
  lastYieldDistribution: Date;
  status: HoldingStatus;
  contractId?: string;
  createdAt: Date;
  updatedAt: Date;
}

interface PurchaseRequest {
  requestId: string;
  billId: string;
  investor: string;
  numberOfTokens: Decimal;
  totalAmount: Decimal;
  paymentMethod: PaymentMethod;
  status: RequestStatus;
  kycLevel: KYCLevel;
  complianceCheck: ComplianceCheck;
  requestDate: Date;
  expiryDate: Date;
  completedAt?: Date;
  contractId?: string;
  createdAt: Date;
  updatedAt: Date;
}
```

#### API Endpoints
```
GET    /api/defi/treasury/bills
POST   /api/defi/treasury/bills
GET    /api/defi/treasury/bills/:billId
PUT    /api/defi/treasury/bills/:billId
DELETE /api/defi/treasury/bills/:billId

GET    /api/defi/treasury/portfolio?investor=:investorId
GET    /api/defi/treasury/holdings?investor=:investorId

POST   /api/defi/treasury/purchases
GET    /api/defi/treasury/purchases?investor=:investorId
PUT    /api/defi/treasury/purchases/:requestId/approve
```

### Real Estate Module

#### Features
- **Property Tokenization**: Fractional ownership of real estate
- **Purchase Requests**: Buy property tokens with compliance
- **Governance**: Vote on property management decisions
- **Dividend Distribution**: Rental income distribution
- **Property Management**: Track property status and performance

#### Data Model
```typescript
interface PropertyInfo {
  id: string;
  name: string;
  address: string;
  type: PropertyType;
  subType: PropertySubType;
  totalValue: Decimal;
  tokenSupply: number;
  availableSupply: number;
  pricePerToken: Decimal;
  minimumInvestment: Decimal;
  expectedDividendYield: Decimal;
  historicalReturns: HistoricalReturns;
  occupancyRate: Decimal;
  location: Location;
  propertyManager: string;
  legalStructure: string;
  jurisdiction: string;
  regulatoryStatus: RegulatoryStatus;
  complianceLevel: ComplianceLevel;
  images: string[];
  documents: Document[];
  status: PropertyStatus;
  fundingProgress: FundingProgress;
  // ... more fields
}

interface TokenPurchaseRequest {
  propertyId: string;
  investorAddress: string;
  numberOfTokens: number;
  totalAmount: Decimal;
  paymentMethod: PaymentMethod;
  kycLevel: KYCLevel;
  accreditedInvestor: boolean;
  investorCountry: string;
  privacyLevel: PrivacyLevel;
  zkProofRequired: boolean;
}

interface GovernanceProposal {
  proposalId: string;
  propertyId: string;
  proposalType: ProposalType;
  title: string;
  description: string;
  proposer: string;
  createdAt: Date;
  votingDeadline: Date;
  votesFor: number;
  votesAgainst: number;
  votesAbstain: number;
  status: ProposalStatus;
}
```

#### API Endpoints
```
GET    /api/defi/realestate/properties
GET    /api/defi/realestate/properties/:propertyId
POST   /api/defi/realestate/purchases
GET    /api/defi/realestate/holdings?investor=:investorId
GET    /api/defi/realestate/governance?propertyId=:propertyId
POST   /api/defi/realestate/governance/:proposalId/vote
```

### Privacy Vaults Module

#### Features
- **Vault Creation**: Create privacy vaults with configurable privacy levels
- **Asset Deposit**: Deposit assets with zero-knowledge proofs
- **Asset Withdrawal**: Withdraw assets with proof verification
- **Proof Generation**: Generate ownership, balance, and compliance proofs
- **Selective Disclosure**: Reveal specific information without full disclosure

#### Data Model
```typescript
interface PrivacyVault {
  id: string;
  name: string;
  description: string;
  owner: string;
  custodian: string;
  authorizedViewers: string[];
  trustees: string[];
  privacyLevel: PrivacyLevel;
  encryptionStandard: EncryptionStandard;
  zkProofProtocol: ZKProofProtocol;
  anonymitySet: number;
  complianceLevel: ComplianceLevel;
  jurisdiction: string;
  totalValue: Decimal;
  assetCount: number;
  multiSigThreshold: number;
  timelock?: Duration;
  status: VaultStatus;
  encryptedMetadata: string;
  metadataHash: string;
}

interface PrivateAsset {
  vaultId: string;
  type: AssetType;
  encryptedValue: string;
  encryptedMetadata: string;
  zkProofs: ZKProof[];
  commitments: Commitment[];
  nullifiers: Nullifier[];
  accessLevel: AccessLevel;
  complianceProofs: ComplianceProof[];
  auditTrail: AuditEvent[];
  status: AssetStatus;
}

interface ZKProof {
  proofType: ProofType;
  proof: string;
  publicInputs: any[];
  verificationKey: string;
  timestamp: Date;
}
```

#### API Endpoints
```
POST   /api/defi/privacy/vaults
GET    /api/defi/privacy/vaults?owner=:ownerId
GET    /api/defi/privacy/vaults/:vaultId/assets
POST   /api/defi/privacy/vaults/:vaultId/deposit
POST   /api/defi/privacy/vaults/:vaultId/withdraw
POST   /api/defi/privacy/vaults/:vaultId/proofs
GET    /api/defi/privacy/vaults/:vaultId/compliance
```

### Compliance Module

#### Features
- **KYC Verification**: Multi-level KYC verification
- **AML Screening**: Anti-money laundering checks
- **Sanctions Screening**: OFAC and other sanctions lists
- **Transaction Validation**: Real-time transaction compliance
- **Audit Trail**: Complete audit logging

#### Data Model
```typescript
interface KYCVerification {
  verificationId: string;
  userId: string;
  personalInfo: PersonalInfo;
  targetLevel: KYCLevel;
  currentLevel: KYCLevel;
  status: VerificationStatus;
  documents: KYCDocument[];
  checks: ComplianceCheck[];
  verifiedAt?: Date;
  expiresAt?: Date;
  createdAt: Date;
  updatedAt: Date;
}

interface AMLCheck {
  checkId: string;
  transactionId: string;
  investor: string;
  amount: Decimal;
  assetType: AssetType;
  walletAddress: string;
  riskScore: number;
  riskLevel: RiskLevel;
  alerts: AMLAlert[];
  status: CheckStatus;
  checkedAt: Date;
}

interface ComplianceCheck {
  checkType: CheckType;
  result: ComplianceResult;
  reasons: string[];
  timestamp: Date;
}
```

#### API Endpoints
```
POST   /api/defi/compliance/kyc
GET    /api/defi/compliance/kyc?userId=:userId
POST   /api/defi/compliance/validate
GET    /api/defi/compliance/sanctions?wallet=:walletAddress
```

### Oracle Module

#### Features
- **Price Feeds**: Real-time asset prices
- **Treasury Yields**: Current treasury yields by maturity
- **Market Indices**: Major market indices
- **Property Valuations**: Real estate property valuations
- **Historical Data**: Historical price and yield data

#### Data Model
```typescript
interface PriceData {
  symbol: string;
  price: Decimal;
  currency: string;
  timestamp: Date;
  source: OracleSource;
}

interface TreasuryYield {
  maturity: Maturity;
  yield: Decimal;
  timestamp: Date;
  source: OracleSource;
}

interface MarketIndex {
  indexName: string;
  value: Decimal;
  change: Decimal;
  changePercent: Decimal;
  timestamp: Date;
}

interface PropertyValuation {
  propertyId: string;
  valuation: Decimal;
  valuationDate: Date;
  methodology: ValuationMethodology;
  confidence: number;
}
```

#### API Endpoints
```
GET    /api/defi/oracle/prices?symbol=:symbol
GET    /api/defi/oracle/prices?symbols=:symbols
GET    /api/defi/oracle/treasury-yields?maturity=:maturity
GET    /api/defi/oracle/treasury-yields
GET    /api/defi/oracle/indices
GET    /api/defi/oracle/property/:propertyId/valuation
```

---

## 🔗 Blockchain Infrastructure

### Canton Participant

#### Role
- **Ledger API**: Exposes Ledger API v2 for contract operations
- **State Management**: Maintains participant state
- **Transaction Submission**: Submits transactions to the network
- **Event Streaming**: Streams contract events to subscribers

#### Configuration
```yaml
canton-participant:
  image: ghcr.io/digital-asset/decentralized-canton-sync/docker/canton-participant:0.4.19
  ports:
    - 4001: ledger-api (gRPC)
    - 4002: admin-api
  environment:
    CANTON_PARTICIPANT_POSTGRES_SERVER: postgres-splice
    CANTON_PARTICIPANT_POSTGRES_PORT: 5432
    CANTON_PARTICIPANT_POSTGRES_USER: canton
    CANTON_PARTICIPANT_POSTGRES_PASSWORD: ${DB_PASSWORD}
    CANTON_PARTICIPANT_POSTGRES_DB: participant-0
    SPLICE_APP_PARTICIPANT_AUTH_AUDIENCE: authenticated
    AUTH_URL: http://65.108.15.30:32233
    AUTH_JWKS_URL: http://65.108.15.30:32233/jwks
```

#### Endpoints
- **Ledger API (gRPC)**: `65.108.15.30:30501` (NodePort)
- **Ledger API (HTTP)**: `http://65.108.15.30:30757` (NodePort)
- **Admin API**: `http://65.108.15.30:4002`

### Canton Validator

#### Role
- **Consensus**: Participates in BFT consensus
- **Block Production**: Produces blocks when selected
- **Validation**: Validates blocks and transactions
- **Staking**: Manages validator stake

#### Configuration
```yaml
validator-app:
  image: ghcr.io/digital-asset/decentralized-canton-sync/docker/validator-app:0.4.19
  ports:
    - 5003: validator-api
    - 5004: validator-admin
  environment:
    SPLICE_APP_VALIDATOR_PARTICIPANT_ADDRESS: participant
    SPLICE_APP_VALIDATOR_PARTICIPANT_IDENTIFIER: gyber-validator
    SPLICE_APP_VALIDATOR_PARTY_HINT: gyber-validator
    SPLICE_APP_VALIDATOR_SV_SPONSOR_ADDRESS: https://sv.sv-1.dev.global.canton.network.sync.global
    SPLICE_APP_VALIDATOR_ONBOARDING_SECRET: ${ONBOARDING_SECRET}
    MIGRATION_ID: 0
```

#### Endpoints
- **Validator API**: `http://65.108.15.30:30503` (NodePort)
- **Validator Admin**: `http://65.108.15.30:30504` (NodePort)

### PostgreSQL

#### Role
- **Contract Storage**: Stores contract data
- **State Storage**: Maintains participant state
- **Transaction Log**: Logs all transactions
- **Indexing**: Provides indexed queries

#### Configuration
```yaml
postgres-splice:
  image: postgres:14
  ports:
    - 5432: postgres
  environment:
    POSTGRES_USER: canton
    POSTGRES_PASSWORD: ${DB_PASSWORD}
    POSTGRES_DB: postgres
    CREATE_DATABASE_participant: participant-0
    CREATE_DATABASE_validator: validator
  volumes:
    - postgres-data:/var/lib/postgresql/data
  resources:
    requests:
      memory: 512Mi
      cpu: 250m
    limits:
      memory: 2Gi
      cpu: 1000m
```

### Nginx

#### Role
- **Reverse Proxy**: Routes requests to backend services
- **SSL Termination**: Handles SSL/TLS encryption
- **Static Files**: Serves static web assets
- **Load Balancing**: Distributes load across instances

#### Configuration
```nginx
upstream participant {
    server participant:4001;
}

upstream validator {
    server validator:5003;
}

server {
    listen 80;
    server_name canton-defi.example.com;

    location /api/v1/ {
        proxy_pass http://participant;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
    }

    location /wallet {
        proxy_pass http://wallet-web-ui;
    }

    location /ans {
        proxy_pass http://ans-web-ui;
    }
}
```

---

## 🔄 Data Flows

### Treasury Bill Creation Flow

```
1. User creates treasury bill via Frontend
   ↓
2. Frontend calls POST /api/defi/treasury/bills
   ↓
3. API route validates request
   ↓
4. TreasuryBillsService.createTreasuryBill()
   ↓
5. ComplianceService.validateTransaction()
   ↓
6. OracleService.getTreasuryYield()
   ↓
7. DamlIntegrationService.createInstitutionalAsset()
   ↓
8. SDK: LedgerClient.create_contract()
   ↓
9. Canton Participant: Ledger API
   ↓
10. Canton Network: Consensus
    ↓
11. Contract created on ledger
    ↓
12. Response returned to user
```

### Purchase Request Flow

```
1. User submits purchase request
   ↓
2. Frontend calls POST /api/defi/treasury/purchases
   ↓
3. API route validates request
   ↓
4. TreasuryBillsService.createPurchaseRequest()
   ↓
5. ComplianceService.validateTransaction()
   ├─ KYC verification
   ├─ AML screening
   └─ Sanctions check
   ↓
6. If compliant:
   DamlIntegrationService.createPurchaseRequest()
   ↓
7. SDK: LedgerClient.create_contract()
   ↓
8. Canton Participant: Ledger API
   ↓
9. Canton Network: Consensus
   ↓
10. Purchase request contract created
    ↓
11. Response returned to user
```

### Approval Flow

```
1. Admin approves purchase request
   ↓
2. Frontend calls PUT /api/defi/treasury/purchases/:requestId/approve
   ↓
3. API route validates admin permissions
   ↓
4. TreasuryBillsService.approvePurchaseRequest()
   ↓
5. DamlIntegrationService.approvePurchase()
   ↓
6. SDK: LedgerClient.exercise()
   ↓
7. Canton Participant: Ledger API
   ↓
8. Canton Network: Consensus
   ↓
9. AssetHolding contract created
   ↓
10. Treasury bill supply updated
    ↓
11. Response returned to admin
```

### Real Estate Token Purchase Flow

```
1. User purchases property tokens
   ↓
2. Frontend calls POST /api/defi/realestate/purchases
   ↓
3. API route validates request
   ↓
4. RealEstateService.purchaseTokens()
   ↓
5. ComplianceService.validateTransaction()
   ├─ KYC verification
   ├─ Accredited investor check
   └─ Jurisdiction compliance
   ↓
6. OracleService.getPropertyValuation()
   ↓
7. DamlIntegrationService.createPurchaseRequest()
   ↓
8. SDK: LedgerClient.create_contract()
   ↓
9. Canton Participant: Ledger API
   ↓
10. Canton Network: Consensus
    ↓
11. Token holding contract created
    ↓
12. Property supply updated
    ↓
13. Response returned to user
```

### Privacy Vault Operations Flow

```
1. User creates privacy vault
   ↓
2. Frontend calls POST /api/defi/privacy/vaults
   ↓
3. API route validates request
   ↓
4. PrivacyVaultService.createVault()
   ↓
5. ZKProofService.generateProof()
   ├─ Ownership proof
   ├─ Balance proof
   └─ Compliance proof
   ↓
6. DamlIntegrationService.createContract()
   ↓
7. SDK: LedgerClient.create_contract()
   ↓
8. Canton Participant: Ledger API
   ↓
9. Canton Network: Consensus
    ↓
10. Vault contract created
    ↓
11. Response returned to user
```

---

## 🔐 Security Architecture

### Authentication Flow

```
┌─────────────┐
│   User      │
└──────┬──────┘
       │
       │ 1. Login Request
       ↓
┌─────────────┐
│  Supabase   │
│   Auth      │
└──────┬──────┘
       │
       │ 2. JWT Token
       ↓
┌─────────────┐
│ Frontend    │
└──────┬──────┘
       │
       │ 3. API Request + JWT
       ↓
┌─────────────┐
│ API Routes  │
│ (Next.js)   │
└──────┬──────┘
       │
       │ 4. Validate JWT
       ↓
┌─────────────┐
│  Supabase   │
│   JWKS      │
└──────┬──────┘
       │
       │ 5. Valid
       ↓
┌─────────────┐
│ Business    │
│  Services   │
└──────┬──────┘
       │
       │ 6. Ledger Operation
       ↓
┌─────────────┐
│ Canton SDK  │
└──────┬──────┘
       │
       │ 7. Signed Command
       ↓
┌─────────────┐
│ Canton      │
│ Participant │
└──────┬──────┘
       │
       │ 8. Verify Signature
       ↓
┌─────────────┐
│ Canton      │
│  Network    │
└─────────────┘
```

### Security Layers

#### 1. Frontend Security
- **Authentication**: Supabase OIDC
- **Authorization**: Role-based access control
- **Input Validation**: Client-side validation
- **CSRF Protection**: Anti-CSRF tokens
- **XSS Prevention**: Content Security Policy

#### 2. API Security
- **JWT Validation**: Token verification on every request
- **Rate Limiting**: Request rate limiting
- **Input Sanitization**: Server-side input validation
- **CORS**: Cross-origin resource sharing
- **API Keys**: Optional API key authentication

#### 3. SDK Security
- **Party-based Auth**: Canton party authorization
- **Digital Signatures**: Command signing
- **Key Management**: Secure key storage
- **TLS Encryption**: Encrypted communication

#### 4. Ledger Security
- **Consensus**: BFT consensus protocol
- **Validation**: Transaction validation
- **Immutability**: Immutable ledger
- **Privacy**: Private contract data

#### 5. Network Security
- **TLS**: End-to-end encryption
- **Firewall**: Network firewall rules
- **DDoS Protection**: DDoS mitigation
- **IP Whitelisting**: IP-based access control

### Compliance Framework

#### KYC Levels
```typescript
enum KYCLevel {
  BASIC = 'BASIC',           // Email verification
  STANDARD = 'STANDARD',     // ID verification
  ENHANCED = 'ENHANCED',     // Additional documentation
  INSTITUTIONAL = 'INSTITUTIONAL' // Full institutional KYC
}
```

#### AML Risk Levels
```typescript
enum RiskLevel {
  LOW = 'LOW',
  MEDIUM = 'MEDIUM',
  HIGH = 'HIGH',
  CRITICAL = 'CRITICAL'
}
```

#### Compliance Checks
- **Identity Verification**: Document verification
- **Sanctions Screening**: OFAC, EU, UN sanctions
- **PEP Screening**: Politically exposed persons
- **Transaction Monitoring**: Suspicious activity detection
- **Geolocation**: Country-based restrictions

---

## ✅ Best Practices

### Code Organization

#### Frontend (TypeScript)
```typescript
// Feature-based structure
src/
├── app/
│   ├── defi/
│   │   ├── page.tsx
│   │   ├── treasury/
│   │   │   └── page.tsx
│   │   ├── realestate/
│   │   │   └── page.tsx
│   │   └── privacy/
│   │       └── page.tsx
│   └── api/
│       └── defi/
│           ├── treasury/
│           │   ├── bills/
│           │   │   └── route.ts
│           │   └── portfolio/
│           │       └── route.ts
│           └── compliance/
│               └── kyc/
│                   └── route.ts
├── components/
│   └── defi/
│       ├── TreasuryBillsPanel.tsx
│       ├── RealEstatePanel.tsx
│       └── PrivacyVaultsPanel.tsx
├── lib/
│   └── canton/
│       ├── services/
│       │   ├── treasuryBillsService.ts
│       │   ├── realEstateService.ts
│       │   └── privacyVaultService.ts
│       ├── hooks/
│       │   ├── useTreasuryBills.ts
│       │   ├── useRealEstate.ts
│       │   └── usePrivacyVaults.ts
│       └── store/
│           └── cantonStore.ts
└── types/
    └── defi/
        ├── treasury.ts
        ├── realestate.ts
        └── privacy.ts
```

#### SDK (Rust)
```rust
// Crate-based structure
cantonnet-omnichain-sdk/
├── canton-core/
│   ├── src/
│   │   ├── types/
│   │   │   ├── command.rs
│   │   │   ├── value.rs
│   │   │   └── identifier.rs
│   │   ├── error.rs
│   │   ├── config.rs
│   │   └── domain/
│   │       ├── treasury.rs
│   │       ├── realestate.rs
│   │       └── privacy.rs
│   └── Cargo.toml
├── canton-ledger-api/
│   ├── src/
│   │   ├── client.rs
│   │   ├── conversion.rs
│   │   └── services/
│   │       ├── command_submission.rs
│   │       └── state_service.rs
│   └── Cargo.toml
└── ...
```

### Error Handling

#### Frontend Error Handling
```typescript
// Custom error types
class DeFiError extends Error {
  constructor(
    message: string,
    public code: string,
    public details?: any
  ) {
    super(message);
    this.name = 'DeFiError';
  }
}

// Error boundaries
class DeFiErrorBoundary extends React.Component {
  state = { hasError: false, error: null };

  static getDerivedStateFromError(error: Error) {
    return { hasError: true, error };
  }

  componentDidCatch(error: Error, errorInfo: ErrorInfo) {
    console.error('DeFi Error:', error, errorInfo);
    // Log to error tracking service
  }

  render() {
    if (this.state.hasError) {
      return <ErrorFallback error={this.state.error} />;
    }
    return this.props.children;
  }
}

// Service error handling
async function createTreasuryBill(data: CreateBillInput): Promise<TreasuryBill> {
  try {
    const response = await fetch('/api/defi/treasury/bills', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(data)
    });

    if (!response.ok) {
      const error = await response.json();
      throw new DeFiError(
        error.message || 'Failed to create treasury bill',
        error.code || 'CREATE_BILL_FAILED',
        error.details
      );
    }

    return response.json();
  } catch (error) {
    if (error instanceof DeFiError) {
      throw error;
    }
    throw new DeFiError(
      'Network error occurred',
      'NETWORK_ERROR',
      { originalError: error }
    );
  }
}
```

#### SDK Error Handling
```rust
// Custom error types
#[derive(Debug, thiserror::Error)]
pub enum SdkError {
    #[error("Connection error: {0}")]
    ConnectionError(String),

    #[error("Ledger API error: {0}")]
    LedgerApiError(String),

    #[error("Conversion error: {0}")]
    ConversionError(String),

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Compliance error: {0}")]
    ComplianceError(String),

    #[error("Timeout error")]
    TimeoutError,

    #[error("Unknown error: {0}")]
    Unknown(String),
}

pub type SdkResult<T> = Result<T, SdkError>;

// Error propagation
impl From<tonic::Status> for SdkError {
    fn from(status: tonic::Status) -> Self {
        match status.code() {
            tonic::Code::Unavailable => SdkError::ConnectionError(status.message().to_string()),
            tonic::Code::InvalidArgument => SdkError::ValidationError(status.message().to_string()),
            tonic::Code::PermissionDenied => SdkError::ComplianceError(status.message().to_string()),
            _ => SdkError::LedgerApiError(status.message().to_string()),
        }
    }
}

// Error handling in services
impl TreasuryService {
    pub async fn create_bill(&mut self, input: CreateBillInput) -> SdkResult<TreasuryBill> {
        // Validate input
        self.validate_bill_input(&input)?;

        // Convert to Daml record
        let record = self.to_daml_record(&input)?;

        // Create contract
        let contract_id = self.ledger_client
            .create_contract(&self.template_ids.institutional_asset, record)
            .await?;

        // Convert to domain type
        let bill = self.from_contract_payload(contract_id)?;

        Ok(bill)
    }

    fn validate_bill_input(&self, input: &CreateBillInput) -> SdkResult<()> {
        if input.total_supply <= dec!(0) {
            return Err(SdkError::ValidationError(
                "Total supply must be positive".to_string()
            ));
        }
        if input.price_per_token <= dec!(0) {
            return Err(SdkError::ValidationError(
                "Price per token must be positive".to_string()
            ));
        }
        Ok(())
    }
}
```

### Async/Await Patterns

#### Frontend
```typescript
// React hooks with async operations
function useTreasuryBills() {
  const [bills, setBills] = useState<TreasuryBill[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<Error | null>(null);

  const fetchBills = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const response = await fetch('/api/defi/treasury/bills');
      if (!response.ok) throw new Error('Failed to fetch bills');
      const data = await response.json();
      setBills(data);
    } catch (err) {
      setError(err as Error);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    fetchBills();
  }, [fetchBills]);

  return { bills, loading, error, refetch: fetchBills };
}

// Async component with loading states
function TreasuryBillsPanel() {
  const { bills, loading, error } = useTreasuryBills();

  if (loading) return <LoadingSpinner />;
  if (error) return <ErrorMessage error={error} />;

  return (
    <div>
      {bills.map(bill => (
        <TreasuryBillCard key={bill.billId} bill={bill} />
      ))}
    </div>
  );
}
```

#### SDK
```rust
// Async service methods
impl TreasuryService {
    pub async fn create_bill(&mut self, input: CreateBillInput) -> SdkResult<TreasuryBill> {
        // Validate input
        self.validate_bill_input(&input)?;

        // Convert to Daml record
        let record = self.to_daml_record(&input)?;

        // Create contract
        let contract_id = self.ledger_client
            .create_contract(&self.template_ids.institutional_asset, record)
            .await?;

        // Convert to domain type
        let bill = self.from_contract_payload(contract_id)?;

        Ok(bill)
    }

    pub async fn get_portfolio_summary(&self, investor: &str) -> SdkResult<PortfolioSummary> {
        // Get holdings
        let holdings = self.get_holdings(investor).await?;

        // Calculate summary
        let total_value = holdings.iter()
            .map(|h| h.current_market_value)
            .sum::<Decimal>();

        let total_invested = holdings.iter()
            .map(|h| h.average_cost_basis * h.tokens_owned)
            .sum::<Decimal>();

        let yield_earned = holdings.iter()
            .map(|h| h.accumulated_yield)
            .sum::<Decimal>();

        let unrealized_gains = holdings.iter()
            .map(|h| h.unrealized_gain_loss)
            .sum::<Decimal>();

        Ok(PortfolioSummary {
            total_value,
            total_invested,
            yield_earned,
            unrealized_gains,
            holding_count: holdings.len(),
        })
    }
}

// Async client with timeout
impl LedgerClient {
    pub async fn submit_with_timeout(
        &mut self,
        commands: Commands,
        timeout: Duration,
    ) -> SdkResult<()> {
        let submit_future = self.submit(commands);
        match tokio::time::timeout(timeout, submit_future).await {
            Ok(result) => result,
            Err(_) => Err(SdkError::TimeoutError),
        }
    }
}
```

### Testing Strategies

#### Frontend Testing
```typescript
// Unit tests with Jest
describe('TreasuryBillsService', () => {
  it('should create treasury bill', async () => {
    const mockData: CreateBillInput = {
      name: 'Test Bill',
      symbol: 'TEST',
      totalSupply: 1000000,
      pricePerToken: 0.99,
    };

    const result = await createTreasuryBill(mockData);
    expect(result).toHaveProperty('billId');
    expect(result.name).toBe('Test Bill');
  });

  it('should validate input', async () => {
    const invalidData = {
      name: '',
      symbol: 'TEST',
      totalSupply: -1,
      pricePerToken: 0.99,
    };

    await expect(createTreasuryBill(invalidData)).rejects.toThrow();
  });
});

// Integration tests with Playwright
test('user can create treasury bill', async ({ page }) => {
  await page.goto('/defi/treasury');
  await page.click('[data-testid="create-bill-button"]');
  await page.fill('[name="name"]', 'Test Bill');
  await page.fill('[name="symbol"]', 'TEST');
  await page.fill('[name="totalSupply"]', '1000000');
  await page.click('[data-testid="submit-button"]');
  await expect(page.locator('[data-testid="success-message"]')).toBeVisible();
});
```

#### SDK Testing
```rust
// Unit tests
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_bill_input() {
        let service = TreasuryService::new(/* ... */);

        let valid_input = CreateBillInput {
            name: "Test Bill".to_string(),
            total_supply: dec!(1000000),
            price_per_token: dec!(0.99),
            // ... other fields
        };

        assert!(service.validate_bill_input(&valid_input).is_ok());

        let invalid_input = CreateBillInput {
            name: "Test Bill".to_string(),
            total_supply: dec!(-1),
            price_per_token: dec!(0.99),
            // ... other fields
        };

        assert!(service.validate_bill_input(&invalid_input).is_err());
    }

    #[test]
    fn test_to_daml_record() {
        let service = TreasuryService::new(/* ... */);

        let input = CreateBillInput {
            name: "Test Bill".to_string(),
            total_supply: dec!(1000000),
            // ... other fields
        };

        let record = service.to_daml_record(&input).unwrap();
        assert_eq!(record.fields.get("name").unwrap(), &DamlValue::Text("Test Bill".to_string()));
    }
}

// Integration tests
#[cfg(feature = "integration")]
mod integration_tests {
    use super::*;

    #[tokio::test]
    async fn test_create_bill_on_ledger() {
        let mut client = LedgerClient::connect(
            "http://65.108.15.30:30501",
            "participant"
        ).await.unwrap();

        let mut service = TreasuryService::new(client, template_ids);

        let input = CreateBillInput {
            name: "Integration Test Bill".to_string(),
            total_supply: dec!(1000000),
            // ... other fields
        };

        let bill = service.create_bill(input).await.unwrap();
        assert!(!bill.bill_id.is_empty());
        assert_eq!(bill.name, "Integration Test Bill");
    }
}
```

### Performance Optimization

#### Frontend Optimization
```typescript
// Code splitting
const TreasuryBillsPanel = lazy(() => import('./TreasuryBillsPanel'));
const RealEstatePanel = lazy(() => import('./RealEstatePanel'));
const PrivacyVaultsPanel = lazy(() => import('./PrivacyVaultsPanel'));

// Memoization
function TreasuryBillCard({ bill }: { bill: TreasuryBill }) {
  const formattedPrice = useMemo(
    () => formatCurrency(bill.pricePerToken),
    [bill.pricePerToken]
  );

  const yieldPercent = useMemo(
    () => (bill.currentYield * 100).toFixed(2),
    [bill.currentYield]
  );

  return (
    <Card>
      <h3>{bill.name}</h3>
      <p>Price: {formattedPrice}</p>
      <p>Yield: {yieldPercent}%</p>
    </Card>
  );
}

// Debouncing
function useDebounce<T>(value: T, delay: number): T {
  const [debouncedValue, setDebouncedValue] = useState(value);

  useEffect(() => {
    const handler = setTimeout(() => {
      setDebouncedValue(value);
    }, delay);

    return () => {
      clearTimeout(handler);
    };
  }, [value, delay]);

  return debouncedValue;
}

// Virtual scrolling for large lists
import { FixedSizeList } from 'react-window';

function TreasuryBillsList({ bills }: { bills: TreasuryBill[] }) {
  const Row = ({ index, style }: { index: number; style: React.CSSProperties }) => (
    <div style={style}>
      <TreasuryBillCard bill={bills[index]} />
    </div>
  );

  return (
    <FixedSizeList
      height={600}
      itemCount={bills.length}
      itemSize={120}
      width="100%"
    >
      {Row}
    </FixedSizeList>
  );
}
```

#### SDK Optimization
```rust
// Connection pooling
impl LedgerClient {
    pub async fn connect_with_pool(
        endpoint: &str,
        ledger_id: &str,
        pool_size: usize,
    ) -> SdkResult<Self> {
        let channel = ChannelBuilder::new(endpoint)
            .pool_size(pool_size)
            .connect()
            .await?;

        Ok(Self {
            channel,
            ledger_id: ledger_id.to_string(),
        })
    }
}

// Batching operations
impl TreasuryService {
    pub async fn create_bills_batch(
        &mut self,
        inputs: Vec<CreateBillInput>,
    ) -> SdkResult<Vec<TreasuryBill>> {
        let mut commands = Vec::new();

        for input in inputs {
            let record = self.to_daml_record(&input)?;
            commands.push(Command::Create(CreateCommand {
                template_id: self.template_ids.institutional_asset.clone(),
                create_arguments: record,
            }));
        }

        self.ledger_client
            .submit_domain_commands(&Commands { commands })
            .await?;

        // Fetch created contracts
        // ...
    }
}

// Caching
use lru::LruCache;

impl TreasuryService {
    pub async fn get_bill_cached(&self, bill_id: &str) -> SdkResult<Option<TreasuryBill>> {
        // Check cache
        if let Some(bill) = self.cache.lock().await.get(bill_id) {
            return Ok(Some(bill.clone()));
        }

        // Fetch from ledger
        let bill = self.get_bill(bill_id).await?;

        // Update cache
        if let Some(ref bill) = bill {
            self.cache.lock().await.put(bill_id.to_string(), bill.clone());
        }

        Ok(bill)
    }
}
```

---

## 🚀 Deployment Patterns

### Development Environment

#### Local Development
```bash
# Start local Canton participant
docker-compose -f docker-compose.local.yml up -d

# Start frontend
cd canton-otc/defi
npm install
npm run dev

# Start SDK tests
cd cantonnet-omnichain-sdk
cargo test
```

#### DevNet Deployment
```bash
# Connect to DevNet
./scripts/get-onboarding-secret.sh devnet --save

# Deploy validator
kubectl apply -f k8s/canton-validator-full-stack.yaml

# Verify deployment
kubectl get pods -n canton-node
kubectl logs -l app=canton-validator -f
```

### Production Environment

#### Kubernetes Deployment
```yaml
# Production values
replicaCount: 3

resources:
  requests:
    memory: 2Gi
    cpu: 1000m
  limits:
    memory: 4Gi
    cpu: 2000m

autoscaling:
  enabled: true
  minReplicas: 3
  maxReplicas: 10
  targetCPUUtilizationPercentage: 70
  targetMemoryUtilizationPercentage: 80

nodeSelector:
  workload: production

tolerations:
  - key: "dedicated"
    operator: "Equal"
    value: "production"
    effect: "NoSchedule"
```

#### Blue-Green Deployment
```bash
# Deploy blue version
kubectl apply -f k8s/canton-validator-blue.yaml

# Verify blue version
kubectl get pods -l version=blue

# Switch traffic to blue
kubectl patch service canton-validator -p '{"spec":{"selector":{"version":"blue"}}}'

# Monitor blue version
kubectl logs -l version=blue -f

# If successful, remove green version
kubectl delete -f k8s/canton-validator-green.yaml
```

#### Canary Deployment
```bash
# Deploy canary version (10% traffic)
kubectl apply -f k8s/canton-validator-canary.yaml

# Update service to route 10% to canary
kubectl patch service canton-validator -p '{
  "spec": {
    "selector": {
      "version": "canary"
    },
    "sessionAffinity": "ClientIP",
    "sessionAffinityConfig": {
      "clientIP": {
        "timeoutSeconds": 10800
      }
    }
  }
}'

# Monitor canary
kubectl logs -l version=canary -f

# Gradually increase traffic
# 25% -> 50% -> 75% -> 100%

# Full rollout
kubectl apply -f k8s/canton-validator-v2.yaml
```

### CI/CD Pipeline

#### GitHub Actions
```yaml
name: CI/CD Pipeline

on:
  push:
    branches: [main, develop]
  pull_request:
    branches: [main]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Setup Node.js
        uses: actions/setup-node@v3
        with:
          node-version: '18'

      - name: Install dependencies
        run: |
          cd canton-otc/defi
          npm ci

      - name: Run tests
        run: |
          cd canton-otc/defi
          npm test

      - name: Setup Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable

      - name: Run SDK tests
        run: |
          cd cantonnet-omnichain-sdk
          cargo test --all-features

  build:
    needs: test
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Build Docker images
        run: |
          docker build -t canton-defi:${{ github.sha }} .
          docker build -t canton-sdk:${{ github.sha }} ./cantonnet-omnichain-sdk

      - name: Push to registry
        run: |
          echo ${{ secrets.GHCR_TOKEN }} | docker login ghcr.io -u ${{ github.actor }} --password-stdin
          docker push ghcr.io/gyber/canton-defi:${{ github.sha }}

  deploy:
    needs: build
    runs-on: ubuntu-latest
    if: github.ref == 'refs/heads/main'
    steps:
      - name: Deploy to Kubernetes
        run: |
          kubectl set image deployment/canton-validator \
            validator=ghcr.io/gyber/canton-defi:${{ github.sha }}
          kubectl rollout status deployment/canton-validator
```

---

## 📊 Monitoring & Observability

### Metrics Collection

#### Prometheus Metrics
```yaml
# Prometheus configuration
global:
  scrape_interval: 15s
  evaluation_interval: 15s

scrape_configs:
  - job_name: 'canton-validator'
    static_configs:
      - targets: ['65.108.15.30:8080']
    metrics_path: '/metrics'

  - job_name: 'canton-participant'
    static_configs:
      - targets: ['65.108.15.30:4001']
    metrics_path: '/metrics'

  - job_name: 'defi-api'
    static_configs:
      - targets: ['defi-api:3000']
    metrics_path: '/api/metrics'
```

#### Custom Metrics
```rust
// SDK metrics
use prometheus::{Counter, Histogram, Registry};

pub struct Metrics {
    pub requests_total: Counter,
    pub request_duration: Histogram,
    pub errors_total: Counter,
}

impl Metrics {
    pub fn new() -> Self {
        Self {
            requests_total: Counter::new(
                "canton_sdk_requests_total",
                "Total number of SDK requests"
            ).unwrap(),

            request_duration: Histogram::new(
                "canton_sdk_request_duration_seconds",
                "Request duration in seconds"
            ).unwrap(),

            errors_total: Counter::new(
                "canton_sdk_errors_total",
                "Total number of SDK errors"
            ).unwrap(),
        }
    }

    pub fn register(&self, registry: &Registry) -> prometheus::Result<()> {
        registry.register(Box::new(self.requests_total.clone()))?;
        registry.register(Box::new(self.request_duration.clone()))?;
        registry.register(Box::new(self.errors_total.clone()))?;
        Ok(())
    }
}

// Usage
impl LedgerClient {
    pub async fn submit(&mut self, commands: Commands) -> SdkResult<()> {
        let timer = self.metrics.request_duration.start_timer();

        match self.submit_internal(commands).await {
            Ok(_) => {
                self.metrics.requests_total.inc();
                timer.observe_duration();
                Ok(())
            }
            Err(e) => {
                self.metrics.errors_total.inc();
                timer.observe_duration();
                Err(e)
            }
        }
    }
}
```

### Logging

#### Structured Logging
```rust
// SDK logging
use tracing::{info, error, warn, instrument};

#[instrument(skip(self))]
impl TreasuryService {
    pub async fn create_bill(&mut self, input: CreateBillInput) -> SdkResult<TreasuryBill> {
        info!(
            bill_name = %input.name,
            total_supply = %input.total_supply,
            "Creating treasury bill"
        );

        match self.create_bill_internal(input).await {
            Ok(bill) => {
                info!(
                    bill_id = %bill.bill_id,
                    "Treasury bill created successfully"
                );
                Ok(bill)
            }
            Err(e) => {
                error!(
                    error = %e,
                    "Failed to create treasury bill"
                );
                Err(e)
            }
        }
    }
}
```

#### Frontend Logging
```typescript
// Structured logging
import { logger } from '@/lib/logger';

logger.info('Creating treasury bill', {
  billName: input.name,
  totalSupply: input.totalSupply,
  userId: user.id,
});

logger.error('Failed to create treasury bill', {
  error: error.message,
  errorCode: error.code,
  userId: user.id,
  timestamp: new Date().toISOString(),
});
```

### Distributed Tracing

#### OpenTelemetry Integration
```rust
// SDK tracing
use opentelemetry::trace::{TraceContextExt, Tracer};
use opentelemetry::global;

#[instrument(skip(self))]
impl LedgerClient {
    pub async fn submit(&mut self, commands: Commands) -> SdkResult<()> {
        let tracer = global::tracer("canton-sdk");

        let span = tracer.start("LedgerClient::submit");

        span.set_attribute("command_count", commands.commands.len() as i64);

        match self.submit_internal(commands).await {
            Ok(_) => {
                span.set_status(opentelemetry::trace::Status::Ok);
                Ok(())
            }
            Err(e) => {
                span.set_status(opentelemetry::trace::Status::Error {
                    description: e.to_string().into(),
                });
                Err(e)
            }
        }
    }
}
```

### Alerting

#### Prometheus Alert Rules
```yaml
groups:
  - name: canton_alerts
    rules:
      - alert: HighErrorRate
        expr: rate(canton_sdk_errors_total[5m]) > 0.1
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "High error rate detected"
          description: "Error rate is {{ $value }} errors/sec"

      - alert: HighLatency
        expr: histogram_quantile(0.99, rate(canton_sdk_request_duration_seconds_bucket[5m])) > 5
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "High latency detected"
          description: "P99 latency is {{ $value }} seconds"

      - alert: ValidatorDown
        expr: up{job="canton-validator"} == 0
        for: 1m
        labels:
          severity: critical
        annotations:
          summary: "Canton validator is down"
          description: "Validator {{ $labels.instance }} is not responding"
```

---

## 🧪 Testing Strategy

### Test Pyramid

```
        /\
       /  \
      / E2E \        - 10% of tests
     /--------\
    /          \
   / Integration \    - 30% of tests
  /--------------\
 /                \
/    Unit Tests    \  - 60% of tests
/------------------\
```

### Unit Tests

#### Frontend Unit Tests
```typescript
// Component tests
describe('TreasuryBillsPanel', () => {
  it('renders treasury bills list', () => {
    const bills = [
      { billId: '1', name: 'Bill 1', pricePerToken: 0.99 },
      { billId: '2', name: 'Bill 2', pricePerToken: 0.98 },
    ];

    render(<TreasuryBillsPanel bills={bills} />);

    expect(screen.getByText('Bill 1')).toBeInTheDocument();
    expect(screen.getByText('Bill 2')).toBeInTheDocument();
  });

  it('shows loading state', () => {
    render(<TreasuryBillsPanel loading={true} bills={[]} />);
    expect(screen.getByTestId('loading-spinner')).toBeInTheDocument();
  });

  it('shows error state', () => {
    render(<TreasuryBillsPanel error={new Error('Test error')} bills={[]} />);
    expect(screen.getByText('Test error')).toBeInTheDocument();
  });
});

// Hook tests
describe('useTreasuryBills', () => {
  it('fetches bills on mount', async () => {
    const mockBills = [
      { billId: '1', name: 'Bill 1', pricePerToken: 0.99 },
    ];

    global.fetch = jest.fn(() =>
      Promise.resolve({
        ok: true,
        json: () => Promise.resolve(mockBills),
      })
    );

    const { result } = renderHook(() => useTreasuryBills());

    await waitFor(() => {
      expect(result.current.bills).toEqual(mockBills);
    });
  });
});
```

#### SDK Unit Tests
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_bill_input_valid() {
        let service = TreasuryService::new(/* ... */);

        let input = CreateBillInput {
            name: "Test Bill".to_string(),
            total_supply: dec!(1000000),
            price_per_token: dec!(0.99),
            // ... other fields
        };

        assert!(service.validate_bill_input(&input).is_ok());
    }

    #[test]
    fn test_validate_bill_input_invalid_supply() {
        let service = TreasuryService::new(/* ... */);

        let input = CreateBillInput {
            name: "Test Bill".to_string(),
            total_supply: dec!(-1),
            price_per_token: dec!(0.99),
            // ... other fields
        };

        assert!(matches!(
            service.validate_bill_input(&input),
            Err(SdkError::ValidationError(_))
        ));
    }

    #[test]
    fn test_to_daml_record() {
        let service = TreasuryService::new(/* ... */);

        let input = CreateBillInput {
            name: "Test Bill".to_string(),
            total_supply: dec!(1000000),
            // ... other fields
        };

        let record = service.to_daml_record(&input).unwrap();

        assert_eq!(
            record.fields.get("name"),
            Some(&DamlValue::Text("Test Bill".to_string()))
        );
        assert_eq!(
            record.fields.get("totalSupply"),
            Some(&DamlValue::Numeric(dec!(1000000)))
        );
    }
}
```

### Integration Tests

#### Frontend Integration Tests
```typescript
// API integration tests
describe('Treasury API', () => {
  beforeAll(async () => {
    // Setup test database
    await setupTestDatabase();
  });

  afterAll(async () => {
    // Cleanup test database
    await cleanupTestDatabase();
  });

  it('creates treasury bill', async () => {
    const response = await fetch('/api/defi/treasury/bills', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        name: 'Test Bill',
        symbol: 'TEST',
        totalSupply: 1000000,
        pricePerToken: 0.99,
      }),
    });

    expect(response.ok).toBe(true);
    const bill = await response.json();
    expect(bill).toHaveProperty('billId');
    expect(bill.name).toBe('Test Bill');
  });

  it('gets treasury bills', async () => {
    const response = await fetch('/api/defi/treasury/bills');
    expect(response.ok).toBe(true);

    const bills = await response.json();
    expect(Array.isArray(bills)).toBe(true);
  });
});
```

#### SDK Integration Tests
```rust
#[cfg(feature = "integration")]
mod integration_tests {
    use super::*;

    #[tokio::test]
    async fn test_create_bill_on_devnet() {
        let mut client = LedgerClient::connect(
            "http://65.108.15.30:30501",
            "participant"
        ).await.unwrap();

        let mut service = TreasuryService::new(client, template_ids);

        let input = CreateBillInput {
            name: "Integration Test Bill".to_string(),
            total_supply: dec!(1000000),
            price_per_token: dec!(0.99),
            // ... other fields
        };

        let bill = service.create_bill(input).await.unwrap();

        assert!(!bill.bill_id.is_empty());
        assert_eq!(bill.name, "Integration Test Bill");

        // Cleanup
        service.delist_bill(&bill.bill_id).await.unwrap();
    }

    #[tokio::test]
    async fn test_purchase_flow() {
        let mut client = LedgerClient::connect(
            "http://65.108.15.30:30501",
            "participant"
        ).await.unwrap();

        let mut service = TreasuryService::new(client, template_ids);

        // Create bill
        let bill = service.create_bill(/* ... */).await.unwrap();

        // Create purchase request
        let request = service.create_purchase_request(
            &bill.bill_id,
            "test-investor",
            100,
            PaymentData::default(),
        ).await.unwrap();

        assert_eq!(request.status, RequestStatus::PENDING);

        // Approve request
        let holding = service.approve_purchase_request(
            &request.request_id,
            "admin",
        ).await.unwrap();

        assert_eq!(holding.investor, "test-investor");
        assert_eq!(holding.tokens_owned, dec!(100));
    }
}
```

### End-to-End Tests

#### Playwright E2E Tests
```typescript
import { test, expect } from '@playwright/test';

test.describe('Treasury Bills Flow', () => {
  test('user can create and purchase treasury bill', async ({ page }) => {
    // Login
    await page.goto('/login');
    await page.fill('[name="email"]', 'test@example.com');
    await page.fill('[name="password"]', 'password');
    await page.click('[type="submit"]');

    // Navigate to treasury
    await page.goto('/defi/treasury');

    // Create bill (admin)
    await page.click('[data-testid="create-bill-button"]');
    await page.fill('[name="name"]', 'E2E Test Bill');
    await page.fill('[name="symbol"]', 'E2E');
    await page.fill('[name="totalSupply"]', '1000000');
    await page.fill('[name="pricePerToken"]', '0.99');
    await page.click('[data-testid="submit-button"]');

    // Wait for success
    await expect(page.locator('[data-testid="success-message"]')).toBeVisible();

    // Purchase bill (user)
    await page.click('[data-testid="purchase-button"]');
    await page.fill('[name="numberOfTokens"]', '100');
    await page.click('[data-testid="confirm-purchase"]');

    // Wait for confirmation
    await expect(page.locator('[data-testid="purchase-confirmation"]')).toBeVisible();

    // Verify in portfolio
    await page.goto('/defi/treasury/portfolio');
    await expect(page.locator('text=E2E Test Bill')).toBeVisible();
  });
});
```

---

## ⚡ Performance Optimization

### Frontend Performance

#### Code Splitting
```typescript
// Lazy loading routes
const TreasuryPage = lazy(() => import('./pages/TreasuryPage'));
const RealEstatePage = lazy(() => import('./pages/RealEstatePage'));
const PrivacyPage = lazy(() => import('./pages/PrivacyPage'));

function App() {
  return (
    <Suspense fallback={<LoadingSpinner />}>
      <Routes>
        <Route path="/defi/treasury" element={<TreasuryPage />} />
        <Route path="/defi/realestate" element={<RealEstatePage />} />
        <Route path="/defi/privacy" element={<PrivacyPage />} />
      </Routes>
    </Suspense>
  );
}
```

#### Image Optimization
```typescript
import Image from 'next/image';

function PropertyImage({ src, alt }: { src: string; alt: string }) {
  return (
    <Image
      src={src}
      alt={alt}
      width={800}
      height={600}
      loading="lazy"
      placeholder="blur"
      blurDataURL="data:image/jpeg;base64,/9j/4AAQSkZJRg..."
    />
  );
}
```

#### Caching Strategy
```typescript
// API caching with SWR
import useSWR from 'swr';

const fetcher = (url: string) => fetch(url).then(r => r.json());

function useTreasuryBills() {
  const { data, error, mutate } = useSWR(
    '/api/defi/treasury/bills',
    fetcher,
    {
      revalidateOnFocus: false,
      revalidateOnReconnect: true,
      dedupingInterval: 60000, // 1 minute
    }
  );

  return {
    bills: data,
    error,
    loading: !error && !data,
    refresh: mutate,
  };
}
```

### SDK Performance

#### Connection Pooling
```rust
use tower::ServiceBuilder;
use tower::limit::RateLimitLayer;

impl LedgerClient {
    pub async fn connect_with_pool(
        endpoint: &str,
        ledger_id: &str,
        pool_size: usize,
    ) -> SdkResult<Self> {
        let channel = ChannelBuilder::new(endpoint)
            .pool_size(pool_size)
            .timeout(Duration::from_secs(30))
            .connect()
            .await?;

        let service = ServiceBuilder::new()
            .layer(RateLimitLayer::new(100, Duration::from_secs(1)))
            .service(channel);

        Ok(Self {
            channel: service,
            ledger_id: ledger_id.to_string(),
        })
    }
}
```

#### Batching Operations
```rust
impl TreasuryService {
    pub async fn create_bills_batch(
        &mut self,
        inputs: Vec<CreateBillInput>,
    ) -> SdkResult<Vec<TreasuryBill>> {
        let mut commands = Vec::new();

        for input in inputs {
            let record = self.to_daml_record(&input)?;
            commands.push(Command::Create(CreateCommand {
                template_id: self.template_ids.institutional_asset.clone(),
                create_arguments: record,
            }));
        }

        // Submit all commands in one transaction
        self.ledger_client
            .submit_domain_commands(&Commands { commands })
            .await?;

        // Fetch created contracts
        let bills = self.fetch_created_bills(inputs.len()).await?;

        Ok(bills)
    }
}
```

#### Caching
```rust
use lru::LruCache;
use tokio::sync::Mutex;

impl TreasuryService {
    pub async fn get_bill_cached(&self, bill_id: &str) -> SdkResult<Option<TreasuryBill>> {
        // Check cache
        {
            let mut cache = self.cache.lock().await;
            if let Some(bill) = cache.get(bill_id) {
                return Ok(Some(bill.clone()));
            }
        }

        // Fetch from ledger
        let bill = self.get_bill(bill_id).await?;

        // Update cache
        if let Some(ref bill) = bill {
            let mut cache = self.cache.lock().await;
            cache.put(bill_id.to_string(), bill.clone());
        }

        Ok(bill)
    }
}
```

---

## 📚 Additional Resources

### Documentation
- [Canton Network Official Docs](https://sync.global/docs/)
- [Daml Language Reference](https://docs.daml.com/)
- [Next.js Documentation](https://nextjs.org/docs)
- [Rust Book](https://doc.rust-lang.org/book/)

### Key Files
- `prompts/DEFI_SDK_MASTER_PROMPT.md` - SDK implementation guide
- `prompts/CANTON_DEFI_BLOCK_DIAGRAM_PROMPT.md` - Visualization prompt
- `config/canton.conf` - Canton configuration
- `config/validator.conf` - Validator configuration
- `k8s/canton-validator-full-stack.yaml` - Kubernetes deployment
- `DEFI_CONNECT_DEVNET.md` - DevNet connection guide

### Community
- [Canton Discord](https://discord.gg/canton)
- [Canton Telegram](https://t.me/canton_network)
- [GitHub Repository](https://github.com/digital-asset/decentralized-canton-sync)

---

**Version**: 1.0  
**Date**: 2025-02-01  
**Author**: Gyber  
**Status**: Production Ready ✅
