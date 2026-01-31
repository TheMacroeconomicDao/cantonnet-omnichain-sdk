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

//! Input validation for security.

use crate::error::{Result, SecurityError};
use canton_wallet_core::types::{Command, Commands, DamlValue, Identifier, PartyId};
use regex::Regex;
use std::sync::Arc;

/// Input validator for security.
pub struct InputValidator {
    /// Maximum party ID length.
    max_party_id_length: usize,

    /// Maximum contract ID length.
    max_contract_id_length: usize,

    /// Maximum transaction ID length.
    max_transaction_id_length: usize,

    /// Maximum workflow ID length.
    max_workflow_id_length: usize,

    /// Maximum application ID length.
    max_application_id_length: usize,

    /// Party ID regex pattern.
    party_id_pattern: Regex,

    /// Contract ID regex pattern.
    contract_id_pattern: Regex,

    /// Transaction ID regex pattern.
    transaction_id_pattern: Regex,
}

impl Default for InputValidator {
    fn default() -> Self {
        Self::new()
    }
}

impl InputValidator {
    /// Create a new input validator with default settings.
    pub fn new() -> Self {
        Self {
            max_party_id_length: 256,
            max_contract_id_length: 512,
            max_transaction_id_length: 256,
            max_workflow_id_length: 256,
            max_application_id_length: 256,
            party_id_pattern: Regex::new(r"^[a-zA-Z0-9\-_:]+$").unwrap(),
            contract_id_pattern: Regex::new(r"^[a-zA-Z0-9\-_:]+$").unwrap(),
            transaction_id_pattern: Regex::new(r"^[a-zA-Z0-9\-_:]+$").unwrap(),
        }
    }

    /// Create a new input validator with custom settings.
    pub fn with_settings(
        max_party_id_length: usize,
        max_contract_id_length: usize,
        max_transaction_id_length: usize,
        max_workflow_id_length: usize,
        max_application_id_length: usize,
    ) -> Self {
        Self {
            max_party_id_length,
            max_contract_id_length,
            max_transaction_id_length,
            max_workflow_id_length,
            max_application_id_length,
            party_id_pattern: Regex::new(r"^[a-zA-Z0-9\-_:]+$").unwrap(),
            contract_id_pattern: Regex::new(r"^[a-zA-Z0-9\-_:]+$").unwrap(),
            transaction_id_pattern: Regex::new(r"^[a-zA-Z0-9\-_:]+$").unwrap(),
        }
    }

    /// Validate a party ID.
    ///
    /// # Arguments
    ///
    /// * `party_id` - Party ID to validate
    ///
    /// # Returns
    ///
    /// Returns Ok(()) if valid, or an error if invalid
    pub fn validate_party_id(&self, party_id: &PartyId) -> Result<()> {
        let id = party_id.as_str();

        if id.is_empty() {
            return Err(SecurityError::InputValidationFailed(
                "Party ID cannot be empty".to_string(),
            ));
        }

        if id.len() > self.max_party_id_length {
            return Err(SecurityError::InputValidationFailed(format!(
                "Party ID exceeds maximum length of {}",
                self.max_party_id_length
            )));
        }

        if !self.party_id_pattern.is_match(id) {
            return Err(SecurityError::InputValidationFailed(
                "Party ID contains invalid characters".to_string(),
            ));
        }

        Ok(())
    }

    /// Validate a contract ID.
    ///
    /// # Arguments
    ///
    /// * `contract_id` - Contract ID to validate
    ///
    /// # Returns
    ///
    /// Returns Ok(()) if valid, or an error if invalid
    pub fn validate_contract_id(&self, contract_id: &str) -> Result<()> {
        if contract_id.is_empty() {
            return Err(SecurityError::InputValidationFailed(
                "Contract ID cannot be empty".to_string(),
            ));
        }

        if contract_id.len() > self.max_contract_id_length {
            return Err(SecurityError::InputValidationFailed(format!(
                "Contract ID exceeds maximum length of {}",
                self.max_contract_id_length
            )));
        }

        if !self.contract_id_pattern.is_match(contract_id) {
            return Err(SecurityError::InputValidationFailed(
                "Contract ID contains invalid characters".to_string(),
            ));
        }

        Ok(())
    }

