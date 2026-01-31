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

//! Transaction validator

use canton_ledger_api::proto::Commands;
use tracing::{debug, trace};
use crate::error::{TransactionError, TransactionResult};

/// Transaction validator
#[derive(Debug, Clone)]
pub struct TransactionValidator {
    max_commands: usize,
    max_size: usize,
    require_party: bool,
    require_workflow_id: bool,
    require_application_id: bool,
    require_command_id: bool,
}

impl Default for TransactionValidator {
    fn default() -> Self {
        Self::new()
    }
}

impl TransactionValidator {
    pub fn new() -> Self {
        Self {
            max_commands: 100,
            max_size: 1024 * 1024,
            require_party: true,
            require_workflow_id: false,
            require_application_id: false,
            require_command_id: false,
        }
    }

    pub fn with_max_commands(mut self, max: usize) -> Self {
        self.max_commands = max;
        self
    }

    pub fn with_max_size(mut self, max: usize) -> Self {
        self.max_size = max;
        self
    }

    pub fn require_party(mut self, required: bool) -> Self {
        self.require_party = required;
        self
    }

    pub fn require_workflow_id(mut self, required: bool) -> Self {
        self.require_workflow_id = required;
        self
    }

    pub fn require_application_id(mut self, required: bool) -> Self {
        self.require_application_id = required;
        self
    }

    pub fn require_command_id(mut self, required: bool) -> Self {
        self.require_command_id = required;
        self
    }

    pub fn validate(&self, transaction: &Commands) -> TransactionResult<()> {
        debug!("Validating transaction with {} commands", transaction.commands.len());

        // Validate party ID
        if self.require_party && transaction.party.is_empty() {
            return Err(TransactionError::MissingPartyId);
        }

        // Validate workflow ID
        if self.require_workflow_id && transaction.workflow_id.is_empty() {
            return Err(TransactionError::InvalidWorkflowId("workflow ID is required".to_string()));
        }

        // Validate application ID
        if self.require_application_id && transaction.application_id.is_empty() {
            return Err(TransactionError::InvalidApplicationId("application ID is required".to_string()));
        }

        // Validate command ID
        if self.require_command_id && transaction.command_id.is_empty() {
            return Err(TransactionError::InvalidCommandId("command ID is required".to_string()));
        }

        // Validate commands
        if transaction.commands.is_empty() {
            return Err(TransactionError::TooFewCommands(0));
        }

        if transaction.commands.len() > self.max_commands {
            return Err(TransactionError::TooManyCommands(transaction.commands.len()));
        }

        // Validate act_as
        if transaction.act_as.is_empty() {
            return Err(TransactionError::InvalidParty("act_as cannot be empty".to_string()));
        }

        // Validate deduplication period
        if let Some(rel) = transaction.min_ledger_time_rel {
            if rel < 0 {
                return Err(TransactionError::InvalidDeduplicationPeriod("deduplication period must be non-negative".to_string()));
            }
        }

        // Validate each command
        for (index, command) in transaction.commands.iter().enumerate() {
            trace!("Validating command at index {}", index);
            self.validate_command(command, index)?;
        }

        debug!("Transaction validation successful");
        Ok(())
    }

    fn validate_command(&self, command: &canton_ledger_api::proto::Command, index: usize) -> TransactionResult<()> {
        match command {
            canton_ledger_api::proto::Command::Create(create_cmd) => {
                self.validate_create_command(create_cmd, index)?;
            }
            canton_ledger_api::proto::Command::Exercise(exercise_cmd) => {
                self.validate_exercise_command(exercise_cmd, index)?;
            }
            canton_ledger_api::proto::Command::ExerciseByKey(exercise_by_key_cmd) => {
                self.validate_exercise_by_key_command(exercise_by_key_cmd, index)?;
            }
            canton_ledger_api::proto::Command::CreateAndExercise(create_and_exercise_cmd) => {
                self.validate_create_and_exercise_command(create_and_exercise_cmd, index)?;
            }
        }
        Ok(())
    }

    fn validate_create_command(&self, create_cmd: &canton_ledger_api::proto::CreateCommand, _index: usize) -> TransactionResult<()> {
        if create_cmd.template_id.package_id.is_empty() {
            return Err(TransactionError::InvalidTemplate("package_id cannot be empty".to_string()));
        }
        if create_cmd.template_id.module_name.is_empty() {
            return Err(TransactionError::InvalidTemplate("module_name cannot be empty".to_string()));
        }
        if create_cmd.template_id.entity_name.is_empty() {
            return Err(TransactionError::InvalidTemplate("entity_name cannot be empty".to_string()));
        }
        Ok(())
    }

    fn validate_exercise_command(&self, exercise_cmd: &canton_ledger_api::proto::ExerciseCommand, _index: usize) -> TransactionResult<()> {
        if exercise_cmd.contract_id.is_empty() {
            return Err(TransactionError::InvalidContractId("contract_id cannot be empty".to_string()));
        }
        if exercise_cmd.choice.is_empty() {
            return Err(TransactionError::InvalidChoice("choice cannot be empty".to_string()));
        }
        if let Some(template_id) = &exercise_cmd.template_id {
            if template_id.package_id.is_empty() {
                return Err(TransactionError::InvalidTemplate("package_id cannot be empty".to_string()));
            }
            if template_id.module_name.is_empty() {
                return Err(TransactionError::InvalidTemplate("module_name cannot be empty".to_string()));
            }
            if template_id.entity_name.is_empty() {
                return Err(TransactionError::InvalidTemplate("entity_name cannot be empty".to_string()));
            }
        }
        Ok(())
    }

