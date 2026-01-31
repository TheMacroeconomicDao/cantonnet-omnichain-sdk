// Multi-signature wallet implementation for Canton Wallet SDK

use async_trait::async_trait;
use canton_wallet_core::{
    Command, ContractId, CreatedEvent, DamlRecord, DamlValue, Event, LedgerOffset,
    PartyId, Signature, Transaction, TransactionFilter, TransactionId, Wallet as WalletTrait,
    WalletBalance, WalletConfig, WalletError, WalletResult,
};
use futures::Stream;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;

/// Multi-signature wallet configuration
#[derive(Debug, Clone)]
pub struct MultiSigConfig {
    pub signers: Vec<PartyId>,
    pub threshold: usize,
    pub timeout_ms: u64,
}

impl MultiSigConfig {
    /// Create a new multi-signature configuration
    ///
    /// # Arguments
    ///
    /// * `signers` - List of signer party IDs
    /// * `threshold` - Number of signatures required (must be <= signers.len())
    pub fn new(signers: Vec<PartyId>, threshold: usize) -> WalletResult<Self> {
        if threshold == 0 {
            return Err(WalletError::InvalidArgument(
                "Threshold must be greater than 0".to_string(),
            ));
        }
        if threshold > signers.len() {
            return Err(WalletError::InvalidArgument(format!(
                "Threshold ({}) cannot exceed number of signers ({})",
                threshold,
                signers.len()
            )));
        }
        Ok(Self {
            signers,
            threshold,
            timeout_ms: 30000,
        })
    }

    /// Set timeout in milliseconds
    pub fn with_timeout(mut self, timeout_ms: u64) -> Self {
        self.timeout_ms = timeout_ms;
        self
    }
}

impl Default for MultiSigConfig {
    fn default() -> Self {
        Self {
            signers: Vec::new(),
            threshold: 1,
            timeout_ms: 30000,
        }
    }
}

/// Multi-signature wallet
pub struct MultiSigWallet {
    wallet_id: canton_wallet_core::WalletId,
    party_id: PartyId,
    participant_id: PartyId,
    config: MultiSigConfig,
    balance: Arc<RwLock<WalletBalance>>,
    pending_signatures: Arc<RwLock<HashMap<TransactionId, Vec<Signature>>>>,
}

impl MultiSigWallet {
    /// Create a new multi-signature wallet
    ///
    /// # Arguments
    ///
    /// * `config` - Wallet configuration
    /// * `multi_sig_config` - Multi-signature configuration
    pub async fn new(config: WalletConfig, multi_sig_config: MultiSigConfig) -> WalletResult<Self> {
        let wallet_id = canton_wallet_core::WalletId::generate();
        let party_id = config
            .party_id
            .clone()
            .unwrap_or_else(|| PartyId::generate());
        let participant_id = PartyId::generate();
        let balance = Arc::new(RwLock::new(WalletBalance::zero("USD")));
        let pending_signatures = Arc::new(RwLock::new(HashMap::new()));

        Ok(Self {
            wallet_id,
            party_id,
            participant_id,
            config: multi_sig_config,
            balance,
            pending_signatures,
        })
    }

    /// Add signature to pending transaction
    ///
    /// # Arguments
    ///
    /// * `transaction_id` - Transaction ID
    /// * `signature` - Signature to add
    pub async fn add_signature(
        &self,
        transaction_id: TransactionId,
        signature: Signature,
    ) -> WalletResult<bool> {
        let mut pending = self.pending_signatures.write();
        let signatures = pending.entry(transaction_id.clone()).or_insert_with(Vec::new);
        
        if signatures.contains(&signature) {
            return Ok(false);
        }
        
        signatures.push(signature);
        Ok(true)
    }

    /// Check if transaction has enough signatures
    ///
    /// # Arguments
    ///
    /// * `transaction_id` - Transaction ID
    pub async fn has_enough_signatures(&self, transaction_id: &TransactionId) -> bool {
        let pending = self.pending_signatures.read();
        if let Some(signatures) = pending.get(transaction_id) {
            signatures.len() >= self.config.threshold
        } else {
            false
        }
    }