    /// Validate a transaction ID.
    ///
    /// # Arguments
    ///
    /// * `transaction_id` - Transaction ID to validate
    ///
    /// # Returns
    ///
    /// Returns Ok(()) if valid, or an error if invalid
    pub fn validate_transaction_id(&self, transaction_id: &str) -> Result<()> {
        if transaction_id.is_empty() {
            return Err(SecurityError::InputValidationFailed(
                "Transaction ID cannot be empty".to_string(),
            ));
        }

        if transaction_id.len() > self.max_transaction_id_length {
            return Err(SecurityError::InputValidationFailed(format!(
                "Transaction ID exceeds maximum length of {}",
                self.max_transaction_id_length
            )));
        }

        if !self.transaction_id_pattern.is_match(transaction_id) {
            return Err(SecurityError::InputValidationFailed(
                "Transaction ID contains invalid characters".to_string(),
            ));
        }

        Ok(())
    }

    /// Validate a workflow ID.
    ///
    /// # Arguments
    ///
    /// * `workflow_id` - Workflow ID to validate
    ///
    /// # Returns
    ///
    /// Returns Ok(()) if valid, or an error if invalid
    pub fn validate_workflow_id(&self, workflow_id: &str) -> Result<()> {
        if workflow_id.is_empty() {
            return Err(SecurityError::InputValidationFailed(
                "Workflow ID cannot be empty".to_string(),
            ));
        }

        if workflow_id.len() > self.max_workflow_id_length {
            return Err(SecurityError::InputValidationFailed(format!(
                "Workflow ID exceeds maximum length of {}",
                self.max_workflow_id_length
            )));
        }

        Ok(())
    }

    /// Validate an application ID.
    ///
    /// # Arguments
    ///
    /// * `application_id` - Application ID to validate
    ///
    /// # Returns
    ///
    /// Returns Ok(()) if valid, or an error if invalid
    pub fn validate_application_id(&self, application_id: &str) -> Result<()> {
        if application_id.is_empty() {
            return Err(SecurityError::InputValidationFailed(
                "Application ID cannot be empty".to_string(),
            ));
        }

        if application_id.len() > self.max_application_id_length {
            return Err(SecurityError::InputValidationFailed(format!(
                "Application ID exceeds maximum length of {}",
                self.max_application_id_length
            )));
        }

        Ok(())
    }

    /// Validate an identifier.
    ///
    /// # Arguments
    ///
    /// * `identifier` - Identifier to validate
    ///
    /// # Returns
    ///
    /// Returns Ok(()) if valid, or an error if invalid
    pub fn validate_identifier(&self, identifier: &Identifier) -> Result<()> {
        let id_str = identifier.to_string();

        if id_str.is_empty() {
            return Err(SecurityError::InputValidationFailed(
                "Identifier cannot be empty".to_string(),
            ));
        }

        if id_str.len() > self.max_contract_id_length {
            return Err(SecurityError::InputValidationFailed(format!(
                "Identifier exceeds maximum length of {}",
                self.max_contract_id_length
            )));
        }

        Ok(())
    }

    /// Validate a Daml value.
    ///
    /// # Arguments
    ///
    /// * `value` - Daml value to validate
    ///
    /// # Returns
    ///
    /// Returns Ok(()) if valid, or an error if invalid
    pub fn validate_daml_value(&self, value: &DamlValue) -> Result<()> {
        // Validate nested values recursively
        self.validate_daml_value_recursive(value, 0)
    }

