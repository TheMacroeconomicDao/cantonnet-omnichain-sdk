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

//! Transaction builder

use canton_ledger_api::proto::{
    Commands, Command, CreateCommand, ExerciseCommand, ExerciseByKeyCommand,
    CreateAndExerciseCommand, Identifier, DamlRecord, DamlValue,
};
use canton_wallet_core::types::PartyId;
use chrono::{DateTime, Utc};
use std::time::Duration;
use uuid::Uuid;
use tracing::debug;
use crate::error::{TransactionError, TransactionResult};
use crate::validator::TransactionValidator;

/// Transaction builder
#[derive(Debug, Clone)]
pub struct TransactionBuilder {
    party_id: Option<PartyId>,
    commands: Vec<Command>,
    workflow_id: Option<String>,
    application_id: Option<String>,
    command_id: Option<String>,
    act_as: Vec<PartyId>,
    read_as: Vec<PartyId>,
    min_ledger_time: Option<DateTime<Utc>>,
    deduplication_period: Option<Duration>,
    validator: TransactionValidator,
}

impl Default for TransactionBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl TransactionBuilder {
    pub fn new() -> Self {
        Self {
            party_id: None,
            commands: Vec::new(),
            workflow_id: None,
            application_id: None,
            command_id: None,
            act_as: Vec::new(),
            read_as: Vec::new(),
            min_ledger_time: None,
            deduplication_period: None,
            validator: TransactionValidator::new(),
        }
    }

    pub fn party_id(mut self, party_id: PartyId) -> Self {
        self.party_id = Some(party_id);
        self
    }

    pub fn add_command(mut self, command: Command) -> Self {
        self.commands.push(command);
        self
    }

    pub fn add_commands(mut self, commands: impl IntoIterator<Item = Command>) -> Self {
        self.commands.extend(commands);
        self
    }

    pub fn workflow_id(mut self, id: impl Into<String>) -> Self {
        self.workflow_id = Some(id.into());
        self
    }

    pub fn application_id(mut self, id: impl Into<String>) -> Self {
        self.application_id = Some(id.into());
        self
    }

    pub fn command_id(mut self, id: impl Into<String>) -> Self {
        self.command_id = Some(id.into());
        self
    }

    pub fn act_as(mut self, party_id: PartyId) -> Self {
        self.act_as.push(party_id);
        self
    }

    pub fn act_as_multiple(mut self, party_ids: impl IntoIterator<Item = PartyId>) -> Self {
        self.act_as.extend(party_ids);
        self
    }

    pub fn read_as(mut self, party_id: PartyId) -> Self {
        self.read_as.push(party_id);
        self
    }

    pub fn read_as_multiple(mut self, party_ids: impl IntoIterator<Item = PartyId>) -> Self {
        self.read_as.extend(party_ids);
        self
    }

    pub fn min_ledger_time(mut self, time: DateTime<Utc>) -> Self {
        self.min_ledger_time = Some(time);
        self
    }

    pub fn deduplication_period(mut self, period: Duration) -> Self {
        self.deduplication_period = Some(period);
        self
    }

    pub fn with_validator(mut self, validator: TransactionValidator) -> Self {
        self.validator = validator;
        self
    }

    pub fn build(self) -> TransactionResult<Commands> {
        debug!("Building transaction with {} commands", self.commands.len());

        let party_id = self.party_id.ok_or(TransactionError::MissingPartyId)?;

        let commands = Commands {
            ledger_id: String::new(),
            workflow_id: self.workflow_id.unwrap_or_else(|| Uuid::new_v4().to_string()),
            application_id: self.application_id.unwrap_or_else(|| "canton-wallet-sdk".to_string()),
            command_id: self.command_id.unwrap_or_else(|| Uuid::new_v4().to_string()),
            party: party_id.to_string(),
            commands: self.commands,
            act_as: if self.act_as.is_empty() {
                vec![party_id.to_string()]
            } else {
                self.act_as.iter().map(|p| p.to_string()).collect()
            },
            read_as: self.read_as.iter().map(|p| p.to_string()).collect(),
            min_ledger_time_abs: self.min_ledger_time.map(|t| t.timestamp_millis()),
            min_ledger_time_rel: self.deduplication_period.map(|d| d.as_millis() as u64),
            deduplication_time: None,
            submission_id: None,
        };

        self.validator.validate(&commands)?;

        debug!("Transaction built successfully");
        Ok(commands)
    }