    fn validate_exercise_by_key_command(&self, exercise_by_key_cmd: &canton_ledger_api::proto::ExerciseByKeyCommand, _index: usize) -> TransactionResult<()> {
        if exercise_by_key_cmd.template_id.package_id.is_empty() {
            return Err(TransactionError::InvalidTemplate("package_id cannot be empty".to_string()));
        }
        if exercise_by_key_cmd.template_id.module_name.is_empty() {
            return Err(TransactionError::InvalidTemplate("module_name cannot be empty".to_string()));
        }
        if exercise_by_key_cmd.template_id.entity_name.is_empty() {
            return Err(TransactionError::InvalidTemplate("entity_name cannot be empty".to_string()));
        }
        if exercise_by_key_cmd.choice.is_empty() {
            return Err(TransactionError::InvalidChoice("choice cannot be empty".to_string()));
        }
        Ok(())
    }

    fn validate_create_and_exercise_command(&self, create_and_exercise_cmd: &canton_ledger_api::proto::CreateAndExerciseCommand, _index: usize) -> TransactionResult<()> {
        if create_and_exercise_cmd.template_id.package_id.is_empty() {
            return Err(TransactionError::InvalidTemplate("package_id cannot be empty".to_string()));
        }
        if create_and_exercise_cmd.template_id.module_name.is_empty() {
            return Err(TransactionError::InvalidTemplate("module_name cannot be empty".to_string()));
        }
        if create_and_exercise_cmd.template_id.entity_name.is_empty() {
            return Err(TransactionError::InvalidTemplate("entity_name cannot be empty".to_string()));
        }
        if create_and_exercise_cmd.choice.is_empty() {
            return Err(TransactionError::InvalidChoice("choice cannot be empty".to_string()));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use canton_ledger_api::proto::{Command, CreateCommand, Identifier};

    #[test]
    fn test_validate_empty_party() {
        let validator = TransactionValidator::new();
        let commands = Commands {
            party: String::new(),
            commands: vec![Command::Create(CreateCommand {
                template_id: Identifier {
                    package_id: "pkg".to_string(),
                    module_name: "mod".to_string(),
                    entity_name: "tpl".to_string(),
                },
                create_arguments: canton_ledger_api::proto::DamlRecord::new(),
            })],
            ..Default::default()
        };

        let result = validator.validate(&commands);
        assert!(matches!(result, Err(TransactionError::MissingPartyId)));
    }

    #[test]
    fn test_validate_empty_commands() {
        let validator = TransactionValidator::new().require_party(false);
        let commands = Commands {
            party: "party".to_string(),
            commands: vec![],
            act_as: vec!["party".to_string()],
            ..Default::default()
        };

        let result = validator.validate(&commands);
        assert!(matches!(result, Err(TransactionError::TooFewCommands(0))));
    }

    #[test]
    fn test_validate_too_many_commands() {
        let validator = TransactionValidator::new().with_max_commands(2).require_party(false);
        let commands = Commands {
            party: "party".to_string(),
            commands: vec![
                Command::Create(CreateCommand {
                    template_id: Identifier {
                        package_id: "pkg".to_string(),
                        module_name: "mod".to_string(),
                        entity_name: "tpl".to_string(),
                    },
                    create_arguments: canton_ledger_api::proto::DamlRecord::new(),
                }),
                Command::Create(CreateCommand {
                    template_id: Identifier {
                        package_id: "pkg".to_string(),
                        module_name: "mod".to_string(),
                        entity_name: "tpl".to_string(),
                    },
                    create_arguments: canton_ledger_api::proto::DamlRecord::new(),
                }),
                Command::Create(CreateCommand {
                    template_id: Identifier {
                        package_id: "pkg".to_string(),
                        module_name: "mod".to_string(),
                        entity_name: "tpl".to_string(),
                    },
                    create_arguments: canton_ledger_api::proto::DamlRecord::new(),
                }),
            ],
            act_as: vec!["party".to_string()],
            ..Default::default()
        };

        let result = validator.validate(&commands);
        assert!(matches!(result, Err(TransactionError::TooManyCommands(3))));
    }

    #[test]
    fn test_validate_valid_transaction() {
        let validator = TransactionValidator::new();
        let commands = Commands {
            party: "party".to_string(),
            commands: vec![Command::Create(CreateCommand {
                template_id: Identifier {
                    package_id: "pkg".to_string(),
                    module_name: "mod".to_string(),
                    entity_name: "tpl".to_string(),
                },
                create_arguments: canton_ledger_api::proto::DamlRecord::new(),
            })],
            act_as: vec!["party".to_string()],
            ..Default::default()
        };

        let result = validator.validate(&commands);
        assert!(result.is_ok());
    }
}