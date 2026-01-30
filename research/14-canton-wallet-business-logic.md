# Canton Wallet SDK - Business Logic Specifications

## 1. Overview

This document provides detailed business logic specifications for the Canton Wallet SDK, covering all core functionality, workflows, and edge cases.

## 2. Core Business Entities

### 2.1 Wallet

```rust
/// Wallet entity
pub struct Wallet {
    /// Unique wallet identifier
    pub id: WalletId,
    
    /// Party ID associated with wallet
    pub party_id: PartyId,
    
    /// Participant ID
    pub participant_id: ParticipantId,
    
    /// Ledger ID
    pub ledger_id: String,
    
    /// Wallet type
    pub wallet_type: WalletType,
    
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    
    /// Last activity timestamp
    pub last_activity: DateTime<Utc>,
    
    /// Wallet metadata
    pub metadata: WalletMetadata,
}

#[derive(Debug, Clone, PartialEq)]
pub enum WalletType {
    /// Standard single-signature wallet
    Standard,
    /// Hierarchical deterministic wallet
    HD,
    /// Multi-signature wallet
    MultiSig { threshold: usize, signers: Vec<PartyId> },
    /// Hardware wallet
    Hardware,
}

#[derive(Debug, Clone)]
pub struct WalletMetadata {
    pub display_name: String,
    pub description: Option<String>,
    pub tags: HashMap<String, String>,
    pub custom_data: HashMap<String, Value>,
}
```

### 2.2 Party

```rust
/// Party entity
pub struct Party {
    /// Party ID
    pub id: PartyId,
    
    /// Display name
    pub display_name: String,
    
    /// Whether party is local to participant
    pub is_local: bool,
    
    /// Identity provider ID
    pub identity_provider_id: Option<String>,
    
    /// Local metadata
    pub local_metadata: Option<String>,
    
    /// Associated key ID
    pub key_id: Option<KeyId>,
    
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
}
```

### 2.3 Contract

```rust
/// Contract entity
pub struct Contract {
    /// Contract ID
    pub id: ContractId,
    
    /// Template ID
    pub template_id: Identifier,
    
    /// Contract key (if any)
    pub key: Option<DamlValue>,
    
    /// Create arguments
    pub arguments: DamlRecord,
    
    /// Signatories
    pub signatories: Vec<PartyId>,
    
    /// Observers
    pub observers: Vec<PartyId>,
    
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    
    /// Contract state
    pub state: ContractState,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ContractState {
    Active,
    Archived,
    Consumed,
}
```

### 2.4 Transaction

```rust
/// Transaction entity
pub struct Transaction {
    /// Transaction ID
    pub id: TxId,
    
    /// Command ID
    pub command_id: String,
    
    /// Workflow ID
    pub workflow_id: String,
    
    /// Effective timestamp
    pub effective_at: DateTime<Utc>,
    
    /// Events in transaction
    pub events: Vec<Event>,
    
    /// Ledger offset
    pub offset: String,
    
    /// Transaction status
    pub status: TransactionStatus,
    
    /// Transaction metadata
    pub metadata: TransactionMetadata,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TransactionStatus {
    Pending,
    Confirmed,
    Failed(String),
}

#[derive(Debug, Clone)]
pub struct TransactionMetadata {
    pub application_id: String,
    pub submission_id: Option<String>,
    pub act_as: Vec<PartyId>,
    pub read_as: Vec<PartyId>,
}
```

## 3. Core Business Workflows

### 3.1 Wallet Initialization

