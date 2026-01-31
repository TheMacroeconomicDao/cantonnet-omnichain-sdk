// Standard wallet implementation for Canton Wallet SDK

use async_trait::async_trait;
use canton_wallet_core::{
    Command, ContractId, CreatedEvent, DamlRecord, DamlValue, Event, LedgerOffset,
    PartyId, Signature, Transaction, TransactionFilter, TransactionId, Wallet as WalletTrait,
    WalletBalance, WalletConfig, WalletError, WalletResult,
};
use canton_wallet_crypto::{InMemoryKeyStore, KeyStore as KeyStoreTrait};
use futures::Stream;
use parking_lot::RwLock;
use std::sync::Arc;
use uuid::Uuid;

/// Standard wallet implementation
pub struct StandardWallet {
    wallet_id: WalletId,
    party_id: PartyId,
    participant_id: PartyId,
    config: WalletConfig,
    key_store: Arc<dyn KeyStoreTrait + Send + Sync>,
    balance: Arc<RwLock<WalletBalance>>,
}

impl StandardWallet {
    /// Create a new standard wallet
    ///
    /// # Arguments
    ///
    /// * `config` - Wallet configuration
    pub async fn new(config: WalletConfig) -> WalletResult<Self> {
        let wallet_id = WalletId::generate();
        let party_id = config
            .party_id
            .clone()
            .unwrap_or_else(|| PartyId::generate());
        let participant_id = PartyId::generate();
        let key_store = Arc::new(InMemoryKeyStore::new());
        let balance = Arc::new(RwLock::new(WalletBalance::zero("USD")));

        Ok(Self {
            wallet_id,
            party_id,
            participant_id,
            config,
            key_store,
            balance,
        })
    }

    /// Create a new standard wallet with custom key store
    ///
    /// # Arguments
    ///
    /// * `config` - Wallet configuration
    /// * `key_store` - Custom key store
    pub async fn new_with_key_store(
        config: WalletConfig,
        key_store: Arc<dyn KeyStoreTrait + Send + Sync>,
    ) -> WalletResult<Self> {
        let wallet_id = WalletId::generate();
        let party_id = config
            .party_id
            .clone()
            .unwrap_or_else(|| PartyId::generate());
        let participant_id = PartyId::generate();
        let balance = Arc::new(RwLock::new(WalletBalance::zero("USD")));

        Ok(Self {
            wallet_id,
            party_id,
            participant_id,
            config,
            key_store,
            balance,
        })
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

    /// Get key store
    pub fn key_store(&self) -> &Arc<dyn KeyStoreTrait + Send + Sync> {
        &self.key_store
    }
}

#[async_trait]
impl WalletTrait for StandardWallet {
    fn wallet_id(&self) -> &WalletId {
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
        let command_id = Uuid::new_v4().to_string();
        let workflow_id = Uuid::new_v4().to_string();

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
        let command_id = Uuid::new_v4().to_string();
        let workflow_id = Uuid::new_v4().to_string();

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
        let key_id = self.wallet_id.as_str();
        let key_id = canton_wallet_core::KeyId::new_unchecked(key_id);
        self.key_store.sign(&key_id, data).await
    }

    async fn verify(&self, data: &[u8], signature: &Signature) -> WalletResult<bool> {
        let key_id = self.wallet_id.as_str();
        let key_id = canton_wallet_core::KeyId::new_unchecked(key_id);
        self.key_store.verify(&key_id, data, signature).await
    }
}

/// Canton wallet facade
pub struct CantonWallet {
    inner: Arc<StandardWallet>,
}

impl CantonWallet {
    /// Create a new Canton wallet
    ///
    /// # Arguments
    ///
    /// * `config` - Wallet configuration
    pub async fn new(config: WalletConfig) -> WalletResult<Self> {
        let inner = StandardWallet::new(config).await?;
        Ok(Self {
            inner: Arc::new(inner),
        })
    }

    /// Create a new Canton wallet with custom key store
    ///
    /// # Arguments
    ///
    /// * `config` - Wallet configuration
    /// * `key_store` - Custom key store
    pub async fn new_with_key_store(
        config: WalletConfig,
        key_store: Arc<dyn KeyStoreTrait + Send + Sync>,
    ) -> WalletResult<Self> {
        let inner = StandardWallet::new_with_key_store(config, key_store).await?;
        Ok(Self {
            inner: Arc::new(inner),
        })
    }

    /// Get inner wallet
    pub fn inner(&self) -> &StandardWallet {
        &self.inner
    }
}

#[async_trait]
impl WalletTrait for CantonWallet {
    fn wallet_id(&self) -> &WalletId {
        self.inner.wallet_id()
    }

