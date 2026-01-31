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

//! Basic Wallet Example
//!
//! This example demonstrates basic wallet operations including:
//! - Creating a wallet
//! - Getting wallet information
//! - Creating a contract
//! - Exercising a choice
//! - Getting balance

use canton_wallet_sdk::{CantonWallet, WalletConfig};
use canton_wallet_core::{Identifier, DamlRecord, DamlValue};
use canton_observability::{ObservabilityConfig, init_observability};
use tokio;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize observability
    let observability_config = ObservabilityConfig::default();
    init_observability(observability_config)?;

    println!("Canton Wallet SDK - Basic Wallet Example");
    println!("========================================");

    // Create wallet configuration
    let config = WalletConfig {
        ledger_endpoint: "http://localhost:50051".to_string(),
        party_id: "Alice".to_string(),
        participant_id: "participant1".to_string(),
        ..Default::default()
    };

    println!("\nCreating wallet...");
    let wallet = CantonWallet::new(config).await?;
    println!("Wallet created successfully!");
    println!("  Wallet ID: {}", wallet.wallet_id());
    println!("  Party ID: {}", wallet.party_id());
    println!("  Participant ID: {}", wallet.participant_id());

    // Get wallet address
    println!("\nGetting wallet address...");
    let address = wallet.address().await?;
    println!("Wallet address: {}", address);

    // Get balance
    println!("\nGetting balance...");
    let balance = wallet.balance().await?;
    println!("Balance: {} {}", balance.total_amount, balance.currency);

    // Create a contract
    println!("\nCreating contract...");
    let template_id = Identifier::new("Main", "Iou", "Iou");
    let arguments = DamlRecord::new();
    arguments.add_field("issuer", DamlValue::text("Alice"));
    arguments.add_field("owner", DamlValue::text("Bob"));
    arguments.add_field("amount", DamlValue::int64(100));

    let created_event = wallet
        .create_contract(template_id, arguments)
        .await?;
    println!("Contract created successfully!");
    println!("  Contract ID: {}", created_event.contract_id);
    println!("  Template: {}", created_event.template_id);

    // Exercise a choice on the contract
    println!("\nExercising choice...");
    let contract_id = canton_wallet_core::ContractId::new_unchecked(&created_event.contract_id);
    let choice_argument = DamlValue::int64(50);

    let transaction = wallet
        .exercise_choice(contract_id, "Transfer", choice_argument)
        .await?;
    println!("Choice exercised successfully!");
    println!("  Transaction ID: {}", transaction.transaction_id);
    println!("  Events: {}", transaction.events.len());

    println!("\n========================================");
    println!("Example completed successfully!");

    Ok(())
}