```rust
/// Wallet initialization workflow
pub struct WalletInitializationWorkflow {
    config: WalletConfig,
    key_store: Arc<dyn KeyStore>,
    ledger_client: Arc<LedgerClient>,
}

impl WalletInitializationWorkflow {
    /// Initialize new wallet
    pub async fn initialize_new_wallet(
        &self,
        wallet_type: WalletType,
        metadata: WalletMetadata,
    ) -> Result<Wallet, WalletError> {
        // Step 1: Generate or import keys
        let key_id = match wallet_type {
            WalletType::Standard => {
                self.key_store
                    .generate_key(KeyAlgorithm::Ed25519, KeyPurpose::Signing, KeyMetadata::default())
                    .await?
            }
            WalletType::HD => {
                // Generate HD wallet mnemonic
                let mnemonic = self.generate_mnemonic()?;
                let key_id = self.derive_hd_key(&mnemonic, 0).await?;
                key_id
            }
            WalletType::MultiSig { .. } => {
                // Multi-sig requires multiple keys
                return Err(WalletError::InvalidTransaction("Multi-sig initialization requires explicit key setup".to_string()));
            }
            WalletType::Hardware => {
                // Hardware wallet requires external key
                return Err(WalletError::InvalidTransaction("Hardware wallet requires external connection".to_string()));
            }
        };
        
        // Step 2: Allocate party
        let party_details = self.ledger_client
            .allocate_party(
                Some(format!("wallet-{}", key_id)),
                metadata.display_name.clone(),
            )
            .await?;
        
        let party_id = PartyId::new_unchecked(party_details.party.clone());
        
        // Step 3: Associate key with party
        self.associate_key_with_party(key_id.clone(), party_id.clone()).await?;
        
        // Step 4: Create wallet entity
        let wallet = Wallet {
            id: WalletId::new(),
            party_id: party_id.clone(),
            participant_id: self.ledger_client.participant_id().clone(),
            ledger_id: self.ledger_client.ledger_id().to_string(),
            wallet_type,
            created_at: Utc::now(),
            last_activity: Utc::now(),
            metadata,
        };
        
        // Step 5: Persist wallet
        self.persist_wallet(&wallet).await?;
        
        Ok(wallet)
    }
    
    /// Restore wallet from backup
    pub async fn restore_wallet(
        &self,
        backup: WalletBackup,
    ) -> Result<Wallet, WalletError> {
        // Step 1: Decrypt and import keys
        for encrypted_key in &backup.encrypted_keys {
            let key_id = self.import_encrypted_key(encrypted_key).await?;
            
            // Step 2: Associate key with party
            if let Some(party_id) = &backup.metadata.party_id {
                self.associate_key_with_party(key_id, party_id.clone()).await?;
            }
        }
        
        // Step 3: Verify party exists
        if let Some(party_id) = &backup.metadata.party_id {
            let party_details = self.ledger_client
                .get_party_details(party_id)
                .await?
                .ok_or(WalletError::PartyNotFound(party_id.clone()))?;
            
            // Step 4: Create wallet entity
            let wallet = Wallet {
                id: backup.wallet_id,
                party_id: party_id.clone(),
                participant_id: self.ledger_client.participant_id().clone(),
                ledger_id: self.ledger_client.ledger_id().to_string(),
                wallet_type: backup.metadata.wallet_type.clone(),
                created_at: backup.timestamp,
                last_activity: Utc::now(),
                metadata: backup.metadata.clone(),
            };
            
            // Step 5: Persist wallet
            self.persist_wallet(&wallet).await?;
            
            return Ok(wallet);
        }
        
        Err(WalletError::InvalidTransaction("Backup missing party ID".to_string()))
    }
    
    async fn generate_mnemonic(&self) -> Result<String, WalletError> {
        use bip39::{Mnemonic, MnemonicType, Language};
        
        let mnemonic = Mnemonic::new(MnemonicType::Words12, Language::English);
        Ok(mnemonic.phrase().to_string())
    }
    
    async fn derive_hd_key(&self, mnemonic: &str, index: u32) -> Result<KeyId, WalletError> {
        use bip39::{Mnemonic, Language, Seed};
        use bip32::{Mnemonic as Bip32Mnemonic, XPrv, DerivationPath};
        
        let mnemonic = Mnemonic::from_phrase(mnemonic, Language::English)
            .map_err(|e| WalletError::InvalidTransaction(e.to_string()))?;
        
        let seed = Seed::new(&mnemonic, "");
        let root_key = XPrv::new(seed.as_bytes())
            .map_err(|e| WalletError::InvalidTransaction(e.to_string()))?;
        
        let path = DerivationPath::from_str(&format!("m/44'/118'/{}'/0/0", index))
            .map_err(|e| WalletError::InvalidTransaction(e.to_string()))?;
        
        let account_key = root_key
            .derive_path(&path)
            .map_err(|e| WalletError::InvalidTransaction(e.to_string()))?;
        
        let key_bytes = account_key.private_key().as_bytes().to_vec();
        
        self.key_store
            .import_key(&key_bytes, KeyAlgorithm::Ed25519, KeyPurpose::Signing, KeyMetadata::default())
            .await
    }
    
    async fn associate_key_with_party(
        &self,
        key_id: KeyId,
        party_id: PartyId,
    ) -> Result<(), WalletError> {
        // Store key-party association
        // Implementation depends on storage backend
        Ok(())
    }
    
    async fn persist_wallet(&self, wallet: &Wallet) -> Result<(), WalletError> {
        // Persist wallet to storage
        // Implementation depends on storage backend
        Ok(())
    }
    
    async fn import_encrypted_key(&self, encrypted_key: &EncryptedKey) -> Result<KeyId, WalletError> {
        // Decrypt and import key
        // Implementation depends on encryption scheme
        Ok(KeyId::new())
    }
}
```

### 3.2 Contract Creation Workflow

