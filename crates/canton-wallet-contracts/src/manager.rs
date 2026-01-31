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

//! Contract manager

use canton_ledger_api::proto::{
    Commands, Command, CreateCommand, ExerciseCommand, ExerciseByKeyCommand,
    CreateAndExerciseCommand, Identifier, DamlRecord, DamlValue,
    Transaction, CreatedEvent, TransactionFilter,
};
use canton_wallet_core::types::PartyId;
use canton_wallet_core::types::ContractId;
use std::sync::Arc;
use tokio::sync::RwLock;
use dashmap::DashMap;
use tracing::{debug, trace};
use crate::error::{ContractError, ContractResult};

/// Contract info
#[derive(Debug, Clone)]
pub struct ContractInfo {
    pub contract_id: ContractId,
    pub template_id: Identifier,
    pub create_arguments: DamlRecord,
    pub signatories: Vec<String>,
    pub observers: Vec<String>,
}

impl ContractInfo {
    pub fn from_created_event(event: &CreatedEvent) -> Self {
        Self {
            contract_id: ContractId::new(&event.contract_id),
            template_id: event.template_id.clone(),
            create_arguments: event.create_arguments.clone(),
            signatories: event.signatories.clone(),
            observers: event.observers.clone(),
        }
    }
}

/// Contract cache
#[derive(Debug)]
pub struct ContractCache {
    cache: DashMap<ContractId, ContractInfo>,
}

impl ContractCache {
    pub fn new() -> Self {
        Self {
            cache: DashMap::new(),
        }
    }

    pub fn get(&self, contract_id: &ContractId) -> Option<ContractInfo> {
        self.cache.get(contract_id).map(|v| v.clone())
    }

    pub fn insert(&self, contract_id: ContractId, info: ContractInfo) {
        self.cache.insert(contract_id, info);
    }

    pub fn remove(&self, contract_id: &ContractId) -> Option<ContractInfo> {
        self.cache.remove(contract_id).map(|(_, v)| v)
    }

    pub fn clear(&self) {
        self.cache.clear();
    }

    pub fn len(&self) -> usize {
        self.cache.len()
    }
}

impl Default for ContractCache {
    fn default() -> Self {
        Self::new()
    }
}

/// Contract manager
pub struct ContractManager {
    ledger_client: Arc<dyn LedgerClient>,
    party_id: PartyId,
    cache: Arc<RwLock<ContractCache>>,
}

/// Ledger client trait
#[async_trait::async_trait]
pub trait LedgerClient: Send + Sync {
    async fn submit_and_wait_for_party(
        &self,
        party_id: &PartyId,
        command: Command,
    ) -> ContractResult<Transaction>;

    async fn get_active_contracts(
        &self,
        filter: TransactionFilter,
    ) -> ContractResult<Vec<CreatedEvent>>;
}

impl ContractManager {
    pub fn new(ledger_client: Arc<dyn LedgerClient>, party_id: PartyId) -> Self {
        Self {
            ledger_client,
            party_id,
            cache: Arc::new(RwLock::new(ContractCache::new())),
        }
    }

    pub async fn create(
        &self,
        template_id: Identifier,
        arguments: DamlRecord,
    ) -> ContractResult<CreatedEvent> {
        debug!("Creating contract with template: {}", template_id);

        let command = Command::Create(CreateCommand {
            template_id: template_id.clone(),
            create_arguments: arguments,
        });

        let transaction = self
            .ledger_client
            .submit_and_wait_for_party(&self.party_id, command)
            .await?;

        let created_event = transaction
            .events
            .into_iter()
            .find_map(|event| match event {
                canton_ledger_api::proto::Event::Created(created) => Some(created),
                _ => None,
            })
            .ok_or_else(|| ContractError::CreationFailed("No created event in transaction".to_string()))?;

        let contract_id = ContractId::new(&created_event.contract_id);
        let contract_info = ContractInfo::from_created_event(&created_event);

        let mut cache = self.cache.write().await;
        cache.insert(contract_id, contract_info);

        debug!("Contract created successfully: {}", created_event.contract_id);
        Ok(created_event)
    }