    /// Validate a Daml value recursively with depth limit.
    fn validate_daml_value_recursive(&self, value: &DamlValue, depth: usize) -> Result<()> {
        const MAX_DEPTH: usize = 100;

        if depth > MAX_DEPTH {
            return Err(SecurityError::InputValidationFailed(
                "Daml value exceeds maximum nesting depth".to_string(),
            ));
        }

        match value {
            DamlValue::Record(record) => {
                for field in &record.fields {
                    self.validate_daml_value_recursive(&field.value, depth + 1)?;
                }
            }
            DamlValue::Variant(variant) => {
                self.validate_daml_value_recursive(&variant.value, depth + 1)?;
            }
            DamlValue::List(list) => {
                for item in &list.values {
                    self.validate_daml_value_recursive(item, depth + 1)?;
                }
            }
            DamlValue::Optional(optional) => {
                if let Some(value) = &optional.value {
                    self.validate_daml_value_recursive(value, depth + 1)?;
                }
            }
            DamlValue::Map(map) => {
                for entry in &map.entries {
                    self.validate_daml_value_recursive(&entry.key, depth + 1)?;
                    self.validate_daml_value_recursive(&entry.value, depth + 1)?;
                }
            }
            DamlValue::Unit => {}
            DamlValue::Bool(_) => {}
            DamlValue::Int64(_) => {}
            DamlValue::Numeric(_) => {}
            DamlValue::Text(text) => {
                if text.value.len() > 10000 {
                    return Err(SecurityError::InputValidationFailed(
                        "Text value exceeds maximum length".to_string(),
                    ));
                }
            }
            DamlValue::Timestamp(_) => {}
            DamlValue::Party(party) => {
                self.validate_party_id(&PartyId::new_unchecked(&party.value))?;
            }
            DamlValue::ContractId(contract_id) => {
                self.validate_contract_id(&contract_id.value)?;
            }
            DamlValue::Date(_) => {}
        }

        Ok(())
    }

    /// Validate a command.
    ///
    /// # Arguments
    ///
    /// * `command` - Command to validate
    ///
    /// # Returns
    ///
    /// Returns Ok(()) if valid, or an error if invalid
    pub fn validate_command(&self, command: &Command) -> Result<()> {
        match command {
            Command::Create(create) => {
                self.validate_identifier(&create.template_id)?;
                self.validate_daml_value(&DamlValue::Record(create.create_arguments.clone()))?;
            }
            Command::Exercise(exercise) => {
                self.validate_contract_id(&exercise.contract_id)?;
                if exercise.template_id.is_some() {
                    self.validate_identifier(exercise.template_id.as_ref().unwrap())?;
                }
                self.validate_daml_value(&exercise.choice_argument.clone().into())?;
            }
            Command::ExerciseByKey(exercise_by_key) => {
                self.validate_identifier(&exercise_by_key.template_id)?;
                self.validate_daml_value(&exercise_by_key.contract_key.clone().into())?;
                self.validate_daml_value(&exercise_by_key.choice_argument.clone().into())?;
            }
            Command::CreateAndExercise(create_and_exercise) => {
                self.validate_identifier(&create_and_exercise.template_id)?;
                self.validate_daml_value(&DamlValue::Record(
                    create_and_exercise.create_arguments.clone(),
                ))?;
                self.validate_daml_value(&create_and_exercise.choice_argument.clone().into())?;
            }
        }

        Ok(())
    }

    /// Validate commands.
    ///
    /// # Arguments
    ///
    /// * `commands` - Commands to validate
    ///
    /// # Returns
    ///
    /// Returns Ok(()) if valid, or an error if invalid
    pub fn validate_commands(&self, commands: &Commands) -> Result<()> {
        self.validate_party_id(&PartyId::new_unchecked(&commands.party))?;

        if let Some(ref workflow_id) = commands.workflow_id {
            self.validate_workflow_id(workflow_id)?;
        }

        if let Some(ref application_id) = commands.application_id {
            self.validate_application_id(application_id)?;
        }

        self.validate_transaction_id(&commands.command_id)?;

        for command in &commands.commands {
            self.validate_command(command)?;
        }

        Ok(())
    }