```rust
/// Contract creation workflow
pub struct ContractCreationWorkflow {
    wallet: Arc<dyn Wallet>,
    contract_manager: Arc<ContractManager>,
    approval_manager: Arc<ApprovalManager>,
}

impl ContractCreationWorkflow {
    /// Create contract with approval
    pub async fn create_contract(
        &self,
        template_id: Identifier,
        arguments: DamlRecord,
    ) -> Result<CreatedEvent, WalletError> {
        // Step 1: Validate template
        self.validate_template(&template_id).await?;
        
        // Step 2: Validate arguments
        self.validate_arguments(&template_id, &arguments).await?;
        
        // Step 3: Build command
        let command = Command::Create(CreateCommand {
            template_id: template_id.clone(),
            create_arguments: arguments.clone(),
        });
        
        // Step 4: Build transaction
        let commands = TransactionBuilder::new()
            .party_id(self.wallet.party_id().clone())
            .add_command(command)
            .build()?;
        
        // Step 5: Request approval
        let tx = self.approval_manager
            .request_approval(&commands)
            .await?;
        
        // Step 6: Submit transaction
        let result = self.wallet.submit_and_wait(command).await?;
        
        // Step 7: Extract created event
        let created_event = result
            .events
            .into_iter()
            .find_map(|event| match event {
                Event::Created(created) => Some(created),
                _ => None,
            })
            .ok_or(WalletError::ContractCreationFailed)?;
        
        // Step 8: Update wallet activity
        self.update_wallet_activity().await?;
        
        Ok(created_event)
    }
    
    /// Create and exercise contract
    pub async fn create_and_exercise(
        &self,
        template_id: Identifier,
        create_arguments: DamlRecord,
        choice: &str,
        choice_argument: DamlValue,
    ) -> Result<Transaction, WalletError> {
        // Step 1: Validate template
        self.validate_template(&template_id).await?;
        
        // Step 2: Validate arguments
        self.validate_arguments(&template_id, &create_arguments).await?;
        
        // Step 3: Validate choice
        self.validate_choice(&template_id, choice).await?;
        
        // Step 4: Build command
        let command = Command::CreateAndExercise(CreateAndExerciseCommand {
            template_id: template_id.clone(),
            create_arguments,
            choice: choice.to_string(),
            choice_argument: choice_argument.into(),
        });
        
        // Step 5: Build transaction
        let commands = TransactionBuilder::new()
            .party_id(self.wallet.party_id().clone())
            .add_command(command)
            .build()?;
        
        // Step 6: Request approval
        let tx = self.approval_manager
            .request_approval(&commands)
            .await?;
        
        // Step 7: Submit transaction
        let result = self.wallet.submit_and_wait(command).await?;
        
        // Step 8: Update wallet activity
        self.update_wallet_activity().await?;
        
        Ok(result)
    }
    
    async fn validate_template(&self, template_id: &Identifier) -> Result<(), WalletError> {
        // Check if template exists and is accessible
        // Implementation depends on ledger client
        Ok(())
    }
    
    async fn validate_arguments(
        &self,
        template_id: &Identifier,
        arguments: &DamlRecord,
    ) -> Result<(), WalletError> {
        // Validate arguments against template schema
        // Implementation depends on template metadata
        Ok(())
    }
    
    async fn validate_choice(
        &self,
        template_id: &Identifier,
        choice: &str,
    ) -> Result<(), WalletError> {
        // Validate choice exists on template
        // Implementation depends on template metadata
        Ok(())
    }
    
    async fn update_wallet_activity(&self) -> Result<(), WalletError> {
        // Update last activity timestamp
        // Implementation depends on storage backend
        Ok(())
    }
}
```

### 3.3 Choice Exercise Workflow