    pub async fn exercise(
        &self,
        contract_id: ContractId,
        choice: &str,
        argument: DamlValue,
    ) -> ContractResult<Transaction> {
        debug!(
            "Exercising choice {} on contract {}",
            choice,
            contract_id
        );

        let command = Command::Exercise(ExerciseCommand {
            template_id: None,
            contract_id: contract_id.to_string(),
            choice: choice.to_string(),
            choice_argument: argument,
        });

        let transaction = self
            .ledger_client
            .submit_and_wait_for_party(&self.party_id, command)
            .await?;

        debug!("Choice exercised successfully");
        Ok(transaction)
    }

    pub async fn exercise_with_template(
        &self,
        template_id: Identifier,
        contract_id: ContractId,
        choice: &str,
        argument: DamlValue,
    ) -> ContractResult<Transaction> {
        debug!(
            "Exercising choice {} on contract {} with template {}",
            choice,
            contract_id,
            template_id
        );

        let command = Command::Exercise(ExerciseCommand {
            template_id: Some(template_id),
            contract_id: contract_id.to_string(),
            choice: choice.to_string(),
            choice_argument: argument,
        });

        let transaction = self
            .ledger_client
            .submit_and_wait_for_party(&self.party_id, command)
            .await?;

        debug!("Choice exercised successfully");
        Ok(transaction)
    }

    pub async fn exercise_by_key(
        &self,
        template_id: Identifier,
        contract_key: DamlValue,
        choice: &str,
        argument: DamlValue,
    ) -> ContractResult<Transaction> {
        debug!(
            "Exercising choice {} by key on template {}",
            choice,
            template_id
        );

        let command = Command::ExerciseByKey(ExerciseByKeyCommand {
            template_id,
            contract_key,
            choice: choice.to_string(),
            choice_argument: argument,
        });

        let transaction = self
            .ledger_client
            .submit_and_wait_for_party(&self.party_id, command)
            .await?;

        debug!("Choice exercised by key successfully");
        Ok(transaction)
    }

    pub async fn create_and_exercise(
        &self,
        template_id: Identifier,
        create_arguments: DamlRecord,
        choice: &str,
        choice_argument: DamlValue,
    ) -> ContractResult<Transaction> {
        debug!(
            "Creating and exercising choice {} on template {}",
            choice,
            template_id
        );

        let command = Command::CreateAndExercise(CreateAndExerciseCommand {
            template_id: template_id.clone(),
            create_arguments,
            choice: choice.to_string(),
            choice_argument,
        });

        let transaction = self
            .ledger_client
            .submit_and_wait_for_party(&self.party_id, command)
            .await?;

        debug!("Create and exercise completed successfully");
        Ok(transaction)
    }

    pub async fn active_contracts(
        &self,
        filter: Option<TransactionFilter>,
    ) -> ContractResult<Vec<CreatedEvent>> {
        debug!("Fetching active contracts");

        let filter = filter.unwrap_or_else(|| {
            TransactionFilter::PartiesByParty(self.party_id.clone())
        });

        let contracts = self
            .ledger_client
            .get_active_contracts(filter)
            .await?;

        debug!("Fetched {} active contracts", contracts.len());
        Ok(contracts)
    }

    pub async fn get_contract(
        &self,
        contract_id: ContractId,
    ) -> ContractResult<ContractInfo> {
        trace!("Getting contract: {}", contract_id);

        let cache = self.cache.read().await;
        if let Some(info) = cache.get(&contract_id) {
            return Ok(info);
        }

        let contracts = self.active_contracts(None).await?;
        let contract = contracts
            .into_iter()
            .find(|c| c.contract_id == contract_id.to_string())
            .ok_or_else(|| ContractError::NotFound(contract_id.to_string()))?;

        let info = ContractInfo::from_created_event(&contract);

        let mut cache = self.cache.write().await;
        cache.insert(contract_id, info.clone());

        Ok(info)
    }

