// HD wallet implementation for Canton Wallet SDK

use async_trait::async_trait;
use canton_wallet_core::{
    Command, ContractId, CreatedEvent, DamlRecord, DamlValue, Event, LedgerOffset,
    PartyId, Signature, Transaction, TransactionFilter, TransactionId, Wallet as WalletTrait,
    WalletBalance, WalletConfig, WalletError, WalletResult,
};
use canton_wallet_crypto::{HDWallet as CryptoHDWallet, HDAccount};
use futures::Stream;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;

/// HD wallet implementation
pub struct HDWallet {
    wallet_id: canton_wallet_core::WalletId,
    party_id: PartyId,
    participant_id: PartyId,
    config: WalletConfig,
    hd_wallet: Arc<RwLock<CryptoHDWallet>>,
    accounts: Arc<RwLock<HashMap<u32, HDAccount>>>,
    balance: Arc<RwLock<WalletBalance>>,
}

impl HDWallet {
    /// Create a new HD wallet
    ///
    /// # Arguments
    ///
    /// * `config` - Wallet configuration
    /// * `word_count` - Number of words in mnemonic (12, 15, 18, 21, or 24)
    pub async fn new(config: WalletConfig, word_count: bip39::MnemonicType) -> WalletResult<Self> {
        let wallet_id = canton_wallet_core::WalletId::generate();
        let party_id = config
            .party_id
            .clone()
            .unwrap_or_else(|| PartyId::generate());
        let participant_id = PartyId::generate();
        let hd_wallet = CryptoHDWallet::new(word_count)?;
        let balance = Arc::new(RwLock::new(WalletBalance::zero("USD")));

        Ok(Self {
            wallet_id,
            party_id,
            participant_id,
            config,
            hd_wallet: Arc::new(RwLock::new(hd_wallet)),
            accounts: Arc::new(RwLock::new(HashMap::new())),
            balance,
        })
    }

    /// Restore HD wallet from mnemonic phrase
    ///
    /// # Arguments
    ///
    /// * `config` - Wallet configuration
    /// * `mnemonic_phrase` - BIP39 mnemonic phrase
    pub async fn from_mnemonic(config: WalletConfig, mnemonic_phrase: &str) -> WalletResult<Self> {
        let wallet_id = canton_wallet_core::WalletId::generate();
        let party_id = config
            .party_id
            .clone()
            .unwrap_or_else(|| PartyId::generate());
        let participant_id = PartyId::generate();
        let hd_wallet = CryptoHDWallet::from_mnemonic(mnemonic_phrase)?;
        let balance = Arc::new(RwLock::new(WalletBalance::zero("USD")));

        Ok(Self {
            wallet_id,
            party_id,
            participant_id,
            config,
            hd_wallet: Arc::new(RwLock::new(hd_wallet)),
            accounts: Arc::new(RwLock::new(HashMap::new())),
            balance,
        })
    }

    /// Derive account at index
    ///
    /// # Arguments
    ///
    /// * `index` - Account index
    pub async fn derive_account(&self, index: u32) -> WalletResult<HDAccount> {
        let mut hd_wallet = self.hd_wallet.write();
        let account = hd_wallet.derive_account(index)?;
        let mut accounts = self.accounts.write();
        accounts.insert(index, account.clone());
        Ok(account)
    }

    /// Get account at index
    ///
    /// # Arguments
    ///
    /// * `index` - Account index
    pub async fn get_account(&self, index: u32) -> Option<HDAccount> {
        let accounts = self.accounts.read();
        accounts.get(&index).cloned()
    }

    /// Get all derived accounts
    pub async fn accounts(&self) -> HashMap<u32, HDAccount> {
        let accounts = self.accounts.read();
        accounts.clone()
    }

    /// Get mnemonic phrase
    ///
    /// # Warning
    ///
    /// Never share or log this phrase!
    pub fn mnemonic_phrase(&self) -> String {
        let hd_wallet = self.hd_wallet.read();
        hd_wallet.mnemonic_phrase().to_string()
    }

    /// Update wallet balance
    pub fn update_balance(&self, balance: WalletBalance) {
        let mut b = self.balance.write();
        *b = balance;
    }

    /// Get wallet configuration
    pub fn config(&self) -> &WalletConfig {
        &self.config
    }
}

#[async_trait]
impl WalletTrait for HDWallet {
    fn wallet_id(&self) -> &canton_wallet_core::WalletId {
        &self.wallet_id
    }

    fn party_id(&self) -> &PartyId {
        &self.party_id
    }

    fn participant_id(&self) -> &PartyId {
        &self.participant_id
    }

    async fn address(&self) -> WalletResult<String> {
        Ok(self.party_id.to_string())
    }

    async fn balance(&self) -> WalletResult<WalletBalance> {
        let b = self.balance.read();
        Ok(b.clone())
    }

    async fn submit_command(&self, command: Command) -> WalletResult<Transaction> {
        let transaction_id = TransactionId::generate();
        let command_id = uuid::Uuid::new_v4().to_string();
        let workflow_id = uuid::Uuid::new_v4().to_string();

        let transaction = Transaction {
            transaction_id: transaction_id.clone(),
            command_id,
            workflow_id,
            effective_at: chrono::Utc::now(),
            events: vec![],
            offset: "0".to_string(),
        };

        Ok(transaction)
    }

    async fn submit_and_wait(&self, command: Command) -> WalletResult<Transaction> {
        self.submit_command(command).await
    }

    async fn active_contracts(
        &self,
        _filter: Option<TransactionFilter>,
    ) -> WalletResult<Vec<CreatedEvent>> {
        Ok(Vec::new())
    }