    /// Sanitize a string input.
    ///
    /// # Arguments
    ///
    /// * `input` - Input string to sanitize
    ///
    /// # Returns
    ///
    /// Returns sanitized string
    pub fn sanitize_string(&self, input: &str) -> String {
        // Remove null bytes and other control characters
        input
            .chars()
            .filter(|c| !c.is_control() || *c == '\n' || *c == '\r' || *c == '\t')
            .collect()
    }

    /// Validate and sanitize a string input.
    ///
    /// # Arguments
    ///
    /// * `input` - Input string to validate and sanitize
    ///
    /// # Returns
    ///
    /// Returns validated and sanitized string
    pub fn validate_and_sanitize_string(&self, input: &str) -> Result<String> {
        let sanitized = self.sanitize_string(input);

        if sanitized != input {
            tracing::warn!("Input string was sanitized");
        }

        Ok(sanitized)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_input_validator_default() {
        let validator = InputValidator::new();
        assert_eq!(validator.max_party_id_length, 256);
        assert_eq!(validator.max_contract_id_length, 512);
    }

    #[test]
    fn test_validate_party_id_valid() {
        let validator = InputValidator::new();
        let party_id = PartyId::new_unchecked("valid-party-123");
        assert!(validator.validate_party_id(&party_id).is_ok());
    }

    #[test]
    fn test_validate_party_id_empty() {
        let validator = InputValidator::new();
        let party_id = PartyId::new_unchecked("");
        assert!(validator.validate_party_id(&party_id).is_err());
    }

    #[test]
    fn test_validate_party_id_too_long() {
        let validator = InputValidator::new();
        let long_id = "a".repeat(257);
        let party_id = PartyId::new_unchecked(&long_id);
        assert!(validator.validate_party_id(&party_id).is_err());
    }

    #[test]
    fn test_validate_party_id_invalid_chars() {
        let validator = InputValidator::new();
        let party_id = PartyId::new_unchecked("invalid@party");
        assert!(validator.validate_party_id(&party_id).is_err());
    }

    #[test]
    fn test_validate_contract_id_valid() {
        let validator = InputValidator::new();
        assert!(validator.validate_contract_id("valid-contract-123").is_ok());
    }

    #[test]
    fn test_validate_contract_id_empty() {
        let validator = InputValidator::new();
        assert!(validator.validate_contract_id("").is_err());
    }

    #[test]
    fn test_validate_transaction_id_valid() {
        let validator = InputValidator::new();
        assert!(validator.validate_transaction_id("valid-tx-123").is_ok());
    }

    #[test]
    fn test_validate_transaction_id_empty() {
        let validator = InputValidator::new();
        assert!(validator.validate_transaction_id("").is_err());
    }

    #[test]
    fn test_validate_workflow_id_valid() {
        let validator = InputValidator::new();
        assert!(validator.validate_workflow_id("valid-workflow").is_ok());
    }

    #[test]
    fn test_validate_workflow_id_empty() {
        let validator = InputValidator::new();
        assert!(validator.validate_workflow_id("").is_err());
    }

    #[test]
    fn test_validate_application_id_valid() {
        let validator = InputValidator::new();
        assert!(validator.validate_application_id("valid-app").is_ok());
    }

    #[test]
    fn test_validate_application_id_empty() {
        let validator = InputValidator::new();
        assert!(validator.validate_application_id("").is_err());
    }

    #[test]
    fn test_sanitize_string() {
        let validator = InputValidator::new();
        let input = "hello\x00world";
        let sanitized = validator.sanitize_string(input);
        assert_eq!(sanitized, "helloworld");
    }

    #[test]
    fn test_validate_and_sanitize_string() {
        let validator = InputValidator::new();
        let input = "hello\x00world";
        let result = validator.validate_and_sanitize_string(input);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "helloworld");
    }
}