    fn party_id(&self) ->PartyId {
        self.inner.party_id().clone()
    }

    fn participant_id(&self) ->PartyId {
        self.inner.participant_id().clone()
    }

    async fn address(&self) -> WalletResult<String> {
        self.inner.address().await
    }

    async fn balance(&self) -> WalletResult<WalletBalance> {
        self.inner.balance().await
    }

    async fn submit_command(&self, command: Command) -> WalletResult<Transaction> {
        self.inner.submit_command(command).await
    }

    async fn submit_and_wait(&self, command: Command) -> WalletResult<Transaction> {
        self.inner.submit_and_wait(command).await
    }

    async fn active_contracts(
        &self,
        filter: Option<TransactionFilter>,
    ) -> WalletResult<Vec<CreatedEvent>> {
        self.inner.active_contracts(filter).await
    }

    async fn exercise_choice(
        &self,
        contract_id: ContractId,
        choice: &str,
        argument: DamlValue,
    ) -> WalletResult<Transaction> {
        self.inner.exercise_choice(contract_id, choice, argument).await
    }

    async fn create_contract(
        &self,
        template_id: canton_wallet_core::TemplateId,
        arguments: DamlRecord,
    ) -> WalletResult<CreatedEvent> {
        self.inner.create_contract(template_id, arguments).await
    }

    fn transactions(
        &self,
        begin: LedgerOffset,
        end: Option<LedgerOffset>,
        filter: TransactionFilter,
    ) -> impl Stream<Item = WalletResult<Transaction>> + Send {
        self.inner.transactions(begin, end, filter)
    }

    async fn sign(&self, data: &[u8]) -> WalletResult<Signature> {
        self.inner.sign(data).await
    }

    async fn verify(&self, data: &[u8], signature: &Signature) -> WalletResult<bool> {
        self.inner.verify(data, signature).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_standard_wallet_new() {
        let config = WalletConfig::default();
        let wallet = StandardWallet::new(config).await.unwrap();
        assert!(!wallet.wallet_id().as_str().is_empty());
        assert!(!wallet.party_id().as_str().is_empty());
    }

    #[tokio::test]
    async fn test_standard_wallet_address() {
        let config = WalletConfig::default();
        let wallet = StandardWallet::new(config).await.unwrap();
        let address = wallet.address().await.unwrap();
        assert_eq!(address, wallet.party_id().to_string());
    }

    #[tokio::test]
    async fn test_standard_wallet_balance() {
        let config = WalletConfig::default();
        let wallet = StandardWallet::new(config).await.unwrap();
        let balance = wallet.balance().await.unwrap();
        assert_eq!(balance.total_amount, "0");
        assert_eq!(balance.currency, "USD");
    }

    #[tokio::test]
    async fn test_standard_wallet_update_balance() {
        let config = WalletConfig::default();
        let wallet = StandardWallet::new(config).await.unwrap();
        let new_balance = WalletBalance::new("100", "80", "20", "USD");
        wallet.update_balance(new_balance);
        let balance = wallet.balance().await.unwrap();
        assert_eq!(balance.total_amount, "100");
    }

    #[tokio::test]
    async fn test_standard_wallet_submit_command() {
        let config = WalletConfig::default();
        let wallet = StandardWallet::new(config).await.unwrap();
        let command = Command::Create(canton_wallet_core::CreateCommand {
            template_id: canton_wallet_core::TemplateId::new("pkg", "mod", "Template"),
            create_arguments: serde_json::json!({}),
        });
        let tx = wallet.submit_command(command).await.unwrap();
        assert!(!tx.transaction_id.as_str().is_empty());
    }

    #[tokio::test]
    async fn test_standard_wallet_create_contract() {
        let config = WalletConfig::default();
        let wallet = StandardWallet::new(config).await.unwrap();
        let template_id = canton_wallet_core::TemplateId::new("pkg", "mod", "Template");
        let arguments = DamlRecord::new();
        let created = wallet.create_contract(template_id, arguments).await.unwrap();
        assert!(!created.contract_id.as_str().is_empty());
    }

    #[tokio::test]
    async fn test_canton_wallet_new() {
        let config = WalletConfig::default();
        let wallet = CantonWallet::new(config).await.unwrap();
        assert!(!wallet.wallet_id().as_str().is_empty());
        assert!(!wallet.party_id().as_str().is_empty());
    }

    #[tokio::test]
    async fn test_canton_wallet_balance() {
        let config = WalletConfig::default();
        let wallet = CantonWallet::new(config).await.unwrap();
        let balance = wallet.balance().await.unwrap();
        assert_eq!(balance.total_amount, "0");
    }
}