    /// Get pending signatures for transaction
    ///
    /// # Arguments
    ///
    /// * `transaction_id` - Transaction ID
    pub async fn get_signatures(
        &self,
        transaction_id: &TransactionId,
    ) -> Vec<Signature> {
        let pending = self.pending_signatures.read();
        pending.get(transaction_id).cloned().unwrap_or_default()
    }

    /// Clear pending signatures for transaction
    ///
    /// # Arguments
    ///
    /// * `transaction_id` - Transaction ID
    pub async fn clear_signatures(&self, transaction_id: &TransactionId) {
        let mut pending = self.pending_signatures.write();
        pending.remove(transaction_id);
    }

    /// Get signers
    pub fn signers(&self) -> &Vec<PartyId> {
        &self.config.signers
    }

    /// Get threshold
    pub fn threshold(&self) -> usize {
        self.config.threshold
    }

    /// Update wallet balance
    pub fn update_balance(&self, balance: WalletBalance) {
        let mut b = self.balance.write();
        *b = balance;
    }

    /// Get wallet configuration
    pub fn config(&self) -> &MultiSigConfig {
        &self.config
    }
}

#[async_trait]
impl WalletTrait for MultiSigWallet {
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
            signatories: self.config.signers.clone(),
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

    async fn sign(&self, _data: &[u8]) -> WalletResult<Signature> {
        Err(WalletError::NotSupported(
            "Multi-sig wallet requires individual signers to sign".to_string(),
        ))
    }