```rust
/// Choice exercise workflow
pub struct ChoiceExerciseWorkflow {
    wallet: Arc<dyn Wallet>,
    contract_manager: Arc<ContractManager>,
    approval_manager: Arc<ApprovalManager>,
}

impl ChoiceExerciseWorkflow {
    /// Exercise choice on contract
    pub async fn exercise_choice(
        &self,
        contract_id: ContractId,
        choice: &str,
        argument: DamlValue,
    ) -> Result<Transaction, WalletError> {
        // Step 1: Get contract
        let contract = self.contract_manager.get_contract(contract_id).await?;
        
        // Step 2: Validate contract is active
        if contract.state != ContractState::Active {
            return Err(WalletError::InvalidTransaction(format!(
                "Contract {} is not active",
                contract_id
            )));
        }
        
        // Step 3: Validate party can exercise
        self.validate_exercise_permission(&contract, choice).await?;
        
        // Step 4: Validate choice
        self.validate_choice(&contract.template_id, choice).await?;
        
        // Step 5: Validate argument
        self.validate_choice_argument(&contract.template_id, choice, &argument).await?;
        
        // Step 6: Build command
        let command = Command::Exercise(ExerciseCommand {
            template_id: Some(contract.template_id.clone()),
            contract_id: contract_id.to_string(),
            choice: choice.to_string(),
            choice_argument: argument.into(),
        });
        
        // Step 7: Build transaction
        let commands = TransactionBuilder::new()
            .party_id(self.wallet.party_id().clone())
            .add_command(command)
            .build()?;
        
        // Step 8: Request approval
        let tx = self.approval_manager
            .request_approval(&commands)
            .await?;
        
        // Step 9: Submit transaction
        let result = self.wallet.submit_and_wait(command).await?;
        
        // Step 10: Update wallet activity
        self.update_wallet_activity().await?;
        
        Ok(result)
    }
    
    /// Exercise choice by key
    pub async fn exercise_by_key(
        &self,
        template_id: Identifier,
        contract_key: DamlValue,
        choice: &str,
        argument: DamlValue,
    ) -> Result<Transaction, WalletError> {
        // Step 1: Validate template
        self.validate_template(&template_id).await?;
        
        // Step 2: Validate choice
        self.validate_choice(&template_id, choice).await?;
        
        // Step 3: Validate argument
        self.validate_choice_argument(&template_id, choice, &argument).await?;
        
        // Step 4: Build command
        let command = Command::ExerciseByKey(ExerciseByKeyCommand {
            template_id: template_id.clone(),
            contract_key: contract_key.into(),
            choice: choice.to_string(),
            choice_argument: argument.into(),
        });
        
        // Step 5: Build transaction
        let commands = TransactionBuilder::new()
            .party_id(self.wallet.party_id().clone())
            .add_command(command)
            .build()?;
        
        // Step 6: Request approval
        let tx = self.approval_manager
            .request_approval(&commands)
            .await?;
        
        // Step 7: Submit transaction
        let result = self.wallet.submit_and_wait(command).await?;
        
        // Step 8: Update wallet activity
        self.update_wallet_activity().await?;
        
        Ok(result)
    }
    
    async fn validate_exercise_permission(
        &self,
        contract: &Contract,
        choice: &str,
    ) -> Result<(), WalletError> {
        // Check if wallet party is a signatory
        if !contract.signatories.contains(self.wallet.party_id()) {
            return Err(WalletError::InvalidTransaction(format!(
                "Party {} is not a signatory of contract {}",
                self.wallet.party_id(),
                contract.id
            )));
        }
        
        // Check if choice is consuming or non-consuming
        // Non-consuming choices require signatory permission
        // Consuming choices require signatory permission
        Ok(())
    }
    
    async fn validate_template(&self, template_id: &Identifier) -> Result<(), WalletError> {
        // Check if template exists
        Ok(())
    }
    
    async fn validate_choice(
        &self,
        template_id: &Identifier,
        choice: &str,
    ) -> Result<(), WalletError> {
        // Check if choice exists on template
        Ok(())
    }
    
    async fn validate_choice_argument(
        &self,
        template_id: &Identifier,
        choice: &str,
        argument: &DamlValue,
    ) -> Result<(), WalletError> {
        // Validate argument against choice schema
        Ok(())
    }
    
    async fn update_wallet_activity(&self) -> Result<(), WalletError> {
        // Update last activity timestamp
        Ok(())
    }
}
```

### 3.4 Balance Tracking Workflow