    pub fn create(
        template_id: Identifier,
        create_arguments: DamlRecord,
    ) -> Command {
        Command::Create(CreateCommand {
            template_id,
            create_arguments,
        })
    }

    pub fn exercise(
        contract_id: String,
        choice: String,
        choice_argument: DamlValue,
    ) -> Command {
        Command::Exercise(ExerciseCommand {
            template_id: None,
            contract_id,
            choice,
            choice_argument,
        })
    }

    pub fn exercise_with_template(
        template_id: Identifier,
        contract_id: String,
        choice: String,
        choice_argument: DamlValue,
    ) -> Command {
        Command::Exercise(ExerciseCommand {
            template_id: Some(template_id),
            contract_id,
            choice,
            choice_argument,
        })
    }

    pub fn exercise_by_key(
        template_id: Identifier,
        contract_key: DamlValue,
        choice: String,
        choice_argument: DamlValue,
    ) -> Command {
        Command::ExerciseByKey(ExerciseByKeyCommand {
            template_id,
            contract_key,
            choice,
            choice_argument,
        })
    }

    pub fn create_and_exercise(
        template_id: Identifier,
        create_arguments: DamlRecord,
        choice: String,
        choice_argument: DamlValue,
    ) -> Command {
        Command::CreateAndExercise(CreateAndExerciseCommand {
            template_id,
            create_arguments,
            choice,
            choice_argument,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder_missing_party_id() {
        let builder = TransactionBuilder::new()
            .add_command(Command::Create(CreateCommand {
                template_id: Identifier {
                    package_id: "pkg".to_string(),
                    module_name: "mod".to_string(),
                    entity_name: "tpl".to_string(),
                },
                create_arguments: DamlRecord::new(),
            }));

        let result = builder.build();
        assert!(matches!(result, Err(TransactionError::MissingPartyId)));
    }

    #[test]
    fn test_builder_valid_transaction() {
        let builder = TransactionBuilder::new()
            .party_id(PartyId::new("party"))
            .add_command(Command::Create(CreateCommand {
                template_id: Identifier {
                    package_id: "pkg".to_string(),
                    module_name: "mod".to_string(),
                    entity_name: "tpl".to_string(),
                },
                create_arguments: DamlRecord::new(),
            }));

        let result = builder.build();
        assert!(result.is_ok());
    }

    #[test]
    fn test_builder_with_act_as() {
        let builder = TransactionBuilder::new()
            .party_id(PartyId::new("party"))
            .act_as(PartyId::new("party1"))
            .act_as(PartyId::new("party2"))
            .add_command(Command::Create(CreateCommand {
                template_id: Identifier {
                    package_id: "pkg".to_string(),
                    module_name: "mod".to_string(),
                    entity_name: "tpl".to_string(),
                },
                create_arguments: DamlRecord::new(),
            }));

        let result = builder.build();
        assert!(result.is_ok());
        let commands = result.unwrap();
        assert_eq!(commands.act_as.len(), 2);
    }

    #[test]
    fn test_builder_with_read_as() {
        let builder = TransactionBuilder::new()
            .party_id(PartyId::new("party"))
            .read_as(PartyId::new("party1"))
            .read_as(PartyId::new("party2"))
            .add_command(Command::Create(CreateCommand {
                template_id: Identifier {
                    package_id: "pkg".to_string(),
                    module_name: "mod".to_string(),
                    entity_name: "tpl".to_string(),
                },
                create_arguments: DamlRecord::new(),
            }));

        let result = builder.build();
        assert!(result.is_ok());
        let commands = result.unwrap();
        assert_eq!(commands.read_as.len(), 2);
    }

    #[test]
    fn test_builder_with_deduplication_period() {
        let builder = TransactionBuilder::new()
            .party_id(PartyId::new("party"))
            .deduplication_period(Duration::from_secs(30))
            .add_command(Command::Create(CreateCommand {
                template_id: Identifier {
                    package_id: "pkg".to_string(),
                    module_name: "mod".to_string(),
                    entity_name: "tpl".to_string(),
                },
                create_arguments: DamlRecord::new(),
            }));

        let result = builder.build();
        assert!(result.is_ok());
        let commands = result.unwrap();
        assert_eq!(commands.min_ledger_time_rel, Some(30000));
    }
}