    pub async fn query_by_template(
        &self,
        template_id: Identifier,
    ) -> ContractResult<Vec<CreatedEvent>> {
        debug!("Querying contracts by template: {}", template_id);

        let filter = TransactionFilter::Template(template_id);
        let contracts = self
            .ledger_client
            .get_active_contracts(filter)
            .await?;

        debug!("Found {} contracts for template", contracts.len());
        Ok(contracts)
    }

    pub async fn archive(
        &self,
        contract_id: ContractId,
    ) -> ContractResult<Transaction> {
        debug!("Archiving contract: {}", contract_id);

        let command = Command::Exercise(ExerciseCommand {
            template_id: None,
            contract_id: contract_id.to_string(),
            choice: "Archive".to_string(),
            choice_argument: DamlValue::unit(),
        });

        let transaction = self
            .ledger_client
            .submit_and_wait_for_party(&self.party_id, command)
            .await?;

        let mut cache = self.cache.write().await;
        cache.remove(&contract_id);

        debug!("Contract archived successfully");
        Ok(transaction)
    }

    pub async fn clear_cache(&self) {
        let mut cache = self.cache.write().await;
        cache.clear();
        debug!("Contract cache cleared");
    }

    pub async fn cache_size(&self) -> usize {
        let cache = self.cache.read().await;
        cache.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockLedgerClient {
        active_contracts: Vec<CreatedEvent>,
    }

    #[async_trait::async_trait]
    impl LedgerClient for MockLedgerClient {
        async fn submit_and_wait_for_party(
            &self,
            _party_id: &PartyId,
            _command: Command,
        ) -> ContractResult<Transaction> {
            Ok(Transaction {
                transaction_id: "test-tx".to_string(),
                command_id: "test-cmd".to_string(),
                workflow_id: "test-workflow".to_string(),
                effective_at: None,
                ledger_offset: canton_ledger_api::proto::LedgerOffset::Begin,
                events: vec![],
            })
        }

        async fn get_active_contracts(
            &self,
            _filter: TransactionFilter,
        ) -> ContractResult<Vec<CreatedEvent>> {
            Ok(self.active_contracts.clone())
        }
    }

    #[tokio::test]
    async fn test_create_contract() {
        let client = Arc::new(MockLedgerClient {
            active_contracts: vec![],
        });
        let manager = ContractManager::new(client, PartyId::new("party"));

        let template_id = Identifier {
            package_id: "pkg".to_string(),
            module_name: "mod".to_string(),
            entity_name: "tpl".to_string(),
        };

        let result = manager
            .create(template_id, DamlRecord::new())
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_exercise_choice() {
        let client = Arc::new(MockLedgerClient {
            active_contracts: vec![],
        });
        let manager = ContractManager::new(client, PartyId::new("party"));

        let contract_id = ContractId::new("test-contract");
        let result = manager
            .exercise(contract_id, "Choice", DamlValue::unit())
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_active_contracts() {
        let client = Arc::new(MockLedgerClient {
            active_contracts: vec![],
        });
        let manager = ContractManager::new(client, PartyId::new("party"));

        let result = manager.active_contracts(None).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_cache() {
        let client = Arc::new(MockLedgerClient {
            active_contracts: vec![],
        });
        let manager = ContractManager::new(client, PartyId::new("party"));

        let template_id = Identifier {
            package_id: "pkg".to_string(),
            module_name: "mod".to_string(),
            entity_name: "tpl".to_string(),
        };

        let created = manager
            .create(template_id, DamlRecord::new())
            .await
            .unwrap();

        let contract_id = ContractId::new(&created.contract_id);
        let result = manager.get_contract(contract_id).await;

        assert!(result.is_ok());
        assert_eq!(manager.cache_size().await, 1);

        manager.clear_cache().await;
        assert_eq!(manager.cache_size().await, 0);
    }
}