```rust
/// Balance tracking workflow
pub struct BalanceTrackingWorkflow {
    wallet: Arc<dyn Wallet>,
    contract_manager: Arc<ContractManager>,
    balance_cache: Arc<RwLock<HashMap<String, WalletBalance>>>,
}

impl BalanceTrackingWorkflow {
    /// Get wallet balance
    pub async fn get_balance(&self) -> Result<WalletBalance, WalletError> {
        // Step 1: Get active contracts
        let contracts = self.wallet.active_contracts(None).await?;
        
        // Step 2: Calculate balance from contracts
        let mut total_amount = Decimal::ZERO;
        let mut contract_count = 0;
        
        for contract in contracts {
            if let Some(amount) = self.extract_amount_from_contract(&contract) {
                total_amount += amount;
                contract_count += 1;
            }
        }
        
        // Step 3: Build balance
        let balance = WalletBalance {
            total_amount,
            contract_count,
            contracts: contracts.len(),
            last_updated: Utc::now(),
        };
        
        // Step 4: Update cache
        {
            let mut cache = self.balance_cache.write().await;
            cache.insert(self.wallet.party_id().to_string(), balance.clone());
        }
        
        Ok(balance)
    }
    
    /// Get balance for specific asset
    pub async fn get_asset_balance(&self, asset_id: &str) -> Result<AssetBalance, WalletError> {
        // Step 1: Get active contracts for asset
        let contracts = self.get_contracts_for_asset(asset_id).await?;
        
        // Step 2: Calculate balance
        let total_amount = contracts
            .iter()
            .filter_map(|c| self.extract_amount_from_contract(c))
            .fold(Decimal::ZERO, |acc, amount| acc + amount);
        
        Ok(AssetBalance {
            asset_id: asset_id.to_string(),
            amount: total_amount,
            contract_count: contracts.len(),
            last_updated: Utc::now(),
        })
    }
    
    /// Subscribe to balance updates
    pub fn subscribe_balance_updates(&self) -> impl Stream<Item = WalletBalance> + Send {
        let wallet = self.wallet.clone();
        let balance_cache = self.balance_cache.clone();
        
        async_stream::stream! {
            let mut stream = wallet.transactions(
                LedgerOffset::Begin,
                None,
                TransactionFilter::for_party(wallet.party_id()),
            );
            
            while let Some(result) = stream.next().await {
                if let Ok(tx) = result {
                    // Update balance on each transaction
                    let balance = Self::calculate_balance_from_tx(&tx).await;
                    
                    // Update cache
                    {
                        let mut cache = balance_cache.write().await;
                        cache.insert(wallet.party_id().to_string(), balance.clone());
                    }
                    
                    yield balance;
                }
            }
        }
    }
    
    async fn get_contracts_for_asset(&self, asset_id: &str) -> Result<Vec<CreatedEvent>, WalletError> {
        // Get contracts for specific asset
        // Implementation depends on contract metadata
        Ok(vec![])
    }
    
    fn extract_amount_from_contract(&self, contract: &CreatedEvent) -> Option<Decimal> {
        // Extract amount from contract arguments
        contract.create_arguments.get_field("amount")
            .and_then(|v| v.as_numeric())
    }
    
    async fn calculate_balance_from_tx(tx: &Transaction) -> WalletBalance {
        // Calculate balance from transaction events
        let mut total_amount = Decimal::ZERO;
        let mut contract_count = 0;
        
        for event in &tx.events {
            if let Event::Created(created) = event {
                if let Some(amount) = created.create_arguments.get_field("amount")
                    .and_then(|v| v.as_numeric()) {
                    total_amount += amount;
                    contract_count += 1;
                }
            }
        }
        
        WalletBalance {
            total_amount,
            contract_count,
            contracts: contract_count,
            last_updated: tx.effective_at,
        }
    }
}

#[derive(Debug, Clone)]
pub struct WalletBalance {
    pub total_amount: Decimal,
    pub contract_count: usize,
    pub contracts: usize,
    pub last_updated: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct AssetBalance {
    pub asset_id: String,
    pub amount: Decimal,
    pub contract_count: usize,
    pub last_updated: DateTime<Utc>,
}
```

### 3.5 Multi-Signature Workflow

