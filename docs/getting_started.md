# Canton Wallet SDK - Getting Started Guide

## Table of Contents

1. [Installation](#installation)
2. [Quick Start](#quick-start)
3. [Basic Wallet Operations](#basic-wallet-operations)
4. [HD Wallets](#hd-wallets)
5. [Multi-Signature Wallets](#multi-signature-wallets)
6. [Cross-Chain Transfers](#cross-chain-transfers)
7. [Configuration](#configuration)
8. [Troubleshooting](#troubleshooting)

---

## Installation

### Prerequisites

- Rust 1.85 or later
- Cargo (comes with Rust)
- A running Canton Network participant

### Add to Cargo.toml

Add the following to your `Cargo.toml`:

```toml
[dependencies]
canton-wallet-sdk = "0.1.0"
```

### Features

The SDK supports optional features:

```toml
[dependencies]
canton-wallet-sdk = { version = "0.1.0", features = ["prometheus"] }
```

Available features:
- `prometheus` - Enable Prometheus metrics export
- `hsm` - Enable HSM support (requires additional dependencies)

---

## Quick Start

### Create a Wallet

```rust
use canton_wallet_sdk::{CantonWallet, WalletConfig};
use canton_observability::{ObservabilityConfig, init_observability};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize observability
    let observability_config = ObservabilityConfig::default();
    init_observability(observability_config)?;

    // Create wallet configuration
    let config = WalletConfig {
        ledger_endpoint: "http://localhost:50051".to_string(),
        party_id: "Alice".to_string(),
        participant_id: "participant1".to_string(),
        ..Default::default()
    };

    // Create wallet
    let wallet = CantonWallet::new(config).await?;
    
    println!("Wallet ID: {}", wallet.wallet_id());
    println!("Party ID: {}", wallet.party_id());
    
    Ok(())
}
```

### Get Balance

```rust
let balance = wallet.balance().await?;
println!("Balance: {} {}", balance.total_amount, balance.currency);
```

### Create a Contract

```rust
use canton_wallet_core::{Identifier, DamlRecord, DamlValue};

let template_id = Identifier::new("Main", "Iou", "Iou");
let mut arguments = DamlRecord::new();
arguments.add_field("issuer", DamlValue::text("Alice"));
arguments.add_field("owner", DamlValue::text("Bob"));
arguments.add_field("amount", DamlValue::int64(100));

let created_event = wallet.create_contract(template_id, arguments).await?;
println!("Contract ID: {}", created_event.contract_id);
```

### Exercise a Choice

```rust
use canton_wallet_core::ContractId;

let contract_id = ContractId::new_unchecked(&created_event.contract_id);
let choice_argument = DamlValue::int64(50);

let transaction = wallet
    .exercise_choice(contract_id, "Transfer", choice_argument)
    .await?;
println!("Transaction ID: {}", transaction.transaction_id);
```

---

## Basic Wallet Operations

### Transaction Builder

Use the transaction builder for complex transactions:

```rust
use canton_wallet_transactions::TransactionBuilder;
use canton_wallet_core::{Command, CreateCommand};

let commands = TransactionBuilder::new()
    .party_id(wallet.party_id().clone())
    .add_command(Command::Create(CreateCommand {
        template_id,
        create_arguments: arguments,
    }))
    .workflow_id("my-workflow")
    .application_id("my-app")
    .build()?;
```

### Event Streaming

Subscribe to real-time events:

```rust
use canton_wallet_events::EventStream;
use canton_wallet_core::TransactionFilter;

let stream = EventStream::new(
    ledger_client.clone(),
    wallet.party_id().clone(),
    TransactionFilter::for_party(wallet.party_id()),
);

let mut event_stream = stream.subscribe();
while let Some(result) = event_stream.next().await {
    match result {
        Ok(transaction) => {
            println!("New transaction: {}", transaction.transaction_id);
        }
        Err(e) => {
            eprintln!("Error: {}", e);
        }
    }
}
```

---

## HD Wallets

### Create HD Wallet

```rust
use canton_wallet_sdk::HDWallet;

let hd_wallet = HDWallet::new(bip39::MnemonicType::Words12)?;
println!("Mnemonic: {}", hd_wallet.mnemonic_phrase());
```

### Derive Account

```rust
let account = hd_wallet.derive_account(0).await?;
println!("Account 0 address: {}", account.address());
```

### Restore from Mnemonic

```rust
let mnemonic = "abandon abandon ability able about above absent absorb abstract absurd abuse accident accuse acid acclaim acquire";
let hd_wallet = HDWallet::from_mnemonic(mnemonic)?;
```

---

## Multi-Signature Wallets

### Create Multi-Sig Wallet

```rust
use canton_wallet_sdk::{MultiSigWallet, MultiSigConfig};

let config = MultiSigConfig {
    threshold: 2,
    total_signers: 3,
    signers: vec![
        "Alice".to_string(),
        "Bob".to_string(),
        "Charlie".to_string(),
    ],
    ..Default::default()
};

let multi_sig_wallet = MultiSigWallet::new(config)?;
```

### Collect Signatures

```rust
let transaction = /* ... */;
let signature = multi_sig_wallet.sign_transaction(&transaction).await?;
```

### Execute Multi-Sig Transaction

```rust
let result = multi_sig_wallet
    .execute_transaction(transaction, signatures)
    .await?;
```

---

## Cross-Chain Transfers

### Setup Multi-Chain Wallet

```rust
use canton_wallet_omnichain::{MultiChainWallet, BridgeManager, BridgeConfig};
use canton_wallet_omnichain::adapter::{EthereumAdapter, ChainConfig};

let bridge_config = BridgeConfig::default();
let bridge_manager = Arc::new(BridgeManager::new(bridge_config));

let mut multi_chain_wallet = MultiChainWallet::new(
    Arc::new(canton_wallet),
    bridge_manager,
);

// Add Ethereum adapter
let eth_config = ChainConfig::new(
    canton_wallet_omnichain::types::ChainId::Ethereum,
    "https://mainnet.infura.io/v3/YOUR-PROJECT-ID",
);
let eth_adapter = EthereumAdapter::new(eth_config);
let eth_wallet = eth_adapter.create_wallet(eth_config).await?;
multi_chain_wallet.add_chain(
    canton_wallet_omnichain::types::ChainId::Ethereum,
    Arc::new(eth_wallet),
);
```

### Transfer to Another Chain

```rust
use canton_wallet_omnichain::types::{CantonAsset, ChainAddress};

let asset = CantonAsset::new(
    "token1".to_string(),
    "100".to_string(),
    wallet.party_id().to_string(),
);

let recipient = ChainAddress::ethereum("0x1234...5678");

let cross_chain_tx = multi_chain_wallet
    .transfer_to_chain(asset, canton_wallet_omnichain::types::ChainId::Ethereum, recipient)
    .await?;

println!("Cross-chain transaction completed!");
println!("  Canton TX ID: {}", cross_chain_tx.canton_tx_id);
println!("  Target TX ID: {}", cross_chain_tx.target_tx_id);
```

---

## Configuration

### Wallet Configuration

```rust
use canton_wallet_sdk::WalletConfig;

let config = WalletConfig {
    ledger_endpoint: "http://localhost:50051".to_string(),
    party_id: "Alice".to_string(),
    participant_id: "participant1".to_string(),
    key_store: KeyStoreConfig::EncryptedFile {
        path: "./wallet.enc".to_string(),
        encryption_key: "your-encryption-key".to_string(),
    },
    timeout_seconds: 300,
    max_retries: 3,
};
```

### Observability Configuration

```rust
use canton_observability::{ObservabilityConfig, LogConfig, MetricsConfig, TracingConfig};

let config = ObservabilityConfig {
    logging: LogConfig::new()
        .with_level("debug")
        .with_format("json")
        .with_stdout(true),
    metrics: MetricsConfig::new()
        .with_enabled(true)
        .with_exporter("otlp")
        .with_endpoint("http://localhost:4317"),
    tracing: TracingConfig::new()
        .with_enabled(true)
        .with_exporter("otlp")
        .with_sample_rate(1.0),
    ..Default::default()
};
```

---

## Troubleshooting

### Connection Issues

If you're having trouble connecting to the Canton Network:

1. Verify the ledger endpoint is correct
2. Check network connectivity
3. Ensure the participant is running
4. Verify TLS certificates

### Key Store Issues

If you're having trouble with key storage:

1. Check file permissions
2. Verify encryption key is correct
3. Ensure sufficient disk space
4. Check for corrupted key files

### Transaction Failures

If transactions are failing:

1. Check transaction validation errors
2. Verify party permissions
3. Ensure sufficient balance
4. Check contract template exists

### Getting Help

For additional help:

- Check the [API Documentation](https://docs.rs/canton-wallet-sdk)
- Review [Architecture Documentation](./architecture.md)
- See [Security Best Practices](./security.md)
- Open an issue on [GitHub](https://github.com/your-org/canton-wallet-sdk/issues)

---

## Next Steps

- Explore the [API Documentation](https://docs.rs/canton-wallet-sdk)
- Read the [Architecture Documentation](./architecture.md)
- Review [Security Best Practices](./security.md)
- Check out more [examples](../examples/)
