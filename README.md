# Canton Wallet SDK

A production-ready, secure, and user-friendly Wallet SDK for Canton Network, written in Rust.

## Overview

The Canton Wallet SDK provides developers with a comprehensive toolkit for building wallet applications on the Canton Network. It offers secure key management, intuitive APIs for contract interaction, transaction management, and support for advanced features like HD wallets, multi-signature wallets, and cross-chain transfers.

## Features

- **Secure Key Management**: Encrypted key storage with HSM support
- **HD Wallet Support**: BIP39/BIP44 compliant hierarchical deterministic wallets
- **Multi-Signature**: Support for multi-signature wallets
- **Contract Management**: Intuitive APIs for creating and interacting with contracts
- **Transaction Management**: Builder pattern for transaction construction with validation
- **Event Streaming**: Real-time event subscription and filtering
- **Cross-Chain**: OmniChain integration for cross-chain asset transfers
- **Recovery**: Backup, restore, and social recovery mechanisms
- **Production-Ready**: Comprehensive error handling, logging, and metrics

## Installation

Add the following to your `Cargo.toml`:

```toml
[dependencies]
canton-wallet-sdk = "0.1.0"
```

## Quick Start

```rust
use canton_wallet_sdk::{CantonWallet, WalletConfig};
use tokio;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create wallet configuration
    let config = WalletConfig {
        ledger_endpoint: "http://localhost:50051".to_string(),
        ..Default::default()
    };
    
    // Create wallet
    let wallet = CantonWallet::new(config).await?;
    
    // Get balance
    let balance = wallet.balance().await?;
    println!("Balance: {}", balance.total_amount);
    
    Ok(())
}
```

## Documentation

- [API Documentation](https://docs.rs/canton-wallet-sdk)
- [Getting Started Guide](./docs/getting_started.md)
- [Architecture Documentation](./docs/architecture.md)
- [Security Best Practices](./docs/security.md)

## Project Structure

```
canton-wallet-sdk/
├── crates/
│   ├── canton-wallet/              # Main facade crate
│   ├── canton-wallet-core/         # Core types and traits
│   ├── canton-wallet-crypto/       # Cryptographic operations
│   ├── canton-ledger-api/          # Ledger API integration
│   ├── canton-wallet-transactions/  # Transaction management
│   ├── canton-wallet-contracts/    # Contract management
│   ├── canton-wallet-events/       # Event streaming
│   ├── canton-wallet-security/     # Security features
│   ├── canton-wallet-recovery/     # Recovery mechanisms
│   └── canton-wallet-omnichain/    # OmniChain integration
├── examples/                       # Example applications
├── tests/                          # Integration tests
└── docs/                           # Documentation
```

## Development

### Prerequisites

- Rust 1.85 or higher
- Cargo

### Building

```bash
cargo build --release
```

### Testing

```bash
# Run all tests
cargo test --all

# Run tests with coverage
cargo tarpaulin --out Html
```

### Code Quality

```bash
# Format code
cargo fmt --all

# Run linter
cargo clippy --all-targets --all-features -- -D warnings

# Check for vulnerabilities
cargo audit
```

## Contributing

We welcome contributions! Please see our [Contributing Guide](./CONTRIBUTING.md) for details.

## License

This project is licensed under the Apache License 2.0 - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

This project builds upon the excellent work of:

- Digital Asset (Canton Network)
- The Rust Project
- The Ethereum, Solana, Cosmos, and Polkadot communities
- All contributors to the open-source ecosystem

## Support

- [GitHub Issues](https://github.com/your-org/canton-wallet-sdk/issues)
- [GitHub Discussions](https://github.com/your-org/canton-wallet-sdk/discussions)
- [Documentation](https://docs.rs/canton-wallet-sdk)