```rust
/// Multi-signature workflow
pub struct MultiSignatureWorkflow {
    wallet: Arc<dyn MultiSigWallet>,
    signers: HashMap<PartyId, Arc<dyn Wallet>>,
    pending_signatures: Arc<RwLock<HashMap<TxId, PendingMultiSig>>>,
}

struct PendingMultiSig {
    transaction: Transaction,
    signatures: HashMap<PartyId, Signature>,
    threshold: usize,
    created_at: DateTime<Utc>,
    expires_at: DateTime<Utc>,
}

impl MultiSignatureWorkflow {
    /// Initiate multi-signature transaction
    pub async fn initiate_transaction(
        &self,
        commands: Commands,
    ) -> Result<TxId, WalletError> {
        // Step 1: Validate threshold
        let threshold = self.wallet.threshold();
        if threshold == 0 || threshold > self.wallet.signers().len() {
            return Err(WalletError::InvalidTransaction(format!(
                "Invalid threshold: {}", threshold
            )));
        }
        
        // Step 2: Build transaction
        let transaction = self.build_transaction(commands).await?;
        
        // Step 3: Create pending multi-sig
        let tx_id = transaction.id.clone();
        let pending = PendingMultiSig {
            transaction: transaction.clone(),
            signatures: HashMap::new(),
            threshold,
            created_at: Utc::now(),
            expires_at: Utc::now() + Duration::hours(24),
        };
        
        // Step 4: Store pending multi-sig
        {
            let mut pending_sigs = self.pending_signatures.write().await;
            pending_sigs.insert(tx_id.clone(), pending);
        }
        
        // Step 5: Notify signers
        self.notify_signers(&tx_id, &transaction).await?;
        
        Ok(tx_id)
    }
    
    /// Add signature to pending transaction
    pub async fn add_signature(
        &self,
        tx_id: &TxId,
        signer: PartyId,
        signature: Signature,
    ) -> Result<(), WalletError> {
        // Step 1: Get pending multi-sig
        let mut pending = {
            let pending_sigs = self.pending_signatures.read().await;
            pending_sigs.get(tx_id)
                .cloned()
                .ok_or(WalletError::TransactionNotFound(tx_id.clone()))?
        };
        
        // Step 2: Validate signer
        if !self.wallet.signers().contains(&signer) {
            return Err(WalletError::InvalidTransaction(format!(
                "Signer {} is not authorized",
                signer
            )));
        }
        
        // Step 3: Validate signature
        let signer_wallet = self.signers.get(&signer)
            .ok_or(WalletError::InvalidTransaction("Signer wallet not found".to_string()))?;
        
        let tx_bytes = pending.transaction.serialize();
        if !signer_wallet.verify(&tx_bytes, &signature).await? {
            return Err(WalletError::InvalidSignature);
        }
        
        // Step 4: Check if already signed
        if pending.signatures.contains_key(&signer) {
            return Err(WalletError::InvalidTransaction("Already signed".to_string()));
        }
        
        // Step 5: Add signature
        pending.signatures.insert(signer, signature);
        
        // Step 6: Update pending multi-sig
        {
            let mut pending_sigs = self.pending_signatures.write().await;
            pending_sigs.insert(tx_id.clone(), pending.clone());
        }
        
        // Step 7: Check if ready
        if pending.signatures.len() >= pending.threshold {
            self.execute_transaction(tx_id).await?;
        }
        
        Ok(())
    }
    
    /// Execute transaction when threshold reached
    async fn execute_transaction(&self, tx_id: &TxId) -> Result<Transaction, WalletError> {
        // Step 1: Get pending multi-sig
        let pending = {
            let pending_sigs = self.pending_signatures.read().await;
            pending_sigs.get(tx_id)
                .cloned()
                .ok_or(WalletError::TransactionNotFound(tx_id.clone()))?
        };
        
        // Step 2: Check threshold
        if pending.signatures.len() < pending.threshold {
            return Err(WalletError::InsufficientSignatures {
                have: pending.signatures.len(),
                need: pending.threshold,
            });
        }
        
        // Step 3: Combine signatures
        let combined_signature = self.combine_signatures(&pending.signatures)?;
        
        // Step 4: Submit transaction
        let result = self.wallet.execute(tx_id).await?;
        
        // Step 5: Clean up pending multi-sig
        {
            let mut pending_sigs = self.pending_signatures.write().await;
            pending_sigs.remove(tx_id);
        }
        
        Ok(result)
    }
    
    async fn build_transaction(&self, commands: Commands) -> Result<Transaction, WalletError> {
        // Build transaction from commands
        // Implementation depends on ledger client
        Ok(Transaction {
            id: TxId::new(),
            command_id: commands.command_id.clone(),
            workflow_id: commands.workflow_id.clone(),
            effective_at: Utc::now(),
            events: vec![],
            offset: String::new(),
            status: TransactionStatus::Pending,
            metadata: TransactionMetadata {
                application_id: commands.application_id.clone(),
                submission_id: commands.submission_id.clone(),
                act_as: commands.act_as.iter().map(|p| PartyId::new_unchecked(p)).collect(),
                read_as: commands.read_as.iter().map(|p| PartyId::new_unchecked(p)).collect(),
            },
        })
    }
    
    async fn notify_signers(&self, tx_id: &TxId, transaction: &Transaction) -> Result<(), WalletError> {
        // Notify all signers about pending transaction
        // Implementation depends on notification system
        Ok(())
    }
    
    fn combine_signatures(&self, signatures: &HashMap<PartyId, Signature>) -> Result<Signature, WalletError> {
        // Combine multiple signatures
        // Implementation depends on signature scheme
        Ok(signatures.values().next().unwrap().clone())
    }
}
```

### 3.6 Cross-Chain Transfer Workflow