    async fn verify(&self, _data: &[u8], _signature: &Signature) -> WalletResult<bool> {
        Err(WalletError::NotSupported(
            "Multi-sig wallet requires individual signers to verify".to_string(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_multi_sig_config_new() {
        let signers = vec![
            PartyId::new_unchecked("signer1"),
            PartyId::new_unchecked("signer2"),
            PartyId::new_unchecked("signer3"),
        ];
        let config = MultiSigConfig::new(signers.clone(), 2).unwrap();
        assert_eq!(config.signers.len(), 3);
        assert_eq!(config.threshold, 2);
    }

    #[test]
    fn test_multi_sig_config_invalid_threshold() {
        let signers = vec![
            PartyId::new_unchecked("signer1"),
            PartyId::new_unchecked("signer2"),
        ];
        let result = MultiSigConfig::new(signers, 3);
        assert!(result.is_err());
    }

    #[test]
    fn test_multi_sig_config_zero_threshold() {
        let signers = vec![PartyId::new_unchecked("signer1")];
        let result = MultiSigConfig::new(signers, 0);
        assert!(result.is_err());
    }

    #[test]
    fn test_multi_sig_config_with_timeout() {
        let signers = vec![PartyId::new_unchecked("signer1")];
        let config = MultiSigConfig::new(signers, 1)
            .unwrap()
            .with_timeout(60000);
        assert_eq!(config.timeout_ms, 60000);
    }

    #[tokio::test]
    async fn test_multi_sig_wallet_new() {
        let wallet_config = WalletConfig::default();
        let signers = vec![
            PartyId::new_unchecked("signer1"),
            PartyId::new_unchecked("signer2"),
        ];
        let multi_sig_config = MultiSigConfig::new(signers, 2).unwrap();
        let wallet = MultiSigWallet::new(wallet_config, multi_sig_config)
            .await
            .unwrap();
        assert!(!wallet.wallet_id().as_str().is_empty());
        assert_eq!(wallet.threshold(), 2);
    }

    #[tokio::test]
    async fn test_multi_sig_wallet_add_signature() {
        let wallet_config = WalletConfig::default();
        let signers = vec![
            PartyId::new_unchecked("signer1"),
            PartyId::new_unchecked("signer2"),
        ];
        let multi_sig_config = MultiSigConfig::new(signers, 2).unwrap();
        let wallet = MultiSigWallet::new(wallet_config, multi_sig_config)
            .await
            .unwrap();
        
        let transaction_id = TransactionId::generate();
        let signature = Signature::new(vec![1, 2, 3], "ed25519");
        
        let added = wallet.add_signature(transaction_id.clone(), signature.clone()).await.unwrap();
        assert!(added);
        
        let added_again = wallet.add_signature(transaction_id.clone(), signature).await.unwrap();
        assert!(!added_again);
    }

    #[tokio::test]
    async fn test_multi_sig_wallet_has_enough_signatures() {
        let wallet_config = WalletConfig::default();
        let signers = vec![
            PartyId::new_unchecked("signer1"),
            PartyId::new_unchecked("signer2"),
            PartyId::new_unchecked("signer3"),
        ];
        let multi_sig_config = MultiSigConfig::new(signers, 2).unwrap();
        let wallet = MultiSigWallet::new(wallet_config, multi_sig_config)
            .await
            .unwrap();
        
        let transaction_id = TransactionId::generate();
        let signature1 = Signature::new(vec![1, 2, 3], "ed25519");
        let signature2 = Signature::new(vec![4, 5, 6], "ed25519");
        
        wallet.add_signature(transaction_id.clone(), signature1).await.unwrap();
        assert!(!wallet.has_enough_signatures(&transaction_id).await);
        
        wallet.add_signature(transaction_id.clone(), signature2).await.unwrap();
        assert!(wallet.has_enough_signatures(&transaction_id).await);
    }

    #[tokio::test]
    async fn test_multi_sig_wallet_get_signatures() {
        let wallet_config = WalletConfig::default();
        let signers = vec![
            PartyId::new_unchecked("signer1"),
            PartyId::new_unchecked("signer2"),
        ];
        let multi_sig_config = MultiSigConfig::new(signers, 2).unwrap();
        let wallet = MultiSigWallet::new(wallet_config, multi_sig_config)
            .await
            .unwrap();
        
        let transaction_id = TransactionId::generate();
        let signature1 = Signature::new(vec![1, 2, 3], "ed25519");
        let signature2 = Signature::new(vec![4, 5, 6], "ed25519");
        
        wallet.add_signature(transaction_id.clone(), signature1.clone()).await.unwrap();
        wallet.add_signature(transaction_id.clone(), signature2.clone()).await.unwrap();
        
        let signatures = wallet.get_signatures(&transaction_id).await;
        assert_eq!(signatures.len(), 2);
        assert!(signatures.contains(&signature1));
        assert!(signatures.contains(&signature2));
    }

    #[tokio::test]
    async fn test_multi_sig_wallet_clear_signatures() {
        let wallet_config = WalletConfig::default();
        let signers = vec![
            PartyId::new_unchecked("signer1"),
            PartyId::new_unchecked("signer2"),
        ];
        let multi_sig_config = MultiSigConfig::new(signers, 2).unwrap();
        let wallet = MultiSigWallet::new(wallet_config, multi_sig_config)
            .await
            .unwrap();
        
        let transaction_id = TransactionId::generate();
        let signature = Signature::new(vec![1, 2, 3], "ed25519");
        
        wallet.add_signature(transaction_id.clone(), signature).await.unwrap();
        wallet.clear_signatures(&transaction_id).await;
        
        let signatures = wallet.get_signatures(&transaction_id).await;
        assert!(signatures.is_empty());
    }

    #[tokio::test]
    async fn test_multi_sig_wallet_balance() {
        let wallet_config = WalletConfig::default();
        let signers = vec![
            PartyId::new_unchecked("signer1"),
            PartyId::new_unchecked("signer2"),
        ];
        let multi_sig_config = MultiSigConfig::new(signers, 2).unwrap();
        let wallet = MultiSigWallet::new(wallet_config, multi_sig_config)
            .await
            .unwrap();
        let balance = wallet.balance().await.unwrap();
        assert_eq!(balance.total_amount, "0");
    }

    #[tokio::test]
    async fn test_multi_sig_wallet_update_balance() {
        let wallet_config = WalletConfig::default();
        let signers = vec![
            PartyId::new_unchecked("signer1"),
            PartyId::new_unchecked("signer2"),
        ];
        let multi_sig_config = MultiSigConfig::new(signers, 2).unwrap();
        let wallet = MultiSigWallet::new(wallet_config, multi_sig_config)
            .await
            .unwrap();
        let new_balance = WalletBalance::new("100", "80", "20", "USD");
        wallet.update_balance(new_balance);
        let balance = wallet.balance().await.unwrap();
        assert_eq!(balance.total_amount, "100");
    }
}