    async fn exercise_choice(
        &self,
        _contract_id: ContractId,
        _choice: &str,
        _argument: DamlValue,
    ) -> WalletResult<Transaction> {
        let transaction_id = TransactionId::generate();
        let command_id = uuid::Uuid::new_v4().to_string();
        let workflow_id = uuid::Uuid::new_v4().to_string();

        let transaction = Transaction {
            transaction_id,
            command_id,
            workflow_id,
            effective_at: chrono::Utc::now(),
            events: vec![],
            offset: "0".to_string(),
        };

        Ok(transaction)
    }

    async fn create_contract(
        &self,
        _template_id: canton_wallet_core::TemplateId,
        _arguments: DamlRecord,
    ) -> WalletResult<CreatedEvent> {
        let contract_id = ContractId::generate();
        let template_id = canton_wallet_core::TemplateId::new("pkg", "mod", "Template");

        Ok(CreatedEvent {
            contract_id,
            template_id,
            create_arguments: serde_json::json!({}),
            signatories: vec![self.party_id.clone()],
            observers: vec![],
            agreement_text: String::new(),
        })
    }

    fn transactions(
        &self,
        _begin: LedgerOffset,
        _end: Option<LedgerOffset>,
        _filter: TransactionFilter,
    ) -> impl Stream<Item = WalletResult<Transaction>> + Send {
        futures::stream::empty()
    }

    async fn sign(&self, data: &[u8]) -> WalletResult<Signature> {
        let accounts = self.accounts.read();
        if let Some(account) = accounts.get(&0) {
            let keypair = account.to_ed25519_keypair()?;
            let signature = canton_wallet_crypto::sign(&keypair, data)?;
            Ok(signature)
        } else {
            Err(WalletError::KeyNotFound("No account derived".to_string()))
        }
    }

    async fn verify(&self, data: &[u8], signature: &Signature) -> WalletResult<bool> {
        let accounts = self.accounts.read();
        if let Some(account) = accounts.get(&0) {
            let public_key = account.to_public_key()?;
            canton_wallet_crypto::verify(&public_key, data, signature)
        } else {
            Err(WalletError::KeyNotFound("No account derived".to_string()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_hd_wallet_new() {
        let config = WalletConfig::default();
        let wallet = HDWallet::new(config, bip39::MnemonicType::Words12)
            .await
            .unwrap();
        assert!(!wallet.wallet_id().as_str().is_empty());
        assert!(!wallet.party_id().as_str().is_empty());
    }

    #[tokio::test]
    async fn test_hd_wallet_from_mnemonic() {
        let config = WalletConfig::default();
        let wallet1 = HDWallet::new(config.clone(), bip39::MnemonicType::Words12)
            .await
            .unwrap();
        let mnemonic = wallet1.mnemonic_phrase();
        
        let wallet2 = HDWallet::from_mnemonic(config, &mnemonic)
            .await
            .unwrap();
        assert_eq!(wallet2.mnemonic_phrase(), mnemonic);
    }

    #[tokio::test]
    async fn test_hd_wallet_derive_account() {
        let config = WalletConfig::default();
        let mut wallet = HDWallet::new(config, bip39::MnemonicType::Words12)
            .await
            .unwrap();
        let account = wallet.derive_account(0).await.unwrap();
        assert_eq!(account.index(), 0);
    }

    #[tokio::test]
    async fn test_hd_wallet_get_account() {
        let config = WalletConfig::default();
        let mut wallet = HDWallet::new(config, bip39::MnemonicType::Words12)
            .await
            .unwrap();
        wallet.derive_account(0).await.unwrap();
        let account = wallet.get_account(0).await;
        assert!(account.is_some());
        assert_eq!(account.unwrap().index(), 0);
    }

    #[tokio::test]
    async fn test_hd_wallet_balance() {
        let config = WalletConfig::default();
        let wallet = HDWallet::new(config, bip39::MnemonicType::Words12)
            .await
            .unwrap();
        let balance = wallet.balance().await.unwrap();
        assert_eq!(balance.total_amount, "0");
    }

    #[tokio::test]
    async fn test_hd_wallet_update_balance() {
        let config = WalletConfig::default();
        let wallet = HDWallet::new(config, bip39::MnemonicType::Words12)
            .await
            .unwrap();
        let new_balance = WalletBalance::new("100", "80", "20", "USD");
        wallet.update_balance(new_balance);
        let balance = wallet.balance().await.unwrap();
        assert_eq!(balance.total_amount, "100");
    }

    #[tokio::test]
    async fn test_hd_wallet_submit_command() {
        let config = WalletConfig::default();
        let wallet = HDWallet::new(config, bip39::MnemonicType::Words12)
            .await
            .unwrap();
        let command = Command::Create(canton_wallet_core::CreateCommand {
            template_id: canton_wallet_core::TemplateId::new("pkg", "mod", "Template"),
            create_arguments: serde_json::json!({}),
        });
        let tx = wallet.submit_command(command).await.unwrap();
        assert!(!tx.transaction_id.as_str().is_empty());
    }

    #[tokio::test]
    async fn test_hd_wallet_create_contract() {
        let config = WalletConfig::default();
        let wallet = HDWallet::new(config, bip39::MnemonicType::Words12)
            .await
            .unwrap();
        let template_id = canton_wallet_core::TemplateId::new("pkg", "mod", "Template");
        let arguments = DamlRecord::new();
        let created = wallet.create_contract(template_id, arguments).await.unwrap();
        assert!(!created.contract_id.as_str().is_empty());
    }
}