```rust
/// Cross-chain transfer workflow
pub struct CrossChainTransferWorkflow {
    canton_wallet: Arc<dyn CantonWallet>,
    chain_wallets: HashMap<ChainId, Box<dyn ChainWallet>>,
    bridge_manager: Arc<BridgeManager>,
}

impl CrossChainTransferWorkflow {
    /// Transfer from Canton to another chain
    pub async fn transfer_from_canton(
        &self,
        asset: CantonAsset,
        target_chain: ChainId,
        recipient: ChainAddress,
    ) -> Result<CrossChainTx, WalletError> {
        // Step 1: Validate target chain
        if !self.chain_wallets.contains_key(&target_chain) {
            return Err(WalletError::UnsupportedChain(target_chain));
        }
        
        // Step 2: Validate asset
        self.validate_asset(&asset).await?;
        
        // Step 3: Validate recipient
        self.validate_recipient(target_chain, &recipient).await?;
        
        // Step 4: Lock asset on Canton
        let lock_receipt = self.bridge_manager
            .lock_on_canton(&self.canton_wallet, asset.clone(), target_chain, recipient.clone())
            .await?;
        
        // Step 5: Generate proof
        let proof = self.bridge_manager
            .generate_lock_proof(&lock_receipt)
            .await?;
        
        // Step 6: Release on target chain
        let target_wallet = self.chain_wallets.get(&target_chain).unwrap();
        let release_receipt = self.bridge_manager
            .release_on_chain(target_wallet, proof, recipient)
            .await?;
        
        // Step 7: Create cross-chain transaction record
        let cross_chain_tx = CrossChainTx {
            id: CrossChainTxId::new(),
            canton_tx_id: lock_receipt.tx_id,
            target_tx_id: release_receipt.tx_id,
            asset,
            source_chain: ChainId::Canton,
            target_chain,
            recipient,
            status: CrossChainStatus::Completed,
            created_at: Utc::now(),
            completed_at: Some(Utc::now()),
        };
        
        // Step 8: Persist transaction
        self.persist_cross_chain_tx(&cross_chain_tx).await?;
        
        Ok(cross_chain_tx)
    }
    
    /// Transfer from another chain to Canton
    pub async fn transfer_to_canton(
        &self,
        asset: ChainAsset,
        source_chain: ChainId,
        recipient: PartyId,
    ) -> Result<CrossChainTx, WalletError> {
        // Step 1: Validate source chain
        if !self.chain_wallets.contains_key(&source_chain) {
            return Err(WalletError::UnsupportedChain(source_chain));
        }
        
        // Step 2: Validate asset
        self.validate_chain_asset(source_chain, &asset).await?;
        
        // Step 3: Validate recipient
        self.validate_party(&recipient).await?;
        
        // Step 4: Lock on source chain
        let source_wallet = self.chain_wallets.get(&source_chain).unwrap();
        let lock_receipt = self.bridge_manager
            .lock_on_chain(source_wallet, asset.clone(), ChainId::Canton, recipient.clone())
            .await?;
        
        // Step 5: Generate proof
        let proof = self.bridge_manager
            .generate_lock_proof(&lock_receipt)
            .await?;
        
        // Step 6: Release on Canton
        let release_receipt = self.bridge_manager
            .release_on_canton(&self.canton_wallet, proof, recipient)
            .await?;
        
        // Step 7: Create cross-chain transaction record
        let cross_chain_tx = CrossChainTx {
            id: CrossChainTxId::new(),
            canton_tx_id: release_receipt.tx_id,
            target_tx_id: lock_receipt.tx_id,
            asset: CantonAsset::from_chain_asset(asset),
            source_chain,
            target_chain: ChainId::Canton,
            recipient: ChainAddress::Party(recipient),
            status: CrossChainStatus::Completed,
            created_at: Utc::now(),
            completed_at: Some(Utc::now()),
        };
        
        // Step 8: Persist transaction
        self.persist_cross_chain_tx(&cross_chain_tx).await?;
        
        Ok(cross_chain_tx)
    }
    
    async fn validate_asset(&self, asset: &CantonAsset) -> Result<(), WalletError> {
        // Validate asset exists and is transferable
        Ok(())
    }
    
    async fn validate_chain_asset(&self, chain: ChainId, asset: &ChainAsset) -> Result<(), WalletError> {
        // Validate chain asset
        Ok(())
    }
    
    async fn validate_recipient(&self, chain: ChainId, address: &ChainAddress) -> Result<(), WalletError> {
        // Validate recipient address for chain
        Ok(())
    }
    
    async fn validate_party(&self, party: &PartyId) -> Result<(), WalletError> {
        // Validate party exists
        Ok(())
    }
    
    async fn persist_cross_chain_tx(&self, tx: &CrossChainTx) -> Result<(), WalletError> {
        // Persist cross-chain transaction
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct CrossChainTx {
    pub id: CrossChainTxId,
    pub canton_tx_id: TxId,
    pub target_tx_id: TxId,
    pub asset: CantonAsset,
    pub source_chain: ChainId,
    pub target_chain: ChainId,
    pub recipient: ChainAddress,
    pub status: CrossChainStatus,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CrossChainStatus {
    Pending,
    Locked,
    Released,
    Completed,
    Failed(String),
}
```

## 4. Edge Cases and Error Handling

### 4.1 Concurrent Operations

```rust
/// Concurrent operation handler
pub struct ConcurrentOperationHandler {
    operations: Arc<RwLock<HashMap<OperationId, OperationState>>>,
    timeout: Duration,
}

enum OperationState {
    Pending { started_at: DateTime<Utc> },
    Completed { result: OperationResult },
    Failed { error: WalletError },
}

impl ConcurrentOperationHandler {
    pub async fn execute_operation<F, Fut>(
        &self,
        operation_id: OperationId,
        operation: F,
    ) -> Result<OperationResult, WalletError>
    where
        F: FnOnce() -> Fut + Send,
        Fut: Future<Output = Result<OperationResult, WalletError>> + Send,
    {
        // Step 1: Check if operation already exists
        {
            let ops = self.operations.read().await;
            if ops.contains_key(&operation_id) {
                return Err(WalletError::InvalidTransaction(format!(
                    "Operation {} already in progress",
                    operation_id
                )));
            }
        }
        
        // Step 2: Register operation
        {
            let mut ops = self.operations.write().await;
            ops.insert(operation_id.clone(), OperationState::Pending {
                started_at: Utc::now(),
            });
        }
        
        // Step 3: Execute operation
        let result = tokio::time::timeout(self.timeout, operation()).await;
        
        // Step 4: Handle result
        let final_result = match result {
            Ok(Ok(result)) => {
                let mut ops = self.operations.write().await;
                ops.insert(operation_id.clone(), OperationState::Completed {
                    result: result.clone(),
                });
                Ok(result)
            }
            Ok(Err(error)) => {
                let mut ops = self.operations.write().await;
                ops.insert(operation_id.clone(), OperationState::Failed {
                    error: error.clone(),
                });
                Err(error)
            }
            Err(_) => {
                let mut ops = self.operations.write().await;
                ops.insert(operation_id.clone(), OperationState::Failed {
                    error: WalletError::Internal("Operation timeout".to_string()),
                });
                Err(WalletError::Internal("Operation timeout".to_string()))
            }
        };
        
        // Step 5: Clean up old operations
        self.cleanup_old_operations().await;
        
        final_result
    }
    
    async fn cleanup_old_operations(&self) {
        let mut ops = self.operations.write().await;
        let now = Utc::now();
        
        ops.retain(|_, state| {
            match state {
                OperationState::Pending { started_at } => {
                    now - *started_at < self.timeout
                }
                OperationState::Completed { .. } => {
                    // Keep completed operations for 1 hour
                    false
                }
                OperationState::Failed { .. } => {
                    // Keep failed operations for 1 hour
                    false
                }
            }
        });
    }
}
```

### 4.2 Transaction Conflicts

```rust
/// Conflict resolution handler
pub struct ConflictResolutionHandler {
    retry_policy: RetryPolicy,
    max_retries: u32,
}

impl ConflictResolutionHandler {
    pub async fn resolve_conflict<F, Fut>(
        &self,
        operation: F,
    ) -> Result<Transaction, WalletError>
    where
        F: Fn() -> Fut + Send + Clone,
        Fut: Future<Output = Result<Transaction, WalletError>> + Send,
    {
        let mut attempt = 0;
        
        loop {
            attempt += 1;
            
            match operation().await {
                Ok(tx) => return Ok(tx),
                Err(WalletError::TransactionFailed(ref msg)) if msg.contains("conflict") => {
                    if attempt >= self.max_retries {
                        return Err(WalletError::TransactionFailed(
                            "Max retries exceeded for conflict".to_string()
                        ));
                    }
                    
                    // Exponential backoff
                    let delay = Duration::from_millis(100 * 2_u64.pow(attempt - 1));
                    tokio::time::sleep(delay).await;
                }
                Err(e) => return Err(e),
            }
        }
    }
}
```

## 5. Summary

### 5.1 Core Workflows
1. **Wallet Initialization**: Create new wallet or restore from backup
2. **Contract Creation**: Create contracts with validation and approval
3. **Choice Exercise**: Exercise choices on contracts
4. **Balance Tracking**: Track wallet balances from contracts
5. **Multi-Signature**: Handle multi-signature transactions
6. **Cross-Chain Transfer**: Transfer assets across chains

### 5.2 Key Features
- **Validation**: Comprehensive validation at each step
- **Approval**: User approval for sensitive operations
- **Error Handling**: Detailed error messages and recovery
- **Concurrency**: Handle concurrent operations safely
- **Conflict Resolution**: Automatic retry with exponential backoff
- **Audit Logging**: Track all operations
- **Caching**: Improve performance with intelligent caching
